
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

    /// x-coordinate for the left column
    /// 
    /// Warning: When the rectangle is empty, then `0` is returned which is outside the rectangle.
    pub fn min_x(&self) -> i32 {
        self.x as i32
    }

    /// y-coordinate for the top row
    /// 
    /// Warning: When the rectangle is empty, then `0` is returned which is outside the rectangle.
    pub fn min_y(&self) -> i32 {
        self.y as i32
    }

    /// y-coordinate for the right column
    /// 
    /// Warning: When the rectangle is empty, then `-1` is returned which is outside the rectangle.
    pub fn max_x(&self) -> i32 {
        (self.x as i32) + (self.width as i32) - 1
    }

    /// y-coordinate for the bottom row
    /// 
    /// Warning: When the rectangle is empty, then `-1` is returned which is outside the rectangle.
    pub fn max_y(&self) -> i32 {
        (self.y as i32) + (self.height as i32) - 1
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

    #[test]
    fn test_30000_min_max_0x0_outside_the_rectangle() {
        let rect = Rectangle::empty();
        assert_eq!(rect.min_x(), 0);
        assert_eq!(rect.min_y(), 0);
        assert_eq!(rect.max_x(), -1);
        assert_eq!(rect.max_y(), -1);
    }

    #[test]
    fn test_30001_min_max_1x1() {
        {
            let rect = Rectangle::new(0, 0, 1, 1);
            assert_eq!(rect.min_x(), 0);
            assert_eq!(rect.min_y(), 0);
            assert_eq!(rect.max_x(), 0);
            assert_eq!(rect.max_y(), 0);
        }
        {
            let rect = Rectangle::new(10, 10, 1, 1);
            assert_eq!(rect.min_x(), 10);
            assert_eq!(rect.min_y(), 10);
            assert_eq!(rect.max_x(), 10);
            assert_eq!(rect.max_y(), 10);
        }
    }

    #[test]
    fn test_30002_min_max_2x2() {
        {
            let rect = Rectangle::new(0, 0, 2, 2);
            assert_eq!(rect.min_x(), 0);
            assert_eq!(rect.min_y(), 0);
            assert_eq!(rect.max_x(), 1);
            assert_eq!(rect.max_y(), 1);
        }
        {
            let rect = Rectangle::new(10, 0, 2, 2);
            assert_eq!(rect.min_x(), 10);
            assert_eq!(rect.min_y(), 0);
            assert_eq!(rect.max_x(), 11);
            assert_eq!(rect.max_y(), 1);
        }
        {
            let rect = Rectangle::new(0, 10, 2, 2);
            assert_eq!(rect.min_x(), 0);
            assert_eq!(rect.min_y(), 10);
            assert_eq!(rect.max_x(), 1);
            assert_eq!(rect.max_y(), 11);
        }
    }
}
