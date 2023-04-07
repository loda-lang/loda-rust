use super::{Image, ImageRotate};

pub trait ImageRepairOffset {
    /// Fix damaged pixels in the horizontal plane, by copying good pixels from the same row with `offset * n`.
    fn repair_offset_x(&mut self, repair_mask: &Image, offset: u8) -> anyhow::Result<()>;

    /// Fix damaged pixels in the vertical plane, by copying good pixels from the same column with `offset * n`.
    fn repair_offset_y(&mut self, repair_mask: &Image, offset: u8) -> anyhow::Result<()>;

    // Idea for future
    // repair_offset_diagonal_a()
    // repair_offset_diagonal_b()
}

impl ImageRepairOffset for Image {
    fn repair_offset_x(&mut self, repair_mask: &Image, offset: u8) -> anyhow::Result<()> {
        if self.width() != repair_mask.width() || self.height() != repair_mask.height() {
            return Err(anyhow::anyhow!("Expected same size for 'image' and 'repair_mask'"));
        }
        if self.is_empty() {
            return Err(anyhow::anyhow!("Image must be 1x1 or greater"));
        }
        if offset == 0 {
            return Err(anyhow::anyhow!("Offset must be greater than zero"));
        }
        if offset >= self.width() {
            return Err(anyhow::anyhow!("The offset: {} must be smaller than the width: {}", offset, self.width()));
        }
        let original: Image = self.clone();
        for y in 0..original.height() as i32 {
            for x in 0..original.width() as i32 {
                let mask_color0: u8 = repair_mask.get(x, y).unwrap_or(255);
                if mask_color0 == 0 {
                    continue;
                }

                let x_offset_plus1: i32 = x + (offset as i32);
                let mask_color_offset_plus1: u8 = repair_mask.get(x_offset_plus1, y).unwrap_or(255);
                if mask_color_offset_plus1 == 0 {
                    let set_color: u8 = original.get(x_offset_plus1, y).unwrap_or(255);
                    self.set(x, y, set_color);
                    continue;
                }

                let x_offset_minus1: i32 = x - (offset as i32);
                let mask_color_offset_minus1: u8 = repair_mask.get(x_offset_minus1, y).unwrap_or(255);
                if mask_color_offset_minus1 == 0 {
                    let set_color: u8 = original.get(x_offset_minus1, y).unwrap_or(255);
                    self.set(x, y, set_color);
                    continue;
                }

                let x_offset_plus2: i32 = x + (offset as i32) * 2;
                let mask_color_offset_plus2: u8 = repair_mask.get(x_offset_plus2, y).unwrap_or(255);
                if mask_color_offset_plus2 == 0 {
                    let set_color: u8 = original.get(x_offset_plus2, y).unwrap_or(255);
                    self.set(x, y, set_color);
                    continue;
                }

                let x_offset_minus2: i32 = x - (offset as i32) * 2;
                let mask_color_offset_minus2: u8 = repair_mask.get(x_offset_minus2, y).unwrap_or(255);
                if mask_color_offset_minus2 == 0 {
                    let set_color: u8 = original.get(x_offset_minus2, y).unwrap_or(255);
                    self.set(x, y, set_color);
                    continue;
                }
            }
        }
        Ok(())
    }

    fn repair_offset_y(&mut self, repair_mask: &Image, offset: u8) -> anyhow::Result<()> {
        let mut image: Image = self.rotate_cw()?;
        let repair_mask: Image = repair_mask.rotate_cw()?;
        image.repair_offset_x(&repair_mask, offset)?;
        image = image.rotate_ccw()?;
        self.set_image(image);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_repair_offset_x_checkerboard() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 9, 9, 1, 2, 1, // period 2
            2, 1, 9, 9, 2, 1, 2, // period 2
            1, 2, 1, 2, 9, 9, 9, // period 2
            2, 1, 2, 1, 9, 9, 9, // period 2
            1, 2, 1, 2, 9, 9, 9, // period 2
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        let repair_pixels: Vec<u8> = vec![
            0, 0, 1, 1, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1,
        ];
        let repair_mask: Image = Image::try_create(7, 5, repair_pixels).expect("image");

        // Act
        let mut actual: Image = input.clone();
        actual.repair_offset_x(&repair_mask, 2).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            2, 1, 2, 1, 2, 1, 2, // period 2
            1, 2, 1, 2, 1, 2, 1, // period 2
            2, 1, 2, 1, 2, 1, 2, // period 2
            1, 2, 1, 2, 1, 2, 1, // period 2
        ];
        let expected: Image = Image::try_create(7, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_repair_offset_x_columns() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 3, 9, 9, 9, 3, 1, // period 2
            1, 3, 9, 9, 9, 3, 1, // period 2
            1, 3, 9, 9, 1, 9, 9, // period 2
            1, 3, 1, 3, 9, 9, 9, // period 2
            1, 3, 1, 3, 9, 9, 9, // period 2
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        let repair_pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1, 0, 0,
            0, 0, 1, 1, 1, 0, 0,
            0, 0, 1, 1, 0, 1, 1,
            0, 0, 0, 0, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1,
        ];
        let repair_mask: Image = Image::try_create(7, 5, repair_pixels).expect("image");

        // Act
        let mut actual: Image = input.clone();
        actual.repair_offset_x(&repair_mask, 2).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 3, 1, 3, 1, 3, 1, // period 2
            1, 3, 1, 3, 1, 3, 1, // period 2
            1, 3, 1, 3, 1, 3, 1, // period 2
            1, 3, 1, 3, 1, 3, 1, // period 2
            1, 3, 1, 3, 1, 3, 1, // period 2
        ];
        let expected: Image = Image::try_create(7, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_repair_offset_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            // column 0 has a period of 3
            // column 1 has a period of 2
            1, 4,
            2, 5,
            3, 4,
            9, 9,
            9, 9,
            3, 5,
            1, 4,
            2, 5,
            3, 4,
            1, 5,
            2, 4,
            3, 5,
        ];
        let input: Image = Image::try_create(2, 12, pixels).expect("image");

        let repair_pixels: Vec<u8> = vec![
            0, 0,
            0, 0,
            0, 0,
            1, 1,
            1, 1,
            0, 0,
            0, 0,
            0, 0,
            0, 0,
            0, 0,
            0, 0,
            0, 0,
        ];
        let repair_mask: Image = Image::try_create(2, 12, repair_pixels).expect("image");

        // Act
        let mut actual: Image = input.clone();
        actual.repair_offset_y(&repair_mask, 6).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            // column 0 has a period of 3
            // column 1 has a period of 2
            1, 4,
            2, 5,
            3, 4,
            1, 5,
            2, 4,
            3, 5,
            1, 4,
            2, 5,
            3, 4,
            1, 5,
            2, 4,
            3, 5,
        ];
        let expected: Image = Image::try_create(2, 12, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
