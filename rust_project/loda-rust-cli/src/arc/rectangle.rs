use super::ImageSize;


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
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

    #[allow(dead_code)]
    pub fn x(&self) -> u8 {
        self.x
    }

    #[allow(dead_code)]
    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn size(&self) -> ImageSize {
        ImageSize { width: self.width, height: self.height }
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

    /// x-coordinate for the right column
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
    
    fn range(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Option<Rectangle> {
        if min_x > max_x || min_y > max_y {
            return None;
        }

        // Left position
        if min_x < 0 || min_x > (u8::MAX as i32) {
            return None;
        }
        let x: u8 = min_x as u8;

        // Top position
        if min_y < 0 || min_y > (u8::MAX as i32) {
            return None;
        }
        let y: u8 = min_y as u8;

        // Width
        let new_width_i32: i32 = max_x - min_x + 1;
        if new_width_i32 < 1 || new_width_i32 > (u8::MAX as i32) {
            return None;
        }
        let width: u8 = new_width_i32 as u8;

        // Height
        let new_height_i32: i32 = max_y - min_y + 1;
        if new_height_i32 < 1 || new_height_i32 > (u8::MAX as i32) {
            return None;
        }
        let height: u8 = new_height_i32 as u8;

        Some(Rectangle::new(x, y, width, height))
    }

    /// Create rectangle between two coordinates
    #[allow(dead_code)]
    pub fn span(x0: i32, y0: i32, x1: i32, y1: i32) -> Option<Rectangle> {
        Self::range(x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1))
    }

    /// Find the overlap between two rectangles
    /// 
    /// If there is no overlap then `Rectangle::empty()` is returned.
    #[allow(dead_code)]
    pub fn intersection(&self, other: Rectangle) -> Rectangle {
        let x0 = self.min_x().max(other.min_x());
        let y0 = self.min_y().max(other.min_y());
        let x1 = self.max_x().min(other.max_x());
        let y1 = self.max_y().min(other.max_y());
        Self::range(x0, y0, x1, y1).unwrap_or_else(|| Rectangle::empty())
    }

    /// Check if the coordinate is on the corner of the rectangle.
    #[allow(dead_code)]
    pub fn corner_classification(&self, x: i32, y: i32) -> u8 {
        let mut result: u8 = 0;
        if x == self.min_x() && y == self.min_y() {
            result |= 1;
        }
        if x == self.max_x() && y == self.min_y() {
            result |= 2;
        }
        if x == self.min_x() && y == self.max_y() {
            result |= 4;
        }
        if x == self.max_x() && y == self.max_y() {
            result |= 8;
        }
        result
    }

    /// Check if the coordinate is inside the rectangle.
    #[allow(dead_code)]
    pub fn is_inside(&self, x: i32, y: i32) -> bool {
        x >= self.min_x() && x <= self.max_x() && y >= self.min_y() && y <= self.max_y()
    }

    /// Check if the coordinate is above the rectangle.
    #[allow(dead_code)]
    pub fn is_above(&self, x: i32, y: i32) -> bool {
        x >= self.min_x() && x <= self.max_x() && y < self.min_y()
    }

    /// Check if the coordinate is below the rectangle.
    #[allow(dead_code)]
    pub fn is_below(&self, x: i32, y: i32) -> bool {
        x >= self.min_x() && x <= self.max_x() && y > self.max_y()
    }

    /// Check if the coordinate is left of the rectangle.
    #[allow(dead_code)]
    pub fn is_left(&self, x: i32, y: i32) -> bool {
        x < self.min_x() && y >= self.min_y() && y <= self.max_y()
    }

    /// Check if the coordinate is right of the rectangle.
    #[allow(dead_code)]
    pub fn is_right(&self, x: i32, y: i32) -> bool {
        x > self.max_x() && y >= self.min_y() && y <= self.max_y()
    }

    /// Calculate the area of the rectangle, `width` * `height`.
    #[allow(dead_code)]
    pub fn area(&self) -> u16 {
        (self.width as u16) * (self.height as u16)
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

    #[test]
    fn test_40000_range_inside_bounds() {
        {
            let rect = Rectangle::range(0, 0, 0, 0);
            assert_eq!(rect, Some(Rectangle::new(0, 0, 1, 1)));
        }
        {
            let rect = Rectangle::range(0, 0, 1, 1);
            assert_eq!(rect, Some(Rectangle::new(0, 0, 2, 2)));
        }
        {
            let rect = Rectangle::range(10, 10, 99, 19);
            assert_eq!(rect, Some(Rectangle::new(10, 10, 90, 10)));
        }
    }

    #[test]
    fn test_40001_range_outside_bounds() {
        {
            let rect = Rectangle::range(-1, 0, 0, 0);
            assert_eq!(rect, None);
        }
        {
            let rect = Rectangle::range(10, 10, 9, 20);
            assert_eq!(rect, None);
        }
        {
            let rect = Rectangle::range(250, 0, 10, 1);
            assert_eq!(rect, None);
        }
    }

    #[test]
    fn test_50000_span() {
        let rect = Rectangle::span(0, 0, 0, 0);
        assert_eq!(rect, Some(Rectangle::new(0, 0, 1, 1)));
    }

    #[test]
    fn test_60000_intersection_empty() {
        let rect0 = Rectangle::empty();
        let rect1 = Rectangle::empty();
        let actual: Rectangle = rect0.intersection(rect1);
        assert_eq!(actual, Rectangle::empty());
    }

    #[test]
    fn test_60001_intersection_none() {
        let rect0 = Rectangle::new(0, 0, 1, 1);
        let rect1 = Rectangle::new(1, 0, 1, 1);
        let actual: Rectangle = rect0.intersection(rect1);
        assert_eq!(actual, Rectangle::empty());
    }

    #[test]
    fn test_60002_intersection_none() {
        let rect0 = Rectangle::new(0, 1, 1, 1);
        let rect1 = Rectangle::new(0, 0, 1, 1);
        let actual: Rectangle = rect0.intersection(rect1);
        assert_eq!(actual, Rectangle::empty());
    }
    #[test]
    fn test_60003_intersection_1x1() {
        let rect0 = Rectangle::new(0, 0, 1, 1);
        let rect1 = Rectangle::new(0, 0, 1, 1);
        let actual: Rectangle = rect0.intersection(rect1);
        assert_eq!(actual, Rectangle::new(0, 0, 1, 1));
    }

    #[test]
    fn test_60004_intersection() {
        let rect0 = Rectangle::new(1, 1, 4, 3);
        let rect1 = Rectangle::new(3, 2, 3, 3);
        let actual: Rectangle = rect0.intersection(rect1);
        assert_eq!(actual, Rectangle::new(3, 2, 2, 2));
    }

    #[test]
    fn test_60005_intersection() {
        let rect0 = Rectangle::new(3, 2, 3, 3);
        let rect1 = Rectangle::new(1, 1, 6, 3);
        let actual: Rectangle = rect0.intersection(rect1);
        assert_eq!(actual, Rectangle::new(3, 2, 3, 2));
    }

    #[test]
    fn test_70000_corner_classification() {
        {
            let rect = Rectangle::new(5, 5, 1, 1);
            assert_eq!(rect.corner_classification(5, 4), 0);
            assert_eq!(rect.corner_classification(4, 5), 0);
            assert_eq!(rect.corner_classification(5, 5), 15);
            assert_eq!(rect.corner_classification(6, 5), 0);
            assert_eq!(rect.corner_classification(5, 6), 0);
        }
        {
            let rect = Rectangle::new(10, 10, 10, 10);
            assert_eq!(rect.corner_classification(10, 10), 1);
            assert_eq!(rect.corner_classification(19, 10), 2);
            assert_eq!(rect.corner_classification(10, 19), 4);
            assert_eq!(rect.corner_classification(19, 19), 8);
        }
    }

    #[test]
    fn test_80000_is_inside() {
        {
            let rect = Rectangle::new(5, 5, 1, 1);
            assert_eq!(rect.is_inside(5, 4), false);
            assert_eq!(rect.is_inside(4, 5), false);
            assert_eq!(rect.is_inside(5, 5), true);
            assert_eq!(rect.is_inside(6, 5), false);
            assert_eq!(rect.is_inside(5, 6), false);
        }
        {
            let rect = Rectangle::new(10, 10, 10, 10);
            assert_eq!(rect.is_inside(15, 15), true);
            assert_eq!(rect.is_inside(10, 10), true);
            assert_eq!(rect.is_inside(19, 10), true);
            assert_eq!(rect.is_inside(10, 19), true);
            assert_eq!(rect.is_inside(19, 19), true);
            assert_eq!(rect.is_inside(15, 5), false);
            assert_eq!(rect.is_inside(5, 15), false);
            assert_eq!(rect.is_inside(25, 15), false);
            assert_eq!(rect.is_inside(15, 25), false);
        }
    }

    #[test]
    fn test_80001_is_above() {
        {
            let rect = Rectangle::new(5, 5, 1, 1);
            assert_eq!(rect.is_above(5, 4), true);
            assert_eq!(rect.is_above(4, 5), false);
            assert_eq!(rect.is_above(5, 5), false);
            assert_eq!(rect.is_above(6, 5), false);
            assert_eq!(rect.is_above(6, 6), false);
        }
        {
            let rect = Rectangle::new(10, 10, 3, 10);
            assert_eq!(rect.is_above(9, 9), false);
            assert_eq!(rect.is_above(10, 9), true);
            assert_eq!(rect.is_above(12, 9), true);
            assert_eq!(rect.is_above(13, 9), false);
        }
    }

    #[test]
    fn test_80002_is_below() {
        {
            let rect = Rectangle::new(5, 5, 1, 1);
            assert_eq!(rect.is_below(4, 6), false);
            assert_eq!(rect.is_below(5, 6), true);
            assert_eq!(rect.is_below(6, 6), false);
        }
        {
            let rect = Rectangle::new(10, 10, 3, 1);
            assert_eq!(rect.is_below(9, 11), false);
            assert_eq!(rect.is_below(10, 11), true);
            assert_eq!(rect.is_below(12, 11), true);
            assert_eq!(rect.is_below(13, 11), false);
        }
    }

    #[test]
    fn test_80003_is_left() {
        {
            let rect = Rectangle::new(5, 5, 1, 1);
            assert_eq!(rect.is_left(4, 4), false);
            assert_eq!(rect.is_left(4, 5), true);
            assert_eq!(rect.is_left(5, 5), false);
            assert_eq!(rect.is_left(4, 6), false);
        }
        {
            let rect = Rectangle::new(10, 10, 1, 3);
            assert_eq!(rect.is_left(9, 9), false);
            assert_eq!(rect.is_left(9, 10), true);
            assert_eq!(rect.is_left(9, 12), true);
            assert_eq!(rect.is_left(9, 13), false);
        }
    }

    #[test]
    fn test_80004_is_right() {
        {
            let rect = Rectangle::new(5, 5, 1, 1);
            assert_eq!(rect.is_right(6, 4), false);
            assert_eq!(rect.is_right(6, 5), true);
            assert_eq!(rect.is_right(5, 5), false);
            assert_eq!(rect.is_right(6, 6), false);
        }
        {
            let rect = Rectangle::new(10, 10, 1, 3);
            assert_eq!(rect.is_right(11, 9), false);
            assert_eq!(rect.is_right(11, 10), true);
            assert_eq!(rect.is_right(11, 12), true);
            assert_eq!(rect.is_right(11, 13), false);
        }
    }

    #[test]
    fn test_90000_area() {
        {
            let rect = Rectangle::new(5, 7, 10, 11);
            assert_eq!(rect.area(), 110);
        }
        {
            let rect = Rectangle::new(10, 20, 1, 3);
            assert_eq!(rect.area(), 3);
        }
    }
}
