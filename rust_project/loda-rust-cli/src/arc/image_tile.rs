use super::{Image, ImageOverlay};

pub trait ImageTile {
    /// Create a big composition of tiles. When the mask is 0 then pick `tile0` as tile. When the mask is [1..255] then pick `tile1` as tile.
    /// 
    /// `tile0` and `tile1` must have same size.
    /// 
    /// If the computed output size exceeds 255x255 then an error is returned.
    fn select_two_tiles(&self, tile0: &Image, tile1: &Image) -> anyhow::Result<Image>;
}

impl ImageTile for Image {
    fn select_two_tiles(&self, tile0: &Image, tile1: &Image) -> anyhow::Result<Image> {
        if tile0.width() != tile1.width() {
            return Err(anyhow::anyhow!("Both tiles must have same width. tile0.width: {} tile1.width: {}", tile0.width(), tile1.width()));
        }
        if tile0.height() != tile1.height() {
            return Err(anyhow::anyhow!("Both tiles must have same height. tile0.height: {} tile1.height: {}", tile0.height(), tile1.height()));
        }
        if self.is_empty() || tile0.is_empty() || tile1.is_empty() {
            return Ok(Image::empty());
        }

        let tile_width: u8 = tile0.width();
        let tile_height: u8 = tile0.height();

        let w: u16 = (self.width() as u16) * (tile_width as u16);
        let h: u16 = (self.height() as u16) * (tile_height as u16);
        if w >= (u8::MAX as u16) {
            return Err(anyhow::anyhow!("Output image.width {} is too big. mask.width: {} tile_width: {}", w, self.width(), tile_width));
        }
        if h >= (u8::MAX as u16) {
            return Err(anyhow::anyhow!("Output image.height {} is too big. mask.height: {} tile_height: {}", h, self.height(), tile_height));
        }
        let output_width: u8 = w as u8;
        let output_height: u8 = h as u8;

        let mut result: Image = Image::zero(output_width, output_height);
        for y in 0..self.height() {
            for x in 0..self.width() {
                let mask_value: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                let tile: &Image;
                if mask_value == 0 {
                    tile = &tile0;
                } else {
                    tile = &tile1;
                }
                result = result.overlay_with_position(tile, (x * tile_width) as i32, (y * tile_height) as i32)?;
            }
        }
        return Ok(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_select_two_tiles() {
        // Arrange
        let mask_pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 1, 0,
            0, 1, 1, 0,
            0, 0, 0, 0,
        ];
        let mask: Image = Image::try_create(4, 4, mask_pixels).expect("image");
        let image_a_pixels: Vec<u8> = vec![
            5, 5,
            5, 4,
        ];
        let image_a: Image = Image::try_create(2, 2, image_a_pixels).expect("image");
        let image_b_pixels: Vec<u8> = vec![
            7, 9,
            7, 9,
        ];
        let image_b: Image = Image::try_create(2, 2, image_b_pixels).expect("image");
            
        // Act
        let actual: Image = mask.select_two_tiles(&image_a, &image_b).expect("image");
            
        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5, 5, 5, 5,
            5, 4, 5, 4, 5, 4, 5, 4,
            5, 5, 7, 9, 7, 9, 5, 5,
            5, 4, 7, 9, 7, 9, 5, 4,
            5, 5, 7, 9, 7, 9, 5, 5,
            5, 4, 7, 9, 7, 9, 5, 4,
            5, 5, 5, 5, 5, 5, 5, 5,
            5, 4, 5, 4, 5, 4, 5, 4,
        ];
        let expected: Image = Image::try_create(8, 8, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_select_two_tiles() {
        // Arrange
        let mask_pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 1,
            1, 1, 1,
        ];
        let mask: Image = Image::try_create(3, 3, mask_pixels).expect("image");
        let image_a_pixels: Vec<u8> = vec![
            5,
            5,
        ];
        let image_a: Image = Image::try_create(1, 2, image_a_pixels).expect("image");
        let image_b_pixels: Vec<u8> = vec![
            7,
            7,
        ];
        let image_b: Image = Image::try_create(1, 2, image_b_pixels).expect("image");
            
        // Act
        let actual: Image = mask.select_two_tiles(&image_a, &image_b).expect("image");
            
        // Assert
        let expected_pixels: Vec<u8> = vec![
            7, 7, 7,
            7, 7, 7,
            7, 5, 7,
            7, 5, 7,
            7, 7, 7,
            7, 7, 7,
        ];
        let expected: Image = Image::try_create(3, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
