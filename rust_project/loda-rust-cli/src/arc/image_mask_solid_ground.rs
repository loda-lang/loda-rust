use super::{Image, ImagePadding, convolution3x3};

#[allow(dead_code)]
pub trait ImageMaskSolidGround {
    /// Determines if there is solid ground 1 pixel directly below the current pixel.
    /// 
    /// The pixel value is 1 when there is ground below, or if it's at the bottom position.
    /// 
    /// The pixel value is 0 when there is nothing immediately below.
    /// 
    /// Returns an image with the same size.
    /// 
    /// The usecase is gravity. When an object is moving down to the ground, and the goal is to
    /// maximize the number of pixels where the object is touching the ground,
    /// and leave as few pixels untouched.
    /// It's a bad candidate position if there are several pixels untouched and another object 
    /// are touching more pixels.
    fn mask_ground_below(&self) -> anyhow::Result<Image>;
}

impl ImageMaskSolidGround for Image {
    fn mask_ground_below(&self) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded_zeros: Image = self.padding_advanced(1, 1, 1, 0, 0)?;
        let input_padded: Image = input_padded_zeros.padding_advanced(0, 0, 0, 1, 1)?;
        let result_image: Image = convolution3x3(&input_padded, |image| {
            let center: u8 = image.get(1, 1).unwrap_or(255);
            let bottom: u8 = image.get(1, 2).unwrap_or(255);
            if center == 0 && bottom > 0 {
                return Ok(1);
            }
            Ok(0)
        })?;
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_mask_ground_below() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 1,
            0, 0, 0, 1, 1,
            0, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.mask_ground_below().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 1, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            0, 0, 0, 1, 0,
            0, 1, 1, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
