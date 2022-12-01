use anyhow::Context;

use super::Image;

pub fn convolution2x2<F>(bitmap: &Image, callback: F) -> anyhow::Result<Image>
    where F: Fn(&Image) -> anyhow::Result<u8>
{
    let width: u8 = bitmap.width();
    let height: u8 = bitmap.height();
    if width < 2 || height < 2 {
        return Err(anyhow::anyhow!("too small bitmap, must be 2x2 or bigger"));
    }
    let mut computed_bitmap = Image::zeroes(width - 1, height - 1);
    let mut conv_bitmap = Image::zeroes(2, 2);
    for self_y in 0..height-1 {
        for self_x in 0..width-1 {
            for conv_y in 0..2u8 {
                for conv_x in 0..2u8 {
                    let get_x: i32 = (self_x as i32) + (conv_x as i32);
                    let get_y: i32 = (self_y as i32) + (conv_y as i32);
                    let pixel_value: u8 = bitmap.get(get_x, get_y)
                        .ok_or_else(|| anyhow::anyhow!("self.get({},{}) returned None", get_x, get_y))?;
                    conv_bitmap.set(conv_x as i32, conv_y as i32, pixel_value)
                        .ok_or_else(|| anyhow::anyhow!("conv_bitmap.set({},{}) returned None", conv_x, conv_y))?;
                }
            }
            let computed_value: u8 = callback(&conv_bitmap)
                .with_context(|| format!("error in callback when computing ({},{})", self_x, self_y))?;
            computed_bitmap.set(self_x as i32, self_y as i32, computed_value)
                .ok_or_else(|| anyhow::anyhow!("computed_bitmap.set({},{}) returned None", self_x, self_y))?;
        }
    }
    Ok(computed_bitmap)
}

fn conv2x2_max(bm: &Image) -> anyhow::Result<u8> {
    let mut value: u8 = 0;
    for pixel in bm.pixels() {
        value = u8::max(value, *pixel);
    }
    Ok(value)
}

fn conv2x2_min(bm: &Image) -> anyhow::Result<u8> {
    let mut value: u8 = 255;
    for pixel in bm.pixels() {
        value = u8::min(value, *pixel);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::BitmapTryCreate;

    #[test]
    fn test_10000_callback() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("bitmap");

        // Act
        let output: Image = convolution2x2(&input, |bm| {
            let mut sum: u64 = 0;
            for pixel in bm.pixels() {
                sum += *pixel as u64;
            }
            let value = (sum & 255) as u8;
            Ok(value)
        }).expect("bitmap");

        // Assert
        assert_eq!(output.width(), 2);
        assert_eq!(output.height(), 2);
        assert_eq!(output.get(0, 0), Some(1+2+4+5));
        assert_eq!(output.get(1, 0), Some(2+3+5+6));
        assert_eq!(output.get(0, 1), Some(4+5+7+8));
        assert_eq!(output.get(1, 1), Some(5+6+8+9));
    }

    #[test]
    fn test_20000_max() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,2,3,4,
            5,6,7,8,
            9,10,11,12,
            13,14,15,16,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let output: Image = convolution2x2(&input, conv2x2_max).expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            6,7,8,
            10,11,12,
            14,15,16,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("bitmap");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20001_min() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,2,3,4,
            5,6,7,8,
            9,10,11,12,
            13,14,15,16,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let output: Image = convolution2x2(&input, conv2x2_min).expect("bitmap");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1,2,3,
            5,6,7,
            9,10,11,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("bitmap");
        assert_eq!(output, expected);
    }
}
