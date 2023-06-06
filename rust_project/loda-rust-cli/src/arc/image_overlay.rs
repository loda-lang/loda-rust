use super::{Image, ImageMix, MixMode, ImageSize};

pub trait ImageOverlay {
    /// Z-stack two images on top of each other.
    /// 
    /// Both images must have the same size, otherwise an error is returned.
    fn overlay_with_mask_color(&self, other: &Image, mask_color: u8) -> anyhow::Result<Image>;

    /// Z-Stack multiple images on top of each other.
    /// 
    /// All the images must have the same size, otherwise an error is returned.
    /// 
    /// And the image size must be at least 1x1, otherwise an error is returned.
    fn overlay_images(mask_color: u8, images: &Vec<Image>) -> anyhow::Result<Image>;

    /// Copy rectangle of pixels
    fn overlay_with_position(&self, other: &Image, x: i32, y: i32) -> anyhow::Result<Image>;

    /// Copy pixels where the mask is not zero.
    /// 
    /// Ignore the pixels where the mask is zero.
    fn overlay_with_mask_and_position(&self, other: &Image, mask: &Image, x: i32, y: i32) -> anyhow::Result<Image>;
}

impl ImageOverlay for Image {
    fn overlay_with_mask_color(&self, other: &Image, mask_color: u8) -> anyhow::Result<Image> {
        let mode = MixMode::PickColor1WhenColor0IsDifferent { color0_filter: mask_color };
        other.mix(self, mode)
    }

    fn overlay_images(mask_color: u8, images: &Vec<Image>) -> anyhow::Result<Image> {
        if images.len() < 2 {
            return Err(anyhow::anyhow!("overlay_images: Expected at least 2 images"));
        }
        let mut result_image: Image = images[0].clone();
        if result_image.is_empty() {
            return Err(anyhow::anyhow!("overlay_images: The images must be 1x1 or bigger"));
        }
        let size: ImageSize = result_image.size();
        for (index, image) in images.iter().enumerate() {
            if index == 0 {
                continue;
            }
            if image.size() != size {
                return Err(anyhow::anyhow!("overlay_images: Expected all images to have the same size"));
            }
            result_image = result_image.overlay_with_mask_color(image, mask_color)?;
        }
        Ok(result_image)
    }

