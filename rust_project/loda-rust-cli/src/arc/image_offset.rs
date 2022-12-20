use super::Image;

pub trait ImageOffset {
    fn offset_wrap(&self, x: i32, y: i32) -> anyhow::Result<Image>;
    fn offset_clamp(&self, x: i32, y: i32) -> anyhow::Result<Image>;
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

        let mut image = Image::zero(self.width(), self.height());
        for y in 0..height {
            for x in 0..width {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                let set_x: i32 = (x + offset_x) % width;
                let set_y: i32 = (y + offset_y) % height;
                match image.set(set_x, set_y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", set_x, set_y));
                    }
                }
            }
        }
        Ok(image)
    }

    fn offset_clamp(&self, offset_x: i32, offset_y: i32) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let width: i32 = self.width() as i32;
        let height: i32 = self.height() as i32;
        let max_x: i32 = width - 1;
        let max_y: i32 = height - 1;
        let mut image = Image::zero(self.width(), self.height());
        for y in 0..height {
            for x in 0..width {
                let get_x: i32 = i32::min(i32::max(x - offset_x, 0), max_x);
                let get_y: i32 = i32::min(i32::max(y - offset_y, 0), max_y);
                let pixel_value: u8 = self.get(get_x, get_y).unwrap_or(255);
                match image.set(x, y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", x, y));
                    }
                }
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
    fn test_10000_offset_wrap_xplus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            2, 3, 4, 1,
            6, 7, 8, 5,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");

        // Act
        let actual: Image = input.offset_wrap(1, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 4,
            5, 6, 7, 8,
        ];
        let expected: Image = Image::try_create(4, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_offset_wrap_xminus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 1, 2, 3,
            8, 5, 6, 7,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");

        // Act
        let actual: Image = input.offset_wrap(-1, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 4,
            5, 6, 7, 8,
        ];
        let expected: Image = Image::try_create(4, 2, expected_pixels).expect("image");
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
        let input: Image = Image::try_create(2, 4, pixels).expect("image");

        // Act
        let actual: Image = input.offset_wrap(0, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 5, 
            2, 6,
            3, 7,
            4, 8,
        ];
        let expected: Image = Image::try_create(2, 4, expected_pixels).expect("image");
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
        let input: Image = Image::try_create(2, 4, pixels).expect("image");

        // Act
        let actual: Image = input.offset_wrap(0, -1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 5, 
            2, 6,
            3, 7,
            4, 8,
        ];
        let expected: Image = Image::try_create(2, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_offset_wrap_big() {
        // Arrange
        let pixels: Vec<u8> = vec![
            6, 6, 5, 5,
            8, 8, 7, 7,
            2, 2, 1, 1,
            4, 4, 3, 3,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.offset_wrap(-2, -2).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 2, 2,
            3, 3, 4, 4,
            5, 5, 6, 6,
            7, 7, 8, 8,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_offset_clamp_xplus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            2, 3, 4, 1,
            6, 7, 8, 5,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");

        // Act
        let actual: Image = input.offset_clamp(1, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            2, 2, 3, 4,
            6, 6, 7, 8,
        ];
        let expected: Image = Image::try_create(4, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_offset_clamp_xminus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 1, 2, 3,
            8, 5, 6, 7,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");

        // Act
        let actual: Image = input.offset_clamp(-1, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 3,
            5, 6, 7, 7,
        ];
        let expected: Image = Image::try_create(4, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_offset_clamp_yplus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            2, 6,
            3, 7,
            4, 8,
            1, 5,
        ];
        let input: Image = Image::try_create(2, 4, pixels).expect("image");

        // Act
        let actual: Image = input.offset_clamp(0, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            2, 6, 
            2, 6,
            3, 7,
            4, 8,
        ];
        let expected: Image = Image::try_create(2, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20003_offset_clamp_yminus1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 8,
            1, 5,
            2, 6,
            3, 7,
        ];
        let input: Image = Image::try_create(2, 4, pixels).expect("image");

        // Act
        let actual: Image = input.offset_clamp(0, -1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 5, 
            2, 6,
            3, 7,
            3, 7,
        ];
        let expected: Image = Image::try_create(2, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20004_offset_clamp_big() {
        // Arrange
        let pixels: Vec<u8> = vec![
            6, 6, 5, 5,
            8, 8, 7, 7,
            2, 2, 1, 1,
            4, 4, 3, 3,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.offset_clamp(-2, -2).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            3, 3, 3, 3,
            3, 3, 3, 3,
            3, 3, 3, 3,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20005_offset_clamp_over_the_edge() {
        // Arrange
        let input: Image = Image::try_create(2, 2, vec![1, 2, 3, 4]).expect("image");

        // Act
        let actual: Image = input.offset_clamp(10, 0).expect("image");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![1, 1, 3, 3]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20006_offset_clamp_over_the_edge() {
        // Arrange
        let input: Image = Image::try_create(2, 2, vec![1, 2, 3, 4]).expect("image");

        // Act
        let actual: Image = input.offset_clamp(10, 10).expect("image");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![1, 1, 1, 1]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20007_offset_clamp_over_the_edge() {
        // Arrange
        let input: Image = Image::try_create(2, 2, vec![1, 2, 3, 4]).expect("image");

        // Act
        let actual: Image = input.offset_clamp(-10, -10).expect("image");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![4, 4, 4, 4]).expect("image");
        assert_eq!(actual, expected);
    }
}
