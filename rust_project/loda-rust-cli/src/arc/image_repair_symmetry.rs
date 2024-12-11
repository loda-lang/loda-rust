use super::{Image, Rectangle, Color};

pub trait ImageRepairSymmetry {
    /// Attempts to repair the pixels with the color `Color::CannotCompute`.
    /// 
    /// It's not be possible to repair a pixel, if the mirror image also contains `Color::CannotCompute`.
    /// 
    /// The `rect` must be horizontal symmetric inside the area between `rect.min_x` and `rect.max_x`.
    /// 
    /// Returns the number of pixels that have been repaired.
    fn repair_symmetry_horizontal(&mut self, rect: Rectangle) -> anyhow::Result<u16>;

    /// Attempts to repair the pixels with the color `Color::CannotCompute`.
    /// 
    /// It's not be possible to repair a pixel, if the mirror image also contains `Color::CannotCompute`.
    /// 
    /// The `rect` must be vertical symmetric inside the area between `rect.min_y` and `rect.max_y`.
    /// 
    /// Returns the number of pixels that have been repaired.
    fn repair_symmetry_vertical(&mut self, rect: Rectangle) -> anyhow::Result<u16>;

    /// Attempts to repair the pixels with the color `Color::CannotCompute`.
    /// 
    /// It's not be possible to repair a pixel, if the mirror image also contains `Color::CannotCompute`.
    /// 
    /// The `square` must be diagonal-a symmetric and be a square.
    /// 
    /// Returns the number of pixels that have been repaired.
    fn repair_symmetry_diagonal_a(&mut self, square: Rectangle) -> anyhow::Result<u16>;

    /// Attempts to repair the pixels with the color `Color::CannotCompute`.
    /// 
    /// It's not be possible to repair a pixel, if the mirror image also contains `Color::CannotCompute`.
    /// 
    /// The `square` must be diagonal-b symmetric and be a square.
    /// 
    /// Returns the number of pixels that have been repaired.
    fn repair_symmetry_diagonal_b(&mut self, square: Rectangle) -> anyhow::Result<u16>;
}

impl ImageRepairSymmetry for Image {
    fn repair_symmetry_horizontal(&mut self, rect: Rectangle) -> anyhow::Result<u16> {
        if rect.max_x() >= (self.width() as i32) || rect.max_y() >= (self.height() as i32) {
            return Err(anyhow::anyhow!("rect must be inside image"));
        }
        if self.is_empty() || rect.is_empty() {
            return Ok(0);
        }
        let mut repair_count: u16 = 0;
        for j in 0..rect.height() {
            for i in 0..rect.width() {
                let y: i32 = rect.min_y() + (j as i32);
                let x_left: i32 = rect.min_x() + (i as i32);
                let x_right: i32 = rect.max_x() - (i as i32);
                let pixel_value: u8 = self.get(x_left, y).unwrap_or(0);
                if pixel_value != Color::CannotCompute as u8 {
                    continue;
                }
                let pixel_value: u8 = self.get(x_right, y).unwrap_or(0);
                if pixel_value != Color::CannotCompute as u8 {
                    _ = self.set(x_left, y, pixel_value);
                    repair_count += 1;
                }
            }
        }
        Ok(repair_count)
    }

    fn repair_symmetry_vertical(&mut self, rect: Rectangle) -> anyhow::Result<u16> {
        if rect.max_x() >= (self.width() as i32) || rect.max_y() >= (self.height() as i32) {
            return Err(anyhow::anyhow!("rect must be inside image"));
        }
        if self.is_empty() || rect.is_empty() {
            return Ok(0);
        }
        let mut repair_count: u16 = 0;
        for i in 0..rect.width() {
            for j in 0..rect.height() {
                let x: i32 = rect.min_x() + (i as i32);
                let y_top: i32 = rect.min_y() + (j as i32);
                let y_bottom: i32 = rect.max_y() - (j as i32);
                let pixel_value: u8 = self.get(x, y_top).unwrap_or(0);
                if pixel_value != Color::CannotCompute as u8 {
                    continue;
                }
                let pixel_value: u8 = self.get(x, y_bottom).unwrap_or(0);
                if pixel_value != Color::CannotCompute as u8 {
                    _ = self.set(x, y_top, pixel_value);
                    repair_count += 1;
                }
            }
        }
        Ok(repair_count)
    }

