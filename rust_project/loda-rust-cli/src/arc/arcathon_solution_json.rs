use super::Grid;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Prediction {
    pub prediction_id: u8,
    pub output: Grid,
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

pub type Tasks = Vec<TaskItem>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::read_testdata;

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

    fn mock_tasks() -> Tasks {
        let mut task_vec = Vec::<TaskItem>::new();
        task_vec.push(mock_taskitem());
        task_vec.push(mock_taskitem());
        task_vec
    }

    #[test]
    fn test_10003_tasks_to_json() {
        let instance = mock_tasks();
        let json: String = serde_json::to_string(&instance).expect("string");
        assert_eq!(json.starts_with("[{"), true);
        assert_eq!(json.ends_with("}]"), true);
        assert_eq!(json.contains("mock_taskitem"), true);
        assert_eq!(json.contains("number_of_predictions"), true);
        assert_eq!(json.contains("prediction_id"), true);
        assert_eq!(json.contains("[[1,2],[3,4]]"), true);
    }

    #[test]
    fn test_20000_deserialize() {
        // Arrange
        let json: String = read_testdata("arcathon_solution_format").expect("string");

        // Act
        let tasks: Tasks = serde_json::from_str(&json).expect("tasks");

        // Assert
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].task_name, "12997ef3");
        assert_eq!(tasks[1].task_name, "13713586");
        assert_eq!(tasks[0].test_vec.len(), 2);
        assert_eq!(tasks[1].test_vec.len(), 1);
    }
}
