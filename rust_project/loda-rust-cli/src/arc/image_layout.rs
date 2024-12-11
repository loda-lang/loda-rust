use super::{Image, ImageSize};

#[allow(dead_code)]
pub enum ImageLayoutMode {
    Normal,
    ReverseOddRows,
}

#[allow(dead_code)]
pub trait ImageLayout {
    /// Transfer pixels from one layout to another layout.
    /// 
    /// The background_color is used for padding when there are 
    /// insufficient pixels in the input image.
    fn layout(&self, size: ImageSize, background_color: u8, mode: ImageLayoutMode) -> anyhow::Result<Image>;
}

impl ImageLayout for Image {
    fn layout(&self, size: ImageSize, background_color: u8, mode: ImageLayoutMode) -> anyhow::Result<Image> {
        let mut result_image = Image::color(size.width, size.height, background_color);
        let result_width: usize = size.width as usize;
        let result_height: usize = size.height as usize;
        if result_image.is_empty() || result_width == 0 || result_height == 0 {
            return Err(anyhow::anyhow!("size must be 1x1 or bigger"));
        }

        for y in 0..self.height() {
            for x in 0..self.width() {
                let position: usize = match self.index_for_pixel(x as i32, y as i32) {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Unable to compute index for pixel"));
                    }
                };
                let color: u8 = self.get(x as i32, y as i32).unwrap_or(255);

                let set_x: i32;
                let set_y: i32;
                match mode {
                    ImageLayoutMode::Normal => {
                        let xx: usize = position % result_width;
                        let yy: usize = position / result_width;
                        set_x = xx as i32;
                        set_y = yy as i32;
                    }
                    ImageLayoutMode::ReverseOddRows => {
                        let mut xx: i32 = (position % result_width) as i32;
                        let yy: usize = position / result_width;
                        if yy & 1 == 1 {
                            xx = (result_width as i32) - xx - 1;
                        }
                        set_x = xx;
                        set_y = yy as i32;
                    }
                }

                _ = result_image.set(set_x, set_y, color);
            }
        }

        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_layout_normal() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
        ];
        let input: Image = Image::try_create(10, 1, pixels).expect("image");
        let size = ImageSize { width: 3, height: 4 };

        // Act
        let actual: Image = input.layout(size, 0, ImageLayoutMode::Normal).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
            10, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_layout_reverse_odd_rows() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
        ];
        let input: Image = Image::try_create(10, 1, pixels).expect("image");
        let size = ImageSize { width: 3, height: 4 };

        // Act
        let actual: Image = input.layout(size, 0, ImageLayoutMode::ReverseOddRows).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3,
            6, 5, 4,
            7, 8, 9,
            0, 0, 10,
        ];
        let expected: Image = Image::try_create(3, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
