use super::{Image, ImageMask, ImageRotate, ImagePeriodicity, ImageRepairOffset};

pub trait ImageRepairPattern {
    /// Repair damaged pixels and recreate big repeating patterns such as mosaics.
    /// 
    /// Good for big mosaic patterns.
    /// 
    /// Attempts to repair all the pixels that has the specified `repair_color`.
    fn repair_pattern_with_color(&self, repair_color: u8) -> anyhow::Result<Image>;

    /// Repair damaged pixels and recreate big repeating patterns such as mosaics.
    /// 
    /// Good for big mosaic patterns.
    /// 
    /// Attempts to repair all the pixels in the specified `repair_mask`.
    fn repair_pattern_with_mask(&self, repair_mask: &Image) -> anyhow::Result<Image>;
}

impl ImageRepairPattern for Image {
    fn repair_pattern_with_color(&self, repair_color: u8) -> anyhow::Result<Image> {
        let mut repair_mask: Image = self.to_mask_where_color_is(repair_color);
        let result_image: Image = self.repair_pattern_with_mask(&mut repair_mask)?;
        Ok(result_image)
    }

    fn repair_pattern_with_mask(&self, repair_mask: &Image) -> anyhow::Result<Image> {
        if self.size() != repair_mask.size() {
            return Err(anyhow::anyhow!("images must have same size"));
        }
        let mut result_image: Image = self.clone();
        let mut repair_mask: Image = repair_mask.clone();

        // Horizontal repair
        let tile_width: Option<u8> = result_image.periodicity_x(&repair_mask)?;
        if let Some(offset) = tile_width {
            if offset < result_image.width() {
                result_image.repair_offset_x(&mut repair_mask, offset)?;
            }
        }

        result_image = result_image.rotate_cw()?;
        repair_mask = repair_mask.rotate_cw()?;

        // Vertical repair
        let tile_height: Option<u8> = result_image.periodicity_x(&repair_mask)?;
        if let Some(offset) = tile_height {
            if offset < result_image.width() {
                result_image.repair_offset_x(&mut repair_mask, offset)?;
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
        let actual: Image = input.repair_pattern_with_color(9).expect("image");

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
        let actual: Image = input.repair_pattern_with_color(9).expect("image");

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
