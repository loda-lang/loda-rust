use super::Bitmap;

pub trait BitmapTryCreate {
    fn try_create(width: u8, height: u8, pixels: Vec<u8>) -> anyhow::Result<Bitmap>;
}

impl BitmapTryCreate for Bitmap {
    fn try_create(width: u8, height: u8, pixels: Vec<u8>) -> anyhow::Result<Bitmap> {
        if width == 0 && height > 0 {
            return Err(anyhow::anyhow!("width=0, but height>0, expected both to be zero"));
        }
        if width > 0 && height == 0 {
            return Err(anyhow::anyhow!("height=0, but width>0, expected both to be zero"));
        }
        let len: usize = (width as usize) * (height as usize);
        if len != pixels.len() {
            return Err(anyhow::anyhow!("Number of pixels {} doesn't match width {} x height {}", pixels.len(), width, height));
        }
        for (index, pixel_value) in pixels.iter().enumerate() {
            if *pixel_value >= 16u8 {
                return Err(anyhow::anyhow!("Expected all pixels to be in the range [0..15], but pixel({}) is {} ", index, pixel_value));
            }
        }
        let instance = Self::create_raw(width, height, pixels);
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_try_create_ok() {
        let bm = Bitmap::try_create(2, 2, vec![1,2,3,4]).expect("ok");
        assert_eq!(bm.width(), 2);
        assert_eq!(bm.height(), 2);
        assert_eq!(bm.pixels().len(), 2 * 2);
        let mut sum: usize = 0;
        for pixel in bm.pixels() {
            sum += *pixel as usize;
        }
        assert_eq!(sum, 1+2+3+4);
    }

    #[test]
    fn test_10001_try_create_error() {
        Bitmap::try_create(0, 2, vec!()).expect_err("width height should both be zero");
        Bitmap::try_create(2, 0, vec!()).expect_err("width height should both be zero");
        Bitmap::try_create(1, 1, vec!()).expect_err("width height doesn't match pixel count");
        Bitmap::try_create(1, 1, vec![16]).expect_err("illegal pixel value, expected range [0..15]");
    }
}
