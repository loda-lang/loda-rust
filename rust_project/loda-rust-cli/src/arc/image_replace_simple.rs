use super::{Image, ImageFind, ImageOverlay};

pub trait ImageReplaceSimple {
    /// Find `source` image positions and replace with the `destination` image.
    /// 
    /// Returns the number of replacements performed.
    fn replace_simple(&mut self, source: &Image, destination: &Image) -> anyhow::Result<u16>;
}

impl ImageReplaceSimple for Image {
    fn replace_simple(&mut self, source: &Image, destination: &Image) -> anyhow::Result<u16> {
        if source.size() != destination.size() || source.is_empty() {
            return Err(anyhow::anyhow!("Both images are supposed to have same size, and be 1x1 or bigger"));
        }
        if self.is_empty() {
            return Ok(0);
        }
        // search pattern bigger than image
        if source.width() > self.width() || source.height() > self.height() {
            return Ok(0);
        }
        let positions: Vec<(u8, u8)> = self.find_all(source)?;
        let count_usize: usize = positions.len();
        if count_usize > (u16::MAX as usize) {
            return Err(anyhow::anyhow!("Too many positions to fit into an u16"));
        }
        let count: u16 = count_usize as u16;
        let mut result_image: Image = self.clone();
        for (x, y) in positions {
            result_image = result_image.overlay_with_position(destination, x as i32, y as i32)?;
        }
        self.set_image(result_image);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_replace_simple() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 3, 0, 3, 7,
            0, 0, 3, 0, 3,
            0, 3, 0, 3, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let source_pixels: Vec<u8> = vec![
            0,
            3,
        ];
        let source: Image = Image::try_create(1, 2, source_pixels).expect("image");

        let destination_pixels: Vec<u8> = vec![
            1,
            3,
        ];
        let destination: Image = Image::try_create(1, 2, destination_pixels).expect("image");
        let mut actual: Image = input.clone();
        
        // Act
        let count: u16 = actual.replace_simple(&source, &destination).expect("count");

        // Assert
        assert_eq!(count, 5);
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            0, 3, 1, 3, 7,
            0, 1, 3, 1, 3,
            0, 3, 0, 3, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_replace_simple_mismatching_sizes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 3, 0, 3, 7,
            0, 0, 3, 0, 3,
            0, 3, 0, 3, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let source_pixels: Vec<u8> = vec![
            0,
            3,
            4,
        ];
        let source: Image = Image::try_create(1, 3, source_pixels).expect("image");

        let destination_pixels: Vec<u8> = vec![
            1,
            3,
        ];
        let destination: Image = Image::try_create(1, 2, destination_pixels).expect("image");
        let mut actual: Image = input.clone();
       
        // Act
        let error = actual.replace_simple(&source, &destination).expect_err("is supposed to fail");

        // Assert
        let message: String = format!("{:?}", error);
        assert_eq!(message.contains("Both images are supposed to have same size, and be 1x1 or bigger"), true);
    }
}
