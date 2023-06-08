use super::{Image, ImageMaskCount, ImageMaskGrow, PixelConnectivity, MixMode, ImageMix, ImageCompare, ImageMask};

/// ARC tasks uses images that are max 30x30 pixels.
static MAX_DISTANCE: u8 = 29;

#[allow(dead_code)]
pub trait ImageMaskDistance {
    /// Measure distance to the nearest pixels with value `1`.
    /// 
    /// Returns an image with the same size.
    fn mask_distance(&self, connectivity: PixelConnectivity) -> anyhow::Result<Image>;
}

impl ImageMaskDistance for Image {
    fn mask_distance(&self, connectivity: PixelConnectivity) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }

        let mut result_image: Image = self.clone();
        let mut current_mask: Image = self.clone();
        for distance in 2..MAX_DISTANCE {
            if current_mask.mask_count_zero() == 0 {
                break;
            }

            let mask_grown: Image = current_mask.mask_grow(connectivity)?;
            let outline: Image = mask_grown.diff(&current_mask)?;
            current_mask = current_mask.mix(&mask_grown, MixMode::BooleanOr)?;
            result_image = outline.select_from_image_and_color(&result_image, distance)?;
        }

        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_mask_distance4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.mask_distance(PixelConnectivity::Connectivity4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 4, 3, 4, 5,
            4, 3, 2, 3, 4,
            3, 2, 1, 2, 3,
            4, 3, 2, 3, 4,
            5, 4, 3, 4, 5,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_mask_distance8() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.mask_distance(PixelConnectivity::Connectivity8).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3, 3,
            3, 2, 2, 2, 3,
            3, 2, 1, 2, 3,
            3, 2, 2, 2, 3,
            3, 3, 3, 3, 3,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
