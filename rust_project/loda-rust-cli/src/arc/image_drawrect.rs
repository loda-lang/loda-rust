use super::{Image, Rectangle};

pub trait ImageDrawRect {
    /// Draw a rectangle with a single color
    fn fill_inside_rect(&self, rect: Rectangle, color: u8) -> anyhow::Result<Image>;
}

impl ImageDrawRect for Image {
    fn fill_inside_rect(&self, rect: Rectangle, color: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Err(anyhow::anyhow!("fill_inside_rect: image must be 1x1 or bigger"));
        }
        if rect.is_empty() {
            return Err(anyhow::anyhow!("fill_inside_rect: fill area must be 1x1 or bigger"));
        }

        // Check that the crop area is inside the image area
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;
        if rect.max_x() > x_max || rect.max_y() > y_max {
            return Err(anyhow::anyhow!("fill_inside_rect: fill area must be inside the image area, but it goes outside"));
        }

        // Draw pixels
        let mut result_image = self.clone();
        for yy in 0..rect.height()  {
            for xx in 0..rect.width() {
                let set_x: i32 = (xx as i32) + rect.min_x();
                let set_y: i32 = (yy as i32) + rect.min_y();
                _ = result_image.set(set_x, set_y, color);
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
    fn test_10000_fill_inside_rect_1x1() {
        // Arrange
        let input: Image = Image::zero(1, 1);
        let rect = Rectangle::new(0, 0, 1, 1);

        // Act
        let actual: Image = input.fill_inside_rect(rect, 1).expect("image");

        // Assert
        let expected: Image = Image::color(1, 1, 1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_fill_inside_rect_5x5_a() {
        // Arrange
        let input: Image = Image::zero(5, 4);
        let rect = Rectangle::new(1, 1, 3, 2);

        // Act
        let actual: Image = input.fill_inside_rect(rect, 1).expect("image");

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
    fn test_10002_fill_inside_rect_5x5_b() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5,
            6, 7, 8, 9, 10,
            11, 12, 13, 14, 15,
        ];
        let input: Image = Image::try_create(5, 3, input_pixels).expect("image");
        let rect = Rectangle::new(1, 1, 3, 1);

        // Act
        let actual: Image = input.fill_inside_rect(rect, 0).expect("image");

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
    fn test_10003_fill_inside_rect_error_empty_self() {
        // Arrange
        let input: Image = Image::empty();
        let rect = Rectangle::new(17, 10, 3, 1);

        // Act
        let error = input.fill_inside_rect(rect, 0).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("image must be 1x1 or bigger"), true);
    }

    #[test]
    fn test_10004_fill_inside_rect_error_empty_rect() {
        // Arrange
        let input: Image = Image::zero(5, 5);
        let rect = Rectangle::empty();

        // Act
        let error = input.fill_inside_rect(rect, 0).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("fill area must be 1x1 or bigger"), true);
    }

    #[test]
    fn test_10005_fill_inside_rect_error_outside_image() {
        // Arrange
        let input: Image = Image::zero(2, 2);
        let rect = Rectangle::new(0, 0, 3, 3);

        // Act
        let error = input.fill_inside_rect(rect, 0).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("but it goes outside"), true);
    }
}
