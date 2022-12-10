use super::{Image, convolution2x2};

pub trait ImageDetectHole {
    fn detect_hole_type1(&self, empty_color: u8) -> anyhow::Result<Image>;
}

impl ImageDetectHole for Image {
    fn detect_hole_type1(&self, empty_color: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let corner_image: Image = convolution2x2(&self, |bm| {
            let pixel00: u8 = bm.get(0, 0).unwrap_or(255);
            let pixel10: u8 = bm.get(1, 0).unwrap_or(255);
            let pixel01: u8 = bm.get(0, 1).unwrap_or(255);
            let pixel11: u8 = bm.get(1, 1).unwrap_or(255);
            let mut mask: u8 = 0;
            if pixel00 == pixel10 { mask |= 1; }
            if pixel01 == pixel11 { mask |= 2; }
            if pixel00 == pixel01 { mask |= 4; }
            if pixel10 == pixel11 { mask |= 8; }
            let value: u8 = match mask {
                5 => pixel00,
                6 => pixel01,
                9 => pixel10,
                10 => pixel11,
                _ => empty_color,
            };
            Ok(value)
        })?;
        Ok(corner_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_detect_hole_type1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 6, 6, 6, 0, 0, 0, 0,
            0, 6, 0, 6, 0, 0, 0, 0,
            0, 6, 0, 6, 0, 1, 1, 0,
            0, 6, 6, 6, 0, 1, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(8, 6, pixels).expect("image");

        // Act
        let actual: Image = input.detect_hole_type1(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 6, 6, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 6, 6, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(7, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_detect_hole_type1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 6, 6, 6, 0, 0, 0, 0,
            0, 6, 0, 6, 0, 1, 1, 1,
            0, 6, 0, 6, 0, 1, 1, 1,
            0, 6, 6, 6, 0, 1, 0, 1,
            0, 0, 0, 0, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(8, 6, pixels).expect("image");

        // Act
        let actual: Image = input.detect_hole_type1(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 6, 6, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 6, 6, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 1, 1,
        ];
        let expected: Image = Image::try_create(7, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_detect_hole_type1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 7, 7, 7, 0,
            0, 7, 7, 7, 0,
            0, 0, 0, 7, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.detect_hole_type1(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 7, 0,
            0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_detect_hole_type1_empty_color() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 0, 0,
            1, 0, 1, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 1, 0, 1,
            0, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.detect_hole_type1(9).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 9, 9,
            1, 1, 1, 9,
            9, 1, 1, 1,
            9, 9, 1, 1,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
