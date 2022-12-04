use super::{Image, ImageTryCreate};
use super::read_testdata;
use std::fs;
use std::path::Path;
use serde::Deserialize;

#[allow(dead_code)]
pub trait GridToImage {
    fn to_image(&self) -> anyhow::Result<Image>;
}

#[allow(dead_code)]
pub type Grid = Vec<Vec<u8>>;

impl GridToImage for Grid {
    fn to_image(&self) -> anyhow::Result<Image> {
        // Extract height
        let height_usize: usize = self.len();
        if height_usize == 0 {
            return Ok(Image::empty());
        }
        if height_usize > (u8::MAX as usize) {
            return Err(anyhow::anyhow!("Too many rows in input data. Max 256 is possible"));
        }
        let height: u8 = height_usize as u8;

        // Extract width
        let width_usize: usize = self[0].len(); // At this point we know there is 1 or more rows
        if width_usize > (u8::MAX as usize) {
            return Err(anyhow::anyhow!("Too many columns in input data. Max 256 is possible"));
        }
        let width: u8 = width_usize as u8;

        // Extract pixels
        let mut pixels = Vec::<u8>::new();
        for row in self {
            if row.len() != width_usize {
                return Err(anyhow::anyhow!("Expected all rows to have same length"));
            }
            for pixel in row {
                pixels.push(*pixel);
            }
        }

        let instance = Image::try_create(width, height, pixels)?;
        Ok(instance)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TaskPair {
    input: Grid,
    output: Grid,
}

impl TaskPair {
    pub fn input(&self) -> &Grid {
        &self.input
    }

    pub fn output(&self) -> &Grid {
        &self.output
    }
}

#[allow(dead_code)]
pub struct ImagePair {
    pub input: Image,
    pub output: Image,
}

#[derive(Debug, PartialEq)]
enum ModelImagePairMode {
    All,
    Train,
    Test,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Model {
    train: Vec<TaskPair>,
    test: Vec<TaskPair>,
}

impl Model {
    #[allow(dead_code)]
    pub fn train(&self) -> &Vec<TaskPair> {
        &self.train
    }

    #[allow(dead_code)]
    pub fn test(&self) -> &Vec<TaskPair> {
        &self.test
    }

    #[allow(dead_code)]
    pub fn images_all(&self) -> anyhow::Result<Vec<ImagePair>> {
        self.images_with_mode(ModelImagePairMode::All)
    }

    #[allow(dead_code)]
    pub fn images_train(&self) -> anyhow::Result<Vec<ImagePair>> {
        self.images_with_mode(ModelImagePairMode::Train)
    }

    #[allow(dead_code)]
    pub fn images_test(&self) -> anyhow::Result<Vec<ImagePair>> {
        self.images_with_mode(ModelImagePairMode::Test)
    }

    fn images_with_mode(&self, mode: ModelImagePairMode) -> anyhow::Result<Vec<ImagePair>> {
        let mut task_pairs: Vec<&TaskPair> = vec!();
        if mode == ModelImagePairMode::All || mode == ModelImagePairMode::Train {
            let mut v: Vec<&TaskPair> = self.train.iter().map(|r|r).collect();
            task_pairs.append(&mut v);
        }
        if mode == ModelImagePairMode::All || mode == ModelImagePairMode::Test {
            let mut v: Vec<&TaskPair> = self.test.iter().map(|r|r).collect();
            task_pairs.append(&mut v);
        }
        let mut image_pairs: Vec<ImagePair> = vec!();
        for task in task_pairs {
            let input: Image = task.input().to_image()?;
            let output: Image = task.output().to_image()?;
            image_pairs.push(ImagePair { input, output });
        }
        Ok(image_pairs)
    }

    #[allow(dead_code)]
    pub fn load_testdata(name: &str) -> anyhow::Result<Model> {
        let json: String = read_testdata(name)?;
        let model: Model = serde_json::from_str(&json)?;
        Ok(model)
    }
    
    #[allow(dead_code)]
    pub fn load(name: &str, arc_repository_data_training: &Path) -> anyhow::Result<Model> {
        let filename_json = format!("{}.json", name);
        let path = arc_repository_data_training.join(filename_json);
        Self::load_with_json_file(&path)
    }

    pub fn load_with_json_file(json_file: &Path) -> anyhow::Result<Model> {
        let json: String = match fs::read_to_string(json_file) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("cannot load file, error: {:?} path: {:?}", error, json_file));
            }
        };
        let model: Model = serde_json::from_str(&json)?;
        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_10000_json_to_grid() -> anyhow::Result<()> {
        let json_string = "[[1,2,3],[4,5,6]]";
        let grid: Grid = serde_json::from_str(&json_string)?;
        assert_eq!(grid.len(), 2);
        assert_eq!(grid[0], vec![1,2,3]);
        assert_eq!(grid[1], vec![4,5,6]);
        Ok(())
    }

    #[test]
    fn test_20000_grid_to_image() -> anyhow::Result<()> {
        // Arrange
        let json_string = "[[1,2,3],[4,5,6]]";
        let grid: Grid = serde_json::from_str(&json_string)?;

        // Act
        let bm: Image = grid.to_image().expect("image");

        // Assert
        assert_eq!(bm.width(), 3);
        assert_eq!(bm.height(), 2);
        assert_eq!(bm.get(0, 0), Some(1));
        assert_eq!(bm.get(1, 0), Some(2));
        assert_eq!(bm.get(2, 0), Some(3));
        assert_eq!(bm.get(0, 1), Some(4));
        assert_eq!(bm.get(1, 1), Some(5));
        assert_eq!(bm.get(2, 1), Some(6));
        Ok(())
    }

    #[test]
    fn test_30000_model_loda_testdata() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("6150a2bd")?;
        assert_eq!(model.train.len(), 2);
        assert_eq!(model.test.len(), 1);
        Ok(())
    }

    #[test]
    fn test_30001_model_load() -> anyhow::Result<()> {
        // Arrange
        let json: String = read_testdata("4258a5f9")?;
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_40000_model_load");
        fs::create_dir(&basedir)?;
        let path: PathBuf = basedir.join("hello.json");
        fs::write(&path, &json)?;

        // Act
        let model: Model = Model::load("hello", &basedir)?;

        // Assert
        assert_eq!(model.train.len(), 2);
        assert_eq!(model.test.len(), 1);
        Ok(())
    }
}
