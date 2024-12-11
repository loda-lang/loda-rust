use super::{Image, ImagePadding, ImageExtractRowColumn, ImageStack, ImageRepeat};

pub trait ImageBorder {
    /// Draw border inside an empty image.
    /// 
    /// Similar to CSS `border` with `box-sizing: border-box`.
    fn border_inside(width: u8, height: u8, fill_color: u8, border_color: u8, border_size: u8) -> anyhow::Result<Image>;

    /// Expand by repeating the outer-most pixel border.
    fn border_grow(&self, border_size: u8, corner_color: u8) -> anyhow::Result<Image>;
}

impl ImageBorder for Image {
    fn border_inside(width: u8, height: u8, fill_color: u8, border_color: u8, border_size: u8) -> anyhow::Result<Image> {
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

    fn border_grow(&self, border_size: u8, corner_color: u8) -> anyhow::Result<Image> {
        if border_size == 0 {
            return Err(anyhow::anyhow!("border_size must be 1 or greater"));
        }
        let corner: Image = Image::color(border_size, border_size, corner_color);
        if self.is_empty() {
            let result_image: Image = corner.repeat_by_count(2, 2)?;
            return Ok(result_image);
        }
        let top: Image = self.top_rows(1)?.repeat_by_count(1, border_size)?;
        let left: Image = self.left_columns(1)?.repeat_by_count(border_size, 1)?;
        let right: Image = self.right_columns(1)?.repeat_by_count(border_size, 1)?;
        let bottom: Image = self.bottom_rows(1)?.repeat_by_count(1, border_size)?;

        // Glue together the 9x9 pieces into a single piece
        let a: Image = corner.vjoin(left)?.vjoin(corner.clone())?;
        let b: Image = top.vjoin(self.clone())?.vjoin(bottom)?;
        let c: Image = corner.vjoin(right)?.vjoin(corner.clone())?;
        let result_image: Image = a.hjoin(b)?.hjoin(c)?;
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_border_inside() {
        // Act
        let actual: Image = Image::border_inside(6, 5, 1, 9, 3).expect("image");

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
    fn test_10001_border_inside() {
        // Act
        let actual: Image = Image::border_inside(6, 5, 1, 9, 2).expect("image");

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
    fn test_10002_border_inside() {
        // Act
        let actual: Image = Image::border_inside(6, 5, 1, 9, 1).expect("image");

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
    fn test_10003_border_inside() {
        // Act
        let actual: Image = Image::border_inside(6, 5, 1, 9, 0).expect("image");

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

    #[test]
    fn test_20000_border_grow_3x2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.border_grow(1, 9).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 1, 2, 3, 9,
            1, 1, 2, 3, 3,
            4, 4, 5, 6, 6,
            9, 4, 5, 6, 9,
        ];
        let expected: Image = Image::try_create(5, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_border_grow_1x1() {
        // Arrange
        let input: Image = Image::color(1, 1, 2);

        // Act
        let actual: Image = input.border_grow(2, 9).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 9, 2, 9, 9,
            9, 9, 2, 9, 9,
            2, 2, 2, 2, 2,
            9, 9, 2, 9, 9,
            9, 9, 2, 9, 9,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_border_grow_empty() {
        // Act
        let actual: Image = Image::empty().border_grow(2, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
