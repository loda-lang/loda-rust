use super::Image;

pub trait ImageSkew {
    /// Skew the image in the horizontal direction. The image becomes wider.
    /// 
    /// Insets each row by the `y index`.
    fn skew_x(&self, fill_color: u8) -> anyhow::Result<Image>;
}

impl ImageSkew for Image {
    fn skew_x(&self, fill_color: u8) -> anyhow::Result<Image> {
        if self.width() <= 1 && self.height() <= 1 {
            // No point in processing an empty image or a 1x1 image.
            return Ok(self.clone());
        }
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;

        let combined_u16: u16 = self.width() as u16 + self.height() as u16 - 1;
        if combined_u16 > 255 {
            return Err(anyhow::anyhow!("Unable to skew image. The combined width and height is too large: {}", combined_u16));
        }

        // Copy rows with the x skewed by the y index
        let mut image = Image::color(combined_u16 as u8, self.height(), fill_color);
        for y in 0..=y_max {
            for x in 0..=x_max {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                let set_x: i32 = x + y;
                match image.set(set_x, y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result image", set_x, y));
                    }
                }
            }
        }
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_skew_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,
            1,
            1,
        ];
        let input: Image = Image::try_create(1, 3, pixels).expect("image");

        // Act
        let actual: Image = input.skew_x(9).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 9, 9,
            9, 1, 9,
            9, 9, 1,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_skew_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            1, 2,
            1, 2,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: Image = input.skew_x(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 0, 0,
            0, 1, 2, 0,
            0, 0, 1, 2,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_skew_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 2,
            0, 1, 2, 0,
            1, 2, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: Image = input.skew_x(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 2, 0, 0,
            0, 0, 1, 2, 0, 0,
            0, 0, 1, 2, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
