use super::index_for_pixel::index_for_pixel;
use std::fmt;

/// Tiny 2D grid with 4 bits per pixel, max size 256 x 256 pixels.
#[derive(Clone, PartialEq)]
pub struct Image {
    width: u8,
    height: u8,
    pixels: Vec<u8>,
}

impl Image {
    pub fn empty() -> Self {
        Self { width: 0, height: 0, pixels: vec!() }
    }

    /// Create an `Image` instance, filled with `color`
    pub fn color(width: u8, height: u8, color: u8) -> Self {
        let len: usize = (width as usize) * (height as usize);
        let pixels: Vec<u8> = vec![color; len];
        Self { width, height, pixels }
    }

    /// Create an `Image` instance, filled with zeroes
    pub fn zero(width: u8, height: u8) -> Self {
        Self::color(width, height, 0)
    }

    /// Create a `Image` instance without any checks of the data
    /// 
    /// It's up to the caller to ensure:
    /// - Make sure that the pixels.len() is the same as width x height.
    /// - Make sure that when width=0, that height is not greater than 0.
    /// - Make sure that when height=0, that width is not greater than 0.
    pub fn create_raw(width: u8, height: u8, pixels: Vec<u8>) -> Self {
        Self { width, height, pixels }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn pixels(&self) -> &Vec<u8> {
        &self.pixels
    }

    pub fn index_for_pixel(&self, x: i32, y: i32) -> Option<usize> {
        index_for_pixel(x, y, self.width, self.height)
    }

    /// Get pixel value at coordinate (x, y).
    pub fn get(&self, x: i32, y: i32) -> Option<u8> {
        let index: usize = self.index_for_pixel(x, y)?;
        if index >= self.pixels.len() {
            return None;
        }
        Some(self.pixels[index])
    }

    /// Set pixel value at coordinate (x, y).
    pub fn set(&mut self, x: i32, y: i32, value: u8) -> Option<()> {
        let index: usize = self.index_for_pixel(x, y)?;
        if index >= self.pixels.len() {
            return None;
        }
        self.pixels[index] = value;
        Some(())
    }

    pub fn human_readable(&self) -> String {
        let mut s = String::new();
        for y in 0..self.height {
            if y > 0 {
                s += "\n";
            }
            for x in 0..self.width {
                let pixel_value: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                if x > 0 {
                    s += " ";
                }
                s += &format!("{:X?}", pixel_value);
            }
        }
        s
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Image {}x{}\n{}", self.width, self.height, self.human_readable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_init_empty() {
        let bm = Image::empty();
        assert_eq!(bm.width(), 0);
        assert_eq!(bm.height(), 0);
        assert_eq!(bm.pixels().is_empty(), true);
        assert_eq!(bm.is_empty(), true);
    }

    #[test]
    fn test_10001_init_color() {
        let bm = Image::color(4, 3, 1);
        assert_eq!(bm.width(), 4);
        assert_eq!(bm.height(), 3);
        assert_eq!(bm.pixels().len(), 4 * 3);
        assert_eq!(bm.is_empty(), false);
        let mut sum: usize = 0;
        for pixel in bm.pixels() {
            sum += *pixel as usize;
        }
        assert_eq!(sum, 12);
    }

    #[test]
    fn test_10002_init_zero() {
        let bm = Image::zero(4, 3);
        assert_eq!(bm.width(), 4);
        assert_eq!(bm.height(), 3);
        assert_eq!(bm.pixels().len(), 4 * 3);
        assert_eq!(bm.is_empty(), false);
        let mut sum: usize = 0;
        for pixel in bm.pixels() {
            sum += *pixel as usize;
        }
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_20000_get_set_pixel_value_ok() {
        let mut bm = Image::zero(3, 2);
        bm.set(0, 0, 1).expect("ok");
        bm.set(1, 0, 2).expect("ok");
        bm.set(2, 0, 3).expect("ok");
        bm.set(0, 1, 4).expect("ok");
        bm.set(1, 1, 5).expect("ok");
        bm.set(2, 1, 6).expect("ok");
        assert_eq!(bm.get(0, 0), Some(1));
        assert_eq!(bm.get(1, 0), Some(2));
        assert_eq!(bm.get(2, 0), Some(3));
        assert_eq!(bm.get(0, 1), Some(4));
        assert_eq!(bm.get(1, 1), Some(5));
        assert_eq!(bm.get(2, 1), Some(6));
    }

    #[test]
    fn test_20001_get_set_pixel_value_ok() {
        let mut bm = Image::zero(3, 1);
        bm.set(0, 0, 253).expect("ok");
        bm.set(1, 0, 254).expect("ok");
        bm.set(2, 0, 255).expect("ok");
        assert_eq!(bm.get(0, 0), Some(253));
        assert_eq!(bm.get(1, 0), Some(254));
        assert_eq!(bm.get(2, 0), Some(255));
    }

    #[test]
    fn test_20001_set_pixel_value_error() {
        let mut bm = Image::zero(3, 2);
        // negative coordinates
        assert_eq!(bm.set(-1, 0, 0), None);
        assert_eq!(bm.set(0, -1, 0), None);

        // beyond width or height
        assert_eq!(bm.set(3, 0, 0), None);
        assert_eq!(bm.set(0, 2, 0), None);
    }

    #[test]
    fn test_30000_compare() {
        {
            let mut bm0 = Image::zero(3, 2);
            bm0.set(0, 0, 255).expect("ok");
            bm0.set(2, 1, 255).expect("ok");
            let bm1 = Image::create_raw(3, 2, vec![255, 0, 0, 0, 0, 255]);
            assert_eq!(bm0, bm1);
        }
        {
            let mut bm0 = Image::zero(3, 2);
            bm0.set(0, 0, 255).expect("ok");
            bm0.set(2, 1, 254).expect("ok");
            let bm1 = Image::create_raw(3, 2, vec![255, 0, 0, 0, 0, 255]);
            assert_ne!(bm0, bm1);
        }
        {
            let mut bm0 = Image::create_raw(3, 2, vec![255, 0, 0, 0, 0, 255]);
            bm0.set(0, 0, 0).expect("ok");
            bm0.set(2, 1, 0).expect("ok");
            let bm1 = Image::zero(3, 2);
            assert_eq!(bm0, bm1);
        }
    }
}
