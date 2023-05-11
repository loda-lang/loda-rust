use super::{Image, ImageMix, MixMode};

pub trait ImageCompare {
    /// Find differences. The parameter order does not matter. Yields the same result.
    /// 
    /// Set `value=1` where the two images disagree. 
    /// 
    /// Set `value=0` where the two images agree.
    /// 
    /// The two images must have the same size. Otherwise an `Err` is returned.
    fn diff(&self, image: &Image) -> anyhow::Result<Image>;
    
    /// Find differences, considers only pixels with `color` in the `self` image.
    /// 
    /// The parameter order matters.
    /// Similar to doing a subtraction `self - other`, and return what remains.
    /// 
    /// Set `value=1` where the two images disagree. And the `self` pixel is `color`.
    /// 
    /// Set `value=0` where the two images agree. Or the `self` pixel is different than `color`.
    /// 
    /// The two images must have the same size. Otherwise an `Err` is returned.
    fn diff_color(&self, other: &Image, color: u8) -> anyhow::Result<Image>;
}

impl ImageCompare for Image {
    fn diff(&self, image: &Image) -> anyhow::Result<Image> {
        self.mix(image, MixMode::IsDifferent)
    }

    fn diff_color(&self, other: &Image, color: u8) -> anyhow::Result<Image> {
        let self_width: u8 = self.width();
        let self_height: u8 = self.height();
        if self_width != other.width() || self_height != other.height() {
            return Err(anyhow::anyhow!("Both images must have same size. mask: {}x{} image: {}x{}", self_width, self_height, other.width(), other.height()));
        }
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let mut result_image = Image::zero(self_width, self_height);
        for y in 0..(self_height as i32) {
            for x in 0..(self_width as i32) {
                let pixel_value0: u8 = self.get(x, y).unwrap_or(255);
                if pixel_value0 != color {
                    continue;
                }
                let pixel_value1: u8 = other.get(x, y).unwrap_or(255);
                if pixel_value0 != pixel_value1 {
                    let _ = result_image.set(x, y, 1);
                }
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
    fn test_10000_diff_is_same() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.diff(&input).expect("image");

        // Assert
        let expected: Image = Image::zero(3, 2);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_diff_empty() {
        // Arrange
        let input: Image = Image::empty();

        // Act
        let actual: Image = input.diff(&input).expect("image");

        // Assert
        let expected: Image = Image::empty();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_diff_is_different() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input0: Image = Image::try_create(3, 2, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            1, 2, 9,
            9, 5, 6,
        ];
        let input1: Image = Image::try_create(3, 2, pixels1).expect("image");

        // Act
        let actual: Image = input0.diff(&input1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1,
            1, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_diff_error_must_be_same_size() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input0: Image = Image::try_create(3, 2, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            1, 2,
            4, 5,
        ];
        let input1: Image = Image::try_create(2, 2, pixels1).expect("image");

        // Act
        let error = input0.diff(&input1).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("must have same size"), true);
    }

    #[test]
    fn test_20000_diff_color_identical() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 3,
            4, 5, 5,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.diff_color(&input, 5).expect("image");

        // Assert
        let expected: Image = Image::zero(3, 2);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_diff_color_subset() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            5, 5, 3,
            4, 5, 5,
        ];
        let input0: Image = Image::try_create(3, 2, pixels0).expect("image");
        let pixels1: Vec<u8> = vec![
            5, 5, 5,
            4, 5, 5,
        ];
        let input1: Image = Image::try_create(3, 2, pixels1).expect("image");

        // Act
        let actual: Image = input0.diff_color(&input1, 5).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_diff_color_different() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            5, 5, 3,
            4, 5, 5,
        ];
        let input0: Image = Image::try_create(3, 2, pixels0).expect("image");
        let pixels1: Vec<u8> = vec![
            5, 5, 3,
            4, 5, 3,
        ];
        let input1: Image = Image::try_create(3, 2, pixels1).expect("image");

        // Act
        let actual: Image = input0.diff_color(&input1, 5).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 0, 1,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
