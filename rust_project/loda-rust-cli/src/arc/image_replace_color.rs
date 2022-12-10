use super::Image;
use std::collections::HashMap;

pub trait ImageReplaceColor {
    fn replace_color(&self, source: u8, destination: u8) -> anyhow::Result<Image>;
    fn replace_colors_with_hashmap(&self, replacements: &HashMap::<u8, u8>) -> anyhow::Result<Image>;
    fn replace_colors_with_palette_image(&self, palette_image: &Image) -> anyhow::Result<Image>;
    fn replace_colors_other_than(&self, source: u8, destination: u8) -> anyhow::Result<Image>;
}

impl ImageReplaceColor for Image {
    fn replace_color(&self, source: u8, destination: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let mut image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let mut pixel_value: u8 = self.get(x, y).unwrap_or(255);
                if pixel_value == source {
                    pixel_value = destination;
                }
                match image.set(x, y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", x, y));
                    }
                }
            }
        }
        Ok(image)
    }

    fn replace_colors_with_hashmap(&self, replacements: &HashMap::<u8, u8>) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        if replacements.is_empty() {
            return Ok(self.clone());
        }
        let mut image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let mut pixel_value: u8 = self.get(x, y).unwrap_or(255);
                if let Some(replacement_value) = replacements.get(&pixel_value) {
                    pixel_value = *replacement_value;
                }
                match image.set(x, y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", x, y));
                    }
                }
            }
        }
        Ok(image)
    }

    fn replace_colors_with_palette_image(&self, palette_image: &Image) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        if palette_image.is_empty() {
            return Ok(self.clone());
        }
        if palette_image.height() != 2 {
            return Err(anyhow::anyhow!("ImageReplaceColor.replace_colors_with_palette_image the height of the palette image must be 2. Top row is for source color, bottom row is for destination color."));
        }
        let mut replacements = HashMap::<u8, u8>::new();
        for x in 0..(palette_image.width() as i32) {
            let source_color: u8 = palette_image.get(x, 0).unwrap_or(255);
            let destination_color: u8 = palette_image.get(x, 1).unwrap_or(255);
            replacements.insert(source_color, destination_color);
        }
        self.replace_colors_with_hashmap(&replacements)
    }

    fn replace_colors_other_than(&self, source: u8, destination: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let mut image = Image::zero(self.width(), self.height());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let mut pixel_value: u8 = self.get(x, y).unwrap_or(255);
                if pixel_value != source {
                    pixel_value = destination;
                }
                match image.set(x, y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result bitmap", x, y));
                    }
                }
            }
        }
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_replace_color() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 3, 0, 3,
            0, 0, 3, 2,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: Image = input.replace_color(3, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 0, 1,
            0, 0, 1, 2,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_replace_colors_with_hashmap() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 7,
            0, 3, 0, 3,
            0, 0, 3, 2,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");
        let mut replacements = HashMap::<u8, u8>::new();
        replacements.insert(0, 1);
        replacements.insert(2, 3);
        replacements.insert(3, 4);

        // Act
        let actual: Image = input.replace_colors_with_hashmap(&replacements).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 7,
            1, 4, 1, 4,
            1, 1, 4, 3,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_replace_colors_with_palette_image() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            0, 0, 0, 7,
            0, 3, 0, 3,
            0, 0, 3, 2,
        ];
        let input: Image = Image::try_create(4, 3, input_pixels).expect("image");
        let palette_pixels: Vec<u8> = vec![
            0, 2, 3,
            1, 3, 4,
        ];
        let palette: Image = Image::try_create(3, 2, palette_pixels).expect("image");

        // Act
        let actual: Image = input.replace_colors_with_palette_image(&palette).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 7,
            1, 4, 1, 4,
            1, 1, 4, 3,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_replace_colors_other_than() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 0, 0, 5,
            0, 1, 2, 0,
            0, 3, 4, 5,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: Image = input.replace_colors_other_than(5, 0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 5,
            0, 0, 0, 0,
            0, 0, 0, 5,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
