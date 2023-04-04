
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rectangle {
    x: u8,
    y: u8,
    width: u8,
    height: u8,
}

impl Rectangle {
    pub fn empty() -> Self {
        Self { x: 0, y: 0, width: 0, height: 0 }
    }

    pub fn new(x: u8, y: u8, width: u8, height: u8) -> Self {
        if width == 0 || height == 0 {
            return Self::empty();
        }
        Self { x, y, width, height }
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_copy() {
        // Arrange
        let rect_original = Rectangle::new(1, 2, 3, 4);

        // Act
        let rect0: Rectangle = rect_original;
        let rect1: Rectangle = rect_original;

        // Assert
        assert_eq!(rect0, Rectangle::new(1, 2, 3, 4));
        assert_eq!(rect1, Rectangle::new(1, 2, 3, 4));
    }

    #[test]
    fn test_20000_new_returning_empty() {
        {
            let rect = Rectangle::new(1, 2, 3, 0);
            assert_eq!(rect, Rectangle::empty());
        }
        {
            let rect = Rectangle::new(1, 2, 0, 4);
            assert_eq!(rect, Rectangle::empty());
        }
        {
            let rect = Rectangle::new(1, 2, 0, 0);
            assert_eq!(rect, Rectangle::empty());
        }
    }
}
