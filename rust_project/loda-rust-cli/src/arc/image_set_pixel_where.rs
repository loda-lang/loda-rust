use super::Image;

pub trait ImageSetPixelWhere {
    /// Replace the pixel value at the positions where two images agree on the same color.
    /// 
    /// If the two images doesn't agree then leave the pixel untouched.
    /// 
    /// Suppress setting the pixel if the color happen to be `color_must_be_different_than`.
    fn set_pixel_where_two_images_agree(&mut self, image0: &Image, image1: &Image, color_must_be_different_than: u8) -> anyhow::Result<()>;
}

impl ImageSetPixelWhere for Image {
    fn set_pixel_where_two_images_agree(&mut self, image0: &Image, image1: &Image, color_must_be_different_than: u8) -> anyhow::Result<()> {
        let width: u8 = self.width();
        if width != image0.width() {
            return Err(anyhow::anyhow!("Expected image0.width to be the same as self.width."));
        }
        if width != image1.width() {
            return Err(anyhow::anyhow!("Expected image1.width to be the same as self.width."));
        }
        let height: u8 = self.height();
        if height != image0.height() {
            return Err(anyhow::anyhow!("Expected image0.height to be the same as self.height."));
        }
        if height != image1.height() {
            return Err(anyhow::anyhow!("Expected image1.height to be the same as self.height."));
        }
        for y in 0..(height as i32) {
            for x in 0..(width as i32) {
                let color0: u8 = image0.get(x, y).unwrap_or(255);
                let color1: u8 = image1.get(x, y).unwrap_or(255);
                if color0 == color1 && color0 != color_must_be_different_than {
                    let _ = self.set(x, y, color0);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_set_pixel_where_two_images_agree_stripes() {
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
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
        ];
        let input1: Image = Image::try_create(5, 5, pixels1).expect("image");

        // Act
        let mut actual = Image::color(5, 5, 9);
        actual.set_pixel_where_two_images_agree(&input0, &input1, 0).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 9, 1, 9, 1,
            9, 9, 9, 9, 9,
            1, 9, 1, 9, 1,
            9, 9, 9, 9, 9,
            1, 9, 1, 9, 1,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_set_pixel_where_two_images_agree_identical() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 1, 1, 2, 2,
            1, 1, 1, 2, 2,
            1, 1, 1, 3, 3,
            4, 4, 4, 3, 3,
            4, 4, 4, 3, 3,
        ];
        let input0: Image = Image::try_create(5, 5, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            1, 1, 1, 2, 2,
            1, 1, 1, 2, 2,
            1, 1, 1, 3, 3,
            4, 4, 4, 3, 3,
            4, 4, 4, 3, 3,
        ];
        let input1: Image = Image::try_create(5, 5, pixels1).expect("image");

        // Act
        let mut actual = Image::color(5, 5, 9);
        actual.set_pixel_where_two_images_agree(&input0, &input1, 0).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 2, 2,
            1, 1, 1, 2, 2,
            1, 1, 1, 3, 3,
            4, 4, 4, 3, 3,
            4, 4, 4, 3, 3,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
