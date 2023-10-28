//! Compare the actual prediction with the expected output.
//! 
//! Measure the difference between the two images.
use super::{Image, arc_work_model::Task, arc_work_model::Pair, arcathon_solution_coordinator, arc_json_model::GridToImage, ImageCompare, ImageMaskCount, arcathon_solution_json};
use anyhow::Context;

#[derive(Clone, Debug, PartialEq)]
pub struct VerifyPredictionIncorrectData {
    pub number_of_pixels_total: u16,
    pub number_of_pixels_with_correct_value: u16,
    pub number_of_pixels_with_incorrect_value: u16,
    pub percentage_correct: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerifyPrediction {
    /// Same size, all pixels are the same
    Correct,

    /// The predicted image has same size as the expected image, but some of the pixels are different
    Incorrect { incorrect_data: VerifyPredictionIncorrectData },

    /// The predicted image has a different size than the expected image.
    WrongSize,
}

impl VerifyPrediction {
    /// Compare the actual prediction with the expected output
    fn create(actual: &Image, expected: &Image) -> anyhow::Result<VerifyPrediction> {
        if *actual == *expected {
            return Ok(VerifyPrediction::Correct);
        }

        if actual.size() != expected.size() {
            return Ok(VerifyPrediction::WrongSize);
        }

        // Same image size, but one or more pixels are different
        // Measure the difference
        let diff: Image = expected.diff(&actual).context("VerifyPrediction.create diff")?;
        let (count0, count1, count_other) = diff.mask_count();
        // 0 is where the two images are the same
        // 1 is where the two images are different
        let total: u16 = count0 + count1 + count_other;

        let percentage_correct: u8 = ((count0 as u32 * 100) / (total as u32)).min(255) as u8;

        let incorrect_data = VerifyPredictionIncorrectData {
            number_of_pixels_total: total,
            number_of_pixels_with_correct_value: count0,
            number_of_pixels_with_incorrect_value: count1 + count_other,
            percentage_correct,
        };
        Ok(VerifyPrediction::Incorrect { incorrect_data })
    }

    /// Obtain the test pair that corresponds to the prediction index
    fn pair_for_test_index(task: &Task, test_index: u8) -> anyhow::Result<&Pair> {
        let found_pair: Option<&Pair> = task.pairs.iter().find(|pair| {
            pair.test_index == Some(test_index)
        });
        match found_pair {
            Some(value) => Ok(value),
            None => {
                Err(anyhow::anyhow!("VerifyPrediction.pair_for_test_index() Task: {} has no pair where test_index corresponds to output_id {}", task.id, test_index))
            }
        }
    }
}

pub trait VerifyPredictionWithTask {
    /// Check a single prediction with the expected output.
    fn verify_prediction(&self, task: &Task) -> anyhow::Result<VerifyPrediction>;
}

impl VerifyPredictionWithTask for arcathon_solution_coordinator::Prediction {
    fn verify_prediction(&self, task: &Task) -> anyhow::Result<VerifyPrediction> {
        let pair: &Pair = VerifyPrediction::pair_for_test_index(task, self.output_id)?;
        let expected: &Image = &pair.output.test_image;
        let actual: Image = self.output.to_image().context("arcathon_solution_coordinator::Prediction.verify_prediction to_image")?;
        VerifyPrediction::create(&actual, expected)
    }
}

impl VerifyPredictionWithTask for arcathon_solution_json::Prediction {
    fn verify_prediction(&self, task: &Task) -> anyhow::Result<VerifyPrediction> {
        let pair: &Pair = VerifyPrediction::pair_for_test_index(task, self.prediction_id)?;
        let expected: &Image = &pair.output.test_image;
        let actual: Image = self.output.to_image().context("arcathon_solution_json::Prediction.verify_prediction to_image")?;
        VerifyPrediction::create(&actual, expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::arc_json_model;
    use crate::arc::arc_json_model::GridFromImage;
    use crate::arc::arc_work_model;
    use crate::arc::ImageTryCreate;

    fn task_with_testdata(name: &str) -> anyhow::Result<arc_work_model::Task> {
        let json_task: arc_json_model::Task = arc_json_model::Task::load_testdata(name)?;
        arc_work_model::Task::try_from(&json_task)
    }

    #[test]
    fn test_10000_verify_prediction_correct() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            0, 1, 2,
            3, 4, 5,
        ];
        let image0: Image = Image::try_create(3, 2, pixels0).expect("image");

        // Act
        let actual: VerifyPrediction = VerifyPrediction::create(&image0, &image0).expect("ok");

        // Assert
        let expected = VerifyPrediction::Correct;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_verify_prediction_incorrect() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            0, 1, 2,
            0, 1, 2,
        ];
        let image0: Image = Image::try_create(3, 2, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0,
            1, 1, 1,
        ];
        let image1: Image = Image::try_create(3, 2, pixels1).expect("image");

