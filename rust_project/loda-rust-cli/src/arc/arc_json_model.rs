use crate::arc::{Bitmap, BitmapTryCreate, convolution3x3, Padding};
use serde::Deserialize;

pub trait GridToBitmap {
    fn to_bitmap(&self) -> anyhow::Result<Bitmap>;
}

pub type Grid = Vec<Vec<u8>>;

impl GridToBitmap for Grid {
    fn to_bitmap(&self) -> anyhow::Result<Bitmap> {
        // Extract height
        let height_usize: usize = self.len();
        if height_usize == 0 {
            return Ok(Bitmap::empty());
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

        let instance = Bitmap::try_create(width, height, pixels)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;
    use std::fs;
    use crate::arc::read_testdata;

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
    fn test_20000_json_to_model() -> anyhow::Result<()> {
        // Arrange
        let json: String = read_testdata("6150a2bd")?;

        // Act
        let model: Model = serde_json::from_str(&json)?;

        // Assert
        assert_eq!(model.train.len(), 2);
        assert_eq!(model.test.len(), 1);
        Ok(())
    }

    #[test]
    fn test_30000_grid_to_bitmap() -> anyhow::Result<()> {
        // Arrange
        let json_string = "[[1,2,3],[4,5,6]]";
        let grid: Grid = serde_json::from_str(&json_string)?;

        // Act
        let bm: Bitmap = grid.to_bitmap().expect("bitmap");

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

    // #[test]
    fn test_40000_parse_real_data() -> anyhow::Result<()> {
        let config = Config::load();
        let arc_repository_data_training: PathBuf = config.arc_repository_data_training();
        let path = arc_repository_data_training.join("4258a5f9.json");
        let json_string: String = match fs::read_to_string(&path) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("cannot load file, error: {:?} path: {:?}", error, path));
            }
        };
        let model: Model = serde_json::from_str(&json_string)?;
        assert_eq!(model.train.len(), 2);
        assert_eq!(model.test.len(), 1);

        let input: Bitmap = model.train[0].input.to_bitmap().expect("bitmap");
        let output: Bitmap = model.train[0].output.to_bitmap().expect("bitmap");

        let input_padded: Bitmap = input.zero_padding(1).expect("bitmap");

        let result_bm: Bitmap = convolution3x3(&input_padded, |bm| {
            let mut found = false;
            for y in 0..3i32 {
                for x in 0..3i32 {
                    if x == 1 && y == 1 {
                        continue;
                    }
                    let pixel_value: u8 = bm.get(x, y).unwrap_or(255);
                    if pixel_value == 5 {
                        found = true;
                    }
                }
            }
            let mut value: u8 = bm.get(1, 1).unwrap_or(255);
            if found {
                value = 1;
            }
            Ok(value)
        }).expect("bitmap");

        assert_eq!(result_bm, output);

        Ok(())
    }
}
