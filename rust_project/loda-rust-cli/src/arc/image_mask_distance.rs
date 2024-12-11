//! Measure distance to nearest pixel
use super::{Image, ImageMaskCount, ImageMaskGrow, PixelConnectivity, MixMode, ImageMix, ImageCompare, ImageMask};

/// ARC tasks uses images that are max 30x30 pixels.
static MAX_DISTANCE: u8 = 29;

#[allow(dead_code)]
pub trait ImageMaskDistance {
    /// Measure distance to the nearest pixels with value `1`.
    /// 
    /// It returns `255` when the input image is all zeros.
    /// When the input image is all zeros, it means there is nothing to measure distance to.
    /// 
    /// Returns an image with the same size.
    fn mask_distance_infinite(&self, connectivity: PixelConnectivity) -> anyhow::Result<Image>;

    /// Measure distance to the nearest pixels with value `1`.
    /// 
    /// This implementation is buggy. It returns `0` when the input image is all zeros.
    /// It's supposed to return 255 to indicate there is an infinite distance to the nearest pixel.
    /// When the input image is all zeros, it means there is nothing to measure distance to.
    /// 
    /// Returns an image with the same size.
    fn mask_distance_zerobug(&self, connectivity: PixelConnectivity) -> anyhow::Result<Image>;
}

impl ImageMaskDistance for Image {
    fn mask_distance_infinite(&self, connectivity: PixelConnectivity) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }

        let mut result_image: Image = self.clone();
        let mut current_mask: Image = self.clone();
        for distance in 2..MAX_DISTANCE {
            let (count0, count1, _count_other) = current_mask.mask_count();
            if count1 == 0 && distance == 2 {
                return Ok(self.clone_color(255));
            }
            if count0 == 0 {
                break;
            }

            let mask_grown: Image = current_mask.mask_grow(connectivity)?;
            let outline: Image = mask_grown.diff(&current_mask)?;
            current_mask = current_mask.mix(&mask_grown, MixMode::BooleanOr)?;
            result_image = outline.select_from_image_and_color(&result_image, distance)?;
        }

        Ok(result_image)
    }

    fn mask_distance_zerobug(&self, connectivity: PixelConnectivity) -> anyhow::Result<Image> {
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
    fn test_10000_mask_distance_infinite_connectivity4() {
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
        let actual: Image = input.mask_distance_infinite(PixelConnectivity::Connectivity4).expect("image");

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
    fn test_10001_mask_distance_infinite_connectivity8() {
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
        let actual: Image = input.mask_distance_infinite(PixelConnectivity::Connectivity8).expect("image");

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

    #[test]
    fn test_10002_mask_distance_infinite_empty_space() {
        // Arrange
        let input: Image = Image::zero(5, 5);

        // Act
        let actual: Image = input.mask_distance_infinite(PixelConnectivity::Connectivity8).expect("image");

        // Assert
        let expected: Image = Image::color(5, 5, 255);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_mask_distance_zerobug_connectivity4() {
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
        let actual: Image = input.mask_distance_zerobug(PixelConnectivity::Connectivity4).expect("image");

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
    fn test_20001_mask_distance_zerobug_connectivity8() {
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
        let actual: Image = input.mask_distance_zerobug(PixelConnectivity::Connectivity8).expect("image");

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

    #[test]
    fn test_20002_mask_distance_infinite_empty_space() {
        // Arrange
        let input: Image = Image::zero(5, 5);

        // Act
        let actual: Image = input.mask_distance_zerobug(PixelConnectivity::Connectivity8).expect("image");

        // Assert
        // This is where `mask_distance_zerobug()` has a problem.
        // It's supposed to return 255 to indicate there is an infinite distance to the nearest pixel.
        // However it returns 0 instead, so it seems that the nearest pixel is at distance 0, which is incorrect.
        // This is why the function is named `mask_distance_zerobug()`, to indicate it has a problem with zero distance.
        let expected: Image = Image::zero(5, 5);
        assert_eq!(actual, expected);
    }
}
