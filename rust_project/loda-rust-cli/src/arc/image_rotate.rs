use super::Image;

pub trait ImageRotate {
    /// Rotate clockwise (CW)
    fn rotate_cw(&self) -> anyhow::Result<Image>;

    /// Rotate counter clockwise (CCW)
    fn rotate_ccw(&self) -> anyhow::Result<Image>;

    /// Rotate by `N * 90` degrees in any direction
    /// 
    /// Rotate 180 degrees is the same as the `flip_xy` operation.
    fn rotate(&self, direction: i8) -> anyhow::Result<Image>;
}

impl ImageRotate for Image {
    fn rotate_cw(&self) -> anyhow::Result<Image> {
        if self.width() <= 1 && self.height() <= 1 {
            // No point in rotating an empty image or a 1x1 image.
            return Ok(self.clone());
        }
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;

        // Copy pixels with coordinates rotated
        let mut image = Image::zero(self.height(), self.width());
        for y in 0..=y_max {
            for x in 0..=x_max {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                let set_y: i32 = y_max - y;
                match image.set(set_y, x, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result image", set_y, x));
                    }
                }
            }
        }
        Ok(image)
    }

    fn rotate_ccw(&self) -> anyhow::Result<Image> {
        self.rotate(-1)
    }

    fn rotate(&self, direction: i8) -> anyhow::Result<Image> {
        let count: u8 = (((direction % 4) + 4) % 4) as u8;
        if count == 0 {
            return Ok(self.clone());
        }
        let mut image: Image = self.clone();
        for _ in 0..count {
            image = image.rotate_cw()?;
        }
        Ok(image)
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
    fn test_10004_rotate_cw_single_pixel() {
        // Arrange
        let input: Image = Image::try_create(1, 1, vec![42]).expect("image");

        // Act
        let actual: Image = input.rotate_cw().expect("image");

        // Assert
        let expected: Image = Image::try_create(1, 1, vec![42]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10005_rotate_cw_empty() {
        // Arrange
        let input: Image = Image::empty();

        // Act
        let actual: Image = input.rotate_cw().expect("image");

        // Assert
        let expected: Image = Image::empty();
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

    #[test]
    fn test_30000_rotate_ccw() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            2, 2, 2, 3,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 3,
            1, 2,
            1, 2,
            1, 2,
        ];
        let expected: Image = Image::try_create(2, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
