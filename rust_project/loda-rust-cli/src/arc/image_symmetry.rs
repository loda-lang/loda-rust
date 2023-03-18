use super::{Image, ImageRotate};

pub trait ImageSymmetry {
    /// Reverse the `x-axis`.
    fn flip_x(&self) -> anyhow::Result<Image>;

    /// Reverse the `y-axis`.
    fn flip_y(&self) -> anyhow::Result<Image>;
    
    /// Reverse both the `x-axis` and the `y-axis`.
    fn flip_xy(&self) -> anyhow::Result<Image>;

    /// Detect symmetry along the `x-axis`.
    fn is_symmetric_x(&self) -> anyhow::Result<bool>;

    /// Detect symmetry along the `y-axis`.
    fn is_symmetric_y(&self) -> anyhow::Result<bool>;
}

impl ImageSymmetry for Image {
    fn flip_x(&self) -> anyhow::Result<Image> {
        if self.width() <= 1 {
            // No point in flipping an empty image or a 1xN image.
            return Ok(self.clone());
        }
        
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;

        // Copy pixels, with x axis flipped
        let mut bitmap = Image::zero(self.width(), self.height());
        for y in 0..=y_max {
            for x in 0..=x_max {
                let pixel_value: u8 = self.get(x_max - x, y).unwrap_or(255);
                match bitmap.set(x, y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", x, y));
                    }
                }
            }
        }
        return Ok(bitmap);
    }

    fn flip_y(&self) -> anyhow::Result<Image> {
        let bitmap0: Image = self.rotate(1)?;
        let bitmap1: Image = bitmap0.flip_x()?;
        let bitmap2: Image = bitmap1.rotate(-1)?;
        Ok(bitmap2)
    }

    fn flip_xy(&self) -> anyhow::Result<Image> {
        let bitmap0: Image = self.flip_x()?;
        let bitmap1: Image = bitmap0.flip_y()?;
        Ok(bitmap1)
    }

    fn is_symmetric_x(&self) -> anyhow::Result<bool> {
        let image: Image = self.flip_x()?;
        let is_symmetric: bool = image == *self;
        Ok(is_symmetric)
    }

    fn is_symmetric_y(&self) -> anyhow::Result<bool> {
        self.rotate(1)?.is_symmetric_x()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_flip_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            3, 2, 1,
            6, 5, 4,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.flip_x().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_flip_x_oneline() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,
            2,
            3,
        ];
        let input: Image = Image::try_create(1, 3, pixels).expect("image");

        // Act
        let actual: Image = input.flip_x().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1,
            2,
            3,
        ];
        let expected: Image = Image::try_create(1, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_flip_x_empty() {
        let actual: Image = Image::empty().flip_x().expect("image");
        assert_eq!(actual, Image::empty());
    }

    #[test]
    fn test_20000_flip_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 6,
            3, 4,
            1, 2,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: Image = input.flip_y().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 
            3, 4,
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
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
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: Image = input.flip_xy().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_is_symmetric_x_no() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let is_symmetric: bool = input.is_symmetric_x().expect("bool");

        // Assert
        assert_eq!(is_symmetric, false);
    }

    #[test]
    fn test_40001_is_symmetric_x_yes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 1,
            3, 4, 3,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let is_symmetric: bool = input.is_symmetric_x().expect("bool");

        // Assert
        assert_eq!(is_symmetric, true);
    }

    #[test]
    fn test_40002_is_symmetric_x_yes() {
        let is_symmetric: bool = Image::empty().is_symmetric_x().expect("bool");
        assert_eq!(is_symmetric, true);
    }

    #[test]
    fn test_40003_is_symmetric_x_yes() {
        let is_symmetric: bool = Image::color(4, 3, 1).is_symmetric_x().expect("bool");
        assert_eq!(is_symmetric, true);
    }

    #[test]
    fn test_50000_is_symmetric_y_no() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 
            3, 4, 
            5, 6,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let is_symmetric: bool = input.is_symmetric_y().expect("bool");

        // Assert
        assert_eq!(is_symmetric, false);
    }

    #[test]
    fn test_50001_is_symmetric_y_yes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 3,
            2, 4,
            1, 3,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let is_symmetric: bool = input.is_symmetric_y().expect("bool");

        // Assert
        assert_eq!(is_symmetric, true);
    }

    #[test]
    fn test_50002_is_symmetric_y_yes() {
        let is_symmetric: bool = Image::empty().is_symmetric_y().expect("bool");
        assert_eq!(is_symmetric, true);
    }

    #[test]
    fn test_50003_is_symmetric_y_yes() {
        let is_symmetric: bool = Image::color(4, 3, 1).is_symmetric_y().expect("bool");
        assert_eq!(is_symmetric, true);
    }
}
