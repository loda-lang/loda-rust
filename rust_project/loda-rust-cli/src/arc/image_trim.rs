use super::{Histogram, Image, ImageHistogram};

pub trait ImageTrim {
    /// Determines the most popular border color and removes the area.
    fn trim(&self) -> anyhow::Result<Image>;

    /// Remove border with the specified color.
    fn trim_color(&self, color_to_be_trimmed: u8) -> anyhow::Result<Image>;
}

impl ImageTrim for Image {
    fn trim(&self) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        
        // Determine what is the most popular pixel value
        // traverses the border of the bitmap, and builds a histogram
        let histogram: Histogram = self.histogram_border();
        let popular_border_pixel_value: u8 = match histogram.most_popular_color() {
            Some(value) => value,
            None => {
                return Ok(Image::empty());
            }
        };
        self.trim_color(popular_border_pixel_value)
    }

    fn trim_color(&self, color_to_be_trimmed: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }

        // Find bounding box
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;
        let mut found_x0: i32 = x_max;
        let mut found_x1: i32 = 0;
        let mut found_y0: i32 = y_max;
        let mut found_y1: i32 = 0;
        for y in 0..=y_max {
            for x in 0..=x_max {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                if pixel_value == color_to_be_trimmed {
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
            return Ok(Image::empty());
        }

        // Width of the object
        let new_width_i32: i32 = found_x1 - found_x0 + 1;
        if new_width_i32 < 1 || new_width_i32 > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. Bounding box coordinates are messed up. new_width_i32: {}", new_width_i32));
        }
        let new_width: u8 = new_width_i32 as u8;

        // Height of the object
        let new_height_i32: i32 = found_y1 - found_y0 + 1;
        if new_height_i32 < 1 || new_height_i32 > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. Bounding box coordinates are messed up. new_height_i32: {}", new_height_i32));
        }
        let new_height: u8 = new_height_i32 as u8;

        // Copy pixels of the object
        let mut bitmap: Image = Image::zero(new_width, new_height);
        for y in found_y0..=found_y1 {
            for x in found_x0..=found_x1 {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                let set_x: i32 = x - found_x0;
                let set_y: i32 = y - found_y0;
                match bitmap.set(set_x, set_y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", set_x, set_y));
                    }
                }
            }
        }
        Ok(bitmap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_trim_color_left() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0,
            5, 1, 2, 0,
            5, 3, 4, 0,
            5, 5, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim_color(5).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 0, 0,
            1, 2, 0,
            3, 4, 0,
            5, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_trim_color_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0,
            5, 1, 2, 0,
            5, 3, 4, 0,
            5, 5, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim_color(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 0,
            5, 1, 2,
            5, 3, 4,
            5, 5, 0,
        ];
        let expected: Image = Image::try_create(3, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_trim_border_with_zeroes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 2, 0,
            0, 3, 4, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![1, 2, 3, 4]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_trim_all_10s() {
        // Arrange
        let pixels: Vec<u8> = vec![
            10, 10, 10, 10, 10,
            10,  1,  2, 10, 10,
            10,  3,  4, 10, 10,
            10, 10, 10, 10, 10,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![1, 2, 3, 4]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_trim_top_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            5, 5, 1, 1,
            5, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![5, 5, 5, 1]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20003_trim_left_right_bottom() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0,
            0, 0, 1, 0,
            0, 0, 1, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::try_create(1, 3, vec![1, 1, 1]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20003_trim_no_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::empty();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20004_trim_1pixel() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 5,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::try_create(1, 1, vec![5]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20005_trim_2pixels() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 5,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = input.clone();
        assert_eq!(actual, expected);
    }
}
