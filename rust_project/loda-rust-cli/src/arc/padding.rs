use crate::arc::Image;

pub trait ImagePadding {
    fn zero_padding(&self, count: u8) -> anyhow::Result<Image>;
}

impl ImagePadding for Image {
    fn zero_padding(&self, count: u8) -> anyhow::Result<Image> {
        // Width
        let width_usize: usize = (self.width() as usize) + (count as usize) * 2;
        if width_usize > (u8::MAX as usize) {
            return Err(anyhow::anyhow!("the new width {} exceeds the max width of 256", width_usize));
        }
        let width: u8 = width_usize as u8;

        // Height
        let height_usize: usize = (self.height() as usize) + (count as usize) * 2;
        if height_usize > (u8::MAX as usize) {
            return Err(anyhow::anyhow!("the new height {} exceeds the max height of 256", height_usize));
        }
        let height: u8 = height_usize as u8;

        // Transfer pixel values
        let mut result_bitmap = Image::zeroes(width, height);
        for y in 0..self.height() {
            for x in 0..self.width() {
                let pixel_value: u8 = self.get(x as i32, y as i32)
                    .ok_or_else(|| anyhow::anyhow!("self.get({},{}) returned None", x, y))?;
                let set_x = (x as usize) + (count as usize);
                let set_y = (y as usize) + (count as usize);
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
            let actual = Image::empty().zero_padding(0).expect("bitmap");
            let expected = Image::empty();
            assert_eq!(actual, expected);
        }
        {
            let actual = Image::empty().zero_padding(1).expect("bitmap");
            let expected = Image::zeroes(2, 2);
            assert_eq!(actual, expected);
        }
        {
            let actual = Image::empty().zero_padding(2).expect("bitmap");
            let expected = Image::zeroes(4, 4);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_10001_padding_around_data() {
        let bm = Image::create_raw(3, 1, vec![1, 2, 3]);
        let actual = bm.zero_padding(1).expect("bitmap");

        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 2, 3, 0,
            0, 0, 0, 0, 0,
        ];
        let expected = Image::create_raw(5, 3, expected_pixels);
        assert_eq!(actual, expected);
    }
}
