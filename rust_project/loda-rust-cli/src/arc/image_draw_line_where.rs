use super::{Histogram, Image, ImageRotate, ImageHistogram, ImageReplaceColor};

pub trait ImageDrawLineWhere {
    /// Draw a horizontal line if the `mask` contains one or more non-zero pixels.
    /// 
    /// Returns the number of rows that was drawn.
    fn draw_line_row_where_mask_is_nonzero(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<u8>;
    
    /// Draw a vertical line if the `mask` contains one or more non-zero pixels.
    /// 
    /// Returns the number of columns that was drawn.
    fn draw_line_column_where_mask_is_nonzero(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<u8>;
    
    /// Shoot out lines in all directions
    /// 
    /// Draw horizontal lines and vertical lines where the `mask` contains one or more non-zero pixels.
    /// 
    /// Returns tuple with `(number of columns, number of rows)` that was drawn.
    fn draw_line_where_mask_is_nonzero(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<(u8,u8)>;

    /// Draw lines between the top-most pixel and the bottom-most pixel.
    /// 
    /// In the mask, it finds the top-most non-zero pixel, and the bottom-most non-zero pixel.
    /// 
    /// A line is drawn when there are 2 or more different coordinates.
    /// 
    /// No line is drawn when there are fewer than 2 coordinates.
    /// 
    /// Mask values of zero are ignored.
    /// 
    /// Returns the number of columns that was drawn.
    fn draw_line_between_top_bottom(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<u8>;

    /// Draw lines between the left-most pixel and the right-most pixel.
    /// 
    /// In the mask, it finds the left-most non-zero pixel, and the right-most non-zero pixel.
    /// 
    /// A line is drawn when there are 2 or more different coordinates.
    /// 
    /// No line is drawn when there are fewer than 2 coordinates.
    /// 
    /// Mask values of zero are ignored.
    /// 
    /// Returns the number of rows that was drawn.
    fn draw_line_between_left_right(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<u8>;

    /// Draw lines between the outer-most pixels that are in the same column/row.
    /// 
    /// In rows/columns that contains 2 or more non-zero pixels in the mask.
    /// 
    /// Mask values of zero are ignored.
    /// 
    /// Returns tuple with `(number of columns, number of rows)` that was drawn.
    fn draw_line_between_top_bottom_and_left_right(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<(u8,u8)>;

    /// Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.
    /// 
    /// In rows/columns that contains both `color0` pixels and `color1` pixels. If one of the colors is missing then no line is drawn.
    /// 
    /// Returns tuple with `(number of columns, number of rows)` that was drawn.
    fn draw_line_connecting_two_colors(&mut self, color0: u8, color1: u8, line_color: u8) -> anyhow::Result<(u8,u8)>;
}

impl ImageDrawLineWhere for Image {
    fn draw_line_row_where_mask_is_nonzero(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<u8> {
        if self.size() != mask.size() {
            return Err(anyhow::anyhow!("Expected mask.size to be the same as self.size"));
        }
        let width = self.width() as i32;
        let height = self.height() as i32;
        let mut number_of_lines: u8 = 0;
        for y in 0..height {
            let mut ignore: bool = true;
            for x in 0..width {
                let color: u8 = mask.get(x, y).unwrap_or(255);
                if color > 0 {
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

    fn draw_line_column_where_mask_is_nonzero(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<u8> {
        if self.size() != mask.size() {
            return Err(anyhow::anyhow!("Expected mask.size to be the same as self.size"));
        }
        let width = self.width() as i32;
        let height = self.height() as i32;
        let mut number_of_lines: u8 = 0;
        for x in 0..width {
            let mut ignore: bool = true;
            for y in 0..height {
                let color: u8 = mask.get(x, y).unwrap_or(255);
                if color > 0 {
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

    fn draw_line_where_mask_is_nonzero(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<(u8,u8)> {
        let count_columns: u8 = self.draw_line_column_where_mask_is_nonzero(mask, line_color)?;
        let count_rows: u8 = self.draw_line_row_where_mask_is_nonzero(mask, line_color)?;
        Ok((count_columns, count_rows))
    }

    fn draw_line_between_top_bottom(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<u8> {
        if self.size() != mask.size() {
            return Err(anyhow::anyhow!("Expected mask.size to be the same as self.size"));
        }
        let mut number_of_lines: u8 = 0;
        for x in 0..(self.width() as i32) {
            let mut min_value: i32 = 255;
            let mut max_value: i32 = 0;
            let mut number_of_markers: u8 = 0;
            for y in 0..(self.height() as i32) {
                let mask_value: u8 = mask.get(x, y).unwrap_or(0);
                if mask_value == 0 {
                    continue;
                }
                min_value = min_value.min(y);
                max_value = max_value.max(y);
                number_of_markers += 1;
            }
            if number_of_markers < 2 || min_value >= max_value {
                continue;
            }
            number_of_lines += 1;
            for y in min_value..=max_value {
                _ = self.set(x, y, line_color);
            }
        }
        Ok(number_of_lines)
    }

    fn draw_line_between_left_right(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<u8> {
        if self.size() != mask.size() {
            return Err(anyhow::anyhow!("Expected mask.size to be the same as self.size"));
        }
        let mut number_of_lines: u8 = 0;
        for y in 0..(self.height() as i32) {
            let mut min_value: i32 = 255;
            let mut max_value: i32 = 0;
            let mut number_of_markers: u8 = 0;
            for x in 0..(self.width() as i32) {
                let mask_value: u8 = mask.get(x, y).unwrap_or(0);
                if mask_value == 0 {
                    continue;
                }
                min_value = min_value.min(x);
                max_value = max_value.max(x);
                number_of_markers += 1;
            }
            if number_of_markers < 2 || min_value >= max_value {
                continue;
            }
            number_of_lines += 1;
            for x in min_value..=max_value {
                _ = self.set(x, y, line_color);
            }
        }
        Ok(number_of_lines)
    }

    fn draw_line_between_top_bottom_and_left_right(&mut self, mask: &Image, line_color: u8) -> anyhow::Result<(u8,u8)> {
        let count_columns: u8 = self.draw_line_between_top_bottom(mask, line_color)?;
        let count_rows: u8 = self.draw_line_between_left_right(mask, line_color)?;
        Ok((count_columns, count_rows))
    }

    fn draw_line_connecting_two_colors(&mut self, color0: u8, color1: u8, line_color: u8) -> anyhow::Result<(u8,u8)> {
        // Draw with a color value that isn't clashing with color0 or color1.
        let mut draw_color: u8 = line_color;
        if line_color == color0 && line_color == color1 {
            let histogram: Histogram = self.histogram_all();
            let unused_color: u8 = match histogram.unused_color() {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Unable to find an unused color"));
                }
            };
            draw_color = unused_color;
        }

        let mut self_rotated: Image = self.rotate_cw()?;
        let original_rotated: Image = self_rotated.clone();

        // Draw columns
        let count_columns: u8 = inner_draw_line_connecting_two_colors(&mut self_rotated, &original_rotated, color0, color1, draw_color)?;
        let mut self_rotated_back: Image = self_rotated.rotate_ccw()?;

        // Draw rows
        let count_rows: u8 = inner_draw_line_connecting_two_colors(&mut self_rotated_back, self, color0, color1, draw_color)?;

        // Restore from the clashing draw_color to the line_color
        if line_color != draw_color {
            self_rotated_back = self_rotated_back.replace_color(draw_color, line_color)?;
        }

        self.set_image(self_rotated_back);
        Ok((count_columns, count_rows))
    }

}

fn inner_draw_line_connecting_two_colors(image: &mut Image, original_image: &Image, color0: u8, color1: u8, line_color: u8) -> anyhow::Result<u8> {
    if image.size() != original_image.size() {
        return Err(anyhow::anyhow!("Expected image.size to be the same as original_image.size"));
    }
    let mut number_of_lines: u8 = 0;
    for y in 0..(image.height() as i32) {
        let mut min_value: i32 = 255;
        let mut max_value: i32 = 0;
        let mut count_color0: u8 = 0;
        let mut count_color1: u8 = 0;
        for x in 0..(image.width() as i32) {
            let color: u8 = image.get(x, y).unwrap_or(0);
            if color != color0 && color != color1 {
                continue;
            }
            if color == color0 {
                count_color0 += 1;
            }
            if color == color1 {
                count_color1 += 1;
            }
            min_value = min_value.min(x);
            max_value = max_value.max(x);
        }
        if count_color0 < 1 || count_color1 < 1 || min_value >= max_value {
            continue;
        }
        number_of_lines += 1;
        for x in min_value..=max_value {
            let color: u8 = original_image.get(x, y).unwrap_or(0);
            if color == color0 || color == color1 {
                continue;
            }
            _ = image.set(x, y, line_color);
        }
    }
    Ok(number_of_lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_draw_line_row_where_mask_is_nonzero() {
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
        let line_count: u8 = actual.draw_line_row_where_mask_is_nonzero(&input1, 5).expect("line count");

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
    fn test_20000_draw_line_column_where_mask_is_nonzero() {
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
        let line_count: u8 = actual.draw_line_column_where_mask_is_nonzero(&input1, 5).expect("line count");

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
    fn test_30000_draw_line_where_mask_is_nonzero() {
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
            0, 0, 42, 0, 0,
            0, 0, 42, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 42,
        ];
        let input1: Image = Image::try_create(5, 5, pixels1).expect("image");

        // Act
        let mut actual = input0.clone();
        let (count_columns, count_rows) = actual.draw_line_where_mask_is_nonzero(&input1, 5).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 5, 5, 5, 5,
            1, 0, 5, 0, 5,
            1, 0, 5, 0, 5,
            5, 5, 5, 5, 5,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
        assert_eq!(count_columns, 2);
        assert_eq!(count_rows, 3);
    }

    #[test]
    fn test_40000_draw_line_between_top_bottom() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            0, 0, 0, 7, 0,
            0, 7, 0, 0, 0,
            0, 0, 0, 5, 5,
            0, 7, 0, 0, 0,
            7, 0, 0, 7, 0,
        ];
        let input0: Image = Image::try_create(5, 5, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 1, 0,
            0, 1, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 1, 0, 0, 0,
            1, 0, 0, 1, 0,
        ];
        let input1: Image = Image::try_create(5, 5, pixels1).expect("image");

        // Act
        let mut actual = input0.clone();
        actual.draw_line_between_top_bottom(&input1, 3).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 3, 0,
            0, 3, 0, 3, 0,
            0, 3, 0, 3, 5,
            0, 3, 0, 3, 0,
            7, 0, 0, 3, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40001_draw_line_between_top_bottom() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            0, 0, 0, 7, 0,
            0, 7, 0, 0, 7,
            0, 0, 0, 5, 5,
            0, 7, 0, 0, 0,
            7, 0, 0, 7, 0,
        ];
        let input0: Image = Image::try_create(5, 5, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 1, 0,
            0, 1, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 1, 0, 0, 0,
            1, 0, 0, 1, 0,
        ];
        let input1: Image = Image::try_create(5, 5, pixels1).expect("image");

        // Act
        let mut actual = input0.clone();
        actual.draw_line_between_left_right(&input1, 3).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 7, 0,
            0, 3, 3, 3, 3,
            0, 0, 0, 5, 5,
            0, 7, 0, 0, 0,
            3, 3, 3, 3, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40002_draw_line_between_top_bottom() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            0, 7, 0, 7, 0,
            0, 7, 0, 0, 7,
            0, 0, 0, 5, 5,
            0, 7, 0, 0, 0,
            7, 0, 0, 7, 0,
        ];
        let input0: Image = Image::try_create(5, 5, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            0, 1, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 1, 0, 0, 0,
            1, 0, 0, 1, 0,
        ];
        let input1: Image = Image::try_create(5, 5, pixels1).expect("image");

        // Act
        let mut actual = input0.clone();
        let (count_columns, count_rows) = actual.draw_line_between_top_bottom_and_left_right(&input1, 3).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 3, 3, 3, 0,
            0, 3, 3, 3, 3,
            0, 3, 0, 3, 5,
            0, 3, 0, 3, 0,
            3, 3, 3, 3, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
        assert_eq!(count_columns, 2);
        assert_eq!(count_rows, 3);
    }

    #[test]
    fn test_50000_draw_line_connecting_two_colors_different_color_values() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 7, 7, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            7, 0, 0, 5, 5, 5, 0, 0, 0,
            0, 0, 0, 5, 5, 5, 0, 0, 0,
            3, 0, 0, 5, 5, 5, 0, 0, 7,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 7, 0, 0, 0, 7,
        ];
        let input: Image = Image::try_create(9, 9, pixels).expect("image");

        // Act
        let mut actual = input.clone();
        let (count_columns, count_rows) = actual.draw_line_connecting_two_colors(5, 7, 1).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 7, 7, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            7, 1, 1, 5, 5, 5, 0, 0, 0,
            0, 0, 0, 5, 5, 5, 0, 0, 0,
            3, 0, 0, 5, 5, 5, 1, 1, 7,
            0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 7, 0, 0, 0, 7,
        ];
        let expected: Image = Image::try_create(9, 9, expected_pixels).expect("image");
        assert_eq!(actual, expected);
        assert_eq!(count_columns, 1);
        assert_eq!(count_rows, 2);
    }

    #[test]
    fn test_50001_draw_line_connecting_two_colors_same_color_value() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 7, 0,
            0, 0, 0, 3, 3, 3, 0, 0, 0,
            0, 7, 0, 3, 3, 3, 0, 7, 0,
            0, 0, 0, 5, 5, 5, 0, 0, 0,
            0, 0, 0, 7, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 7, 0, 0, 0,
        ];
        let input: Image = Image::try_create(9, 7, pixels).expect("image");

        // Act
        let mut actual = input.clone();
        let (count_columns, count_rows) = actual.draw_line_connecting_two_colors(7, 7, 1).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 7, 0,
            0, 0, 0, 3, 3, 1, 0, 1, 0,
            0, 7, 1, 1, 1, 1, 1, 7, 0,
            0, 0, 0, 5, 5, 1, 0, 0, 0,
            0, 0, 0, 7, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 7, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(9, 7, expected_pixels).expect("image");
        assert_eq!(actual, expected);
        assert_eq!(count_columns, 2);
        assert_eq!(count_rows, 1);
    }

    #[test]
    fn test_50002_draw_line_connecting_two_colors_same_as_line_color() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 7, 0,
            0, 0, 0, 3, 3, 3, 0, 0, 0,
            0, 7, 0, 3, 3, 3, 0, 7, 0,
            0, 0, 0, 5, 5, 5, 0, 0, 0,
            0, 0, 0, 7, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 7, 0, 0, 0,
        ];
        let input: Image = Image::try_create(9, 7, pixels).expect("image");

        // Act
        let mut actual = input.clone();
        let (count_columns, count_rows) = actual.draw_line_connecting_two_colors(7, 7, 7).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 0, 0, 7, 0, 7, 0,
            0, 0, 0, 3, 3, 7, 0, 7, 0,
            0, 7, 7, 7, 7, 7, 7, 7, 0,
            0, 0, 0, 5, 5, 7, 0, 0, 0,
            0, 0, 0, 7, 0, 7, 0, 0, 0,
            0, 0, 0, 0, 0, 7, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(9, 7, expected_pixels).expect("image");
        assert_eq!(actual, expected);
        assert_eq!(count_columns, 2);
        assert_eq!(count_rows, 1);
    }
}
