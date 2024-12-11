#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ImageSize {
    pub width: u8, 
    pub height: u8,
}

impl ImageSize {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self { width: 0, height: 0 }
    }

    #[allow(dead_code)]
    pub fn new(width: u8, height: u8) -> Self {
        if width == 0 || height == 0 {
            return Self::empty();
        }
        Self { width, height }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    #[allow(dead_code)]
    pub fn rotate(&self) -> Self {
        Self::new(self.height, self.width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_new() {
        {
            let actual = ImageSize::new(0, 4);
            assert_eq!(actual, ImageSize::empty());
        }
        {
            let actual = ImageSize::new(5, 0);
            assert_eq!(actual, ImageSize::empty());
        }
        {
            let actual = ImageSize::new(5, 4);
            assert_ne!(actual, ImageSize::empty());
            assert_eq!(actual.width, 5);
            assert_eq!(actual.height, 4);
        }
    }

    #[test]
    fn test_20000_rotate() {
        let actual = ImageSize::new(5, 4).rotate();
        assert_eq!(actual.width, 4);
        assert_eq!(actual.height, 5);
    }
}
