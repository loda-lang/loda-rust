use super::{Image, PixelConnectivity};

pub trait ImageFill {
    /// Replace color with another color
    /// 
    /// Visit 4 or 8 neighbors around a pixel.
    fn flood_fill(&mut self, x: i32, y: i32, from_color: u8, to_color: u8, connectivity: PixelConnectivity);

    /// Build a mask of connected pixels that has the same color
    /// 
    /// Visit 4 or 8 neighbors around a pixel.
    fn mask_flood_fill(&mut self, image: &Image, x: i32, y: i32, color: u8, connectivity: PixelConnectivity) -> anyhow::Result<()>;

    /// Flood fill at every pixel along the border
    /// 
    /// Visit 4 or 8 neighbors around a pixel.
    fn border_flood_fill(&mut self, from_color: u8, to_color: u8, connectivity: PixelConnectivity);
}

impl ImageFill for Image {
    fn flood_fill(&mut self, x: i32, y: i32, from_color: u8, to_color: u8, connectivity: PixelConnectivity) {
        match connectivity {
            PixelConnectivity::Connectivity4 => FloodFill::flood_fill4(self, x, y, from_color, to_color),
            PixelConnectivity::Connectivity8 => FloodFill::flood_fill8(self, x, y, from_color, to_color),
        }
    }

    fn mask_flood_fill(&mut self, image: &Image, x: i32, y: i32, color: u8, connectivity: PixelConnectivity) -> anyhow::Result<()> {
        if self.size() != image.size() {
            return Err(anyhow::anyhow!("both images must have same size"));
        }
        match connectivity {
            PixelConnectivity::Connectivity4 => FloodFill::mask_flood_fill4(self, image, x, y, color),
            PixelConnectivity::Connectivity8 => FloodFill::mask_flood_fill8(self, image, x, y, color),
        }
        Ok(())
    }

    fn border_flood_fill(&mut self, from_color: u8, to_color: u8, connectivity: PixelConnectivity) {
        let x1: i32 = (self.width() as i32) - 1;
        let y1: i32 = (self.height() as i32) - 1;
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                if x > 0 && x < x1 && y > 0 && y < y1 { 
                    continue;
                }
                self.flood_fill(x, y, from_color, to_color, connectivity);
            }
        }
    }
    
}

struct FloodFill;

impl FloodFill {
    fn flood_fill4(image: &mut Image, x: i32, y: i32, from_color: u8, to_color: u8) {
        if x < 0 || y < 0 || x >= (image.width() as i32) || y >= (image.height() as i32) {
            return;
        }
        let value: u8 = image.get(x, y).unwrap_or(255);
        if value == to_color {
            return;
        }
        if value != from_color {
            return;
        }
        let _ = image.set(x, y, to_color);
        Self::flood_fill4(image, x-1, y, from_color, to_color);
        Self::flood_fill4(image, x+1, y, from_color, to_color);
        Self::flood_fill4(image, x, y-1, from_color, to_color);
        Self::flood_fill4(image, x, y+1, from_color, to_color);
    }

    fn flood_fill8(image: &mut Image, x: i32, y: i32, from_color: u8, to_color: u8) {
        if x < 0 || y < 0 || x >= (image.width() as i32) || y >= (image.height() as i32) {
            return;
        }
        let value: u8 = image.get(x, y).unwrap_or(255);
        if value == to_color {
            return;
        }
        if value != from_color {
            return;
        }
        let _ = image.set(x, y, to_color);
        Self::flood_fill8(image, x-1, y-1, from_color, to_color);
        Self::flood_fill8(image, x, y-1, from_color, to_color);
        Self::flood_fill8(image, x+1, y-1, from_color, to_color);
        Self::flood_fill8(image, x-1, y, from_color, to_color);
        Self::flood_fill8(image, x+1, y, from_color, to_color);
        Self::flood_fill8(image, x-1, y+1, from_color, to_color);
        Self::flood_fill8(image, x, y+1, from_color, to_color);
        Self::flood_fill8(image, x+1, y+1, from_color, to_color);
    }

    fn mask_flood_fill4(mask: &mut Image, image: &Image, x: i32, y: i32, color: u8) {
        assert!(mask.width() == image.width());
        assert!(mask.height() == image.height());
        if x < 0 || y < 0 || x >= (mask.width() as i32) || y >= (mask.height() as i32) {
            return;
        }
        let mask_value: u8 = mask.get(x, y).unwrap_or(255);
        if mask_value > 0 {
            // already visited
            return;
        }
        let value: u8 = image.get(x, y).unwrap_or(255);
        if value != color {
            return;
        }
        let _ = mask.set(x, y, 1); // flag as visited
        Self::mask_flood_fill4(mask, image, x-1, y, color);
        Self::mask_flood_fill4(mask, image, x+1, y, color);
        Self::mask_flood_fill4(mask, image, x, y-1, color);
        Self::mask_flood_fill4(mask, image, x, y+1, color);
    }

