use super::{Image, ImageOverlay, ImageRotate};

pub trait ImageRepeat {
    /// Make a big image by repeating the current image.
    fn repeat_by_count(&self, count_x: u8, count_y: u8) -> anyhow::Result<Image>;

    /// Make a big image by repeating the current image, rotations: 0, 90, 180, 270 degrees.
    /// 
    /// The image must be a square, otherwise an error is returned.
    fn repeat_rotated(&self, top: u8, bottom: u8, left: u8, right: u8) -> anyhow::Result<Image>;
}

impl ImageRepeat for Image {
    fn repeat_by_count(&self, count_x: u8, count_y: u8) -> anyhow::Result<Image> {
        if count_x == 0 || count_y == 0 {
            return Err(anyhow::anyhow!("Both count_x and count_y must be 1 or greater."));
        }
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let output_width: u16 = (self.width() as u16) * (count_x as u16);
        let output_height: u16 = (self.height() as u16) * (count_y as u16);
        if output_width > (u8::MAX as u16) {
            return Err(anyhow::anyhow!("Output image.width {} is too big. self.width: {} count_x: {}", output_width, self.width(), count_x));
        }
        if output_height > (u8::MAX as u16) {
            return Err(anyhow::anyhow!("Output image.height {} is too big. self.height: {} count_y: {}", output_height, self.height(), count_y));
        }
        let mut image: Image = Image::zero(output_width as u8, output_height as u8);
        for y in 0..count_y {
            for x in 0..count_x {
                image = image.overlay_with_position(&self, (x * self.width()) as i32, (y * self.height()) as i32)?;
            }
        }
        Ok(image)
    }

    fn repeat_rotated(&self, top: u8, bottom: u8, left: u8, right: u8) -> anyhow::Result<Image> {
        if top == 0 && bottom == 0 && left == 0 && right == 0 {
            return Err(anyhow::anyhow!("One or more counters must be 1 or greater."));
        }
        if self.is_empty() {
            return Ok(Image::empty());
        }
        if self.width() != self.height() {
            return Err(anyhow::anyhow!("The image must be a square."));
        }
        let count_x: u16 = (left as u16) + (right as u16) + 1;
        let count_y: u16 = (top as u16) + (bottom as u16) + 1;
        let output_width: u16 = (self.width() as u16) * count_x;
        let output_height: u16 = (self.height() as u16) * count_y;
        if output_width > (u8::MAX as u16) {
            return Err(anyhow::anyhow!("Output image.width {} is too big. self.width: {} left: {} right: {}", output_width, self.width(), left, right));
        }
        if output_height > (u8::MAX as u16) {
            return Err(anyhow::anyhow!("Output image.height {} is too big. self.height: {} top: {} bottom: {}", output_height, self.height(), top, bottom));
        }
        let self90: Image = self.rotate_cw()?;
        let self180: Image = self90.rotate_cw()?;
        let self270: Image = self180.rotate_cw()?;
        let width_i32 = self.width() as i32;
        let height_i32 = self.height() as i32;
        let mut result_image: Image = Image::zero(output_width as u8, output_height as u8);
        for y in 0..(count_y as i32) {
            for x in 0..(count_x as i32) {
                let variant: i32 = ((y + (top as i32)) & 1) * 2 + ((x + (left as i32)) & 1);
                let image: &Image = match variant {
                    0 => &self,
                    1 => &self90,
                    2 => &self270,
                    3 => &self180,
                    _ => unreachable!(),
                };
                result_image = result_image.overlay_with_position(&image, x * width_i32, y * height_i32)?;
            }
        }
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_repeat_by_count11() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let output: Image = input.repeat_by_count(1, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let expected = Image::create_raw(3, 2, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10001_repeat_by_count_24() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let output: Image = input.repeat_by_count(2, 4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 1, 2, 3,
            4, 5, 6, 4, 5, 6,
            1, 2, 3, 1, 2, 3,
            4, 5, 6, 4, 5, 6,
            1, 2, 3, 1, 2, 3,
            4, 5, 6, 4, 5, 6,
            1, 2, 3, 1, 2, 3,
            4, 5, 6, 4, 5, 6,
        ];
        let expected = Image::create_raw(6, 8, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20000_repeat_rotated_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let output: Image = input.repeat_rotated(0, 0, 0, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 1,
            3, 4, 4, 2,
        ];
        let expected = Image::create_raw(4, 2, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20001_repeat_rotated_bottom() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let output: Image = input.repeat_rotated(0, 1, 0, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
            2, 4,
            1, 3,
        ];
        let expected = Image::create_raw(2, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20002_repeat_rotated_left() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let output: Image = input.repeat_rotated(0, 0, 1, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 1, 1, 2,
            4, 2, 3, 4,
        ];
        let expected = Image::create_raw(4, 2, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20003_repeat_rotated_top() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let output: Image = input.repeat_rotated(1, 0, 0, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            2, 4,
            1, 3,
            1, 2,
            3, 4,
        ];
        let expected = Image::create_raw(2, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20004_repeat_rotated_right_bottom() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let output: Image = input.repeat_rotated(0, 1, 0, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 1,
            3, 4, 4, 2,
            2, 4, 4, 3,
            1, 3, 2, 1,
        ];
        let expected = Image::create_raw(4, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20005_repeat_rotated_left_top() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let output: Image = input.repeat_rotated(1, 0, 1, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            4, 3, 2, 4,
            2, 1, 1, 3,
            3, 1, 1, 2,
            4, 2, 3, 4,
        ];
        let expected = Image::create_raw(4, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20006_repeat_rotated_all() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let output: Image = input.repeat_rotated(1, 1, 1, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            4, 3, 2, 4, 4, 3,
            2, 1, 1, 3, 2, 1,
            3, 1, 1, 2, 3, 1,
            4, 2, 3, 4, 4, 2,
            4, 3, 2, 4, 4, 3,
            2, 1, 1, 3, 2, 1,
        ];
        let expected = Image::create_raw(6, 6, expected_pixels);
        assert_eq!(output, expected);
    }
}
