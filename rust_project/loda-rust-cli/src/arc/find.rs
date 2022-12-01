use super::Image;

pub trait BitmapFind {
    fn find_exact(&self, needle: &Image) -> anyhow::Result<Option<(u8, u8)>>;
}

impl BitmapFind for Image {
    fn find_exact(&self, needle: &Image) -> anyhow::Result<Option<(u8, u8)>> {
        if self.is_empty() {
            return Ok(None);
        }
        if needle.is_empty() {
            return Ok(None);
        }
        // search pattern bigger than image
        if needle.width() > self.width() || needle.height() > self.height() {
            return Ok(None);
        }
        
        let x_max: i32 = (self.width() as i32) - (needle.width() as i32);
        let y_max: i32 = (self.height() as i32) - (needle.height() as i32);
        if x_max < 0 || y_max < 0 {
            return Err(anyhow::anyhow!("Integrity error. x_max and y_max is not supposed to be negative. x_max: {} y_max: {}", x_max, y_max));
        }

        // Compare with the pattern
        for y in 0..=y_max {
            for x in 0..=x_max {
                let mut skip = false;
                for needle_y in 0..(needle.height() as i32) {
                    for needle_x in 0..(needle.width() as i32) {
                        let needle_pixel_value: u8 = needle.get(needle_x, needle_y).unwrap_or(255);
                        let bitmap_pixel_value: u8 = self.get(x + needle_x, y + needle_y).unwrap_or(255);
                        if needle_pixel_value != bitmap_pixel_value {
                            skip = true;
                            break;
                        }
                    }
                    if skip {
                        break;
                    }
                }
                if skip {
                    continue;
                }
                // Found the pattern
                let found_x = (x & 255) as u8;
                let found_y = (y & 255) as u8;
                return Ok(Some((found_x, found_y)));
            }
        }

        // Traversed all pixels, but didn't find the pattern
        return Ok(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::BitmapTryCreate;

    #[test]
    fn test_10000_find_exact_big() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 1, 2, 0,
            0, 0, 3, 4, 0,
            5, 0, 0, 0, 0,
        ];
        let input_bitmap: Image = Image::try_create(5, 4, input_pixels).expect("bitmap");
        let find_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let find_bitmap: Image = Image::try_create(2, 2, find_pixels).expect("bitmap");

        // Act
        let actual: Option<(u8, u8)> = input_bitmap.find_exact(&find_bitmap).expect("some position");

        // Assert
        assert_eq!(actual, Some((2, 1)));
    }

    #[test]
    fn test_10001_find_exact_bottom_left() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 6,
        ];
        let input_bitmap: Image = Image::try_create(5, 4, input_pixels).expect("bitmap");
        let find_bitmap: Image = Image::try_create(1, 1, vec![6]).expect("bitmap");

        // Act
        let actual: Option<(u8, u8)> = input_bitmap.find_exact(&find_bitmap).expect("some position");

        // Assert
        assert_eq!(actual, Some((4, 3)));
    }

    #[test]
    fn test_10002_find_exact_none() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 1,
        ];
        let input_bitmap: Image = Image::try_create(5, 4, input_pixels).expect("bitmap");
        let find_bitmap: Image = Image::try_create(1, 1, vec![255]).expect("bitmap");

        // Act
        let actual: Option<(u8, u8)> = input_bitmap.find_exact(&find_bitmap).expect("some position");

        // Assert
        assert_eq!(actual, None);
    }
}
