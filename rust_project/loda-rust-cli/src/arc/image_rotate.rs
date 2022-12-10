use super::Image;

pub trait ImageRotate {
    fn rotate_cw(&self) -> anyhow::Result<Image>;
    fn rotate(&self, direction: i8) -> anyhow::Result<Image>;
}

impl ImageRotate for Image {
    fn rotate_cw(&self) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;

        // Copy pixels with coordinates rotated
        let mut bitmap = Image::zero(self.height(), self.width());
        for y in 0..=y_max {
            for x in 0..=x_max {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                let set_y: i32 = y_max - y;
                match bitmap.set(set_y, x, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", set_y, x));
                    }
                }
            }
        }
        return Ok(bitmap);
    }

    fn rotate(&self, direction: i8) -> anyhow::Result<Image> {
        let count: u8 = (((direction % 4) + 4) % 4) as u8;
        if count == 0 {
            return Ok(self.clone());
        }
        let mut bitmap: Image = self.clone();
        for _ in 0..count {
            bitmap = bitmap.rotate_cw()?;
        }
        Ok(bitmap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_rotate_cw_big() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_cw().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            4, 1,
            5, 2,
            6, 3,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_rotate_cw_long() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4,
            3,
            2,
            1,
        ];
        let input: Image = Image::try_create(1, 4, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_cw().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 4,
        ];
        let expected: Image = Image::try_create(4, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_rotate_cw_square() {
        // Arrange
        let pixels: Vec<u8> = vec![
            8, 5, 0,
            8, 5, 3,
            0, 3, 2,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        // let actual: Image = input.rotate(1).expect("image");
        let actual: Image = input.rotate_cw().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 8, 8,
            3, 5, 5,
            2, 3, 0,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_rotate_cw_multiple_times() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 1, 0,
            2, 0, 1,
            0, 2, 0,
            0, 0, 2
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let bitmap0: Image = input.rotate_cw().expect("image");
        let bitmap1: Image = bitmap0.rotate_cw().expect("image");
        let bitmap2: Image = bitmap1.rotate_cw().expect("image");
        let bitmap3: Image = bitmap2.rotate_cw().expect("image");
        let actual: Image = bitmap3;

        // Assert
        let expected: Image = input.clone();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_rotate0() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 3,
            0, 2, 0,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.rotate(0).expect("image");

        // Assert
        let expected: Image = input.clone();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_rotate1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 3,
            0, 2, 0,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.rotate(1).expect("image");

        // Assert
        let expected: Image = input.rotate_cw().expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_rotate_minus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 3,
            0, 2, 0,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.rotate(-1).expect("image");

        // Assert
        let bitmap1: Image = input.rotate_cw().expect("image");
        let bitmap2: Image = bitmap1.rotate_cw().expect("image");
        let expected: Image = bitmap2.rotate_cw().expect("image");
        assert_eq!(actual, expected);
    }
}
