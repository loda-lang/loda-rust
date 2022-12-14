use super::Image;

pub trait ImageMask {
    /// Convert to a mask image by converting 0 to 0 and converting [1..255] to 1.
    fn to_mask(&self) -> Image;

    /// Inverts a mask image by converting 0 to 1 and converting [1..255] to 0.
    fn invert_mask(&self) -> Image;
}

impl ImageMask for Image {
    fn to_mask(&self) -> Image {
        if self.is_empty() {
            return Image::empty();
        }
        let mut image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let get_color: u8 = self.get(x, y).unwrap_or(255);
                let set_color: u8;
                if get_color > 0 {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_to_mask() {
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
        let actual: Image = input.to_mask();

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
    fn test_20000_invert_mask() {
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
}
