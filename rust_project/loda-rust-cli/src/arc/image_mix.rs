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

    // Future experiments:
    // And
    // Or
    // Xor
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
        };
        Ok(result_color)
    }
}

#[allow(dead_code)]
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
