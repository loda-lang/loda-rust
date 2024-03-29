use super::Image;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum MixMode {
    /// Add two values, clamp to 255.
    Plus,

    /// Subtract two values, clamp to 0.
    Minus,

    /// Maximum of two values
    Max,

    /// Minimum of two values
    Min,

    /// Multiply two values, clamp to 255.
    Multiply,

    /// When the colors are identical then return 1. When the colors differ then return 0.
    IsSame,

    /// When the colors are different then return 1. When the colors are identical then return 0.
    IsDifferent,

    /// Find differences, only where color0 is the specified `color`.
    /// When the colors are different then return 1. When the colors are identical then return 0.
    /// 
    /// If `color0` is different than the specified `color` then return 0.
    IsDifferentOnlyConsideringColor0 { color: u8 },

    /// When the colors are identical then return the color. 
    /// When the colors differ then return the `disagreement_color`.
    AgreeOrColor { disagreement_color: u8 },

    /// Overlay another image on top.
    /// 
    /// Pick `color1` as the overlay when `color0` is the same as the `color0_filter`.
    /// 
    /// Pick `color0` as the backdrop when `color0` is different than the `color0_filter`.
    PickColor1WhenColor0IsDifferent { color0_filter: u8 },

    /// Choose between 2 colors. When `color0 == 0` then pick `color1`. 
    /// Otherwise use the specified `color`.
    PickColor1WhenColor0IsZero { color: u8 },

    /// Choose between 2 colors. When `color0 != 0` then pick `color1`. 
    /// Otherwise use the specified `color`.
    PickColor1WhenColor0IsNonZero { color: u8 },

    /// When both colors are non-zero then `1` is returned.
    /// Otherwise `0` is returned.
    /// 
    /// Consider color values that are non-zero as true.
    /// Consider color values that are zero as false.
    /// 
    /// Performs an AND operation between the values.
    BooleanAnd,

    /// If both colors are zero then `0` is returned.
    /// Otherwise `1` is returned.
    /// 
    /// Consider color values that are non-zero as true.
    /// Consider color values that are zero as false.
    /// 
    /// Performs an OR operation between the values.
    BooleanOr,

    /// When one of the colors are non-zero and the other color is zero then `1` is returned.
    /// Otherwise `0` is returned.
    /// 
    /// Consider color values that are non-zero as true.
    /// Consider color values that are zero as false.
    /// 
    /// Performs an XOR operation between the values.
    BooleanXor,

    /// Performs a bitwise AND operation between the values.
    BitwiseAnd,

    /// Performs a bitwise OR operation between the values.
    BitwiseOr,

    /// Performs a bitwise XOR operation between the values.
    BitwiseXor,

    // Future experiments:
    // Absolute difference
    // Divide
    // IsGreaterThan,
    // IsLessThan,
    // GreaterThanOrEqualTo { limit }
    // LessThanOrEqualTo { limit }
    // ClampRange { min, max }
    // IsInRange { min, max }
}

impl MixMode {
    fn compute(&self, color0: u8, color1: u8) -> anyhow::Result<u8> {
        let result_color: u8 = match self {
            MixMode::Plus => {
                ((color0 as u16) + (color1 as u16)).min(u8::MAX as u16) as u8
            },
            MixMode::Minus => {
                ((color0 as i16) - (color1 as i16)).max(0 as i16) as u8
            },
            MixMode::Max => {
                color0.max(color1)
            },
            MixMode::Min => {
                color0.min(color1)
            },
            MixMode::Multiply => {
                ((color0 as u16) * (color1 as u16)).min(u8::MAX as u16) as u8
            },
            MixMode::IsSame => {
                if color0 == color1 { 1 } else { 0 }
            },
            MixMode::IsDifferent => {
                if color0 != color1 { 1 } else { 0 }
            },
            MixMode::IsDifferentOnlyConsideringColor0 { color } => {
                if color0 == *color && color0 != color1 { 1 } else { 0 }
            },
            MixMode::AgreeOrColor { disagreement_color } => {
                if color0 == color1 { color0 } else { *disagreement_color }
            },
            MixMode::PickColor1WhenColor0IsDifferent { color0_filter } => {
                if color0 == *color0_filter { color1 } else { color0 }
            },
            MixMode::PickColor1WhenColor0IsZero { color } => {
                if color0 == 0 { color1 } else { *color }
            },
            MixMode::PickColor1WhenColor0IsNonZero { color } => {
                if color0 == 0 { *color } else { color1 }
            },
            MixMode::BooleanAnd => {
                if (color0 > 0) & (color1 > 0) { 1 } else { 0 }
            },
            MixMode::BooleanOr => {
                if (color0 > 0) | (color1 > 0) { 1 } else { 0 }
            },
            MixMode::BooleanXor => {
                if (color0 > 0) ^ (color1 > 0) { 1 } else { 0 }
            },
            MixMode::BitwiseAnd => {
                color0 & color1
            },
            MixMode::BitwiseOr => {
                color0 | color1
            },
            MixMode::BitwiseXor => {
                color0 ^ color1
            },
        };
        Ok(result_color)
    }
}

