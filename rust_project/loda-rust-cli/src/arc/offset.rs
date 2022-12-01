use super::Image;

pub trait ImageOffset {
    fn offset_wrap(&self, x: i32, y: i32) -> anyhow::Result<Image>;
}

impl ImageOffset for Image {
    fn offset_wrap(&self, x: i32, y: i32) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let width: i32 = self.width() as i32;
        let height: i32 = self.height() as i32;
        
        // ensure that offset is positive
        let offset_x: i32 = ((x % width) + width) % width;
        let offset_y: i32 = ((y % height) + height) % height;
        if offset_x == 0 && offset_y == 0 {
            return Ok(self.clone());
        }

        let mut bitmap = Image::zeroes(self.width(), self.height());
        for y in 0..height {
            for x in 0..width {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                let set_x: i32 = (x + offset_x) % width;
                let set_y: i32 = (y + offset_y) % height;
                match bitmap.set(set_x, set_y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", set_x, set_y));
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
    fn test_10000_offset_wrap_xplus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            2, 3, 4, 1,
            6, 7, 8, 5,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("bitmap");

        // Act
        let actual: Image = input.offset_wrap(1, 0).expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 4,
            5, 6, 7, 8,
        ];
        let expected: Image = Image::try_create(4, 2, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_offset_wrap_xminus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 1, 2, 3,
            8, 5, 6, 7,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("bitmap");

        // Act
        let actual: Image = input.offset_wrap(-1, 0).expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 4,
            5, 6, 7, 8,
        ];
        let expected: Image = Image::try_create(4, 2, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_offset_wrap_yplus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            2, 6,
            3, 7,
            4, 8,
            1, 5,
        ];
        let input: Image = Image::try_create(2, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.offset_wrap(0, 1).expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 5, 
            2, 6,
            3, 7,
            4, 8,
        ];
        let expected: Image = Image::try_create(2, 4, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_offset_wrap_yminus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 8,
            1, 5,
            2, 6,
            3, 7,
        ];
        let input: Image = Image::try_create(2, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.offset_wrap(0, -1).expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 5, 
            2, 6,
            3, 7,
            4, 8,
        ];
        let expected: Image = Image::try_create(2, 4, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_offset_wrap_big() {
        // Arrange
        let pixels: Vec<u8> = vec![
            6, 6, 5, 5,
            8, 8, 7, 7,
            2, 2, 1, 1,
            4, 4, 3, 3,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.offset_wrap(-2, -2).expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 2, 2,
            3, 3, 4, 4,
            5, 5, 6, 6,
            7, 7, 8, 8,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("bitmap");
        assert_eq!(actual, expected);
    }
}
