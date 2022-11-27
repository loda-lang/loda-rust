use super::Bitmap;
use num_bigint::BigUint;
use num_traits::Zero;

pub trait BitmapToNumber {
    fn to_number(&self) -> anyhow::Result<BigUint>;
}

impl BitmapToNumber for Bitmap {
    fn to_number(&self) -> anyhow::Result<BigUint> {
        let mut value = BigUint::zero();
        if self.pixels().len() != ((self.width() as usize) * (self.height() as usize)) {
            return Err(anyhow::anyhow!("Integrity error. Number of pixels {} doesn't match width {} x height {}", self.pixels().len(), self.width(), self.height()))
        }
        for pixel_value in self.pixels().iter().rev() {
            if *pixel_value >= 16u8 {
                return Err(anyhow::anyhow!("Integrity error. Expected all pixels to be in the range [0..15], but encountered a pixel that is {} ", pixel_value));
            }
            value *= 16u16;
            value += *pixel_value as u32;
        }
        value *= 256u16;
        value += self.height();
        value *= 256u16;
        value += self.width();
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigUint;

    #[test]
    fn test_10000_bitmap_to_number_ok() {
        {
            let bm = Bitmap::create_raw(0, 0, vec!());
            assert_eq!(bm.to_number().unwrap(), BigUint::zero());
        }
        {
            let bm = Bitmap::create_raw(1, 1, vec![0]);
            let value: u64 = 1 * 256 + 1;
            assert_eq!(bm.to_number().unwrap(), value.to_biguint().unwrap());
        }
        {
            let bm = Bitmap::create_raw(1, 1, vec![1]);
            let value: u64 = (1 * 256 + 1) * 256 + 1;
            assert_eq!(bm.to_number().unwrap(), value.to_biguint().unwrap());
        }
        {
            let bm = Bitmap::create_raw(3, 1, vec![5, 6, 7]);
            let value: u64 = (((7 * 16 + 6) * 16 + 5) * 256 + 1) * 256 + 3;
            assert_eq!(bm.to_number().unwrap(), value.to_biguint().unwrap());
        }
        {
            let bm = Bitmap::create_raw(1, 3, vec![5, 6, 7]);
            let value: u64 = (((7 * 16 + 6) * 16 + 5) * 256 + 3) * 256 + 1;
            assert_eq!(bm.to_number().unwrap(), value.to_biguint().unwrap());
        }
        {
            let bm = Bitmap::create_raw(2, 2, vec![5, 6, 7, 8]);
            let value: u64 = ((((8 * 16 + 7) * 16 + 6) * 16 + 5) * 256 + 2) * 256 + 2;
            assert_eq!(bm.to_number().unwrap(), value.to_biguint().unwrap());
        }
    }

    #[test]
    fn test_10001_bitmap_to_number_error() {
        {
            let bm = Bitmap::create_raw(0, 0, vec![5]);
            bm.to_number().expect_err("expected 0 pixels");
        }
        {
            let bm = Bitmap::create_raw(1, 1, vec!());
            bm.to_number().expect_err("expected 1 pixel");
        }
        {
            let bm = Bitmap::create_raw(1, 1, vec![16]);
            bm.to_number().expect_err("expected pixel value in range [0..15]");
        }
    }
}