    fn repair_symmetry_diagonal_a(&mut self, square: Rectangle) -> anyhow::Result<u16> {
        if square.max_x() >= (self.width() as i32) || square.max_y() >= (self.height() as i32) {
            return Err(anyhow::anyhow!("square must be inside image"));
        }
        if square.width() != square.height() {
            return Err(anyhow::anyhow!("must be a square"));
        }
        if self.is_empty() || square.is_empty() {
            return Ok(0);
        }
        let mut repair_count: u16 = 0;
        for i in 0..square.width() {
            for j in 0..square.height() {
                let x: i32 = square.min_x() + (i as i32);
                let y: i32 = square.min_y() + (j as i32);
                let x_mirror: i32 = square.min_x() + (j as i32);
                let y_mirror: i32 = square.min_y() + (i as i32);
                let pixel_value: u8 = self.get(x, y).unwrap_or(0);
                if pixel_value != Color::CannotCompute as u8 {
                    continue;
                }
                let pixel_value: u8 = self.get(x_mirror, y_mirror).unwrap_or(0);
                if pixel_value != Color::CannotCompute as u8 {
                    _ = self.set(x, y, pixel_value);
                    repair_count += 1;
                }
            }
        }
        Ok(repair_count)
    }

    fn repair_symmetry_diagonal_b(&mut self, square: Rectangle) -> anyhow::Result<u16> {
        if square.max_x() >= (self.width() as i32) || square.max_y() >= (self.height() as i32) {
            return Err(anyhow::anyhow!("square must be inside image"));
        }
        if square.width() != square.height() {
            return Err(anyhow::anyhow!("must be a square"));
        }
        if self.is_empty() || square.is_empty() {
            return Ok(0);
        }
        let mut repair_count: u16 = 0;
        for i in 0..square.width() {
            for j in 0..square.height() {
                let x: i32 = square.min_x() + (i as i32);
                let y: i32 = square.min_y() + (j as i32);
                let x_mirror: i32 = square.max_x() - (j as i32);
                let y_mirror: i32 = square.max_y() - (i as i32);
                let pixel_value: u8 = self.get(x, y).unwrap_or(0);
                if pixel_value != Color::CannotCompute as u8 {
                    continue;
                }
                let pixel_value: u8 = self.get(x_mirror, y_mirror).unwrap_or(0);
                if pixel_value != Color::CannotCompute as u8 {
                    _ = self.set(x, y, pixel_value);
                    repair_count += 1;
                }
            }
        }
        Ok(repair_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_repair_symmetry_horizontal() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            1, 1, 2, 1, 1,
            2, a, 0, 1, 2,
            3, a, 3, 3, 3,
            4, 0, 0, a, a,
            1, 1, 0, a, a
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let rect = Rectangle::new(0, 0, 5, 5);

        // Act
        let mut actual: Image = input.clone();
        let repair_count: u16 = actual.repair_symmetry_horizontal(rect).expect("u16");

        // Assert
        assert_eq!(repair_count, 6);
        let expected_pixels: Vec<u8> = vec![
            1, 1, 2, 1, 1,
            2, 1, 0, 1, 2,
            3, 3, 3, 3, 3,
            4, 0, 0, 0, 4,
            1, 1, 0, 1, 1
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_repair_symmetry_horizontal() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            1, 1, 2, 1, 1,
            2, a, 0, 1, 2,
            3, a, 3, 3, 3,
            4, 0, 0, a, a,
            1, 1, 0, a, a
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let rect = Rectangle::new(0, 2, 5, 2);

        // Act
        let mut actual: Image = input.clone();
        let repair_count: u16 = actual.repair_symmetry_horizontal(rect).expect("u16");

        // Assert
        assert_eq!(repair_count, 3);
        let expected_pixels: Vec<u8> = vec![
            1, 1, 2, 1, 1,
            2, a, 0, 1, 2,
            3, 3, 3, 3, 3,
            4, 0, 0, 0, 4,
            1, 1, 0, a, a
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_repair_symmetry_vertical() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            1, 1, 5, 7, 3,
            2, 0, 5, 7, a,
            3, 1, 5, 0, 3,
            2, 0, a, a, 3,
            1, 1, a, a, 3,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let rect = Rectangle::new(0, 0, 5, 5);

        // Act
        let mut actual: Image = input.clone();
        let repair_count: u16 = actual.repair_symmetry_vertical(rect).expect("u16");

        // Assert
        assert_eq!(repair_count, 5);
        let expected_pixels: Vec<u8> = vec![
            1, 1, 5, 7, 3,
            2, 0, 5, 7, 3,
            3, 1, 5, 0, 3,
            2, 0, 5, 7, 3,
            1, 1, 5, 7, 3,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_repair_symmetry_vertical() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            1, 1, 5, 7, 3,
            2, 0, 5, 7, a,
            3, 1, 5, 0, 3,
            2, 0, a, a, 3,
            1, 1, a, a, 3,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let rect = Rectangle::new(1, 0, 2, 5);

        // Act
        let mut actual: Image = input.clone();
        let repair_count: u16 = actual.repair_symmetry_vertical(rect).expect("u16");

        // Assert
        assert_eq!(repair_count, 2);
        let expected_pixels: Vec<u8> = vec![
            1, 1, 5, 7, 3,
            2, 0, 5, 7, a,
            3, 1, 5, 0, 3,
            2, 0, 5, a, 3,
            1, 1, 5, a, 3,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_repair_symmetry_diagonal_a() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            1, a, a, a, a,
            1, 0, a, a, a,
            1, 1, 1, 0, a,
            0, 0, 0, 5, 5,
            0, 0, 0, 5, 5,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let rect = Rectangle::new(0, 0, 5, 5);

        // Act
        let mut actual: Image = input.clone();
        let repair_count: u16 = actual.repair_symmetry_diagonal_a(rect).expect("u16");

        // Assert
        assert_eq!(repair_count, 8);
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 0, 0,
            1, 0, 1, 0, 0,
            1, 1, 1, 0, 0,
            0, 0, 0, 5, 5,
            0, 0, 0, 5, 5,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30001_repair_symmetry_diagonal_a() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            1, a, a, a, a,
            1, 0, a, a, a,
            1, 1, 1, 0, a,
            0, 0, 0, 5, 5,
            0, 0, 0, 5, 5,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let rect = Rectangle::new(1, 1, 3, 3);

        // Act
        let mut actual: Image = input.clone();
        let repair_count: u16 = actual.repair_symmetry_diagonal_a(rect).expect("u16");

        // Assert
        assert_eq!(repair_count, 2);
        let expected_pixels: Vec<u8> = vec![
            1, a, a, a, a,
            1, 0, 1, 0, a,
            1, 1, 1, 0, a,
            0, 0, 0, 5, 5,
            0, 0, 0, 5, 5,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_repair_symmetry_diagonal_b() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            a, a, a, a, 1,
            a, a, a, 0, 1,
            a, 0, 1, 1, 1,
            5, 5, 0, 0, 0,
            5, 5, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let rect = Rectangle::new(0, 0, 5, 5);

        // Act
        let mut actual: Image = input.clone();
        let repair_count: u16 = actual.repair_symmetry_diagonal_b(rect).expect("u16");

        // Assert
        assert_eq!(repair_count, 8);
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1,
            0, 0, 1, 0, 1,
            0, 0, 1, 1, 1,
            5, 5, 0, 0, 0,
            5, 5, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40001_repair_symmetry_diagonal_b() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            a, a, a, a, 1,
            a, a, a, 0, 1,
            a, 0, 1, 1, 1,
            5, 5, 0, 0, 0,
            5, 5, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let rect = Rectangle::new(1, 1, 3, 3);

        // Act
        let mut actual: Image = input.clone();
        let repair_count: u16 = actual.repair_symmetry_diagonal_b(rect).expect("u16");

        // Assert
        assert_eq!(repair_count, 2);
        let expected_pixels: Vec<u8> = vec![
            a, a, a, a, 1,
            a, 0, 1, 0, 1,
            a, 0, 1, 1, 1,
            5, 5, 0, 0, 0,
            5, 5, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

}
