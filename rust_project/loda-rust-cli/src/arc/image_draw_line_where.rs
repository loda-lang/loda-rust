use super::Image;

pub trait ImageDrawLineWhere {
    /// Draw a horizontal line if the `condition_image` contains one or more pixels with the `condition_color`.
    /// 
    /// Returns the number of lines that was drawn.
    fn draw_line_where_row_contains_color(&mut self, condition_image: &Image, condition_color: u8, line_color: u8) -> anyhow::Result<u8>;

    /// Draw a vertical line if the `condition_image` contains one or more pixels with the `condition_color`.
    /// 
    /// Returns the number of lines that was drawn.
    fn draw_line_where_column_contains_color(&mut self, condition_image: &Image, condition_color: u8, line_color: u8) -> anyhow::Result<u8>;

    /// Draw horizontal lines and vertical lines where the `condition_image` contains one or more pixels with the `condition_color`.
    fn draw_line_where_row_or_column_contains_color(&mut self, condition_image: &Image, condition_color: u8, line_color: u8) -> anyhow::Result<()>;
}

impl ImageDrawLineWhere for Image {
    fn draw_line_where_row_contains_color(&mut self, condition_image: &Image, condition_color: u8, line_color: u8) -> anyhow::Result<u8> {
        if self.size() != condition_image.size() {
            return Err(anyhow::anyhow!("Expected condition_image.size to be the same as self.size"));
        }
        let width = self.width() as i32;
        let height = self.height() as i32;
        let mut number_of_lines: u8 = 0;
        for y in 0..height {
            let mut ignore: bool = true;
            for x in 0..width {
                let color: u8 = condition_image.get(x, y).unwrap_or(255);
                if color == condition_color {
                    ignore = false;
                    break;
                }
            }
            if ignore {
                continue;
            }
            number_of_lines += 1;
            for x in 0..width {
                _ = self.set(x, y, line_color);
            }
        }
        Ok(number_of_lines)
    }

    fn draw_line_where_column_contains_color(&mut self, condition_image: &Image, condition_color: u8, line_color: u8) -> anyhow::Result<u8> {
        if self.size() != condition_image.size() {
            return Err(anyhow::anyhow!("Expected condition_image.size to be the same as self.size"));
        }
        let width = self.width() as i32;
        let height = self.height() as i32;
        let mut number_of_lines: u8 = 0;
        for x in 0..width {
            let mut ignore: bool = true;
            for y in 0..height {
                let color: u8 = condition_image.get(x, y).unwrap_or(255);
                if color == condition_color {
                    ignore = false;
                    break;
                }
            }
            if ignore {
                continue;
            }
            number_of_lines += 1;
            for y in 0..height {
                _ = self.set(x, y, line_color);
            }
        }
        Ok(number_of_lines)
    }

    fn draw_line_where_row_or_column_contains_color(&mut self, condition_image: &Image, condition_color: u8, line_color: u8) -> anyhow::Result<()> {
        _ = self.draw_line_where_row_contains_color(&condition_image, condition_color, line_color)?;
        _ = self.draw_line_where_column_contains_color(&condition_image, condition_color, line_color)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_draw_line_where_row_contains_color() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
        ];
        let input0: Image = Image::try_create(5, 5, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 42, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 42,
        ];
        let input1: Image = Image::try_create(5, 5, pixels1).expect("image");

        // Act
        let mut actual = input0.clone();
        let line_count: u8 = actual.draw_line_where_row_contains_color(&input1, 42, 5).expect("line count");

        // Assert
        assert_eq!(line_count, 2);
        let expected_pixels: Vec<u8> = vec![
            1, 0, 1, 0, 1,
            5, 5, 5, 5, 5,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            5, 5, 5, 5, 5,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_draw_line_where_column_contains_color() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
        ];
        let input0: Image = Image::try_create(5, 5, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 42, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 42,
        ];
        let input1: Image = Image::try_create(5, 5, pixels1).expect("image");

        // Act
        let mut actual = input0.clone();
        let line_count: u8 = actual.draw_line_where_column_contains_color(&input1, 42, 5).expect("line count");

        // Assert
        assert_eq!(line_count, 2);
        let expected_pixels: Vec<u8> = vec![
            1, 0, 5, 0, 5,
            1, 0, 5, 0, 5,
            1, 0, 5, 0, 5,
            1, 0, 5, 0, 5,
            1, 0, 5, 0, 5,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_draw_line_where_row_or_column_contains_color() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
        ];
        let input0: Image = Image::try_create(5, 5, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 42, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 42,
        ];
        let input1: Image = Image::try_create(5, 5, pixels1).expect("image");

        // Act
        let mut actual = input0.clone();
        actual.draw_line_where_row_or_column_contains_color(&input1, 42, 5).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 5, 0, 5,
            5, 5, 5, 5, 5,
            1, 0, 5, 0, 5,
            1, 0, 5, 0, 5,
            5, 5, 5, 5, 5,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