    fn mask_flood_fill8(mask: &mut Image, image: &Image, x: i32, y: i32, color: u8) {
        assert!(mask.width() == image.width());
        assert!(mask.height() == image.height());
        if x < 0 || y < 0 || x >= (mask.width() as i32) || y >= (mask.height() as i32) {
            return;
        }
        let mask_value: u8 = mask.get(x, y).unwrap_or(255);
        if mask_value > 0 {
            // already visited
            return;
        }
        let value: u8 = image.get(x, y).unwrap_or(255);
        if value != color {
            return;
        }
        let _ = mask.set(x, y, 1); // flag as visited
        Self::mask_flood_fill8(mask, image, x-1, y-1, color);
        Self::mask_flood_fill8(mask, image, x, y-1, color);
        Self::mask_flood_fill8(mask, image, x+1, y-1, color);
        Self::mask_flood_fill8(mask, image, x-1, y, color);
        Self::mask_flood_fill8(mask, image, x+1, y, color);
        Self::mask_flood_fill8(mask, image, x-1, y+1, color);
        Self::mask_flood_fill8(mask, image, x, y+1, color);
        Self::mask_flood_fill8(mask, image, x+1, y+1, color);
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
        output.flood_fill(0, 0, 5, 3, PixelConnectivity::Connectivity4);

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
        output.flood_fill(1, 1, 8, 1, PixelConnectivity::Connectivity4);

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
        output.flood_fill(4, 1, 8, 1, PixelConnectivity::Connectivity4);

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
    fn test_10003_flood_fill4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 0, 0, 0, 5,
            5, 0, 0, 0, 5,
            5, 5, 5, 5, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.flood_fill(2, 1, 0, 0, PixelConnectivity::Connectivity4);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 0, 0, 0, 5,
            5, 0, 0, 0, 5,
            5, 5, 5, 5, 5,
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
        output.flood_fill(3, 1, 5, 0, PixelConnectivity::Connectivity8);

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
    fn test_20001_flood_fill8() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 0, 0, 0, 5,
            5, 0, 0, 0, 5,
            5, 5, 5, 5, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.flood_fill(2, 1, 0, 0, PixelConnectivity::Connectivity8);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 0, 0, 0, 5,
            5, 0, 0, 0, 5,
            5, 5, 5, 5, 5,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
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
        output.mask_flood_fill(&input, 0, 0, color, PixelConnectivity::Connectivity4).expect("ok");

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
        output.mask_flood_fill(&input, 1, 1, color, PixelConnectivity::Connectivity4).expect("ok");

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
        output.mask_flood_fill(&input, 4, 1, color, PixelConnectivity::Connectivity4).expect("ok");

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
        output.mask_flood_fill(&input, 2, 0, color, PixelConnectivity::Connectivity4).expect("ok");

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
        output.mask_flood_fill(&input, 2, 0, color, PixelConnectivity::Connectivity8).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 1, 
            1, 0, 1, 
            1, 1, 0,
        ];
        let expected = Image::create_raw(3, 3, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_50000_border_flood_fill4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 3, 0, 0, 0,
            0, 0, 3, 3, 3, 7,
            3, 3, 0, 7, 0, 7,
            0, 0, 3, 3, 3, 7,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.border_flood_fill(0, 1, PixelConnectivity::Connectivity4);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 3, 1, 1, 1,
            1, 1, 3, 3, 3, 7,
            3, 3, 0, 7, 0, 7,
            1, 1, 3, 3, 3, 7,
        ];
        let expected = Image::create_raw(6, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_50001_border_flood_fill8() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 3, 0, 0, 0,
            0, 0, 3, 3, 3, 7,
            3, 3, 0, 7, 0, 7,
            0, 0, 3, 3, 3, 7,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.border_flood_fill(0, 1, PixelConnectivity::Connectivity8);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 3, 1, 1, 1,
            1, 1, 3, 3, 3, 7,
            3, 3, 1, 7, 0, 7,
            1, 1, 3, 3, 3, 7,
        ];
        let expected = Image::create_raw(6, 4, expected_pixels);
        assert_eq!(output, expected);
    }
}
