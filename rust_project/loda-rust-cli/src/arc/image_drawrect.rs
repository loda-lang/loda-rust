use super::{Image, Rectangle, ImageMask, Histogram, ImageHistogram, ImageMix, MixMode, ImageMaskCount};

pub trait ImageDrawRect {
    /// Draw a filled rectangle
    fn draw_rect_filled(&self, rect: Rectangle, fill_color: u8) -> anyhow::Result<Image>;

    /// Draw a filled rectangle over the bounding box of the mask
    fn draw_rect_filled_mask(&self) -> anyhow::Result<Image>;

    /// Draw non-overlapping filled rectangles over the bounding boxes of each color.
    /// 
    /// An error is returned when encountering overlap.
    /// 
    /// An error is returned when there are no colors, different than the background color.
    fn draw_rect_filled_foreach_color(&self, background_color: u8) -> anyhow::Result<Image>;

    /// Draw a border around a rectangle
    fn draw_rect_border(&self, min_x: i32, min_y: i32, max_x: i32, max_y: i32, border_color: u8) -> anyhow::Result<Image>;
}

impl ImageDrawRect for Image {
    fn draw_rect_filled(&self, rect: Rectangle, fill_color: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Err(anyhow::anyhow!("image must be 1x1 or bigger"));
        }
        if rect.is_empty() {
            return Err(anyhow::anyhow!("fill area must be 1x1 or bigger"));
        }

        // Check that the crop area is inside the image area
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;
        if rect.max_x() > x_max || rect.max_y() > y_max {
            return Err(anyhow::anyhow!("fill area must be inside the image area, but it goes outside"));
        }

