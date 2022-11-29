use super::{Bitmap, BitmapRotate};

pub trait BitmapRemoveDuplicates {
    fn remove_duplicate_rows(&self) -> anyhow::Result<Bitmap>;
    fn remove_duplicate_columns(&self) -> anyhow::Result<Bitmap>;
    fn remove_duplicates(&self) -> anyhow::Result<Bitmap>;
}

impl BitmapRemoveDuplicates for Bitmap {
    fn remove_duplicate_rows(&self) -> anyhow::Result<Bitmap> {
        let len: usize = (self.width() as usize) * (self.height() as usize);
        if len == 0 {
            return Ok(Bitmap::empty());
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
        let mut bitmap = Bitmap::zeroes(self.width(), height_new);
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

    fn remove_duplicate_columns(&self) -> anyhow::Result<Bitmap> {
        let bitmap0: Bitmap = self.rotate(1)?;
        let bitmap1: Bitmap = bitmap0.remove_duplicate_rows()?;
        let bitmap2: Bitmap = bitmap1.rotate(-1)?;
        Ok(bitmap2)
    }

    fn remove_duplicates(&self) -> anyhow::Result<Bitmap> {
        let bitmap0: Bitmap = self.remove_duplicate_rows()?;
        let bitmap1: Bitmap = bitmap0.remove_duplicate_columns()?;
        Ok(bitmap1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::BitmapTryCreate;

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
        let input: Bitmap = Bitmap::try_create(3, 7, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.remove_duplicate_rows().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0,
            1, 1, 1,
            0, 0, 1,
            0, 0, 0,
        ];
        let expected: Bitmap = Bitmap::try_create(3, 4, expected_pixels).expect("bitmap");
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
        let input: Bitmap = Bitmap::try_create(5, 6, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.remove_duplicate_rows().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 0, 1, 0,
        ];
        let expected: Bitmap = Bitmap::try_create(5, 3, expected_pixels).expect("bitmap");
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
        let input: Bitmap = Bitmap::try_create(3, 4, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.remove_duplicate_rows().expect("bitmap");

        // Assert
        let expected: Bitmap = input.clone();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_remove_duplicate_rows_identical_1px() {
        // Arrange
        let input: Bitmap = Bitmap::try_create(1, 1, vec![42]).expect("bitmap");

        // Act
        let actual: Bitmap = input.remove_duplicate_rows().expect("bitmap");

        // Assert
        let expected: Bitmap = input.clone();
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
        let input: Bitmap = Bitmap::try_create(7, 3, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.remove_duplicate_columns().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0,
            0, 1, 0, 0,
            0, 1, 1, 0,
        ];
        let expected: Bitmap = Bitmap::try_create(4, 3, expected_pixels).expect("bitmap");
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
        let input: Bitmap = Bitmap::try_create(7, 6, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.remove_duplicates().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 0, 1, 0,
        ];
        let expected: Bitmap = Bitmap::try_create(5, 3, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

}
