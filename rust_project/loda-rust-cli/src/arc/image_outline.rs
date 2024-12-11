use super::{Image, ImagePadding, convolution3x3};

pub trait ImageOutline {
    fn outline_mask_neighbour(&self) -> anyhow::Result<Image>;
    fn outline_type1(&self, outline_color: u8, background_color: u8) -> anyhow::Result<Image>;
}

impl ImageOutline for Image {
    fn outline_mask_neighbour(&self) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded: Image = self.padding_with_color(1, 0)?;
        let image: Image = convolution3x3(&input_padded, |bm| {
            let c: u8 = bm.get(1, 1).unwrap_or(255);
            if c == 1 {
                return Ok(0);
            }
            let t: u8 = bm.get(1, 0).unwrap_or(255);
            let l: u8 = bm.get(0, 1).unwrap_or(255);
            let r: u8 = bm.get(2, 1).unwrap_or(255);
            let b: u8 = bm.get(1, 2).unwrap_or(255);
            if t > 0 || l > 0 || r > 0 || b > 0 {
                return Ok(1);
            }
            Ok(0)
        })?;
        Ok(image)
    }

    fn outline_type1(&self, outline_color: u8, background_color: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded: Image = self.padding_with_color(1, background_color)?;
        let image: Image = convolution3x3(&input_padded, |bm| {
            let center_pixel_value: u8 = bm.get(1, 1).unwrap_or(255);
            if center_pixel_value != background_color {
                return Ok(center_pixel_value);
            }
            for y in 0..3i32 {
                for x in 0..3i32 {
                    if x == 1 && y == 1 {
                        continue;
                    }
                    let pixel_value: u8 = bm.get(x, y).unwrap_or(255);
                    if pixel_value != center_pixel_value {
                        return Ok(outline_color);
                    }
                }
            }
            Ok(background_color)
        })?;

        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_outline_mask_neighbour() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.outline_mask_neighbour().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0,
            1, 0, 1, 0,
            0, 1, 0, 0,
            0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_outline_mask_neighbour() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 1, 0,
            0, 1, 1, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.outline_mask_neighbour().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            1, 0, 0, 1,
            1, 0, 0, 1,
            0, 1, 1, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_outline_mask_neighbour() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.outline_mask_neighbour().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0,
            1, 0, 1, 0,
            0, 1, 0, 1,
            0, 0, 1, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_outline_type1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.outline_type1(2, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            2, 2, 2, 0,
            2, 1, 2, 0,
            2, 2, 2, 0,
            0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_outline_type1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 2, 1, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.outline_type1(9, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 9, 9, 0,
            9, 1, 9, 9,
            9, 2, 1, 9,
            9, 9, 9, 9,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_outline_type1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.outline_type1(9, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 9, 9, 0,
            9, 1, 9, 9,
            9, 9, 1, 9,
            0, 9, 9, 1,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
