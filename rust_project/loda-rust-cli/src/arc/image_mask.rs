use super::Image;

pub trait ImageMask {
    /// Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
    fn to_mask_where_color_is(&self, color: u8) -> Image;

    /// Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
    fn to_mask_where_color_is_different(&self, color: u8) -> Image;

    /// Convert to a mask image by converting `pixel_color >= threshold_color` to 1 and converting anything else to to 0.
    fn to_mask_where_color_is_equal_or_greater_than(&self, threshold_color: u8) -> Image;

    /// Inverts a mask image by converting 0 to 1 and converting [1..255] to 0.
    fn invert_mask(&self) -> Image;

    /// Pick pixels. When the mask is 0 then pick the `color`. When the mask is [1..255] then pick from the image.
    fn select_from_color_and_image(&self, color: u8, image: &Image) -> anyhow::Result<Image>;
    
    /// Pick pixels from two images. When the mask is 0 then pick `image_a`. When the mask is [1..255] then pick from `image_b`.
    fn select_from_images(&self, image_a: &Image, image_b: &Image) -> anyhow::Result<Image>;

    /// The smallest box that can contain the mask.
    fn bounding_box(&self) -> Option<(u8,u8,u8,u8)>;
}

impl ImageMask for Image {
    fn to_mask_where_color_is(&self, color: u8) -> Image {
        if self.is_empty() {
            return Image::empty();
        }
        let mut image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let get_color: u8 = self.get(x, y).unwrap_or(255);
                let set_color: u8;
                if get_color == color {
                    set_color = 1;
                } else {
                    set_color = 0;
                }
                let _ = image.set(x, y, set_color);
            }
        }
        return image;
    }

    fn to_mask_where_color_is_different(&self, color: u8) -> Image {
        if self.is_empty() {
            return Image::empty();
        }
        let mut image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let get_color: u8 = self.get(x, y).unwrap_or(255);
                let set_color: u8;
                if get_color != color {
                    set_color = 1;
                } else {
                    set_color = 0;
                }
                let _ = image.set(x, y, set_color);
            }
        }
        return image;
    }

    fn to_mask_where_color_is_equal_or_greater_than(&self, threshold_color: u8) -> Image {
        if self.is_empty() {
            return Image::empty();
        }
        let mut image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let get_color: u8 = self.get(x, y).unwrap_or(255);
                let set_color: u8;
                if get_color >= threshold_color {
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

    fn select_from_color_and_image(&self, color: u8, image: &Image) -> anyhow::Result<Image> {
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
                let set_color: u8;
                if mask_value > 0 {
                    set_color = image.get(x, y).unwrap_or(255)
                } else {
                    set_color = color;
                }
                let _ = result_image.set(x, y, set_color);
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

    fn bounding_box(&self) -> Option<(u8,u8,u8,u8)> {
        if self.is_empty() {
            return None;
        }

        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;
        let mut found_x0: i32 = x_max;
        let mut found_x1: i32 = 0;
        let mut found_y0: i32 = y_max;
        let mut found_y1: i32 = 0;
        for y in 0..=y_max {
            for x in 0..=x_max {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                if pixel_value == 0 {
                    continue;
                }

                // grow the bounding box
                found_x0 = i32::min(found_x0, x);
                found_x1 = i32::max(found_x1, x);
                found_y0 = i32::min(found_y0, y);
                found_y1 = i32::max(found_y1, y);
            }
        }

        if found_x0 > found_x1 || found_y0 > found_y1 {
            return None;
        }

        // X position
        if found_x0 < 0 || found_x0 > (u8::MAX as i32) {
            return None;
        }
        let new_x = found_x0 as u8;

        // Y position
        if found_y0 < 0 || found_y0 > (u8::MAX as i32) {
            return None;
        }
        let new_y = found_y0 as u8;

        // Width of the object
        let new_width_i32: i32 = found_x1 - found_x0 + 1;
        if new_width_i32 < 1 || new_width_i32 > (u8::MAX as i32) {
            return None;
        }
        let new_width: u8 = new_width_i32 as u8;

        // Height of the object
        let new_height_i32: i32 = found_y1 - found_y0 + 1;
        if new_height_i32 < 1 || new_height_i32 > (u8::MAX as i32) {
            return None;
        }
        let new_height: u8 = new_height_i32 as u8;

        return Some((new_x, new_y, new_width, new_height));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_to_mask_where_color_is() {
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
        let actual: Image = input.to_mask_where_color_is(0);

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
    fn test_20000_to_mask_where_color_is_different() {
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
        let actual: Image = input.to_mask_where_color_is_different(0);

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
    fn test_30000_to_mask_where_color_is_equal_or_greater_than() {
        // Arrange
        let pixels: Vec<u8> = vec![
            2, 1, 0,  1, 2,
            1, 3, 6,  9, 1,
            0, 4, 7, 10, 0,
            1, 5, 8, 11, 1,
            2, 1, 0,  1, 2,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.to_mask_where_color_is_equal_or_greater_than(3);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 1, 1, 1, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_invert_mask() {
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
    fn test_50000_select_from_image() {
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
        let actual: Image = mask.select_from_color_and_image(9, &image).expect("image");

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
    fn test_60000_select_from_images() {
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

    #[test]
    fn test_70000_bounding_box() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1,
            0, 0, 0,
            0, 0, 0,
            0, 0, 0,
            1, 0, 0,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: (u8,u8,u8,u8) = input.bounding_box().expect("bounding box");

        // Assert
        assert_eq!(actual, (0, 0, 3, 5));
    }

    #[test]
    fn test_70001_bounding_box() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 0, 1,
            0, 1, 1,
            0, 1, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: (u8,u8,u8,u8) = input.bounding_box().expect("bounding box");

        // Assert
        assert_eq!(actual, (1, 1, 2, 3));
    }

    #[test]
    fn test_70002_bounding_box() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 0, 1,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: (u8,u8,u8,u8) = input.bounding_box().expect("bounding box");

        // Assert
        assert_eq!(actual, (2, 1, 1, 1));
    }

    #[test]
    fn test_70003_bounding_box() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0,
            0, 0,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let actual: Option<(u8,u8,u8,u8)> = input.bounding_box();

        // Assert
        assert_eq!(actual, None);
    }
}
