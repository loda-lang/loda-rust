use crate::{common::find_json_files_recursively, arc::generate_dataset_histogram::{self, DatasetItemForTask}};
use super::arc_json_model::{TaskId, Task};
use std::{path::{Path, PathBuf}, collections::HashSet};
use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::{Value, Map};
use std::fs;

pub struct SubcommandARCMetadata;

impl SubcommandARCMetadata {
    /// The `arc-metadata-histograms` subcommand when invoked from the command line.
    /// 
    /// Traverses the task json files, and assign a number of histogram comparisons.
    pub fn run(count: u16, seed: u64, task_json_directory: &Path) -> anyhow::Result<()> {
        debug!("count: {} seed: {} directory: {:?}", count, seed, task_json_directory);
        if !task_json_directory.is_dir() {
            anyhow::bail!("arc-metadata-histograms. Expected directory to be a directory, but it's not. path: {:?}", task_json_directory);
        }
        if count == 0 {
            anyhow::bail!("arc-metadata-histograms. Expected count to be greater than zero, but it's not. count: {}", count);
        }
        if count > 1000 {
            anyhow::bail!("arc-metadata-histograms. Expected count to be less than or equal to 1000. count: {}", count);
        }

        // Traverse the directory and find all the task json files, recursively
        let paths: Vec<PathBuf> = find_json_files_recursively(&task_json_directory);
        debug!("arc-metadata-histograms. Found {} task json files", paths.len());
        if paths.len() > 10000 {
            anyhow::bail!("arc-metadata-histograms. Aborting. Found way too many json files. We don't want to cause too much havoc. count: {}", paths.len());
        }
        
        let pb = ProgressBar::new(paths.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-")
        );
        
        paths.par_iter().for_each(|path| {
            pb.inc(1);
            match Self::generate_metadata_for_task(count, seed, path) {
                Ok(()) => {},
                Err(error) => {
                    error!("arc-metadata-histograms. Problem with file. error: {:?} path: {:?}", error, path);
                }
            }
        });
        Ok(())
    }

    fn generate_metadata_for_task(count: u16, seed: u64, path: &Path) -> anyhow::Result<()> {
        let original_json_string: String = fs::read_to_string(&path)?;
        let original_size: usize = original_json_string.as_bytes().len();

        // Load the json file into a Task. 
        // If it cannot be loaded, it may not be an ARC json file, but some other json file, thus it's safer to skip the file.
        let task_id = TaskId::Custom { identifier: "no-id".to_string() };
        let task: Task = Task::from_json(task_id, &original_json_string)
            .with_context(|| format!("Unable to parse task json file. path: {:?}", path))?;

        // Generate a number of histogram comparisons.
        debug!("arc-metadata-histograms. Start generating. count: {} seed: {} path: {:?}", count, seed, path);

        // Invoke the generate_dataset_histogram::GenerateDataset::markdown_for_comparison_items() function.
        let mut dataset_items: Vec<DatasetItemForTask> = vec!();
        let mut metadata_ids = HashSet::<String>::new();
        for i in 0..(count * 2) {
            let seed_plus_i = seed + (i as u64);
            let result = generate_dataset_histogram::GenerateDataset::generate_with_task(
                &task,
                seed_plus_i,
                false,
            );
            let dataset_item: DatasetItemForTask = match result {
                Ok(value) => value,
                Err(error) => {
                    error!("arc-metadata-histograms. Something went wrong generating histogram. seed_plus_i: {} error: {:?} path: {:?}", seed_plus_i, error, path);
                    continue;
                }
            };
            metadata_ids.insert(dataset_item.metadata_id.clone());
            dataset_items.push(dataset_item);
            if metadata_ids.len() == count as usize {
                break;
            }
        }

        // Convert dataset items to key value pairs
        let mut insert_key_value_pairs: Vec<(String, String)> = vec!();
        for dataset_item in dataset_items {
            let key: String = dataset_item.metadata_id.clone();
            let value: String = dataset_item.markdown.clone();
            debug!("arc-metadata-histograms. key: {} value: {} bytes path: {:?}", key, value.as_bytes().len(), path);
            insert_key_value_pairs.push((key, value));
        }

        // Update the json file with the new metadata
        let updated_json: String = SubcommandARCMetadata::update_json(
            &original_json_string, 
            "metadata", 
            insert_key_value_pairs
        )?;

        let bytes_written: usize = updated_json.as_bytes().len();
        let bytes_inserted: i64 = bytes_written as i64 - original_size as i64;
        debug!("arc-metadata-histograms. Inserted {} bytes into json file. path: {:?}", bytes_inserted, path);

        fs::write(&path, updated_json)
            .with_context(|| format!("Unable to write to task json file. path: {:?}", path))?;

        Ok(())
    }

