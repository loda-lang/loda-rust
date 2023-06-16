use super::arc_work_model::{PairType, Task};
use super::{ImageSize};
use std::path::{PathBuf, Path};

pub struct SubcommandARCSize;

impl SubcommandARCSize {
    /// The `arc-size` subcommand when invoked from the command line.
    /// 
    /// Predict the output sizes of the test pairs in the specified ARC task json file.
    /// 
    /// Prints the output sizes to stdout.
    pub fn run(json_file: &Path) -> anyhow::Result<()> {
        let s: String = Self::predict_output_sizes_of_task(json_file)?;
        println!("{}", s);
        Ok(())
    }

    /// Predict the output sizes of the test pairs in the specified ARC task json file.
    pub fn predict_output_sizes_of_task(json_file: &Path) -> anyhow::Result<String> {
        if !json_file.is_file() {
            return Err(anyhow::anyhow!("Cannot read the specified file. It must be an ARC task json file."));
        }
        println!("json_file: {:?}", json_file);

        let task: Task = Task::load_with_json_file(&json_file)?;
        if !task.has_predicted_output_size() {
            return Err(anyhow::anyhow!("Cannot predict the output sizes for this task."));
        }

        for pair in &task.pairs {
            if pair.pair_type != PairType::Test {
                continue;
            }
            let size: ImageSize = pair.predicted_output_size().expect("Expected an output size");
            println!("size: {:?}", size);
        }

        let s = String::from("hello");
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::path_testdata;

    #[test]
    fn test_10000_run() {
        // Arrange
        let path: PathBuf = path_testdata("72ca375d").expect("path");

        // Act
        let actual: String = SubcommandARCSize::predict_output_sizes_of_task(&path).expect("string");

        // Assert
        assert_eq!(actual, "hello");
    }
}
