use super::Image;

pub trait ImageSegment {
    fn flood_fill(&mut self, x: i32, y: i32, from_color: u8, to_color: u8);
}

impl ImageSegment for Image {
    fn flood_fill(&mut self, x: i32, y: i32, from_color: u8, to_color: u8) {
        if x < 0 || y < 0 || x >= (self.width() as i32) || y >= (self.height() as i32) {
            return;
        }
        let value: u8 = self.get(x, y).unwrap_or(255);
        if value != from_color {
            return;
        }
        let _ = self.set(x, y, to_color);
        self.flood_fill(x-1, y, from_color, to_color);
        self.flood_fill(x+1, y, from_color, to_color);
        self.flood_fill(x, y-1, from_color, to_color);
        self.flood_fill(x, y+1, from_color, to_color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_flood_fill() {
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
        output.flood_fill(0, 0, 5, 3);

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
    fn test_10001_flood_fill() {
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
        output.flood_fill(1, 1, 8, 1);

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
    fn test_10002_flood_fill() {
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
        output.flood_fill(4, 1, 8, 1);

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
}
