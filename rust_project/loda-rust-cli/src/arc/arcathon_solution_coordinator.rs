//! Decides what gets saved to the `archaton_solution_json` file.
use super::{TestItem, TaskItem, arc_json_model, arcathon_solution_json};
use super::ArcathonSolutionJsonFile;
use std::collections::HashMap;
use std::path::{PathBuf, Path};

/// ARCathon solutions json file allows for [1..3] predictions per output_id.
#[allow(dead_code)]
static MAX_NUMBER_OF_PREDICTIONS: u8 = 3;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PredictionType {
    None,
    SolveLogisticRegression,
    SolveSplit,
    ExistingLODA,
    MutatedLODA,
}

impl PredictionType {
    #[allow(dead_code)]
    fn sort_weight(&self) -> u32 {
        match self {
            // Split tries out lots of things deterministic, so it's high priority.
            Self::SolveSplit => 0, 

            // The LODA programs that have been manually been coded are somewhat good and deals with many edge cases.
            Self::ExistingLODA => 1, 

            // The mutated LODA programs, may not deal with edge cases, but they are still good, since all train+test pairs gets evaluated.
            Self::MutatedLODA => 2, 

            // Logistic regression is rarely correct, so it's low priority.
            Self::SolveLogisticRegression => 9, 

            // When loaded from a file, without info about what type it is, then it's unclear what priority to assign, so assign the lowest priority.
            Self::None => 10, 
        }
    }
}

#[derive(Clone, Debug)]
pub struct Prediction {
    pub output_id: u8,
    pub output: arc_json_model::Grid,
    pub prediction_type: PredictionType,
}

