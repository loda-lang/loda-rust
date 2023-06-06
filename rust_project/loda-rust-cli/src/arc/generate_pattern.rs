use super::{Image, ImageRotate, ImageSize, ImageRepeat};

#[allow(dead_code)]
pub enum GeneratePattern {
    /// Draw alternating columns with two colors.
    /// 
    /// The left-most-column is always color0.
    StripedColumns { count0: u8, color0: u8, count1: u8, color1: u8 },

    /// Draw alternating rows with two colors.
    /// 
    /// The left-most-row is always color0.
    StripedRows { count0: u8, color0: u8, count1: u8, color1: u8 },
}

impl GeneratePattern {
    /// Draws the pattern.
    /// 
    /// Returns an image with the given size.
    #[allow(dead_code)]
    pub fn draw(&self, size: ImageSize) -> anyhow::Result<Image> {
        match self {
            GeneratePattern::StripedColumns { count0, color0, count1, color1 } => {
                let image: Image = Self::striped_columns(*count0, *color0, *count1, *color1, size)?;
                return Ok(image);
            },
            GeneratePattern::StripedRows { count0, color0, count1, color1 } => {
                let size_rotated = size.rotate();
                let image_rotated: Image = Self::striped_columns(*count0, *color0, *count1, *color1, size_rotated)?;
                let image: Image = image_rotated.rotate_cw()?;
                return Ok(image);
            },
        }
    }

    /// Draw alternating columns with two colors.
    /// 
    /// Returns an image with the given size.
    fn striped_columns(count0: u8, color0: u8, count1: u8, color1: u8, size: ImageSize) -> anyhow::Result<Image> {
        if count0 == 0 || count1 == 0 {
            return Err(anyhow::anyhow!("count0 and count1 must be non-zero"));
        }
        if size.is_empty() {
            return Err(anyhow::anyhow!("size must be non-empty"));
        }
        let count_sum: u16 = count0 as u16 + count1 as u16;
        let mut result_image = Image::zero(size.width, 1);
        for x in 0..size.width as u16 {
            let color = if x % count_sum < (count0 as u16) { color0 } else { color1 };
            result_image.set(x as i32, 0, color);
        }
        result_image = result_image.repeat_by_count(1, size.height)?;
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_striped_columns() {
        // Arrange
        let pattern = GeneratePattern::StripedColumns { count0: 1, color0: 5, count1: 1, color1: 3 };

        // Act
        let actual: Image = pattern.draw(ImageSize::new(5, 2)).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 3, 5, 3, 5,
            5, 3, 5, 3, 5,
        ];
        let expected: Image = Image::try_create(5, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_striped_columns() {
        // Arrange
        let pattern = GeneratePattern::StripedColumns { count0: 2, color0: 5, count1: 1, color1: 3 };

        // Act
        let actual: Image = pattern.draw(ImageSize::new(7, 2)).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 3, 5, 5, 3, 5,
            5, 5, 3, 5, 5, 3, 5,
        ];
        let expected: Image = Image::try_create(7, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_striped_rows() {
        // Arrange
        let pattern = GeneratePattern::StripedRows { count0: 2, color0: 0, count1: 3, color1: 1 };

        // Act
        let actual: Image = pattern.draw(ImageSize::new(2, 8)).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0,
            0, 0,
            1, 1,
            1, 1,
            1, 1,
            0, 0,
            0, 0,
            1, 1,
        ];
        let expected: Image = Image::try_create(2, 8, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

}
