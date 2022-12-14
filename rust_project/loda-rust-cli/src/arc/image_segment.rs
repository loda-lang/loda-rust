use super::Image;

pub trait ImageSegment {
    /// Replace color with another color
    fn flood_fill(&mut self, x: i32, y: i32, from_color: u8, to_color: u8);

    /// Build a mask of connected pixels that has the same color
    fn flood_fill_visit(&mut self, image: &Image, x: i32, y: i32, color: u8);

    /// Identify clusters of connected pixels
    fn find_object_masks(&self) -> anyhow::Result<Vec<Image>>;
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

    fn flood_fill_visit(&mut self, image: &Image, x: i32, y: i32, color: u8) {
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
        self.flood_fill_visit(image, x-1, y, color);
        self.flood_fill_visit(image, x+1, y, color);
        self.flood_fill_visit(image, x, y-1, color);
        self.flood_fill_visit(image, x, y+1, color);
    }

    fn find_object_masks(&self) -> anyhow::Result<Vec<Image>> {
        let mut object_mask_vec = Vec::<Image>::new();
        let mut accumulated_mask = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let mask_value: u8 = accumulated_mask.get(x, y).unwrap_or(255);
                if mask_value > 0 {
                    continue;
                }
                let color: u8 = self.get(x, y).unwrap_or(255);
                let mut object_mask = Image::zero(self.width(), self.height());
                object_mask.flood_fill_visit(&self, x, y, color);

                // copy the mask into the accumulated mask, so that the pixel doesn't get visited again
                for yy in 0..(self.height() as i32) {
                    for xx in 0..(self.width() as i32) {
                        let mask_value: u8 = object_mask.get(xx, yy).unwrap_or(255);
                        if mask_value > 0 {
                            let _ = accumulated_mask.set(xx, yy, 1);
                        }
                    }
                }
                object_mask_vec.push(object_mask);
            }
        }
        Ok(object_mask_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use crate::arc::ImageStack;

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

    #[test]
    fn test_20000_flood_fill_visit() {
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
        output.flood_fill_visit(&input, 0, 0, color);

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
    fn test_20001_flood_fill_visit() {
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
        output.flood_fill_visit(&input, 1, 1, color);

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
    fn test_20002_flood_fill_visit() {
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
        output.flood_fill_visit(&input, 4, 1, color);

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
    fn test_30000_find_object_masks() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = input.find_object_masks().expect("image");

        // Assert
        assert_eq!(mask_vec.len(), 3);
        let output: Image = Image::vstack(mask_vec).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 0, 0, 1, 0,
            1, 0, 1, 1, 0,
            1, 1, 1, 1, 0,

            0, 0, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 0, 0, 0,

            0, 0, 0, 0, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
        ];
        let expected = Image::create_raw(5, 4*3, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_30001_find_object_masks() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 6, 6, 6, 5,
            5, 6, 5, 6, 5,
            5, 6, 6, 6, 5,
            5, 5, 5, 5, 5,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = input.find_object_masks().expect("image");

        // Assert
        assert_eq!(mask_vec.len(), 3);
        let output: Image = Image::vstack(mask_vec).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 1,

            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 1, 0, 1, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 0,
            
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected = Image::create_raw(5, 5*3, expected_pixels);
        assert_eq!(output, expected);
    }
}
