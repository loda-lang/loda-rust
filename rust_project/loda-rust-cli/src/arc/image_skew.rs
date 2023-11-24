//! Mimic rotation by 45 degrees by skewing the image.
use super::{Image, ImageRotate90};

pub trait ImageSkew {
    /// Skew the image in the horizontal direction by 45 degrees.
    /// 
    /// The image becomes wider. The image `height` is preserved.
    /// 
    /// When `reverse=false` then insets each row by the `y index`.
    /// 
    /// When `reverse=true` then insets each row by the `max_y - y index`.
    /// 
    /// The area outside the original image is filled with the `fill_color`.
    fn skew_x(&self, fill_color: u8, reverse: bool) -> anyhow::Result<Image>;

    /// Skew the image in the vertical direction by 45 degrees.
    /// 
    /// The image becomes taller. The image `width` is preserved.
    /// 
    /// When `reverse=false` then insets each column by the `x index`.
    /// 
    /// When `reverse=true` then insets each column by the `max_x - x index`.
    /// 
    /// The area outside the original image is filled with the `fill_color`.
    fn skew_y(&self, fill_color: u8, reverse: bool) -> anyhow::Result<Image>;
}

impl ImageSkew for Image {
    fn skew_x(&self, fill_color: u8, reverse: bool) -> anyhow::Result<Image> {
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
                let set_x: i32 = if reverse { x + y_max - y } else { x + y };
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

    fn skew_y(&self, fill_color: u8, reverse: bool) -> anyhow::Result<Image> {
        let image: Image = self.rotate_cw()?;
        let image: Image = image.skew_x(fill_color, !reverse)?;
        let image: Image = image.rotate_ccw()?;
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_skew_x_reversefalse() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,
            1,
            1,
        ];
        let input: Image = Image::try_create(1, 3, pixels).expect("image");

        // Act
        let actual: Image = input.skew_x(9, false).expect("image");

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
    fn test_10001_skew_x_reversetrue() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,
            1,
            1,
        ];
        let input: Image = Image::try_create(1, 3, pixels).expect("image");

        // Act
        let actual: Image = input.skew_x(9, true).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 9, 1,
            9, 1, 9,
            1, 9, 9,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_skew_x_reversefalse() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            1, 2,
            1, 3,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: Image = input.skew_x(0, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 0, 0,
            0, 1, 2, 0,
            0, 0, 1, 3,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_skew_x_reversetrue() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            1, 2,
            1, 3,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: Image = input.skew_x(0, true).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 2,
            0, 1, 2, 0,
            1, 3, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_skew_x_reversefalse() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 2,
            0, 1, 2, 0,
            1, 3, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: Image = input.skew_x(0, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 2, 0, 0,
            0, 0, 1, 2, 0, 0,
            0, 0, 1, 3, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_skew_y_reversefalse() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
        ];
        let input: Image = Image::try_create(3, 1, pixels).expect("image");

        // Act
        let actual: Image = input.skew_y(9, false).expect("image");

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
    fn test_20001_skew_y_reversetrue() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
        ];
        let input: Image = Image::try_create(3, 1, pixels).expect("image");

        // Act
        let actual: Image = input.skew_y(9, true).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 9, 1,
            9, 1, 9,
            1, 9, 9,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_skew_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            2, 2, 3,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.skew_y(0, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0,
            2, 1, 0,
            0, 2, 1,
            0, 0, 3,
        ];
        let expected: Image = Image::try_create(3, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20003_skew_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 3,
            0, 2, 1,
            2, 1, 0,
            1, 0, 0,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");

        // Act
        let actual: Image = input.skew_y(0, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 0, 0,
            2, 2, 3,
            1, 1, 1,
            0, 0, 0,
            0, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

}
