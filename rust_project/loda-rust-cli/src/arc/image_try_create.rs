use super::Image;

pub trait ImageTryCreate {
    fn try_create(width: u8, height: u8, pixels: Vec<u8>) -> anyhow::Result<Image>;
}

impl ImageTryCreate for Image {
    fn try_create(width: u8, height: u8, pixels: Vec<u8>) -> anyhow::Result<Image> {
        if width == 0 && height > 0 {
            return Err(anyhow::anyhow!("ImageTryCreate.try_create() width=0, but height>0, expected both to be zero"));
        }
        if width > 0 && height == 0 {
            return Err(anyhow::anyhow!("ImageTryCreate.try_create() height=0, but width>0, expected both to be zero"));
        }
        let len: usize = (width as usize) * (height as usize);
        if len != pixels.len() {
            return Err(anyhow::anyhow!("ImageTryCreate.try_create() Number of pixels {} doesn't match width {} x height {}", pixels.len(), width, height));
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
        let bm = Image::try_create(2, 2, vec![1,2,3,4]).expect("ok");
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
        Image::try_create(0, 2, vec!()).expect_err("width height should both be zero");
        Image::try_create(2, 0, vec!()).expect_err("width height should both be zero");
        Image::try_create(1, 1, vec!()).expect_err("width height doesn't match pixel count");
    }
}
