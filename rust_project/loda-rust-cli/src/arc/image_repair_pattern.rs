use super::{Image, ImageMask, ImageRotate, ImagePeriodicity, ImageRepairOffset};

pub trait ImageRepairPattern {
    /// Repair damaged pixels and recreate big repeating patterns such as mosaics.
    /// 
    /// Good for big mosaic patterns.
    fn repair_pattern(&self, repair_color: u8) -> anyhow::Result<Image>;
}

impl ImageRepairPattern for Image {
    fn repair_pattern(&self, repair_color: u8) -> anyhow::Result<Image> {
        let mut result_image: Image = self.clone();

        // Horizontal repair
        let repair_mask_x: Image = result_image.to_mask_where_color_is(repair_color);
        let tile_width: Option<u8> = result_image.horizontal_periodicity(&repair_mask_x)?;
        if let Some(offset) = tile_width {
            if offset < result_image.width() {
                result_image.repair_offset_x(&repair_mask_x, offset)?;
            }
        }

        result_image = result_image.rotate_cw()?;

        // Vertical repair
        let repair_mask_y: Image = result_image.to_mask_where_color_is(repair_color);
        let tile_height: Option<u8> = result_image.horizontal_periodicity(&repair_mask_y)?;
        if let Some(offset) = tile_height {
            if offset < result_image.width() {
                result_image.repair_offset_x(&repair_mask_y, offset)?;
            }
        }

        result_image = result_image.rotate_ccw()?;
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_repair_pattern() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 9, 1, 2, 4, 2, 1,
            9, 9, 2, 1, 2, 4, 2,
            1, 2, 4, 2, 9, 2, 4,
            2, 1, 2, 4, 2, 1, 2,
            4, 9, 9, 2, 4, 2, 1,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        // Act
        let actual: Image = input.repair_pattern(9).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            4, 2, 1, 2, 4, 2, 1,
            2, 4, 2, 1, 2, 4, 2,
            1, 2, 4, 2, 1, 2, 4,
            2, 1, 2, 4, 2, 1, 2,
            4, 2, 1, 2, 4, 2, 1,
        ];
        let expected: Image = Image::try_create(7, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_repair_pattern() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 2, 2, 5, 2, 2, 5,
            5, 9, 2, 5, 2, 2, 5,
            3, 5, 5, 3, 5, 5, 3,
            5, 2, 2, 5, 2, 9, 9,
            5, 2, 2, 5, 2, 9, 9,
            3, 5, 5, 3, 5, 9, 9,
        ];
        let input: Image = Image::try_create(7, 6, pixels).expect("image");

        // Act
        let actual: Image = input.repair_pattern(9).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 2, 2, 5, 2, 2, 5,
            5, 2, 2, 5, 2, 2, 5,
            3, 5, 5, 3, 5, 5, 3,
            5, 2, 2, 5, 2, 2, 5,
            5, 2, 2, 5, 2, 2, 5,
            3, 5, 5, 3, 5, 5, 3,
        ];
        let expected: Image = Image::try_create(7, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