    fn overlay_with_position(&self, other: &Image, x: i32, y: i32) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        if other.is_empty() {
            return Ok(self.clone());
        }
        let mut image: Image = self.clone();
        for yy in 0..(other.height() as i32) {
            for xx in 0..(other.width() as i32) {
                let pixel_value: u8 = other.get(xx, yy).unwrap_or(255); 
                let set_x = x + xx;
                let set_y = y + yy;
                let _ = image.set(set_x, set_y, pixel_value);
            }
        }
        Ok(image)
    }

    fn overlay_with_mask_and_position(&self, other: &Image, mask: &Image, x: i32, y: i32) -> anyhow::Result<Image> {
        if other.size() != mask.size() {
            return Err(anyhow::anyhow!("overlay_with_mask_and_position: Expected other.size to be the same as mask.size"));
        }
        if self.is_empty() {
            return Ok(Image::empty());
        }
        if other.is_empty() {
            return Ok(self.clone());
        }
        let mut image: Image = self.clone();
        for yy in 0..(other.height() as i32) {
            for xx in 0..(other.width() as i32) {
                let mask_value: u8 = mask.get(xx, yy).unwrap_or(255); 
                if mask_value == 0 {
                    continue;
                }
                let pixel_value: u8 = other.get(xx, yy).unwrap_or(255); 
                let set_x = x + xx;
                let set_y = y + yy;
                let _ = image.set(set_x, set_y, pixel_value);
            }
        }
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use crate::arc::ImageSymmetry;

    #[test]
    fn test_10000_overlay_with_mask_color_simple() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
            5, 6,
            0, 0,
            0, 0,
            0, 0,
            0, 0,
            0, 0,
        ];
        let input: Image = Image::try_create(2, 8, pixels).expect("image");
        let other: Image = input.flip_y().expect("image");

        // Act
        let actual: Image = input.overlay_with_mask_color(&other, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
            5, 6,
            0, 0,
            0, 0,
            5, 6,
            3, 4,
            1, 2,
        ];
        let expected: Image = Image::try_create(2, 8, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_overlay_with_mask_color_advanced() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 5, 5, 5, 5,
            5, 1, 5, 5, 5,
            5, 5, 1, 5, 5,
            5, 5, 5, 1, 5,
            5, 5, 5, 5, 1,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let other: Image = input.flip_y().expect("image");

        // Act
        let actual: Image = input.overlay_with_mask_color(&other, 5).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 5, 5, 5, 1,
            5, 1, 5, 1, 5,
            5, 5, 1, 5, 5,
            5, 1, 5, 1, 5,
            1, 5, 5, 5, 1,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_overlay_with_mask_color_and_overlap() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 2,
            3, 4,
            5, 6,
            0, 0,
            0, 0,
            0, 0,
            0, 0,
            0, 0,
        ];
        let input0: Image = Image::try_create(2, 8, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            42, 0,
            42, 0,
            42, 0,
            42, 0,
            42, 0,
            5, 6,
            3, 4,
            1, 2,
        ];
        let input1: Image = Image::try_create(2, 8, pixels1).expect("image");

        // Act
        let actual: Image = input0.overlay_with_mask_color(&input1, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            42, 2,
            42, 4,
            42, 6,
            42, 0,
            42, 0,
            5, 6,
            3, 4,
            1, 2,
        ];
        let expected: Image = Image::try_create(2, 8, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_overlay_images() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            0, 0, 0,
            9, 9, 9,
        ];
        let input0: Image = Image::try_create(3, 2, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            9, 9, 1,
            9, 1, 1,
        ];
        let input1: Image = Image::try_create(3, 2, pixels1).expect("image");

        let pixels2: Vec<u8> = vec![
            9, 9, 2,
            9, 9, 9,
        ];
        let input2: Image = Image::try_create(3, 2, pixels2).expect("image");

        let images: Vec<Image> = vec![input0, input1, input2];

        // Act
        let actual: Image = Image::overlay_images(9, &images).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 2,
            9, 1, 1,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_overlay_with_position_inside() {
        // Arrange
        let a_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
        ];
        let a: Image = Image::try_create(5, 5, a_pixels).expect("image");
        let b_pixels: Vec<u8> = vec![
            2, 2, 2,
            2, 2, 2,
            2, 2, 2, 
        ];
        let b: Image = Image::try_create(3, 3, b_pixels).expect("image");

        // Act
        let actual: Image = a.overlay_with_position(&b, 1, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 2, 2, 2, 1,
            1, 2, 2, 2, 1,
            1, 2, 2, 2, 1,
            1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30001_overlay_with_position_clip_top_left() {
        // Arrange
        let a_pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1, 1,
            0, 0, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        let a: Image = Image::try_create(6, 4, a_pixels).expect("image");
        let b_pixels: Vec<u8> = vec![
            2, 2, 2,
            2, 2, 2,
            2, 2, 2, 
        ];
        let b: Image = Image::try_create(3, 3, b_pixels).expect("image");

        // Act
        let actual: Image = a.overlay_with_position(&b, -1, -1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            2, 2, 1, 1, 1, 1,
            2, 2, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(6, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30002_overlay_with_position_clip_bottom_right() {
        // Arrange
        let a_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 0,
            1, 1, 1, 0,
        ];
        let a: Image = Image::try_create(4, 5, a_pixels).expect("image");
        let b_pixels: Vec<u8> = vec![
            2, 3,
            4, 5,
            6, 7,
        ];
        let b: Image = Image::try_create(2, 3, b_pixels).expect("image");

        // Act
        let actual: Image = a.overlay_with_position(&b, 3, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 2,
            1, 1, 1, 4,
        ];
        let expected: Image = Image::try_create(4, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30003_overlay_with_position_outside() {
        let a_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let a: Image = Image::try_create(2, 2, a_pixels).expect("image");
        let b_pixels: Vec<u8> = vec![
            9, 9,
            9, 9,
        ];
        let b: Image = Image::try_create(2, 2, b_pixels).expect("image");

        {
            let actual: Image = a.overlay_with_position(&b, 2, 0).expect("image");
            assert_eq!(actual, a);
        }
        {
            let actual: Image = a.overlay_with_position(&b, -2, 0).expect("image");
            assert_eq!(actual, a);
        }
        {
            let actual: Image = a.overlay_with_position(&b, 0, 2).expect("image");
            assert_eq!(actual, a);
        }
        {
            let actual: Image = a.overlay_with_position(&b, 0, -2).expect("image");
            assert_eq!(actual, a);
        }
    }

    #[test]
    fn test_40000_overlay_with_mask_and_position() {
        // Arrange
        let a_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 5, 5, 5, 5,
            7, 7, 7, 7, 7,
            6, 6, 6, 6, 6,
            6, 6, 6, 6, 6,
        ];
        let a: Image = Image::try_create(5, 5, a_pixels).expect("image");
        let b_pixels: Vec<u8> = vec![
            2, 3, 4,
            5, 6, 7,
            8, 9, 10,
        ];
        let b: Image = Image::try_create(3, 3, b_pixels).expect("image");
        let mask_pixels: Vec<u8> = vec![
            0, 1, 0,
            1, 1, 1,
            0, 1, 0,
        ];
        let mask: Image = Image::try_create(3, 3, mask_pixels).expect("image");

        // Act
        let actual: Image = a.overlay_with_mask_and_position(&b, &mask, 1, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 5, 3, 5, 5,
            7, 5, 6, 7, 7,
            6, 6, 9, 6, 6,
            6, 6, 6, 6, 6,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
