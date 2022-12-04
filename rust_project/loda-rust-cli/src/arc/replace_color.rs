use super::Image;

pub trait ImageReplaceColor {
    fn replace_color(&self, source: u8, destination: u8) -> anyhow::Result<Image>;
}

impl ImageReplaceColor for Image {
    fn replace_color(&self, source: u8, destination: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;

        let mut bitmap = Image::zero(self.width(), self.height());
        for y in 0..=y_max {
            for x in 0..=x_max {
                let mut pixel_value: u8 = self.get(x, y).unwrap_or(255);
                if pixel_value == source {
                    pixel_value = destination;
                }
                match bitmap.set(x, y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", x, y));
                    }
                }
            }
        }
        return Ok(bitmap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_replace() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 3, 0, 3,
            0, 0, 3, 2,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: Image = input.replace_color(3, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 0, 1,
            0, 0, 1, 2,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
