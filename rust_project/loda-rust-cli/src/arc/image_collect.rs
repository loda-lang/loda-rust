use super::Image;

#[allow(dead_code)]
pub trait ImageCollect {
    /// Populate a vector with pixels where the mask value is non-zero.
    /// 
    /// When encountering a zero mask value, then the pixel is ignored.
    /// 
    /// Returns an empty vector when no pixels were collected.
    fn collect_pixels_as_vec(&self, mask: &Image) -> anyhow::Result<Vec<u8>>;

    /// Extract pixels where the mask value is non-zero.
    /// 
    /// When encountering a zero mask value, then the pixel is ignored.
    /// 
    /// Returns an image that is N pixels wide an 1 pixels tall.
    /// 
    /// Returns an error when no pixels were collected.
    fn collect_pixels_as_image(&self, mask: &Image) -> anyhow::Result<Image>;
}

impl ImageCollect for Image {
    fn collect_pixels_as_vec(&self, mask: &Image) -> anyhow::Result<Vec<u8>> {
        if self.size() != mask.size() {
            return Err(anyhow::anyhow!("Both images must have same size"));
        }
        let mut collected_pixels = Vec::<u8>::new();
        for y in 0..self.height() {
            for x in 0..self.width() {
                let mask_value: u8 = mask.get(x as i32, y as i32).unwrap_or(255);
                if mask_value == 0 {
                    continue;
                }
                let color: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                collected_pixels.push(color);
            }
        }
        Ok(collected_pixels)
    }

    fn collect_pixels_as_image(&self, mask: &Image) -> anyhow::Result<Image> {
        let pixels: Vec<u8> = self.collect_pixels_as_vec(mask)?;
        if pixels.is_empty() {
            return Err(anyhow::anyhow!("Gathered zero pixels"));
        }
        if pixels.len() > (u8::MAX as usize) {
            return Err(anyhow::anyhow!("Gathered more pixels than what can be fitted inside an Image"));
        }
        let width: u8 = pixels.len() as u8;
        let mut result_image = Image::zero(width, 1);
        for x in 0..width {
            let pixel: u8 = pixels[x as usize];
            _ = result_image.set(x as i32, 0, pixel);
        }
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_collect_with_mask() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 5, 2, 5, 1,
            5, 4, 4, 5, 3,
            5, 2, 7, 5, 3,
            5, 5, 5, 3, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        let mask_pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            0, 1, 1, 0, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 1, 0,
        ];
        let mask: Image = Image::try_create(5, 4, mask_pixels).expect("image");

        // Act
        let actual: Vec<u8> = input.collect_pixels_as_vec(&mask).expect("ok");

        // Assert
        assert_eq!(actual, vec![5, 5, 4, 4, 3, 3]);
    }

    #[test]
    fn test_20000_collect_with_mask_as_image() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 5, 2, 5, 1,
            5, 4, 4, 5, 3,
            5, 2, 7, 5, 3,
            5, 5, 5, 3, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        let mask_pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            0, 1, 1, 0, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 1, 0,
        ];
        let mask: Image = Image::try_create(5, 4, mask_pixels).expect("image");

        // Act
        let actual: Image = input.collect_pixels_as_image(&mask).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 4, 4, 3, 3,
        ];
        let expected: Image = Image::try_create(6, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_collect_with_mask_as_image_zero_pixels() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 5, 2, 5, 1,
            5, 4, 4, 5, 3,
            5, 2, 7, 5, 3,
            5, 5, 5, 3, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        let mask_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let mask: Image = Image::try_create(5, 4, mask_pixels).expect("image");

        // Act
        let error = input.collect_pixels_as_image(&mask).expect_err("is supposed to fail");

        // Assert
        let message: String = format!("{:?}", error);
        assert_eq!(message.contains("Gathered zero pixels"), true);
    }
}
