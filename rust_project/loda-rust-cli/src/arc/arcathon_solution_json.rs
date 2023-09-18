//! Read and write the `archathon_solution_json` file.
use super::arc_json_model;
use serde::{Deserialize, Serialize};
use std::{path::Path, fs};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Prediction {
    pub prediction_id: u8,
    pub output: arc_json_model::Grid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestItem {
    pub output_id: u8,
    pub number_of_predictions: u8,
    pub predictions: Vec<Prediction>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskItem {
    pub task_name: String,

    #[serde(rename = "test")]
    pub test_vec: Vec<TestItem>,
}

/// Wrapper for the `archathon_solution_json` file.
#[derive(Clone, Debug)]
pub struct ArcathonSolutionJsonFile {
    pub task_vec: Vec<TaskItem>,
}

impl ArcathonSolutionJsonFile {
    /// Load the `archaton_solution_json` file.
    pub fn load(path_solution_teamid_json: &Path) -> anyhow::Result<Self> {
        let solution_teamid_json_string: String = match fs::read_to_string(path_solution_teamid_json) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Something went wrong reading the file: {:?} error: {:?}", path_solution_teamid_json, error));
            }
        };
        let tasks: Vec<TaskItem> = match serde_json::from_str(&solution_teamid_json_string) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Could not parse archaton_solution_json file, path: {:?} error: {:?} json: {:?}", path_solution_teamid_json, error, solution_teamid_json_string));
            }
        };
        let instance = Self {
            task_vec: tasks,
        };
        Ok(instance)
    }

    pub fn empty() -> Self {
        Self {
            task_vec: vec!(),
        }
    }

    /// Save the `archaton_solution_json` file.
    /// 
    /// Returns the file size in bytes.
    pub fn save(&self, path_solution_dir: &Path, path_solution_teamid_json: &Path) -> anyhow::Result<usize> {
        if !path_solution_dir.exists() {
                match fs::create_dir(path_solution_dir) {
                Ok(_) => {},
                Err(error) => {
                    return Err(anyhow::anyhow!("Unable to create solution directory: {:?}, error: {:?}", path_solution_dir, error));
                }
            }
        }
        let json: String = match serde_json::to_string(&self.task_vec) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Unable to serialize task_vec to json: {:?}", error));
            }
        };
        let bytes: usize = json.len();
        match fs::write(&path_solution_teamid_json, json) {
            Ok(()) => {},
            Err(error) => {
                return Err(anyhow::anyhow!("Unable to save solutions file. path: {:?} error: {:?}", path_solution_teamid_json, error));
            }
        }
        Ok(bytes)
    }    
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::path_testdata;
    use std::path::PathBuf;

    fn mock_prediction() -> Prediction {
        Prediction {
            prediction_id: 8,
            output: vec![vec![1, 2], vec![3, 4]]
        }
    }

    #[test]
    fn test_10000_prediction_to_json() {
        let instance = mock_prediction();
        let json: String = serde_json::to_string(&instance).expect("string");
        assert_eq!(json, "{\"prediction_id\":8,\"output\":[[1,2],[3,4]]}");
    }

    fn mock_testitem() -> TestItem {
        let mut predictions = Vec::<Prediction>::new();
        predictions.push(mock_prediction());
        predictions.push(mock_prediction());
        TestItem {
            output_id: 23,
            number_of_predictions: 2,
            predictions,
        }
    }

    #[test]
    fn test_10001_testitem_to_json() {
        let instance = mock_testitem();
        let json: String = serde_json::to_string(&instance).expect("string");
        assert_eq!(json, "{\"output_id\":23,\"number_of_predictions\":2,\"predictions\":[{\"prediction_id\":8,\"output\":[[1,2],[3,4]]},{\"prediction_id\":8,\"output\":[[1,2],[3,4]]}]}");
    }

    fn mock_taskitem() -> TaskItem {
        let mut test_vec = Vec::<TestItem>::new();
        test_vec.push(mock_testitem());
        test_vec.push(mock_testitem());
        TaskItem {
            task_name: "mock_taskitem".to_string(),
            test_vec
        }
    }

    #[test]
    fn test_10002_taskitem_to_json() {
        let instance = mock_taskitem();
        let json: String = serde_json::to_string(&instance).expect("string");
        assert_eq!(json.starts_with("{"), true);
        assert_eq!(json.ends_with("}"), true);
        assert_eq!(json.contains("mock_taskitem"), true);
        assert_eq!(json.contains("number_of_predictions"), true);
        assert_eq!(json.contains("prediction_id"), true);
        assert_eq!(json.contains("[[1,2],[3,4]]"), true);
    }

    fn mock_task_vec() -> Vec<TaskItem> {
        let mut task_vec = Vec::<TaskItem>::new();
        task_vec.push(mock_taskitem());
        task_vec.push(mock_taskitem());
        task_vec
    }

    #[test]
    fn test_20000_save_solutions_json() -> anyhow::Result<()> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_20000_save_solutions_json");
        fs::create_dir(&basedir)?;

        let path_solutions_json: PathBuf = basedir.join("solutions.json");

        let task_vec = mock_task_vec();
        let tasks = ArcathonSolutionJsonFile {
            task_vec
        };

        // Act
        let returned_bytes: usize = tasks.save(&basedir, &path_solutions_json)?;

        // Assert
        let filesize: u64 = path_solutions_json.metadata()?.len();
        assert_eq!(returned_bytes as u64, filesize);
        assert_eq!(returned_bytes, 659);
        let json: String = fs::read_to_string(&path_solutions_json)?;
        assert_eq!(json.starts_with("[{"), true);
        assert_eq!(json.ends_with("}]"), true);
        assert_eq!(json.contains("mock_taskitem"), true);
        assert_eq!(json.contains("number_of_predictions"), true);
        assert_eq!(json.contains("prediction_id"), true);
        assert_eq!(json.contains("[[1,2],[3,4]]"), true);
        Ok(())
    }

    #[test]
    fn test_30000_read_solutions_json() {
        // Arrange
        let path: PathBuf = path_testdata("arcathon_solution_format").expect("ok");

        // Act
        let tasks_instance: ArcathonSolutionJsonFile = ArcathonSolutionJsonFile::load(&path).expect("ok");

        // Assert
        let tasks: Vec<TaskItem> = tasks_instance.task_vec.clone();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].task_name, "12997ef3");
        assert_eq!(tasks[1].task_name, "13713586");
        assert_eq!(tasks[0].test_vec.len(), 2);
        assert_eq!(tasks[1].test_vec.len(), 1);
        assert_eq!(tasks[0].test_vec[0].number_of_predictions, 3);
        assert_eq!(tasks[0].test_vec[0].predictions.len(), 3);
        assert_eq!(tasks[0].test_vec[1].number_of_predictions, 3);
        assert_eq!(tasks[0].test_vec[1].predictions.len(), 3);
        assert_eq!(tasks[1].test_vec[0].number_of_predictions, 3);
        assert_eq!(tasks[1].test_vec[0].predictions.len(), 3);
    }

    #[test]
    fn test_30001_read_solutions_json() {
        // Arrange
        let path: PathBuf = path_testdata("solution_notXORdinary").expect("ok");

        // Act
        let tasks_instance: ArcathonSolutionJsonFile = ArcathonSolutionJsonFile::load(&path).expect("ok");

        // Assert
        let tasks: Vec<TaskItem> = tasks_instance.task_vec.clone();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].task_name, "3428a4f5");
        assert_eq!(tasks[1].task_name, "f2829549");
        assert_eq!(tasks[0].test_vec.len(), 2);
        assert_eq!(tasks[1].test_vec.len(), 1);
        assert_eq!(tasks[0].test_vec[0].number_of_predictions, 1);
        assert_eq!(tasks[0].test_vec[0].predictions.len(), 1);
        assert_eq!(tasks[0].test_vec[1].number_of_predictions, 1);
        assert_eq!(tasks[0].test_vec[1].predictions.len(), 1);
        assert_eq!(tasks[1].test_vec[0].number_of_predictions, 1);
        assert_eq!(tasks[1].test_vec[0].predictions.len(), 1);
    }
}
