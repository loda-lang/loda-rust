use super::{Image, ImageOverlay};

pub trait ImageRepeat {
    fn repeat_by_count(&self, count_x: u8, count_y: u8) -> anyhow::Result<Image>;
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
}
