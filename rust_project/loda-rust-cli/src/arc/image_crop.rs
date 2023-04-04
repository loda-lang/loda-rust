use super::{Image, Rectangle};

pub trait ImageCrop {
    /// Extract an area from the image
    fn crop(&self, rect: Rectangle) -> anyhow::Result<Image>;
}

impl ImageCrop for Image {
    fn crop(&self, rect: Rectangle) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Err(anyhow::anyhow!("crop: image must be 1x1 or bigger"));
        }
        if rect.is_empty() {
            return Err(anyhow::anyhow!("crop: crop area must be 1x1 or bigger"));
        }

        // Check that the crop area is inside the image area
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;
        let x1: i32 = (rect.x() as i32) + (rect.width() as i32) - 1;
        let y1: i32 = (rect.y() as i32) + (rect.height() as i32) - 1;
        if x1 > x_max || y1 > y_max {
            return Err(anyhow::anyhow!("crop: crop area must be inside the image area, but it goes outside"));
        }

        // Copy pixels
        let mut result_image = Image::zero(rect.width(), rect.height());
        for yy in 0..rect.height()  {
            for xx in 0..rect.width() {
                let get_x: i32 = (xx as i32) + (rect.x() as i32);
                let get_y: i32 = (yy as i32) + (rect.y() as i32);
                let pixel_value: u8 = self.get(get_x, get_y).unwrap_or(255);
                _ = result_image.set(xx as i32, yy as i32, pixel_value);
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
    fn test_10000_crop_simple() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0,
            5, 1, 2, 0,
            5, 3, 4, 0,
            5, 5, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let rect = Rectangle::new(1, 1, 2, 2);

        // Act
        let actual: Image = input.crop(rect).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_crop_tiny() {
        // Arrange
        let pixels: Vec<u8> = vec![42];
        let input: Image = Image::try_create(1, 1, pixels).expect("image");
        let rect = Rectangle::new(0, 0, 1, 1);

        // Act
        let actual: Image = input.crop(rect).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![42];
        let expected: Image = Image::try_create(1, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_crop_bottom_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 5, 5,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");
        let rect = Rectangle::new(2, 1, 2, 1);

        // Act
        let actual: Image = input.crop(rect).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5,
        ];
        let expected: Image = Image::try_create(2, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_crop_bottom_left() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0,
            5, 0, 
            5, 0,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");
        let rect = Rectangle::new(0, 1, 1, 2);

        // Act
        let actual: Image = input.crop(rect).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 
            5,
        ];
        let expected: Image = Image::try_create(1, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_crop_error_empty_self() {
        // Arrange
        let rect = Rectangle::new(0, 0, 5, 5);

        // Act
        let error = Image::empty().crop(rect).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("image must be 1x1 or bigger"), true);
    }

    #[test]
    fn test_10005_crop_error_empty_croparea() {
        // Arrange
        let input: Image = Image::zero(5, 5);
        let rect = Rectangle::new(0, 0, 0, 0);

        // Act
        let error = input.crop(rect).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("crop area must be 1x1 or bigger"), true);
    }

    #[test]
    fn test_10006_crop_error_croparea_outside_imagearea_x() {
        // Arrange
        let input: Image = Image::zero(1, 1);
        let rect = Rectangle::new(0, 0, 2, 1);

        // Act
        let error = input.crop(rect).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("it goes outside"), true);
    }

    #[test]
    fn test_10007_crop_error_croparea_outside_imagearea_y() {
        // Arrange
        let input: Image = Image::zero(1, 1);
        let rect = Rectangle::new(0, 0, 1, 2);

        // Act
        let error = input.crop(rect).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("it goes outside"), true);
    }
}
