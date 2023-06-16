use super::arc_work_model::{PairType, Task};
use super::ImageSize;
use std::path::Path;
use serde::Serialize;

pub struct SubcommandARCSize;

impl SubcommandARCSize {
    /// The `arc-size` subcommand when invoked from the command line.
    /// 
    /// Predict the output sizes of the `test` pairs in the specified ARC task json file.
    /// 
    /// Prints json to stdout, containing the predicted output sizes.
    pub fn run(task_json_file: &Path) -> anyhow::Result<()> {
        let json_with_prediction: String = Self::predict_output_sizes_of_task_format_as_json(task_json_file)?;
        println!("{}", json_with_prediction);
        Ok(())
    }

    /// Predict the output sizes of the test pairs in the specified ARC task json file.
    pub fn predict_output_sizes_of_task_format_as_json(task_json_file: &Path) -> anyhow::Result<String> {
        if !task_json_file.is_file() {
            return Err(anyhow::anyhow!("Cannot read the specified file. It must be an ARC task json file."));
        }

        // The prediction of the output sizes, happens during loading of the file.
        let task: Task = Task::load_with_json_file(&task_json_file)?;
        if !task.has_predicted_output_size() {
            return Err(anyhow::anyhow!("Cannot predict the output sizes for this task."));
        }

        // Extract the output sizes only from the `test` pairs.
        let mut test_pair_vec = Vec::<PairItem>::new();
        for pair in &task.pairs {
            if pair.pair_type != PairType::Test {
                continue;
            }
            let size: ImageSize = pair.predicted_output_size().expect("Expected an output size");
            let output: OutputItem = OutputItem {
                width: size.width,
                height: size.height,
            };
            let pair: PairItem = PairItem {
                output: output,
            };
            test_pair_vec.push(pair);
        }

        let task_item = TaskItem { test: test_pair_vec };
        let json_data: String = serde_json::to_string(&task_item)?;
        Ok(json_data)
    }
}

#[derive(Debug, Serialize)]
struct OutputItem {
    width: u8,
    height: u8,
}

#[derive(Debug, Serialize)]
struct PairItem {
    output: OutputItem,
}

#[derive(Debug, Serialize)]
struct TaskItem {
    test: Vec<PairItem>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::path_testdata;
    use std::path::PathBuf;

    #[test]
    fn test_10000_two_outputs() {
        // Arrange
        let path: PathBuf = path_testdata("d5d6de2d").expect("path");

        // Act
        let actual: String = SubcommandARCSize::predict_output_sizes_of_task_format_as_json(&path).expect("string");

        // Assert
        let expected = r#"{"test":[{"output":{"width":10,"height":10}},{"output":{"width":25,"height":25}}]}"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_one_output() {
        // Arrange
        let path: PathBuf = path_testdata("017c7c7b").expect("path");

        // Act
        let actual: String = SubcommandARCSize::predict_output_sizes_of_task_format_as_json(&path).expect("string");

        // Assert
        let expected = r#"{"test":[{"output":{"width":3,"height":9}}]}"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_one_output() {
        // Arrange
        let path: PathBuf = path_testdata("f9012d9b").expect("path");

        // Act
        let actual: String = SubcommandARCSize::predict_output_sizes_of_task_format_as_json(&path).expect("string");

        // Assert
        let expected = r#"{"test":[{"output":{"width":3,"height":3}}]}"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_error_cannot_predict() {
        // Arrange
        let path: PathBuf = path_testdata("8731374e").expect("path");

        // Act
        let error = SubcommandARCSize::predict_output_sizes_of_task_format_as_json(&path).expect_err("is supposed to fail");

        // Assert
        let message: String = format!("{}", error);
        assert_eq!(message.contains("Cannot predict the output sizes for this task"), true);
    }
}
