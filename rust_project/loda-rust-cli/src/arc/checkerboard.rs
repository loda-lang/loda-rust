use super::{Image, ImageTryCreate};
use num_integer::Integer;

#[allow(dead_code)]
pub struct Checkerboard;

impl Checkerboard {
    #[allow(dead_code)]
    pub fn checkerboard(width: u8, height: u8, color0: u8, color1: u8) -> Image {
        if width == 0 || height == 0 {
            return Image::empty();
        }
        let mut pixels = Vec::<u8>::new();
        for y in 0..(height as u16) {
            for x in 0..(width as u16) {
                let color = if (x + y).is_even() { color0 } else { color1 };
                pixels.push(color);
            }
        }
        assert_eq!(pixels.len(), (width as usize) * (height as usize));
        Image::try_create(width, height, pixels).expect("image")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_checkerboard() {
        // Act
        let actual: Image = Checkerboard::checkerboard(5, 4, 0, 1);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            1, 0, 1, 0, 1,
            0, 1, 0, 1, 0,
            1, 0, 1, 0, 1,
        ];
        let expected: Image = Image::try_create(5, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
