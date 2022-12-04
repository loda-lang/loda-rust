use super::{Image, ImageTryCreate};
use std::ops::{Rem, Div};
use num_bigint::{BigUint, ToBigUint};
use num_traits::{ToPrimitive, Zero};

pub trait NumberToImage {
    fn to_image(&self) -> anyhow::Result<Image>;
}

impl NumberToImage for BigUint {
    fn to_image(&self) -> anyhow::Result<Image> {
        let bits8: BigUint = 256u32.to_biguint().unwrap();
        let mut current_value = self.clone();

        // Extract `width`
        let width_biguint: BigUint = current_value.clone().rem(&bits8);
        let width_u16: u16 = match width_biguint.to_u16() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("cannot convert width to u16"));
            }
        };
        if width_u16 > (u8::MAX as u16) {
            return Err(anyhow::anyhow!("cannot convert width to u8"));
        }
        let width = width_u16 as u8;

        // Shift the `width` parameter. 8 bits.
        current_value = current_value.div(&bits8);

        // Extract `height`
        let height_biguint: BigUint = current_value.clone().rem(&bits8);
        let height_u16: u16 = match height_biguint.to_u16() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("cannot convert height to u16"));
            }
        };
        if height_u16 > (u8::MAX as u16) {
            return Err(anyhow::anyhow!("cannot convert height to u8"));
        }
        let height = height_u16 as u8;

        // Shift the `height` parameter. 8 bits.
        current_value = current_value.div(&bits8);

        // Extract the pixels. Each pixel is 4 bits.
        let mut pixels = Vec::<u8>::new();
        while !current_value.is_zero() {
            let pixel_biguint: BigUint = current_value.clone().rem(&bits8);
            let pixel_u16: u16 = match pixel_biguint.to_u16() {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("cannot convert pixel to u16"));
                }
            };
            if pixel_u16 > (u8::MAX as u16) {
                return Err(anyhow::anyhow!("cannot convert pixel to u8"));
            }
            pixels.push(pixel_u16 as u8);
    
            // Shift this `pixel` parameter. 8 bits.
            current_value = current_value.div(&bits8);
        }

        let size: usize = (width as usize) * (height as usize);
        if pixels.len() > size {
            return Err(anyhow::anyhow!("there are more pixel data (length: {}) than width {} x height {}", pixels.len(), width, height));
        }

        // If there are too few pixels, then pad until we reach the size
        while pixels.len() < size {
            pixels.push(0);
        }

        Image::try_create(width, height, pixels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_to_image_ok() {
        let k: u64 = 256;
        let value_u64: u64 = (255 * k + 1) * k + 1;
        let value_biguint = value_u64.to_biguint().unwrap();
        let bm: Image = value_biguint.to_image().expect("image");
        assert_eq!(bm.width(), 1);
        assert_eq!(bm.height(), 1);
        assert_eq!(bm.get(0, 0), Some(255));
    }

    #[test]
    fn test_10001_to_image_ok() {
        let k: u64 = 256;
        let value_u64: u64 = ((((8 * k + 7) * k + 6) * k + 5) * k + 2) * k + 2;
        let value_biguint = value_u64.to_biguint().unwrap();
        let bm: Image = value_biguint.to_image().expect("image");
        assert_eq!(bm.width(), 2);
        assert_eq!(bm.height(), 2);
        assert_eq!(bm.get(0, 0), Some(5));
        assert_eq!(bm.get(1, 0), Some(6));
        assert_eq!(bm.get(0, 1), Some(7));
        assert_eq!(bm.get(1, 1), Some(8));
    }
}
