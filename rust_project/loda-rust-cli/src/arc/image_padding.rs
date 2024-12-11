use crate::arc::Image;

pub trait ImagePadding {
    fn padding_with_color(&self, count: u8, color: u8) -> anyhow::Result<Image>;
    fn padding_advanced(&self, top: u8, left: u8, right: u8, bottom: u8, color: u8) -> anyhow::Result<Image>;
}

impl ImagePadding for Image {
    fn padding_with_color(&self, count: u8, color: u8) -> anyhow::Result<Image> {
        self.padding_advanced(count, count, count, count, color)
    }

    fn padding_advanced(&self, top: u8, left: u8, right: u8, bottom: u8, color: u8) -> anyhow::Result<Image> {
        // Width
        let width_usize: usize = (self.width() as usize) + (left as usize) + (right as usize);
        if width_usize > (u8::MAX as usize) {
            return Err(anyhow::anyhow!("the new width {} exceeds the max width of 256", width_usize));
        }
        let width: u8 = width_usize as u8;

        // Height
        let height_usize: usize = (self.height() as usize) + (top as usize) + (bottom as usize);
        if height_usize > (u8::MAX as usize) {
            return Err(anyhow::anyhow!("the new height {} exceeds the max height of 256", height_usize));
        }
        let height: u8 = height_usize as u8;

        // Transfer pixel values
        let mut result_bitmap = Image::color(width, height, color);
        for y in 0..self.height() {
            for x in 0..self.width() {
                let pixel_value: u8 = self.get(x as i32, y as i32)
                    .ok_or_else(|| anyhow::anyhow!("self.get({},{}) returned None", x, y))?;
                let set_x = (x as usize) + (left as usize);
                let set_y = (y as usize) + (top as usize);
                result_bitmap.set(set_x as i32, set_y as i32, pixel_value)
                    .ok_or_else(|| anyhow::anyhow!("result_bitmap.set({},{}) returned None", set_x, set_y))?;
            }
        }
        Ok(result_bitmap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_empty() {
        {
            let actual = Image::empty().padding_with_color(0, 0).expect("image");
            let expected = Image::empty();
            assert_eq!(actual, expected);
        }
        {
            let actual = Image::empty().padding_with_color(1, 0).expect("image");
            let expected = Image::zero(2, 2);
            assert_eq!(actual, expected);
        }
        {
            let actual = Image::empty().padding_with_color(2, 0).expect("image");
            let expected = Image::zero(4, 4);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_10001_padding_around_data() {
        let bm = Image::create_raw(3, 1, vec![1, 2, 3]);
        let actual = bm.padding_with_color(1, 0).expect("image");

        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 2, 3, 0,
            0, 0, 0, 0, 0,
        ];
        let expected = Image::create_raw(5, 3, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_padding_advanced_top_bottom() {
        let bm = Image::create_raw(3, 1, vec![1, 2, 3]);
        let actual = bm.padding_advanced(2, 0, 0, 2, 9).expect("image");

        let expected_pixels: Vec<u8> = vec![
            9, 9, 9,
            9, 9, 9,
            1, 2, 3,
            9, 9, 9,
            9, 9, 9,
        ];
        let expected = Image::create_raw(3, 5, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_padding_advanced_top_left() {
        let bm = Image::create_raw(3, 1, vec![1, 2, 3]);
        let actual = bm.padding_advanced(2, 1, 0, 0, 8).expect("image");

        let expected_pixels: Vec<u8> = vec![
            8, 8, 8, 8,
            8, 8, 8, 8,
            8, 1, 2, 3,
        ];
        let expected = Image::create_raw(4, 3, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_padding_advanced_bottom_right() {
        let image = Image::color(2, 2, 1);
        let actual = image.padding_advanced(0, 0, 1, 1, 0).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 0,
            1, 1, 0,
            0, 0, 0,
        ];
        let expected = Image::create_raw(3, 3, expected_pixels);
        assert_eq!(actual, expected);
    }
}