        // Draw pixels
        let mut result_image = self.clone();
        for yy in 0..rect.height()  {
            for xx in 0..rect.width() {
                let set_x: i32 = (xx as i32) + rect.min_x();
                let set_y: i32 = (yy as i32) + rect.min_y();
                _ = result_image.set(set_x, set_y, fill_color);
            }
        }
        Ok(result_image)
    }

    fn draw_rect_filled_mask(&self) -> anyhow::Result<Image> {
        let rect: Rectangle = match self.bounding_box() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Cannot determine bounding box"));
            }
        };
        let result_image: Image = self.draw_rect_filled(rect, 1)?;
        Ok(result_image)
    }

    fn draw_rect_filled_foreach_color(&self, background_color: u8) -> anyhow::Result<Image> {
        let mut histogram: Histogram = self.histogram_all();
        histogram.set_counter_to_zero(background_color);
        let pairs: Vec<(u32, u8)> = histogram.pairs_ordered_by_color();
        if pairs.is_empty() {
            return Err(anyhow::anyhow!("There are supposed to be 1 or more colors for this algorithm to make sense"));
        }

        let mut result_image: Image = self.clone();
        let mut visited: Image = Image::zero(self.width(), self.height());
        for (_count, color) in &pairs {
            let mut mask: Image = self.to_mask_where_color_is(*color);
            mask = mask.draw_rect_filled_mask()?;

            let intersection_mask: Image = visited.mix(&mask, MixMode::BooleanAnd)?;
            if intersection_mask.mask_count_one() > 0 {
                return Err(anyhow::anyhow!("The bounding boxes are supposed to be non-overlapping, but they are intersecting. Cannot draw rects"));
            }

            result_image = mask.select_from_image_and_color(&result_image, *color)?;
            visited = visited.mix(&mask, MixMode::BooleanOr)?;
        }
        Ok(result_image)
    }

    fn draw_rect_border(&self, min_x: i32, min_y: i32, max_x: i32, max_y: i32, border_color: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Err(anyhow::anyhow!("image must be 1x1 or bigger"));
        }
        if min_x > max_x || min_y > max_y {
            return Err(anyhow::anyhow!("draw area must be 1x1 or bigger"));
        }

        // Draw pixels around the border
        let mut result_image = self.clone();
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if y > min_y && y < max_y && x > min_x && x < max_x {
                    // skip non-border pixels
                    continue;
                }
                _ = result_image.set(x, y, border_color);
            }
        }
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_draw_rect_filled_1x1() {
        // Arrange
        let input: Image = Image::zero(1, 1);
        let rect = Rectangle::new(0, 0, 1, 1);

        // Act
        let actual: Image = input.draw_rect_filled(rect, 1).expect("image");

        // Assert
        let expected: Image = Image::color(1, 1, 1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_draw_rect_filled_5x5_a() {
        // Arrange
        let input: Image = Image::zero(5, 4);
        let rect = Rectangle::new(1, 1, 3, 2);

        // Act
        let actual: Image = input.draw_rect_filled(rect, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_draw_rect_filled_5x5_b() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5,
            6, 7, 8, 9, 10,
            11, 12, 13, 14, 15,
        ];
        let input: Image = Image::try_create(5, 3, input_pixels).expect("image");
        let rect = Rectangle::new(1, 1, 3, 1);

        // Act
        let actual: Image = input.draw_rect_filled(rect, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5,
            6, 0, 0, 0, 10,
            11, 12, 13, 14, 15,
        ];
        let expected: Image = Image::try_create(5, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_draw_rect_filled_3x2_top() {
        // Arrange
        let input: Image = Image::zero(3, 2);
        let rect = Rectangle::new(0, 0, 3, 1);

        // Act
        let actual: Image = input.draw_rect_filled(rect, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1,
            0, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_draw_rect_filled_3x2_left() {
        // Arrange
        let input: Image = Image::zero(3, 2);
        let rect = Rectangle::new(0, 0, 1, 2);

        // Act
        let actual: Image = input.draw_rect_filled(rect, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0,
            1, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10005_draw_rect_filled_error_empty_self() {
        // Arrange
        let input: Image = Image::empty();
        let rect = Rectangle::new(17, 10, 3, 1);

        // Act
        let error = input.draw_rect_filled(rect, 0).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("image must be 1x1 or bigger"), true);
    }

    #[test]
    fn test_10006_draw_rect_filled_error_empty_rect() {
        // Arrange
        let input: Image = Image::zero(5, 5);
        let rect = Rectangle::empty();

        // Act
        let error = input.draw_rect_filled(rect, 0).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("fill area must be 1x1 or bigger"), true);
    }

    #[test]
    fn test_10007_draw_rect_filled_error_outside_image() {
        // Arrange
        let input: Image = Image::zero(2, 2);
        let rect = Rectangle::new(0, 0, 3, 3);

        // Act
        let error = input.draw_rect_filled(rect, 0).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("but it goes outside"), true);
    }

    #[test]
    fn test_20000_draw_rect_filled_mask() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 5, 0, 0, 0, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 2, 0,
            0, 0, 0, 7, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(8, 5, input_pixels).expect("image");

        // Act
        let actual: Image = input.draw_rect_filled_mask().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 1, 1, 1, 1, 1, 0,
            0, 1, 1, 1, 1, 1, 1, 0,
            0, 1, 1, 1, 1, 1, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(8, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_draw_rect_filled_foreach_color() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            0, 5, 0, 0, 0, 0, 0, 0,
            0, 5, 0, 0, 5, 1, 0, 0,
            0, 0, 5, 0, 0, 0, 2, 0,
            0, 0, 0, 7, 0, 0, 0, 0,
            7, 0, 0, 0, 0, 2, 0, 0,
        ];
        let input: Image = Image::try_create(8, 5, input_pixels).expect("image");

        // Act
        let actual: Image = input.draw_rect_filled_foreach_color(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 5, 5, 5, 5, 0, 0, 0,
            0, 5, 5, 5, 5, 1, 0, 0,
            0, 5, 5, 5, 5, 2, 2, 0,
            7, 7, 7, 7, 0, 2, 2, 0,
            7, 7, 7, 7, 0, 2, 2, 0,
        ];
        let expected: Image = Image::try_create(8, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_draw_rect_border() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 5, 0, 3, 0,
            0, 5, 0, 3, 0,
            0, 5, 0, 3, 0,
            0, 5, 0, 3, 0,
            0, 5, 0, 3, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.draw_rect_border(0, 1, 4, 3, 9).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 5, 0, 3, 0,
            9, 9, 9, 9, 9,
            9, 5, 0, 3, 9,
            9, 9, 9, 9, 9,
            0, 5, 0, 3, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40001_draw_rect_border_outside() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 5, 0, 3, 0,
            0, 5, 0, 3, 0,
            0, 5, 0, 3, 0,
            0, 5, 0, 3, 0,
            0, 5, 0, 3, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.draw_rect_border(-1, 1, 5, 3, 9).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 5, 0, 3, 0,
            9, 9, 9, 9, 9,
            0, 5, 0, 3, 0,
            9, 9, 9, 9, 9,
            0, 5, 0, 3, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