impl Prediction {
    #[allow(dead_code)]
    fn testitems_from_predictionitems(predictions: &Vec<Prediction>) -> Vec<TestItem> {
        let mut max_output_id: u8 = 0;
        for prediction in predictions {
            if prediction.output_id > max_output_id {
                max_output_id = prediction.output_id;
            }
        }

        let mut testitem_vec = Vec::<TestItem>::new();

        for output_id in 0..=max_output_id {
            let mut type_and_item_vec = Vec::<(PredictionType, Prediction)>::new();
            for prediction in predictions {
                if prediction.output_id != output_id {
                    continue;
                }
                type_and_item_vec.push((prediction.prediction_type, prediction.clone()));
            }

            if type_and_item_vec.is_empty() {
                continue;
            }

            // Move the best prediction to the front. And the worst prediction to the back.
            type_and_item_vec.sort_unstable_by_key(|item| item.0.sort_weight());

            // Pick the N best predictions.
            type_and_item_vec.truncate(MAX_NUMBER_OF_PREDICTIONS as usize);

            // Assign an incrementing prediction id.
            let mut predictions_for_output = Vec::<arcathon_solution_json::Prediction>::new();
            for (assign_prediction_id, found_prediction) in type_and_item_vec.iter().enumerate() {
                predictions_for_output.push(arcathon_solution_json::Prediction {
                    prediction_id: assign_prediction_id.min(255) as u8,
                    output: found_prediction.1.output.clone(),
                });
            }

            // Save the number of predictions.
            let testitem = TestItem {
                output_id,
                number_of_predictions: predictions_for_output.len().min(255) as u8,
                predictions: predictions_for_output,
            };
            testitem_vec.push(testitem);
        }
        testitem_vec
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArcathonSolutionCoordinator {
    path_solution_dir: PathBuf,
    path_solution_teamid_json: PathBuf,

    taskname_to_prediction_vec: HashMap<String, Vec<Prediction>>,
}

impl ArcathonSolutionCoordinator {
    pub fn new(path_solution_dir: &Path, path_solution_teamid_json: &Path) -> Self {
        Self {
            path_solution_dir: path_solution_dir.to_path_buf(),
            path_solution_teamid_json: path_solution_teamid_json.to_path_buf(),
            taskname_to_prediction_vec: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn append_predictions(&mut self, task_name: String, prediction_vec: Vec<Prediction>) {
        self.taskname_to_prediction_vec
            .entry(task_name)
            .or_insert(Vec::new())
            .extend(prediction_vec);
    }

    #[allow(dead_code)]
    pub fn save_solutions_json(&self) -> anyhow::Result<()> {
        let mut task_vec = Vec::<TaskItem>::new();
        for (taskname, prediction_vec) in &self.taskname_to_prediction_vec {

            let testitem_vec: Vec<TestItem> = Prediction::testitems_from_predictionitems(prediction_vec);
            if testitem_vec.is_empty() {
                continue;
            }

            let task_item = TaskItem {
                task_name: taskname.clone(),
                test_vec: testitem_vec,
            };
            task_vec.push(task_item);
        }

        let solution_json_file = ArcathonSolutionJsonFile {
            task_vec,
        };
        match solution_json_file.save(&self.path_solution_dir, &self.path_solution_teamid_json) {
            Ok(()) => {},
            Err(error) => {
                return Err(anyhow::anyhow!("Unable to save solutions file. path: {:?} error: {:?}", self.path_solution_teamid_json, error));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_testitems_from_predictionitems() {
        // Arrange
        let mut prediction_vec = Vec::<Prediction>::new();

        // output_id 5
        {
            let prediction = Prediction {
                output_id: 5,
                output: vec![vec![2]],
                prediction_type: PredictionType::MutatedLODA,
            };
            prediction_vec.push(prediction);
        }
        {
            let prediction = Prediction {
                output_id: 5,
                output: vec![vec![4]],
                prediction_type: PredictionType::None, // This gets ignored, since it's the worst prediction.
            };
            prediction_vec.push(prediction);
        }
        {
            let prediction = Prediction {
                output_id: 5,
                output: vec![vec![1]],
                prediction_type: PredictionType::SolveSplit,
            };
            prediction_vec.push(prediction);
        }
        {
            let prediction = Prediction {
                output_id: 5,
                output: vec![vec![3]],
                prediction_type: PredictionType::SolveLogisticRegression,
            };
            prediction_vec.push(prediction);
        }

        // output_id 9
        {
            let prediction = Prediction {
                output_id: 9,
                output: vec![vec![6]],
                prediction_type: PredictionType::None,
            };
            prediction_vec.push(prediction);
        }
        {
            let prediction = Prediction {
                output_id: 9,
                output: vec![vec![5]],
                prediction_type: PredictionType::MutatedLODA,
            };
            prediction_vec.push(prediction);
        }

        // Act
        let testitem_vec: Vec<TestItem> = Prediction::testitems_from_predictionitems(&prediction_vec);

        // Assert
        assert_eq!(testitem_vec.len(), 2);
        {
            let testitem = &testitem_vec[0];
            assert_eq!(testitem.output_id, 5);
            assert_eq!(testitem.predictions.len(), 3);
            assert_eq!(testitem.number_of_predictions, 3);
            let mut pixeldata: Vec<u8> = Vec::new();
            for prediction in &testitem.predictions {
                for row in &prediction.output {
                    pixeldata.extend(row.clone());
                }
            }
            assert_eq!(pixeldata, vec![1, 2, 3]);
            let prediction_ids: Vec<u8> = testitem.predictions.iter().map(|prediction| prediction.prediction_id).collect();
            assert_eq!(prediction_ids, vec![0, 1, 2]);
        }
        {
            let testitem = &testitem_vec[1];
            assert_eq!(testitem.output_id, 9);
            assert_eq!(testitem.predictions.len(), 2);
            assert_eq!(testitem.number_of_predictions, 2);
            let mut pixeldata: Vec<u8> = Vec::new();
            for prediction in &testitem.predictions {
                for row in &prediction.output {
                    pixeldata.extend(row.clone());
                }
            }
            assert_eq!(pixeldata, vec![5, 6]);
            let prediction_ids: Vec<u8> = testitem.predictions.iter().map(|prediction| prediction.prediction_id).collect();
            assert_eq!(prediction_ids, vec![0, 1]);
        }
    }
}