        // Act
        let actual: VerifyPrediction = VerifyPrediction::create(&image0, &image1).expect("ok");

        // Assert
        let incorrect_data = VerifyPredictionIncorrectData {
            number_of_pixels_total: 6,
            number_of_pixels_with_correct_value: 2,
            number_of_pixels_with_incorrect_value: 4,
            percentage_correct: 33,
        };
        let expected = VerifyPrediction::Incorrect { incorrect_data };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_verify_prediction_wrong_size() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            0, 1, 2,
            0, 1, 2,
        ];
        let image0: Image = Image::try_create(3, 2, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0,
            1, 1,
            2, 2,
        ];
        let image1: Image = Image::try_create(2, 3, pixels1).expect("image");

        // Act
        let actual: VerifyPrediction = VerifyPrediction::create(&image0, &image1).expect("ok");

        // Assert
        let expected = VerifyPrediction::WrongSize;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_arcathon_solution_coordinator_prediction_all_correct() {
        // Arrange
        let task: arc_work_model::Task = task_with_testdata("6150a2bd").expect("ok");

        let pixels: Vec<u8> = vec![
            0, 0, 4,
            0, 8, 6,
            5, 3, 6,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        let image_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&input);

        let prediction = arcathon_solution_coordinator::Prediction {
            output_id: 0,
            output: image_grid,
            prediction_type: arcathon_solution_coordinator::PredictionType::SolveGenetic
        };

        // Act
        let actual: VerifyPrediction = prediction.verify_prediction(&task).expect("ok");

        // Assert
        let expected = VerifyPrediction::Correct;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_arcathon_solution_coordinator_prediction_all_incorrect() {
        // Arrange
        let task: arc_work_model::Task = task_with_testdata("6150a2bd").expect("ok");

        let pixels: Vec<u8> = vec![
            5, 5, 3,
            4, 5, 5,
            4, 5, 5,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        let image_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&input);

        let prediction = arcathon_solution_coordinator::Prediction {
            output_id: 0,
            output: image_grid,
            prediction_type: arcathon_solution_coordinator::PredictionType::SolveGenetic
        };

        // Act
        let actual: VerifyPrediction = prediction.verify_prediction(&task).expect("ok");

        // Assert
        let incorrect_data = VerifyPredictionIncorrectData {
            number_of_pixels_total: 9,
            number_of_pixels_with_correct_value: 0,
            number_of_pixels_with_incorrect_value: 9,
            percentage_correct: 0,
        };
        let expected = VerifyPrediction::Incorrect { incorrect_data };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_arcathon_solution_coordinator_prediction_some_incorrect() {
        // Arrange
        let task: arc_work_model::Task = task_with_testdata("6150a2bd").expect("ok");

        let pixels: Vec<u8> = vec![
            0, 0, 4,
            0, 8, 1,
            5, 1, 1,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        let image_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&input);

        let prediction = arcathon_solution_coordinator::Prediction {
            output_id: 0,
            output: image_grid,
            prediction_type: arcathon_solution_coordinator::PredictionType::SolveGenetic
        };

        // Act
        let actual: VerifyPrediction = prediction.verify_prediction(&task).expect("ok");

        // Assert
        let incorrect_data = VerifyPredictionIncorrectData {
            number_of_pixels_total: 9,
            number_of_pixels_with_correct_value: 6,
            number_of_pixels_with_incorrect_value: 3,
            percentage_correct: 66,
        };
        let expected = VerifyPrediction::Incorrect { incorrect_data };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20003_arcathon_solution_coordinator_prediction_wrong_size() {
        // Arrange
        let task: arc_work_model::Task = task_with_testdata("6150a2bd").expect("ok");

        let pixels: Vec<u8> = vec![
            0, 1, 2, 3,
            4, 5, 6, 7,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");

        let image_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&input);

        let prediction = arcathon_solution_coordinator::Prediction {
            output_id: 0,
            output: image_grid,
            prediction_type: arcathon_solution_coordinator::PredictionType::SolveGenetic
        };

        // Act
        let actual: VerifyPrediction = prediction.verify_prediction(&task).expect("ok");

        // Assert
        let expected = VerifyPrediction::WrongSize;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_arcathon_solution_json_prediction_all_correct() {
        // Arrange
        let task: arc_work_model::Task = task_with_testdata("6150a2bd").expect("ok");

        let pixels: Vec<u8> = vec![
            0, 0, 4,
            0, 8, 6,
            5, 3, 6,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        let image_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&input);

        let prediction = arcathon_solution_json::Prediction {
            prediction_id: 0,
            output: image_grid,
        };

        // Act
        let actual: VerifyPrediction = prediction.verify_prediction(&task).expect("ok");

        // Assert
        assert_eq!(actual, VerifyPrediction::Correct);
    }

    #[test]
    fn test_30001_arcathon_solution_json_prediction_some_incorrect() {
        // Arrange
        let task: arc_work_model::Task = task_with_testdata("6150a2bd").expect("ok");

        let pixels: Vec<u8> = vec![
            0, 0, 4,
            0, 8, 1,
            5, 1, 1,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        let image_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&input);

        let prediction = arcathon_solution_json::Prediction {
            prediction_id: 0,
            output: image_grid,
        };

        // Act
        let actual: VerifyPrediction = prediction.verify_prediction(&task).expect("ok");

        // Assert
        let incorrect_data = VerifyPredictionIncorrectData {
            number_of_pixels_total: 9,
            number_of_pixels_with_correct_value: 6,
            number_of_pixels_with_incorrect_value: 3,
            percentage_correct: 66,
        };
        let expected = VerifyPrediction::Incorrect { incorrect_data };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30002_arcathon_solution_json_prediction_all_incorrect() {
        // Arrange
        let task: arc_work_model::Task = task_with_testdata("3428a4f5").expect("ok");

        let input: Image = Image::color(5, 6, 9);

        let image_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&input);

        let prediction = arcathon_solution_json::Prediction {
            prediction_id: 1,
            output: image_grid,
        };

        // Act
        let actual: VerifyPrediction = prediction.verify_prediction(&task).expect("ok");

        // Assert
        let incorrect_data = VerifyPredictionIncorrectData {
            number_of_pixels_total: 30,
            number_of_pixels_with_correct_value: 0,
            number_of_pixels_with_incorrect_value: 30,
            percentage_correct: 0,
        };
        let expected = VerifyPrediction::Incorrect { incorrect_data };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30003_no_pair_with_index() {
        // Arrange
        let task: arc_work_model::Task = task_with_testdata("3428a4f5").expect("ok");

        let input: Image = Image::color(5, 6, 9);

        let image_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&input);

        let prediction = arcathon_solution_json::Prediction {
            prediction_id: 2,
            output: image_grid,
        };

        // Act
        let error = prediction.verify_prediction(&task).expect_err("is supposed to fail");
        let message = format!("{:?}", error);

        // Assert
        assert_eq!(message.contains("test_index corresponds to output_id 2"), true);
    }
}
