//! Decides what gets saved to the `archaton_solution_json` file.
//! 
//! Future experiments:
//! Assign it a higher confidence score, when there are identical predictions from multple solvers.
//! Eliminate duplicate predictions, when there are identical predictions from multple solvers.
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
    SolveGenetic,
}

impl PredictionType {
    #[allow(dead_code)]
    fn sort_weight(&self) -> u32 {
        match self {
            // Split tries out lots of things deterministic, so it's high priority.
            Self::SolveSplit => 0, 

            // The LODA programs that have been manually been coded are somewhat good and deals with many edge cases.
            // The mutated LODA programs, may not deal with edge cases, but they are still good, since all train+test pairs gets evaluated.
            Self::SolveGenetic => 1, 

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
            type_and_item_vec.sort_by_key(|item| item.0.sort_weight());

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

    /// Populate with previous predictions, so it's possible to continue from where it left off.
    /// 
    /// The predictions are assigned `PredictionType::None`, so that they get the lowest priority.
    #[allow(dead_code)]
    pub fn import_predictions_from_solution_json_file(&mut self, solution_json_file: &ArcathonSolutionJsonFile) {
        for task_item in &solution_json_file.task_vec {
            let mut predictions_to_append = Vec::<Prediction>::new();
            for test_item in &task_item.test_vec {
                for prediction in &test_item.predictions {
                    let prediction = Prediction {
                        output_id: test_item.output_id,
                        output: prediction.output.clone(),
                        prediction_type: PredictionType::None,
                    };
                    predictions_to_append.push(prediction);
                }
            }
            self.append_predictions(task_item.task_name.clone(), predictions_to_append);
        }
    }

    /// Returns the number of bytes of the saved file.
    #[allow(dead_code)]
    pub fn save_solutions_json(&self) -> anyhow::Result<usize> {
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

        // Sort by task_name, so the tasks don't appear in random order.
        task_vec.sort_by_key(|task_item| task_item.task_name.clone());

        let solution_json_file = ArcathonSolutionJsonFile {
            task_vec,
        };
        let bytes: usize = match solution_json_file.save(&self.path_solution_dir, &self.path_solution_teamid_json) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Err(anyhow::anyhow!("Unable to save solutions file. path: {:?} error: {:?}", self.path_solution_teamid_json, error));
            }
        };
        Ok(bytes)
    }

    #[allow(dead_code)]
    pub fn save_solutions_json_with_console_output(&self) {
        match self.save_solutions_json() {
            Ok(bytes) => {
                println!("Saved solutions file: {:?} bytes: {}", self.path_solution_teamid_json, bytes);
            },
            Err(error) => {
                error!("Unable to save solutions file. path: {:?} error: {:?}", self.path_solution_teamid_json, error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::path_testdata;
    use std::{path::PathBuf, fs};

    #[test]
    fn test_10000_import_predictions_from_solution_json_file() -> anyhow::Result<()> {
        // Arrange
        let path: PathBuf = path_testdata("arcathon_solution_format").expect("ok");
        let solution_json_file: ArcathonSolutionJsonFile = ArcathonSolutionJsonFile::load(&path).expect("ok");

        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_import_predictions_from_solution_json_file");
        fs::create_dir(&basedir)?;

        let path_solutions_json: PathBuf = basedir.join("solutions.json");

        let mut coordinator = ArcathonSolutionCoordinator::new(
            &basedir,
            &path_solutions_json,
        );

        // Act
        coordinator.import_predictions_from_solution_json_file(&solution_json_file);

        // Assert
        assert_eq!(coordinator.taskname_to_prediction_vec.len(), 2);
        {
            let predictions: &Vec<Prediction> = coordinator.taskname_to_prediction_vec.get("12997ef3").expect("ok");
            assert_eq!(predictions.len(), 6);
        }
        {
            let predictions: &Vec<Prediction> = coordinator.taskname_to_prediction_vec.get("13713586").expect("ok");
            assert_eq!(predictions.len(), 3);
        }
        Ok(())
    }

    #[test]
    fn test_20000_testitems_from_predictionitems() {
        // Arrange
        let mut prediction_vec = Vec::<Prediction>::new();

        // output_id 5
        {
            let prediction = Prediction {
                output_id: 5,
                output: vec![vec![2]],
                prediction_type: PredictionType::SolveGenetic,
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
                prediction_type: PredictionType::SolveGenetic,
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

    #[test]
    fn test_30000_same_output_across_multiple_loadsave_iterations() -> anyhow::Result<()> {
        // Arrange
        let original_file_path: PathBuf = path_testdata("arcathon_solution_format").expect("ok");

        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_30000_same_output_across_multiple_loadsave_iterations");
        fs::create_dir(&basedir)?;

        let path_solutions_json1: PathBuf = basedir.join("solutions1.json");
        let path_solutions_json2: PathBuf = basedir.join("solutions2.json");
        
        // Act
        let mut coordinator1 = ArcathonSolutionCoordinator::new(
            &basedir,
            &path_solutions_json1,
        );
        {
            let solution_json_file: ArcathonSolutionJsonFile = ArcathonSolutionJsonFile::load(&original_file_path).expect("ok");
            coordinator1.import_predictions_from_solution_json_file(&solution_json_file);
        }
        coordinator1.save_solutions_json()?;

        let mut coordinator2 = ArcathonSolutionCoordinator::new(
            &basedir,
            &path_solutions_json2,
        );
        {
            let solution_json_file: ArcathonSolutionJsonFile = ArcathonSolutionJsonFile::load(&path_solutions_json1).expect("ok");
            coordinator2.import_predictions_from_solution_json_file(&solution_json_file);
        }
        coordinator2.save_solutions_json()?;

        // Assert
        let content1: String = fs::read_to_string(&path_solutions_json1).expect("ok");
        let content2: String = fs::read_to_string(&path_solutions_json2).expect("ok");
        assert_eq!(content1, content2);
        assert_eq!(content1.len(), 644);
        Ok(())
    }
}
