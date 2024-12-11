use super::Image;

#[allow(dead_code)]
pub trait CenterOfMass {
    /// The center of mass.
    /// 
    /// Ignores pixels that are zero.
    /// 
    /// Only considers non-zero pixels and assign them `mass = 1`.
    /// 
    /// https://en.wikipedia.org/wiki/Center_of_mass
    fn center_of_mass(&self, scale: u32) -> Option<(u32, u32)>;
}

impl CenterOfMass for Image {
    fn center_of_mass(&self, scale: u32) -> Option<(u32, u32)> {
        if self.is_empty() {
            return None;
        }
        let mut sum_x: u32 = 0;
        let mut sum_y: u32 = 0;
        let mut mass: u16 = 0;
        for y in 0..self.height() {
            for x in 0..self.width() {
                if self.get(x as i32, y as i32).unwrap_or(0) == 0 {
                    continue;
                }
                mass += 1;
                sum_x += x as u32;
                sum_y += y as u32;
            }
        }
        if mass == 0 {
            return None;
        }
        let center_x: u32 = sum_x * scale / (mass as u32);
        let center_y: u32 = sum_y * scale / (mass as u32);
        Some((center_x, center_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_centered() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1,
            1, 1,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let xy = input.center_of_mass(1000).expect("some");

        // Assert
        assert_eq!(xy, (500, 500));
    }

    #[test]
    fn test_10001_centered() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            1, 0, 0, 1,
            1, 0, 0, 1,
            0, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let xy = input.center_of_mass(1000).expect("some");

        // Assert
        assert_eq!(xy, (1500, 1500));
    }

    #[test]
    fn test_10002_centered() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0,
            1, 1, 0, 1, 1,
            0, 0, 1, 0, 0,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let xy = input.center_of_mass(1000).expect("some");

        // Assert
        assert_eq!(xy, (2000, 1000));
    }

    #[test]
    fn test_20000_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 1,
            1, 0, 0, 0,
            1, 0, 0, 0,
            1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let xy = input.center_of_mass(1000).expect("some");

        // Assert
        assert_eq!(xy, (1000, 1000));
    }

    #[test]
    fn test_20001_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 1,
            0, 0, 0, 1,
            0, 0, 0, 1,
            1, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let xy = input.center_of_mass(1000).expect("some");

        // Assert
        assert_eq!(xy, (2000, 2000));
    }

    #[test]
    fn test_30000_odd_shapes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 1, 1,
            1, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 2, pixels).expect("image");

        // Act
        let xy = input.center_of_mass(1000).expect("some");

        // Assert
        assert_eq!(xy, (2000, 200));
    }
}
