use super::{Image, ImageTryCreate};
use super::read_testdata;
use std::fs;
use std::path::Path;
use serde::Deserialize;

pub trait GridToImage {
    fn to_image(&self) -> anyhow::Result<Image>;
}

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

#[derive(Deserialize, Debug)]
pub struct Model {
    train: Vec<TaskPair>,
    test: Vec<TaskPair>,
}

impl Model {
    pub fn train(&self) -> &Vec<TaskPair> {
        &self.train
    }

    pub fn test(&self) -> &Vec<TaskPair> {
        &self.test
    }

    pub fn load_testdata(name: &str) -> anyhow::Result<Model> {
        let json: String = read_testdata(name)?;
        let model: Model = serde_json::from_str(&json)?;
        Ok(model)
    }

    pub fn load(name: &str, arc_repository_data_training: &Path) -> anyhow::Result<Model> {
        let filename_json = format!("{}.json", name);
        let path = arc_repository_data_training.join(filename_json);
        let json: String = match fs::read_to_string(&path) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("cannot load file, error: {:?} path: {:?}", error, path));
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
    fn test_20000_grid_to_bitmap() -> anyhow::Result<()> {
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
