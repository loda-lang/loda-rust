use super::Image;

pub trait ImageOverlay {
    fn overlay_with_mask_color(&self, other: &Image, mask_color: u8) -> anyhow::Result<Image>;

    /// Copy rectangle of pixels
    fn overlay_with_position(&self, other: &Image, x: i32, y: i32) -> anyhow::Result<Image>;
}

impl ImageOverlay for Image {
    fn overlay_with_mask_color(&self, other: &Image, mask_color: u8) -> anyhow::Result<Image> {
        if self.size() != other.size() {
            return Err(anyhow::anyhow!("overlay_with_operation: Both images must have same size."));
        }
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let mut result_image: Image = self.clone();
        for y in 0..self.height() {
            for x in 0..self.width() {
                let pixel_value: u8 = other.get(x as i32, y as i32).unwrap_or(255); 
                if pixel_value == mask_color {
                    continue;
                }
                match result_image.set(x as i32, y as i32, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("overlay_with_mask_color: Unable to set pixel inside the result bitmap"));
                    }
                }
            }
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
    fn test_20000_overlay_with_position_inside() {
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
    fn test_20001_overlay_with_position_clip_top_left() {
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
    fn test_20002_overlay_with_position_clip_bottom_right() {
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
    fn test_20003_overlay_with_position_outside() {
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
}
