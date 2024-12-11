use super::Image;
use num_bigint::BigUint;
use num_traits::Zero;

pub trait ImageToNumber {
    fn to_number(&self) -> anyhow::Result<BigUint>;
}

impl ImageToNumber for Image {
    fn to_number(&self) -> anyhow::Result<BigUint> {
        let mut value = BigUint::zero();
        if self.pixels().len() != ((self.width() as usize) * (self.height() as usize)) {
            return Err(anyhow::anyhow!("ImageToNumber.to_number() Number of pixels {} doesn't match width {} x height {}", self.pixels().len(), self.width(), self.height()))
        }
        for pixel_value in self.pixels().iter().rev() {
            value *= 256u16;
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
    fn test_10000_to_number_ok() {
        let k: u64 = 256;
        {
            let bm = Image::create_raw(0, 0, vec!());
            assert_eq!(bm.to_number().unwrap(), BigUint::zero());
        }
        {
            let bm = Image::create_raw(1, 1, vec![0]);
            let value: u64 = 1 * k + 1;
            assert_eq!(bm.to_number().unwrap(), value.to_biguint().unwrap());
        }
        {
            let bm = Image::create_raw(1, 1, vec![255]);
            let value: u64 = (255 * k + 1) * k + 1;
            assert_eq!(bm.to_number().unwrap(), value.to_biguint().unwrap());
        }
        {
            let bm = Image::create_raw(3, 1, vec![5, 6, 7]);
            let value: u64 = (((7 * k + 6) * k + 5) * k + 1) * k + 3;
            assert_eq!(bm.to_number().unwrap(), value.to_biguint().unwrap());
        }
        {
            let bm = Image::create_raw(1, 3, vec![5, 6, 7]);
            let value: u64 = (((7 * k + 6) * k + 5) * k + 3) * k + 1;
            assert_eq!(bm.to_number().unwrap(), value.to_biguint().unwrap());
        }
        {
            let bm = Image::create_raw(2, 2, vec![5, 6, 7, 8]);
            let value: u64 = ((((8 * k + 7) * k + 6) * k + 5) * k + 2) * k + 2;
            assert_eq!(bm.to_number().unwrap(), value.to_biguint().unwrap());
        }
    }

    #[test]
    fn test_10001_to_number_error() {
        {
            let bm = Image::create_raw(0, 0, vec![5]);
            bm.to_number().expect_err("expected 0 pixels");
        }
        {
            let bm = Image::create_raw(1, 1, vec!());
            bm.to_number().expect_err("expected 1 pixel");
        }
    }
}
