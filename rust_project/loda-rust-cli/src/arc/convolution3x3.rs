use anyhow::Context;

use super::Bitmap;

pub fn convolution3x3<F>(bitmap: &Bitmap, callback: F) -> anyhow::Result<Bitmap>
    where F: Fn(&Bitmap) -> anyhow::Result<u8>
{
    let width: u8 = bitmap.width();
    let height: u8 = bitmap.height();
    if width < 3 || height < 3 {
        return Err(anyhow::anyhow!("too small bitmap, must be 3x3 or bigger"));
    }
    let mut computed_bitmap = Bitmap::zeroes(width - 2, height - 2);
    let mut conv_bitmap = Bitmap::zeroes(3, 3);
    for self_y in 0..height-2 {
        for self_x in 0..width-2 {
            for conv_y in 0..3u8 {
                for conv_x in 0..3u8 {
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

fn conv3x3_max(bm: &Bitmap) -> anyhow::Result<u8> {
    let mut value: u8 = 0;
    for pixel in bm.pixels() {
        value = u8::max(value, *pixel);
    }
    Ok(value)
}

fn conv3x3_min(bm: &Bitmap) -> anyhow::Result<u8> {
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
        let input: Bitmap = Bitmap::try_create(3, 3, vec![1,2,3,4,5,6,7,8,9]).expect("bitmap");

        // Act
        let output: Bitmap = convolution3x3(&input, |bm| {
            let mut sum: u64 = 0;
            for pixel in bm.pixels() {
                sum += *pixel as u64;
            }
            let value = (sum & 255) as u8;
            Ok(value)
        }).expect("bitmap");

        // Assert
        assert_eq!(output.width(), 1);
        assert_eq!(output.height(), 1);
        assert_eq!(output.get(0, 0), Some(1+2+3+4+5+6+7+8+9));
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
        let input: Bitmap = Bitmap::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let output: Bitmap = convolution3x3(&input, conv3x3_max).expect("bitmap");

        // Assert
        assert_eq!(output.width(), 2);
        assert_eq!(output.height(), 2);
        assert_eq!(output.get(0, 0), Some(11));
        assert_eq!(output.get(1, 0), Some(12));
        assert_eq!(output.get(0, 1), Some(15));
        assert_eq!(output.get(1, 1), Some(16));
    }

    #[test]
    fn test_30000_min() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,2,3,4,
            5,6,7,8,
            9,10,11,12,
            13,14,15,16,
        ];
        let input: Bitmap = Bitmap::try_create(4, 4, pixels).expect("bitmap");

        // Act
        let output: Bitmap = convolution3x3(&input, conv3x3_min).expect("bitmap");

        // Assert
        assert_eq!(output.width(), 2);
        assert_eq!(output.height(), 2);
        assert_eq!(output.get(0, 0), Some(1));
        assert_eq!(output.get(1, 0), Some(2));
        assert_eq!(output.get(0, 1), Some(5));
        assert_eq!(output.get(1, 1), Some(6));
    }
}