pub trait ImageMix {
    /// Combine colors from 2 images using the `mode`.
    fn mix(&self, other: &Image, mode: MixMode) -> anyhow::Result<Image>;
}

impl ImageMix for Image {
    fn mix(&self, other: &Image, mode: MixMode) -> anyhow::Result<Image> {
        if self.size() != other.size() {
            return Err(anyhow::anyhow!("mix: Both images must have same size."));
        }
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let mut result_image: Image = self.clone();
        for y in 0..self.height() {
            for x in 0..self.width() {
                let color0: u8 = self.get(x as i32, y as i32).unwrap_or(255); 
                let color1: u8 = other.get(x as i32, y as i32).unwrap_or(255); 
                let set_color: u8 = mode.compute(color0, color1)?;
                match result_image.set(x as i32, y as i32, set_color) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("mix: Unable to set pixel"));
                    }
                }
            }
        }
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_mixmode_plus() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (39, 3, 42),
            (3, 39, 42),
            (254, 1, 255),
            (1, 254, 255),
            (255, 1, 255),
            (1, 255, 255),
        ];
        // Act
        let mode = MixMode::Plus;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_mixmode_minus() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (2, 1, 1),
            (1, 2, 0),
            (39, 3, 36),
            (3, 39, 0),
            (254, 1, 253),
            (1, 254, 0),
            (255, 1, 254),
            (1, 255, 0),
        ];
        // Act
        let mode = MixMode::Minus;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_mixmode_max() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (39, 3, 39),
            (3, 39, 39),
            (254, 1, 254),
            (1, 254, 254),
            (255, 1, 255),
            (1, 255, 255),
        ];
        // Act
        let mode = MixMode::Max;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_mixmode_min() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (39, 3, 3),
            (3, 39, 3),
            (254, 1, 1),
            (1, 254, 1),
            (255, 1, 1),
            (1, 255, 1),
        ];
        // Act
        let mode = MixMode::Min;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_mixmode_multiply() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (0, 1, 0),
            (1, 0, 0),
            (1, 1, 1),
            (3, 3, 9),
            (3, 39, 117),
            (39, 3, 117),
            (254, 1, 254),
            (1, 254, 254),
            (51, 5, 255),
            (5, 51, 255),
            (16, 16, 255),
        ];
        // Act
        let mode = MixMode::Multiply;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10006_mixmode_issame() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 1),
            (1, 1, 1),
            (39, 3, 0),
            (3, 39, 0),
            (39, 39, 1),
            (254, 1, 0),
            (1, 254, 0),
            (255, 1, 0),
            (1, 255, 0),
            (255, 255, 1),
        ];
        // Act
        let mode = MixMode::IsSame;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10007_mixmode_isdifferent() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (1, 1, 0),
            (39, 3, 1),
            (3, 39, 1),
            (39, 39, 0),
            (254, 1, 1),
            (1, 254, 1),
            (255, 1, 1),
            (1, 255, 1),
            (255, 255, 0),
        ];
        // Act
        let mode = MixMode::IsDifferent;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10008_mixmode_isdifferentonlyconsideringcolor0() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (1, 1, 0),
            (39, 3, 0),
            (3, 39, 1), // the only place where color0 is 3, and there is a difference
            (3, 3, 0),
            (39, 39, 0),
            (254, 1, 0),
            (1, 254, 0),
            (255, 1, 0),
            (1, 255, 0),
            (255, 255, 0),
        ];
        // Act
        let mode = MixMode::IsDifferentOnlyConsideringColor0 { color: 3 };
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10009_mixmode_agreeorcolor() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (1, 1, 1),
            (39, 3, 255),
            (3, 39, 255),
            (39, 39, 39),
            (254, 1, 255),
            (1, 254, 255),
            (255, 1, 255),
            (1, 255, 255),
            (255, 255, 255),
        ];
        // Act
        let mode = MixMode::AgreeOrColor { disagreement_color: 255 };
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10010_mixmode_pickcolor1whencolor0isdifferent() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (1, 1, 1),
            (39, 3, 39),
            (3, 0, 0),
            (3, 39, 39),
            (3, 255, 255),
            (39, 39, 39),
            (254, 1, 254),
            (1, 254, 1),
            (255, 1, 255),
            (1, 255, 1),
            (255, 255, 255),
        ];
        // Act
        let mode = MixMode::PickColor1WhenColor0IsDifferent { color0_filter: 3 };
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10011_mixmode_pickcolor1whencolor0iszero() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (0, 1, 1),
            (0, 3, 3),
            (0, 255, 255),
            (1, 1, 42),
            (39, 3, 42),
            (3, 0, 42),
            (3, 39, 42),
            (3, 255, 42),
            (39, 39, 42),
            (254, 1, 42),
            (1, 254, 42),
            (255, 1, 42),
            (1, 255, 42),
            (255, 255, 42),
        ];
        // Act
        let mode = MixMode::PickColor1WhenColor0IsZero { color: 42 };
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10012_mixmode_pickcolor1whencolor0isnonzero() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 42),
            (0, 1, 42),
            (0, 3, 42),
            (0, 255, 42),
            (1, 1, 1),
            (39, 3, 3),
            (3, 0, 0),
            (3, 39, 39),
            (3, 255, 255),
        ];
        // Act
        let mode = MixMode::PickColor1WhenColor0IsNonZero { color: 42 };
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10013_mixmode_boolean_and() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (0, 1, 0),
            (1, 0, 0),
            (1, 1, 1),
            (0, 42, 0),
            (42, 0, 0),
            (42, 42, 1),
            (0, 3, 0),
            (0, 255, 0),
            (39, 3, 1),
            (3, 0, 0),
            (3, 39, 1),
            (3, 255, 1),
        ];
        // Act
        let mode = MixMode::BooleanAnd;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10014_mixmode_boolean_or() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (0, 1, 1),
            (1, 0, 1),
            (1, 1, 1),
            (0, 42, 1),
            (42, 0, 1),
            (42, 42, 1),
            (0, 3, 1),
            (0, 255, 1),
            (39, 3, 1),
            (3, 0, 1),
            (3, 39, 1),
            (3, 255, 1),
        ];
        // Act
        let mode = MixMode::BooleanOr;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10015_mixmode_boolean_xor() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (0, 1, 1),
            (1, 0, 1),
            (1, 1, 0),
            (0, 42, 1),
            (42, 0, 1),
            (42, 42, 0),
            (0, 3, 1),
            (0, 255, 1),
            (39, 3, 0),
            (3, 0, 1),
            (3, 39, 0),
            (3, 255, 0),
        ];
        // Act
        let mode = MixMode::BooleanXor;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10016_mixmode_bitwise_and() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (0, 1, 0),
            (1, 0, 0),
            (1, 1, 1),
            (0, 42, 0),
            (42, 0, 0),
            (42, 42, 42),
            (7, 3, 3),
            (3, 7, 3),
            (7, 5, 5),
            (5, 7, 5),
            (8, 7, 0),
            (255, 7, 7),
        ];
        // Act
        let mode = MixMode::BitwiseAnd;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10017_mixmode_bitwise_or() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (0, 1, 1),
            (1, 0, 1),
            (1, 1, 1),
            (0, 42, 42),
            (42, 0, 42),
            (42, 42, 42),
            (7, 3, 7),
            (3, 7, 7),
            (7, 5, 7),
            (5, 7, 7),
            (8, 7, 15),
            (255, 7, 255),
        ];
        // Act
        let mode = MixMode::BitwiseOr;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10018_mixmode_bitwise_xor() {
        // Arrange
        let items: Vec<(u8, u8, u8)> = vec![
            (0, 0, 0),
            (0, 1, 1),
            (1, 0, 1),
            (1, 1, 0),
            (7, 6, 1),
            (5, 7, 2),
            (3, 12, 15),
            (255, 255, 0),
        ];
        // Act
        let mode = MixMode::BitwiseXor;
        let actual: Vec<u8> = items.iter().map(|(color0, color1, _expected)| mode.compute(*color0, *color1).expect("ok") ).collect();

        // Arrange
        let expected: Vec<u8> = items.iter().map(|(_color0, _color1, expected)| *expected ).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_mix_plus() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 2,
            3, 4,
            5, 6,
        ];
        let input0: Image = Image::try_create(2, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            6, 5,
            4, 3,
            2, 1,
        ];
        let input1: Image = Image::try_create(2, 3, pixels1).expect("image");

        // Act
        let actual: Image = input0.mix(&input1, MixMode::Plus).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            7, 7,
            7, 7,
            7, 7,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
