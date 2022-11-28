use super::Bitmap;

pub trait BitmapResize {
    fn resize(&self, width: u8, height: u8) -> anyhow::Result<Bitmap>;
}

impl BitmapResize for Bitmap {
    fn resize(&self, width: u8, height: u8) -> anyhow::Result<Bitmap> {
        let len: usize = (width as usize) * (height as usize);
        if len == 0 {
            return Ok(Bitmap::empty());
        }
        let mut bitmap = Bitmap::zeroes(width, height);
        if self.width() == 0 || self.height() == 0 {
            return Ok(bitmap);
        }
        let original_width: i32 = self.width() as i32;
        let original_height: i32 = self.height() as i32;
        let new_width: i32 = width as i32;
        let new_height: i32 = height as i32;
        for y in 0..height {
            for x in 0..width {
                let xx: i32 = (x as i32) * original_width / new_width;
                let yy: i32 = (y as i32) * original_height / new_height;
                let pixel_value = self.get(xx, yy)
                    .unwrap_or(0);
                match bitmap.set(x as i32, y as i32, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("cannot set pixel at ({}, {})", x, y));
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
    use crate::arc::BitmapTryCreate;

    #[test]
    fn test_10000_make_width_smaller() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 2, 2, 2, 3, 3, 3,
            4, 4, 4, 5, 5, 5, 6, 6, 6,
        ];
        let input: Bitmap = Bitmap::try_create(9, 2, pixels).expect("bitmap");

        // Act
        let output: Bitmap = input.resize(3, 2).expect("bitmap");

        // Assert
        let expected = Bitmap::create_raw(3, 2, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10001_make_height_smaller() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 4,
            1, 4,
            1, 4,
            2, 5,
            2, 5,
            2, 5,
            3, 6,
            3, 6,
            3, 6,
        ];
        let input: Bitmap = Bitmap::try_create(2, 9, pixels).expect("bitmap");

        // Act
        let output: Bitmap = input.resize(2, 3).expect("bitmap");

        // Assert
        let expected = Bitmap::create_raw(2, 3, vec![1, 4, 2, 5, 3, 6]);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20000_make_width_bigger() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let input: Bitmap = Bitmap::try_create(2, 2, pixels).expect("bitmap");

        // Act
        let output: Bitmap = input.resize(4, 2).expect("bitmap");

        // Assert
        let expected = Bitmap::create_raw(4, 2, vec![1, 1, 2, 2, 3, 3, 4, 4]);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20001_make_height_bigger() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let input: Bitmap = Bitmap::try_create(2, 2, pixels).expect("bitmap");

        // Act
        let output: Bitmap = input.resize(2, 4).expect("bitmap");

        // Assert
        let expected = Bitmap::create_raw(2, 4, vec![1, 2, 1, 2, 3, 4, 3, 4]);
        assert_eq!(output, expected);
    }
}