    fn update_json(json_string: &str, dict_name: &str, insert_key_value_pairs: Vec<(String, String)>) -> anyhow::Result<String> {
        let mut json: Value = serde_json::from_str(&json_string)?;
    
        // Ensure the root of the json file is a dictionary
        if let Some(obj) = json.as_object_mut() {
            // Access or create the dictionary named `dict_name`
            let dict_entry = obj.entry(dict_name.to_string()).or_insert_with(|| Value::Object(Map::new()));
    
            // Ensure `dict_name` is a dictionary
            if let Some(dict) = dict_entry.as_object_mut() {
                // Append the new key-value pairs to the dictionary
                for (key, value) in insert_key_value_pairs {
                    dict.insert(key, Value::from(value));
                }
            } else {
                anyhow::bail!("'{}' exists but is not a dictionary.", dict_name);
            }
        } else {
            anyhow::bail!("Expected root of json file to be a dictionary, but it's not.");
        }
    
        // Serialize the modified data back to JSON
        let updated_json: String = serde_json::to_string(&json)?;
        Ok(updated_json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_update_json_preserve_ordering() {
        // Arrange
        let data = r#"{"b": 1,"a": 2,"x": 3}"#;

        // Act
        let json: String = SubcommandARCMetadata::update_json(data, "metadata", vec![("key".to_string(), "value".to_string())]).expect("ok");

        // Assert
        assert_eq!(json, r#"{"b":1,"a":2,"x":3,"metadata":{"key":"value"}}"#);
    }

    #[test]
    fn test_10001_update_json_preserve_existing_metadata() {
        // Arrange
        let data = r#"{"b": 1,"metadata":{"key0":"value0"},"a": 2,"x": 3}"#;

        // Act
        let json: String = SubcommandARCMetadata::update_json(data, "metadata", vec![("key1".to_string(), "value1".to_string())]).expect("ok");

        // Assert
        assert_eq!(json, r#"{"b":1,"metadata":{"key0":"value0","key1":"value1"},"a":2,"x":3}"#);
    }

    #[test]
    fn test_10002_update_json_error_handling_root_not_dictionary() {
        // Arrange
        let data = r#"["a", "b", "c"]"#;

        // Act
        let error = SubcommandARCMetadata::update_json(data, "metadata", vec![("key1".to_string(), "value1".to_string())]).expect_err("is supposed to fail");

        // Assert
        let message: String = format!("{}", error);
        assert_eq!(message, "Expected root of json file to be a dictionary, but it's not.");
    }

    #[test]
    fn test_10003_update_json_error_handling_metadata_not_dictionary() {
        // Arrange
        let data = r#"{"b": 1,"metadata":["key0","value0"],"a": 2,"x": 3}"#;

        // Act
        let error = SubcommandARCMetadata::update_json(data, "metadata", vec![("key1".to_string(), "value1".to_string())]).expect_err("is supposed to fail");

        // Assert
        let message: String = format!("{}", error);
        assert_eq!(message, "'metadata' exists but is not a dictionary.");
    }

    #[test]
    fn test_10004_update_json_error_handling_metadata_not_dictionary() {
        // Arrange
        let data = r#"{"b": 1,"metadata":42,"a": 2,"x": 3}"#;

        // Act
        let error = SubcommandARCMetadata::update_json(data, "metadata", vec![("key1".to_string(), "value1".to_string())]).expect_err("is supposed to fail");

        // Assert
        let message: String = format!("{}", error);
        assert_eq!(message, "'metadata' exists but is not a dictionary.");
    }
}
