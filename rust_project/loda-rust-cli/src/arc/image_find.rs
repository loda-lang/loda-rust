use super::Image;

pub trait ImageFind {
    fn find_exact(&self, needle: &Image) -> anyhow::Result<Option<(u8, u8)>>;

    fn find_exact_with_ignore_mask(&self, needle: &Image, ignore_mask: &Image) -> anyhow::Result<Option<(u8, u8)>>;

    fn count_occurrences(&self, needle: &Image) -> anyhow::Result<u16>;
}

impl ImageFind for Image {
    fn find_exact(&self, needle: &Image) -> anyhow::Result<Option<(u8, u8)>> {
        let ignore_mask = Image::zero(self.width(), self.height());
        self.find_exact_with_ignore_mask(needle, &ignore_mask)
    }

    fn find_exact_with_ignore_mask(&self, needle: &Image, ignore_mask: &Image) -> anyhow::Result<Option<(u8, u8)>> {
        let self_width: u8 = self.width();
        let self_height: u8 = self.height();
        if self_width != ignore_mask.width() || self_height != ignore_mask.height() {
            return Err(anyhow::anyhow!("find_exact_with_ignore_mask: Expected ignore_mask to have same size as self"));
        }

        if self.is_empty() {
            return Ok(None);
        }
        if needle.is_empty() {
            return Ok(None);
        }
        // search pattern bigger than image
        if needle.width() > self_width || needle.height() > self_height {
            return Ok(None);
        }
        
        let x_max: i32 = (self_width as i32) - (needle.width() as i32);
        let y_max: i32 = (self_height as i32) - (needle.height() as i32);
        if x_max < 0 || y_max < 0 {
            return Err(anyhow::anyhow!("find_exact_with_ignore_mask: Integrity error. x_max and y_max is not supposed to be negative. x_max: {} y_max: {}", x_max, y_max));
        }

        // Compare with the pattern
        for y in 0..=y_max {
            for x in 0..=x_max {
                let mut skip = false;
                for needle_y in 0..(needle.height() as i32) {
                    for needle_x in 0..(needle.width() as i32) {
                        let ignore_mask_value: u8 = ignore_mask.get(x + needle_x, y + needle_y).unwrap_or(255);
                        if ignore_mask_value > 0 {
                            skip = true;
                            break;
                        }
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

    fn count_occurrences(&self, needle: &Image) -> anyhow::Result<u16> {
        if self.is_empty() {
            return Err(anyhow::anyhow!("count_occurrences: input size must be 1x1 or greater"));
        }
        if needle.is_empty() {
            return Err(anyhow::anyhow!("count_occurrences: needle size must be 1x1 or greater"));
        }
        let mut ignore_mask = Image::zero(self.width(), self.height());
        let mut count: u16 = 0;
        loop {
            let position: Option<(u8, u8)> = self.find_exact_with_ignore_mask(needle, &ignore_mask)?;
            match position {
                Some((x, y)) => {
                    _ = ignore_mask.set(x as i32, y as i32, 1);
                    count += 1;
                },
                None => {
                    break;
                }
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_find_exact_big() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 1, 2, 0,
            0, 0, 3, 4, 0,
            5, 0, 0, 0, 0,
        ];
        let input_bitmap: Image = Image::try_create(5, 4, input_pixels).expect("image");
        let find_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let find_bitmap: Image = Image::try_create(2, 2, find_pixels).expect("image");

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
        let input_bitmap: Image = Image::try_create(5, 4, input_pixels).expect("image");
        let find_bitmap: Image = Image::try_create(1, 1, vec![6]).expect("image");

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
        let input_bitmap: Image = Image::try_create(5, 4, input_pixels).expect("image");
        let find_bitmap: Image = Image::try_create(1, 1, vec![255]).expect("image");

        // Act
        let actual: Option<(u8, u8)> = input_bitmap.find_exact(&find_bitmap).expect("some position");

        // Assert
        assert_eq!(actual, None);
    }

    #[test]
    fn test_20000_count_occurrences_multiple4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");
        let needle: Image = Image::color(1, 1, 1);

        // Act
        let actual: u16 = input.count_occurrences(&needle).expect("u16");

        // Assert
        assert_eq!(actual, 4);
    }

    #[test]
    fn test_20001_count_occurrences_multiple2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            1, 2,
            1, 2,
            1, 2,
        ];
        let input: Image = Image::try_create(2, 4, pixels).expect("image");
        let needle_pixels: Vec<u8> = vec![
            1, 2,
            1, 2,
            1, 2,
        ];
        let needle: Image = Image::try_create(2, 3, needle_pixels).expect("image");

        // Act
        let actual: u16 = input.count_occurrences(&needle).expect("u16");

        // Assert
        assert_eq!(actual, 2);
    }

    #[test]
    fn test_20002_count_occurrences_once() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 2, 3, 0,
            0, 4, 0, 5, 0,
            0, 6, 7, 8, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let needle_pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 0, 5,
            6, 7, 8,
        ];
        let needle: Image = Image::try_create(3, 3, needle_pixels).expect("image");

        // Act
        let actual: u16 = input.count_occurrences(&needle).expect("u16");

        // Assert
        assert_eq!(actual, 1);
    }

    #[test]
    fn test_20003_count_occurrences_zero() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 2, 3, 0,
            0, 4, 0, 5, 0,
            0, 6, 7, 8, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let needle_pixels: Vec<u8> = vec![
            1, 2, 1,
            2, 1, 2,
        ];
        let needle: Image = Image::try_create(3, 2, needle_pixels).expect("image");

        // Act
        let actual: u16 = input.count_occurrences(&needle).expect("u16");

        // Assert
        assert_eq!(actual, 0);
    }

    #[test]
    fn test_20004_count_occurrences_empty_input() {
        // Arrange
        let input: Image = Image::empty();
        let needle: Image = Image::zero(1, 1);

        // Act
        let error = input.count_occurrences(&needle).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("input size"), true);
    }

    #[test]
    fn test_20005_count_occurrences_empty_needle() {
        // Arrange
        let input: Image = Image::zero(1, 1);
        let needle: Image = Image::empty();

        // Act
        let error = input.count_occurrences(&needle).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("needle size"), true);
    }

}
