use super::Image;

pub trait ImageMask {
    /// Convert to a mask image by converting 0 to 0 and converting [1..255] to 1.
    fn to_mask(&self) -> Image;

    /// Inverts a mask image by converting 0 to 1 and converting [1..255] to 0.
    fn invert_mask(&self) -> Image;

    /// Pick pixels from one image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.
    fn select_from_image(&self, image: &Image, default_color: u8) -> anyhow::Result<Image>;
    
    /// Pick pixels from two images. When the mask is 0 then pick `image_a`. When the mask is [1..255] then pick from `image_b`.
    fn select_from_images(&self, image_a: &Image, image_b: &Image) -> anyhow::Result<Image>;
}

impl ImageMask for Image {
    fn to_mask(&self) -> Image {
        if self.is_empty() {
            return Image::empty();
        }
        let mut image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let get_color: u8 = self.get(x, y).unwrap_or(255);
                let set_color: u8;
                if get_color > 0 {
                    set_color = 1;
                } else {
                    set_color = 0;
                }
                let _ = image.set(x, y, set_color);
            }
        }
        return image;
    }

    fn invert_mask(&self) -> Image {
        if self.is_empty() {
            return Image::empty();
        }
        let mut image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let get_color: u8 = self.get(x, y).unwrap_or(255);
                let set_color: u8;
                if get_color > 0 {
                    set_color = 0;
                } else {
                    set_color = 1;
                }
                let _ = image.set(x, y, set_color);
            }
        }
        return image;
    }

    fn select_from_image(&self, image: &Image, default_color: u8) -> anyhow::Result<Image> {
        if self.width() != image.width() || self.height() != image.height() {
            return Err(anyhow::anyhow!("Both images must have same size. mask: {}x{} image: {}x{}", self.width(), self.height(), image.width(), image.height()));
        }
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let mut result_image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let mask_value: u8 = self.get(x, y).unwrap_or(255);
                let color: u8;
                if mask_value > 0 {
                    color = image.get(x, y).unwrap_or(255)
                } else {
                    color = default_color;
                }
                let _ = result_image.set(x, y, color);
            }
        }
        return Ok(result_image);
    }

    fn select_from_images(&self, image_a: &Image, image_b: &Image) -> anyhow::Result<Image> {
        if self.width() != image_a.width() || self.width() != image_b.width() {
            return Err(anyhow::anyhow!("All images must have same size. mask: {}x{} image_a: {}x{} image_b: {}x{}", self.width(), self.height(), image_a.width(), image_a.height(), image_b.width(), image_b.height()));
        }
        if self.height() != image_a.height() || self.height() != image_b.height() {
            return Err(anyhow::anyhow!("All images must have same size. mask: {}x{} image_a: {}x{} image_b: {}x{}", self.width(), self.height(), image_a.width(), image_a.height(), image_b.width(), image_b.height()));
        }
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let mut result_image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let mask_value: u8 = self.get(x, y).unwrap_or(255);
                let color: u8;
                if mask_value == 0 {
                    color = image_a.get(x, y).unwrap_or(255)
                } else {
                    color = image_b.get(x, y).unwrap_or(255)
                }
                let _ = result_image.set(x, y, color);
            }
        }
        return Ok(result_image);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_to_mask() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 1, 0,
            0, 2, 0,
            0, 3, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: Image = input.to_mask();

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 1, 0,
            0, 1, 0,
            0, 1, 0,
            0, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_invert_mask() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 1, 0,
            0, 2, 0,
            0, 3, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: Image = input.invert_mask();

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
            1, 1, 1,
        ];
        let expected: Image = Image::try_create(3, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_select_from_image() {
        // Arrange
        let mask_pixels: Vec<u8> = vec![
            1, 1, 0, 0,
            1, 1, 0, 0,
            0, 0, 1, 1,
            0, 0, 1, 1,
        ];
        let mask: Image = Image::try_create(4, 4, mask_pixels).expect("image");
        let image_pixels: Vec<u8> = vec![
            0, 1, 255, 255,
            2, 3, 255, 255,
            255, 255, 4, 5,
            255, 255, 6, 7,
        ];
        let image: Image = Image::try_create(4, 4, image_pixels).expect("image");

        // Act
        let actual: Image = mask.select_from_image(&image, 9).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 9, 9,
            2, 3, 9, 9,
            9, 9, 4, 5,
            9, 9, 6, 7,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_select_from_images() {
        // Arrange
        let mask_pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 1, 0,
            0, 1, 1, 0,
            0, 0, 0, 0,
        ];
        let mask: Image = Image::try_create(4, 4, mask_pixels).expect("image");
        let image_a_pixels: Vec<u8> = vec![
            8, 9, 8, 9,
            9, 8, 9, 8,
            8, 9, 8, 9,
            9, 8, 9, 8,
        ];
        let image_a: Image = Image::try_create(4, 4, image_a_pixels).expect("image");
        let image_b_pixels: Vec<u8> = vec![
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 2,
            2, 1, 2, 1,
        ];
        let image_b: Image = Image::try_create(4, 4, image_b_pixels).expect("image");
            
        // Act
        let actual: Image = mask.select_from_images(&image_a, &image_b).expect("image");
            
        // Assert
        let expected_pixels: Vec<u8> = vec![
            8, 9, 8, 9,
            9, 1, 2, 8,
            8, 2, 1, 9,
            9, 8, 9, 8,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
