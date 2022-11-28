use super::Bitmap;

pub trait BitmapRotate {
    fn rotate_cw(&self) -> anyhow::Result<Bitmap>;
    fn rotate(&self, direction: i8) -> anyhow::Result<Bitmap>;
}

impl BitmapRotate for Bitmap {
    fn rotate_cw(&self) -> anyhow::Result<Bitmap> {
        let len: usize = (self.width() as usize) * (self.height() as usize);
        if len == 0 {
            return Ok(Bitmap::empty());
        }
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;

        // Copy pixels, with x y swapped
        let mut bitmap = Bitmap::zeroes(self.height(), self.width());
        for y in 0..=y_max {
            for x in 0..=x_max {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                match bitmap.set(y, x, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", y, x));
                    }
                }
            }
        }
        return Ok(bitmap);
    }

    fn rotate(&self, direction: i8) -> anyhow::Result<Bitmap> {
        let count: u8 = (((direction % 4) + 4) % 4) as u8;
        if count == 0 {
            return Ok(self.clone());
        }
        let mut bitmap: Bitmap = self.clone();
        for _ in 0..count {
            bitmap = bitmap.rotate_cw()?;
        }
        Ok(bitmap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::BitmapTryCreate;

    #[test]
    fn test_10000_rotate_cw_big() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Bitmap = Bitmap::try_create(3, 2, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.rotate_cw().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 4,
            2, 5,
            3, 6,
        ];
        let expected: Bitmap = Bitmap::try_create(2, 3, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_rotate_cw_long() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,
            2,
            3,
            4,
        ];
        let input: Bitmap = Bitmap::try_create(1, 4, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.rotate_cw().expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 4,
        ];
        let expected: Bitmap = Bitmap::try_create(4, 1, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_rotate_cw_multiple_times() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 1, 0,
            2, 0, 1,
            0, 2, 0,
            0, 0, 2
        ];
        let input: Bitmap = Bitmap::try_create(3, 5, pixels).expect("bitmap");

        // Act
        let bitmap0: Bitmap = input.rotate_cw().expect("bitmap");
        let bitmap1: Bitmap = bitmap0.rotate_cw().expect("bitmap");
        let bitmap2: Bitmap = bitmap1.rotate_cw().expect("bitmap");
        let bitmap3: Bitmap = bitmap2.rotate_cw().expect("bitmap");
        let actual: Bitmap = bitmap3;

        // Assert
        let expected: Bitmap = input.clone();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_rotate0() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 3,
            0, 2, 0,
        ];
        let input: Bitmap = Bitmap::try_create(3, 2, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.rotate(0).expect("bitmap");

        // Assert
        let expected: Bitmap = input.clone();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_rotate1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 3,
            0, 2, 0,
        ];
        let input: Bitmap = Bitmap::try_create(3, 2, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.rotate(1).expect("bitmap");

        // Assert
        let expected: Bitmap = input.rotate_cw().expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_rotate_minus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 3,
            0, 2, 0,
        ];
        let input: Bitmap = Bitmap::try_create(3, 2, pixels).expect("bitmap");

        // Act
        let actual: Bitmap = input.rotate(-1).expect("bitmap");

        // Assert
        let bitmap1: Bitmap = input.rotate_cw().expect("bitmap");
        let bitmap2: Bitmap = bitmap1.rotate_cw().expect("bitmap");
        let expected: Bitmap = bitmap2.rotate_cw().expect("bitmap");
        assert_eq!(actual, expected);
    }
}
