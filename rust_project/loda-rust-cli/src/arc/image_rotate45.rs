//! Rotate an image by 45 degrees.
use super::Image;

pub trait ImageRotate45 {
    /// Rotate an image by 45 degrees.
    fn rotate_45(&self, fill_color: u8) -> anyhow::Result<Image>;
}

impl ImageRotate45 for Image {
    fn rotate_45(&self, fill_color: u8) -> anyhow::Result<Image> {
        if self.width() <= 1 && self.height() <= 1 {
            // No point in processing an empty image or a 1x1 image.
            return Ok(self.clone());
        }

        let combined_u16: u16 = self.width() as u16 + self.height() as u16 - 1;
        if combined_u16 > 255 {
            return Err(anyhow::anyhow!("Unable to skew image. The combined width and height is too large: {}", combined_u16));
        }

        // Rotate by 45 degrees
        let rads: f32 = std::f32::consts::PI / 4.0;

        let source_center_x: f32 = ((self.width() - 1) as f32) / 2.0;
        let source_center_y: f32 = ((self.height() - 1) as f32) / 2.0;
        let dest_center_x: f32 = ((combined_u16 - 1) as f32) / 2.0;
        let dest_center_y: f32 = ((combined_u16 - 1) as f32) / 2.0;

        // Increase the spacing between the points in the grid from 1 to sqrt(2)
        let scale: f32 = 1.41421356; // sqrt(2)

        let mut image = Image::color(combined_u16 as u8, combined_u16 as u8, fill_color);
        for get_y in 0..self.height() {
            for get_x in 0..self.width() {
                let pixel_value: u8 = self.get(get_x as i32, get_y as i32).unwrap_or(255);

                let x = (get_x as f32) - source_center_x;
                let y = (get_y as f32) - source_center_y;

                let rotated_x: f32 = (rads.cos() * x + rads.sin() * y) * scale;
                let rotated_y: f32 = (rads.cos() * y - rads.sin() * x) * scale;
                
                let set_x: i32 = (dest_center_x + rotated_x).round() as i32;
                let set_y: i32 = (dest_center_y + rotated_y).round() as i32;
                match image.set(set_x, set_y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result image", set_x, y));
                    }
                }
            }
        }
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_rotate_square() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 3, 0, 0,
            0, 2, 0, 6, 0,
            1, 0, 5, 0, 9,
            0, 4, 0, 8, 0,
            0, 0, 7, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_rotate_landscape_onerow() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
        ];
        let input: Image = Image::try_create(3, 1, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 3,
            0, 2, 0,
            1, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_rotate_landscape_tworows() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 3, 0,
            0, 2, 0, 6,
            1, 0, 5, 0,
            0, 4, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_rotate_portrait_onecolumn() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 
            2, 
            3,
        ];
        let input: Image = Image::try_create(1, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 2, 0,
            0, 0, 3,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_rotate_portrait_twocolumns() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 4,
            2, 5,
            3, 6,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 4, 0, 0,
            1, 0, 5, 0,
            0, 2, 0, 6,
            0, 0, 3, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
