use super::{Image, Rectangle};

pub trait ImageCrop {
    /// Extract an area from the image.
    /// 
    /// The crop area can be outside the image area. In that case, the outside pixels are filled with the given color.
    fn crop_outside(&self, x: i32, y: i32, width: u8, height: u8, outside_color: u8) -> anyhow::Result<Image>;

    /// Extract an area from the image.
    /// 
    /// The crop area must be inside the image area.
    fn crop(&self, rect: Rectangle) -> anyhow::Result<Image>;
}

impl ImageCrop for Image {
    fn crop_outside(&self, x: i32, y: i32, width: u8, height: u8, outside_color: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Err(anyhow::anyhow!("crop_outside: image must be 1x1 or bigger"));
        }
        if width == 0 || height == 0 {
            return Err(anyhow::anyhow!("crop: crop area must be 1x1 or bigger"));
        }

        // Copy pixels
        let mut result_image = Image::zero(width, height);
        for yy in 0..height  {
            for xx in 0..width {
                let get_x: i32 = (xx as i32) + x;
                let get_y: i32 = (yy as i32) + y;
                let pixel_value: u8 = self.get(get_x, get_y).unwrap_or(outside_color);
                _ = result_image.set(xx as i32, yy as i32, pixel_value);
            }
        }
        Ok(result_image)
    }

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
        if rect.max_x() > x_max || rect.max_y() > y_max {
            return Err(anyhow::anyhow!("crop: crop area must be inside the image area, but it goes outside"));
        }

        // Copy pixels
        self.crop_outside(rect.min_x(), rect.min_y(), rect.width(), rect.height(), 255)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_crop_outside() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0,
            5, 1, 2, 0,
            5, 3, 4, 0,
            5, 5, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.crop_outside(-1, 0, 3, 2, 250).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            250, 5, 5,
            250, 5, 1,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_crop_outside() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0,
            5, 1, 2, 0,
            5, 3, 4, 0,
            5, 5, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.crop_outside(2, 0, 3, 2, 250).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 250,
            2, 0, 250,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_crop_outside() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0,
            5, 1, 2, 0,
            5, 3, 4, 0,
            5, 5, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.crop_outside(0, -1, 2, 3, 250).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            250, 250,
            5, 5,
            5, 1,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_crop_outside() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0,
            5, 1, 2, 0,
            5, 3, 4, 0,
            5, 5, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.crop_outside(0, 2, 2, 3, 250).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 3,
            5, 5,
            250, 250,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_crop_simple() {
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
    fn test_20001_crop_tiny() {
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
    fn test_20002_crop_bottom_right() {
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
    fn test_20003_crop_bottom_left() {
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
    fn test_20004_crop_error_empty_self() {
        // Arrange
        let rect = Rectangle::new(0, 0, 5, 5);

        // Act
        let error = Image::empty().crop(rect).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("image must be 1x1 or bigger"), true);
    }

    #[test]
    fn test_20005_crop_error_empty_croparea() {
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
    fn test_20006_crop_error_croparea_outside_imagearea_x() {
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
    fn test_20007_crop_error_croparea_outside_imagearea_y() {
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
