use super::Image;

pub trait ImageFill {
    /// Replace color with another color
    /// 
    /// Visit 4 neighbors around a pixel.
    fn flood_fill4(&mut self, x: i32, y: i32, from_color: u8, to_color: u8);

    /// Replace color with another color
    /// 
    /// Visit 8 neighbors around a pixel.
    fn flood_fill8(&mut self, x: i32, y: i32, from_color: u8, to_color: u8);

    /// Build a mask of connected pixels that has the same color
    /// 
    /// Visit 4 neighbors around a pixel.
    fn mask_flood_fill4(&mut self, image: &Image, x: i32, y: i32, color: u8);

    /// Build a mask of connected pixels that has the same color
    /// 
    /// Visit 8 neighbors around a pixel.
    fn mask_flood_fill8(&mut self, image: &Image, x: i32, y: i32, color: u8);
}

impl ImageFill for Image {
    fn flood_fill4(&mut self, x: i32, y: i32, from_color: u8, to_color: u8) {
        if x < 0 || y < 0 || x >= (self.width() as i32) || y >= (self.height() as i32) {
            return;
        }
        let value: u8 = self.get(x, y).unwrap_or(255);
        if value != from_color {
            return;
        }
        let _ = self.set(x, y, to_color);
        self.flood_fill4(x-1, y, from_color, to_color);
        self.flood_fill4(x+1, y, from_color, to_color);
        self.flood_fill4(x, y-1, from_color, to_color);
        self.flood_fill4(x, y+1, from_color, to_color);
    }

    fn flood_fill8(&mut self, x: i32, y: i32, from_color: u8, to_color: u8) {
        if x < 0 || y < 0 || x >= (self.width() as i32) || y >= (self.height() as i32) {
            return;
        }
        let value: u8 = self.get(x, y).unwrap_or(255);
        if value != from_color {
            return;
        }
        let _ = self.set(x, y, to_color);
        self.flood_fill8(x-1, y-1, from_color, to_color);
        self.flood_fill8(x, y-1, from_color, to_color);
        self.flood_fill8(x+1, y-1, from_color, to_color);
        self.flood_fill8(x-1, y, from_color, to_color);
        self.flood_fill8(x+1, y, from_color, to_color);
        self.flood_fill8(x-1, y+1, from_color, to_color);
        self.flood_fill8(x, y+1, from_color, to_color);
        self.flood_fill8(x+1, y+1, from_color, to_color);
    }

    fn mask_flood_fill4(&mut self, image: &Image, x: i32, y: i32, color: u8) {
        assert!(self.width() == image.width());
        assert!(self.height() == image.height());
        if x < 0 || y < 0 || x >= (self.width() as i32) || y >= (self.height() as i32) {
            return;
        }
        let mask_value: u8 = self.get(x, y).unwrap_or(255);
        if mask_value > 0 {
            // already visited
            return;
        }
        let value: u8 = image.get(x, y).unwrap_or(255);
        if value != color {
            return;
        }
        let _ = self.set(x, y, 1); // flag as visited
        self.mask_flood_fill4(image, x-1, y, color);
        self.mask_flood_fill4(image, x+1, y, color);
        self.mask_flood_fill4(image, x, y-1, color);
        self.mask_flood_fill4(image, x, y+1, color);
    }

    fn mask_flood_fill8(&mut self, image: &Image, x: i32, y: i32, color: u8) {
        assert!(self.width() == image.width());
        assert!(self.height() == image.height());
        if x < 0 || y < 0 || x >= (self.width() as i32) || y >= (self.height() as i32) {
            return;
        }
        let mask_value: u8 = self.get(x, y).unwrap_or(255);
        if mask_value > 0 {
            // already visited
            return;
        }
        let value: u8 = image.get(x, y).unwrap_or(255);
        if value != color {
            return;
        }
        let _ = self.set(x, y, 1); // flag as visited
        self.mask_flood_fill8(image, x-1, y-1, color);
        self.mask_flood_fill8(image, x, y-1, color);
        self.mask_flood_fill8(image, x+1, y-1, color);
        self.mask_flood_fill8(image, x-1, y, color);
        self.mask_flood_fill8(image, x+1, y, color);
        self.mask_flood_fill8(image, x-1, y+1, color);
        self.mask_flood_fill8(image, x, y+1, color);
        self.mask_flood_fill8(image, x+1, y+1, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_flood_fill4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.flood_fill4(0, 0, 5, 3);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3, 3,
            3, 8, 8, 3, 8,
            3, 8, 3, 3, 8,
            3, 3, 3, 3, 8,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10001_flood_fill4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.flood_fill4(1, 1, 8, 1);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 1, 1, 5, 8,
            5, 1, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10002_flood_fill4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.flood_fill4(4, 1, 8, 1);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 1,
            5, 8, 5, 5, 1,
            5, 5, 5, 5, 1,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20000_flood_fill8() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 3, 3, 3, 3, 5,
            3, 5, 3, 5, 3, 3,
            3, 3, 5, 3, 5, 3,
            5, 3, 3, 3, 3, 5,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.flood_fill8(3, 1, 5, 0);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 3, 3, 3, 3, 5,
            3, 0, 3, 0, 3, 3,
            3, 3, 0, 3, 0, 3,
            5, 3, 3, 3, 3, 0,
        ];
        let expected = Image::create_raw(6, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_30000_mask_flood_fill4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");
        let mut output = Image::zero(5, 4);
        let color: u8 = input.get(0, 0).unwrap_or(255);

        // Act
        output.mask_flood_fill4(&input, 0, 0, color);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 0, 0, 1, 0,
            1, 0, 1, 1, 0,
            1, 1, 1, 1, 0,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_30001_mask_flood_fill4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");
        let mut output = Image::zero(5, 4);
        let color: u8 = input.get(1, 1).unwrap_or(255);

        // Act
        output.mask_flood_fill4(&input, 1, 1, color);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_30002_mask_flood_fill4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");
        let mut output = Image::zero(5, 4);
        let color: u8 = input.get(4, 1).unwrap_or(255);

        // Act
        output.mask_flood_fill4(&input, 4, 1, color);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_30003_mask_flood_fill4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let mut output = Image::zero(3, 3);
        let color: u8 = input.get(2, 0).unwrap_or(255);

        // Act
        output.mask_flood_fill4(&input, 2, 0, color);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 1, 
            0, 0, 1, 
            0, 0, 0,
        ];
        let expected = Image::create_raw(3, 3, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_40000_mask_flood_fill8() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let mut output = Image::zero(3, 3);
        let color: u8 = input.get(2, 0).unwrap_or(255);

        // Act
        output.mask_flood_fill8(&input, 2, 0, color);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 1, 
            1, 0, 1, 
            1, 1, 0,
        ];
        let expected = Image::create_raw(3, 3, expected_pixels);
        assert_eq!(output, expected);
    }
}
