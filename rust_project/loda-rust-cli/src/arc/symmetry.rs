use super::{Bitmap, BitmapRotate};

pub trait BitmapSymmetry {
    fn flip_x(&self) -> anyhow::Result<Bitmap>;
    fn flip_y(&self) -> anyhow::Result<Bitmap>;
    fn flip_xy(&self) -> anyhow::Result<Bitmap>;
}

impl BitmapSymmetry for Bitmap {
    fn flip_x(&self) -> anyhow::Result<Bitmap> {
        let len: usize = (self.width() as usize) * (self.height() as usize);
        if len == 0 {
            return Ok(Bitmap::empty());
        }
        
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;

        // Copy pixels, with x axis flipped
        let mut bitmap = Bitmap::zeroes(self.width(), self.height());
        for y in 0..=y_max {
            for x in 0..=x_max {
                let pixel_value: u8 = self.get(x_max - x, y).unwrap_or(255);
                match bitmap.set(x, y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", y, x));
                    }
                }
            }
        }
        return Ok(bitmap);
    }

    fn flip_y(&self) -> anyhow::Result<Bitmap> {
        let bitmap0: Bitmap = self.rotate(1)?;
        let bitmap1: Bitmap = bitmap0.flip_x()?;
        let bitmap2: Bitmap = bitmap1.rotate(-1)?;
        Ok(bitmap2)
    }

    fn flip_xy(&self) -> anyhow::Result<Bitmap> {
        let bitmap0: Bitmap = self.flip_x()?;
        let bitmap1: Bitmap = bitmap0.flip_y()?;
        Ok(bitmap1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::BitmapTryCreate;

    #[test]
    fn test_10000_flip_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            3, 2, 1,
            6, 5, 4,
        ];
        let input: Bitmap = Bitmap::try_create(3, 2, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.flip_x().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let expected: Bitmap = Bitmap::try_create(3, 2, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_flip_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 6,
            3, 4,
            1, 2,
        ];
        let input: Bitmap = Bitmap::try_create(2, 3, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.flip_y().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 
            3, 4,
            5, 6,
        ];
        let expected: Bitmap = Bitmap::try_create(2, 3, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_flip_xy() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 8, 7,
            6, 5, 4,
            3, 2, 1,
        ];
        let input: Bitmap = Bitmap::try_create(3, 3, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.flip_xy().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
        ];
        let expected: Bitmap = Bitmap::try_create(3, 3, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }
}
