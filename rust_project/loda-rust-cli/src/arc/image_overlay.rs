use super::Image;

pub trait ImageOverlay {
    fn overlay_with_mask_color(&self, other: &Image, mask_color: u8) -> anyhow::Result<Image>;
}

impl ImageOverlay for Image {
    fn overlay_with_mask_color(&self, other: &Image, mask_color: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        if self.width() != other.width() {
            return Err(anyhow::anyhow!("ImageOverlay: Both images must have same size. Width is different."));
        }
        if self.height() != other.height() {
            return Err(anyhow::anyhow!("ImageOverlay: Both images must have same size. Height is different."));
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
                        return Err(anyhow::anyhow!("Unable to set pixel inside the result bitmap"));
                    }
                }
            }
        }
        return Ok(result_image);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use crate::arc::ImageSymmetry;

    #[test]
    fn test_10000_overlay_simple() {
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
    fn test_10001_overlay_advanced() {
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
}
