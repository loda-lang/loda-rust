use super::{Image, ImagePadding};

pub trait ImageBorder {
    fn border_inner(width: u8, height: u8, fill_color: u8, border_color: u8, border_size: u8) -> anyhow::Result<Image>;
}

impl ImageBorder for Image {
    fn border_inner(width: u8, height: u8, fill_color: u8, border_color: u8, border_size: u8) -> anyhow::Result<Image> {
        if width == 0 || height == 0 {
            return Ok(Image::empty());
        }
        let inner_width_i32: i32 = (width as i32) - (2 * (border_size as i32));
        let inner_height_i32: i32 = (height as i32) - (2 * (border_size as i32));
        if inner_width_i32 <= 0 || inner_height_i32 <= 0 {
            return Ok(Image::color(width, height, border_color));
        }
        let inner_width: u8 = inner_width_i32 as u8;
        let inner_height: u8 = inner_height_i32 as u8;

        let mut image: Image = Image::color(inner_width, inner_height, fill_color);
        if border_size > 0 {
            image = image.padding_with_color(border_size, border_color)?;
        }
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_border_inner() {
        // Act
        let actual: Image = Image::border_inner(6, 5, 1, 9, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_border_inner() {
        // Act
        let actual: Image = Image::border_inner(6, 5, 1, 9, 2).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 1, 1, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_border_inner() {
        // Act
        let actual: Image = Image::border_inner(6, 5, 1, 9, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 9, 9, 9, 9, 9,
            9, 1, 1, 1, 1, 9,
            9, 1, 1, 1, 1, 9,
            9, 1, 1, 1, 1, 9,
            9, 9, 9, 9, 9, 9,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_border_inner() {
        // Act
        let actual: Image = Image::border_inner(6, 5, 1, 9, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
