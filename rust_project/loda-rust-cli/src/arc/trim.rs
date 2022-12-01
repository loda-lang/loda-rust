use super::{Image, ImageHistogram};

pub trait ImageTrim {
    fn trim(&self) -> anyhow::Result<Image>;
}

impl ImageTrim for Image {
    fn trim(&self) -> anyhow::Result<Image> {
        let len: usize = (self.width() as usize) * (self.height() as usize);
        if len == 0 {
            return Ok(Image::empty());
        }
        
        // Determine what is the most popular pixel value
        // traverses the border of the bitmap, and builds a histogram
        let histogram: Vec<u32> = self.histogram_border()?;
        let mut found_count: u32 = 0;
        let mut found_value: usize = 0;
        for (pixel_value, number_of_occurences) in histogram.iter().enumerate() {
            if *number_of_occurences > found_count {
                found_count = *number_of_occurences;
                found_value = pixel_value;
            }
        }
        let popular_border_pixel_value: u8 = (found_value & 255) as u8;

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
                if pixel_value == popular_border_pixel_value {
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
        let mut bitmap: Image = Image::zeroes(new_width, new_height);
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
    fn test_10000_trim_border_with_zeroes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 2, 0,
            0, 3, 4, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.trim().expect("bitmap");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![1, 2, 3, 4]).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_trim_all_10s() {
        // Arrange
        let pixels: Vec<u8> = vec![
            10, 10, 10, 10, 10,
            10,  1,  2, 10, 10,
            10,  3,  4, 10, 10,
            10, 10, 10, 10, 10,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.trim().expect("bitmap");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![1, 2, 3, 4]).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_trim_top_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            5, 5, 1, 1,
            5, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.trim().expect("bitmap");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![5, 5, 5, 1]).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_trim_left_right_bottom() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0,
            0, 0, 1, 0,
            0, 0, 1, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.trim().expect("bitmap");

        // Assert
        let expected: Image = Image::try_create(1, 3, vec![1, 1, 1]).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_trim_no_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.trim().expect("bitmap");

        // Assert
        let expected: Image = Image::empty();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_trim_1pixel() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 5,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.trim().expect("bitmap");

        // Assert
        let expected: Image = Image::try_create(1, 1, vec![5]).expect("bitmap");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10005_trim_2pixels() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 5,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let actual: Image = input.trim().expect("bitmap");

        // Assert
        let expected: Image = input.clone();
        assert_eq!(actual, expected);
    }
}
