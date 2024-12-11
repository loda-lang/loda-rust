use super::{Image, ImagePadding, convolution3x3, ImageMaskCount, PixelConnectivity};

#[allow(dead_code)]
pub trait ImageMaskGrow {
    /// Grow the mask in all directions.
    /// 
    /// Returns an image with the same size.
    fn mask_grow(&self, connectivity: PixelConnectivity) -> anyhow::Result<Image>;
}

impl ImageMaskGrow for Image {
    fn mask_grow(&self, connectivity: PixelConnectivity) -> anyhow::Result<Image> {
        match connectivity {
            PixelConnectivity::Connectivity4 => MaskGrowAlgorithm::mask_grow4(self),
            PixelConnectivity::Connectivity8 => MaskGrowAlgorithm::mask_grow8(self)
        }
    }
}

struct MaskGrowAlgorithm;

impl MaskGrowAlgorithm {
    fn mask_grow4(original: &Image) -> anyhow::Result<Image> {
        if original.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded: Image = original.padding_with_color(1, 0)?;
        let result_image: Image = convolution3x3(&input_padded, |image| {
            let top: u8 = image.get(1, 0).unwrap_or(255);
            let left: u8 = image.get(0, 1).unwrap_or(255);
            let center: u8 = image.get(1, 1).unwrap_or(255);
            let right: u8 = image.get(2, 1).unwrap_or(255);
            let bottom: u8 = image.get(1, 2).unwrap_or(255);
            if top > 0 || left > 0 || center > 0 || right > 0 || bottom > 0 {
                return Ok(1);
            }
            Ok(0)
        })?;
        Ok(result_image)
    }

    fn mask_grow8(original: &Image) -> anyhow::Result<Image> {
        if original.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded: Image = original.padding_with_color(1, 0)?;
        let result_image: Image = convolution3x3(&input_padded, |image| {
            if image.mask_count_one() > 0 {
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
    fn test_10000_mask_grow4() {
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
        let actual: Image = input.mask_grow(PixelConnectivity::Connectivity4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 1, 1, 1, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_mask_grow8() {
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
        let actual: Image = input.mask_grow(PixelConnectivity::Connectivity8).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 1, 1, 1, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
