use super::{Image, BitmapRotate};

pub trait BitmapRemoveDuplicates {
    fn remove_duplicate_rows(&self) -> anyhow::Result<Image>;
    fn remove_duplicate_columns(&self) -> anyhow::Result<Image>;
    fn remove_duplicates(&self) -> anyhow::Result<Image>;
}

impl BitmapRemoveDuplicates for Image {
    fn remove_duplicate_rows(&self) -> anyhow::Result<Image> {
        let len: usize = (self.width() as usize) * (self.height() as usize);
        if len == 0 {
            return Ok(Image::empty());
        }
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;

        // Collect the y-indexes of rows that are unique
        let mut keep_indexes: Vec<i32> = vec![0];
        for y in 1..=y_max {
            for x in 0..=x_max {
                let pixel_value_prev: u8 = self.get(x, y-1).unwrap_or(255);
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                if pixel_value != pixel_value_prev {
                    keep_indexes.push(y);
                    break;
                }
            }
        }

        // Height of the new bitmap
        let height_new_usize: usize = keep_indexes.len();
        if height_new_usize > (u8::MAX as usize) {
            return Err(anyhow::anyhow!("Integrity error. Found more rows than 256"));
        }
        let height_new: u8 = height_new_usize as u8;

        // Copy pixels of the rows to keep
        let mut bitmap = Image::zeroes(self.width(), height_new);
        let mut current_y: i32 = -1;
        for y in keep_indexes {
            current_y += 1;
            for x in 0..=x_max {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                let set_x: i32 = x;
                let set_y: i32 = current_y;
                match bitmap.set(set_x, set_y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", set_x, set_y));
                    }
                }
            }
        }

        return Ok(bitmap);
    }

    fn remove_duplicate_columns(&self) -> anyhow::Result<Image> {
        let bitmap0: Image = self.rotate(1)?;
        let bitmap1: Image = bitmap0.remove_duplicate_rows()?;
        let bitmap2: Image = bitmap1.rotate(-1)?;
        Ok(bitmap2)
    }

    fn remove_duplicates(&self) -> anyhow::Result<Image> {
        let bitmap0: Image = self.remove_duplicate_rows()?;
        let bitmap1: Image = bitmap0.remove_duplicate_columns()?;
        Ok(bitmap1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_remove_duplicate_rows_big1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 0, 0,
            1, 1, 1,
            0, 0, 1,
            0, 0, 1,
            0, 0, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 7, pixels).expect("bitmap");

        // Act
        let actual: Image = input.remove_duplicate_rows().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0,
            1, 1, 1,
            0, 0, 1,
            0, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 4, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_remove_duplicate_columns_big2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            0, 1, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
        ];
        let input: Image = Image::try_create(5, 6, pixels).expect("bitmap");

        // Act
        let actual: Image = input.remove_duplicate_rows().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 0, 1, 0,
        ];
        let expected: Image = Image::try_create(5, 3, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_remove_duplicate_rows_identical_big() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            1, 1, 1,
            0, 0, 1,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.remove_duplicate_rows().expect("bitmap");

        // Assert
        let expected: Image = input.clone();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_remove_duplicate_rows_identical_1px() {
        // Arrange
        let input: Image = Image::try_create(1, 1, vec![42]).expect("bitmap");

        // Act
        let actual: Image = input.remove_duplicate_rows().expect("bitmap");

        // Assert
        let expected: Image = input.clone();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_remove_duplicate_columns_big() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0, 0,
            0, 0, 1, 1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(7, 3, pixels).expect("bitmap");

        // Act
        let actual: Image = input.remove_duplicate_columns().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0,
            0, 1, 0, 0,
            0, 1, 1, 0,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_remove_duplicates() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0, 0,
            0, 0, 1, 1, 1, 0, 0,
            0, 0, 1, 1, 1, 0, 0,
            0, 0, 1, 0, 0, 1, 0,
            0, 0, 1, 0, 0, 1, 0,
        ];
        let input: Image = Image::try_create(7, 6, pixels).expect("bitmap");

        // Act
        let actual: Image = input.remove_duplicates().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 0, 1, 0,
        ];
        let expected: Image = Image::try_create(5, 3, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

}
