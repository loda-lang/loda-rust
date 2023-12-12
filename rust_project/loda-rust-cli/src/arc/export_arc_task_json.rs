//! Export task to a json file with the same format as the original ARC 1 dataset.
use super::{arc_json_model, Image};
use super::arc_json_model::GridFromImage;
use serde::Serialize;
use std::{path::Path, fs};

#[derive(Clone, Debug, Serialize)]
struct Pair {
    input: arc_json_model::Grid,
    output: arc_json_model::Grid,
}

#[derive(Clone, Debug, Serialize)]
struct Task {
    #[serde(rename = "train")]
    train_vec: Vec<Pair>,

    #[serde(rename = "test")]
    test_vec: Vec<Pair>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ExportARCTaskJson {
    task: Task,
}

impl ExportARCTaskJson {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            task: Task {
                train_vec: vec!(),
                test_vec: vec!(),
            }
        }
    }

    /// Append a `train` pair to the task.
    #[allow(dead_code)]
    pub fn push_train(&mut self, input: &Image, output: &Image) {
        let input_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(input);
        let output_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(output);
        let pair = Pair {
            input: input_grid,
            output: output_grid,
        };
        self.task.train_vec.push(pair);
    }

    /// Append a `test` pair to the task.
    #[allow(dead_code)]
    pub fn push_test(&mut self, input: &Image, output: &Image) {
        let input_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(input);
        let output_grid: arc_json_model::Grid = arc_json_model::Grid::from_image(output);
        let pair = Pair {
            input: input_grid,
            output: output_grid,
        };
        self.task.test_vec.push(pair);
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> anyhow::Result<String> {
        let json: String = serde_json::to_string(&self.task)?;
        Ok(json)
    }

    /// Save as a json file.
    /// 
    /// Returns the file size in bytes.
    #[allow(dead_code)]
    pub fn save_json_file(&self, path: &Path) -> anyhow::Result<usize> {
        let json: String = self.to_string()?;
        let bytes: usize = json.len();
        match fs::write(&path, json) {
            Ok(()) => {},
            Err(error) => {
                return Err(anyhow::anyhow!("Unable to save json file. path: {:?} error: {:?}", path, error));
            }
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use std::path::PathBuf;

    #[test]
    fn test_10000_typical() {
        // Arrange
        let input: Image = Image::try_create(2, 1, vec![1, 2]).expect("image");
        let output: Image = Image::try_create(1, 3, vec![3, 4, 5]).expect("image");

        // Act
        let mut export = ExportARCTaskJson::new();
        export.push_train(&input, &output);
        let json: String = export.to_string().expect("string");

        // Assert
        assert_eq!(json, r#"{"train":[{"input":[[1,2]],"output":[[3],[4],[5]]}],"test":[]}"#);
    }

    #[test]
    fn test_10001_no_pairs() {
        // Act
        let export = ExportARCTaskJson::new();
        let json: String = export.to_string().expect("string");

        // Assert
        assert_eq!(json, r#"{"train":[],"test":[]}"#);
    }

    #[test]
    fn test_10002_empty_images() {
        // Arrange
        let input: Image = Image::empty();
        let output: Image = Image::empty();

        // Act
        let mut export = ExportARCTaskJson::new();
        export.push_test(&input, &output);
        let json: String = export.to_string().expect("string");

        // Assert
        assert_eq!(json, r#"{"train":[],"test":[{"input":[],"output":[]}]}"#);
    }

    #[test]
    fn test_20000_save_json_file() {
        // Act
        let image0: Image = Image::color(1, 1, 0);
        let image1: Image = Image::color(1, 1, 1);
        let image2: Image = Image::color(1, 1, 2);
        let image3: Image = Image::color(1, 1, 3);
        let mut export = ExportARCTaskJson::new();
        export.push_train(&image0, &image1);
        export.push_test(&image2, &image3);

        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_20000_save_json_file");
        fs::create_dir(&basedir).expect("ok");
        let path: PathBuf = basedir.join("taskname.json");

        // Act
        let returned_bytes: usize = export.save_json_file(&path).expect("ok");

        // Assert
        let filesize: u64 = path.metadata().expect("ok").len();
        assert_eq!(returned_bytes as u64, filesize);
        assert_eq!(returned_bytes, 82);
        let json: String = fs::read_to_string(&path).expect("ok");
        assert_eq!(json, r#"{"train":[{"input":[[0]],"output":[[1]]}],"test":[{"input":[[2]],"output":[[3]]}]}"#);
    }

}
