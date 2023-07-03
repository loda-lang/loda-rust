use super::{Image, ImageSize, ImageTrim, ImageRemoveDuplicates, ImageTryCreate, ImageRotate, ImageSymmetry};
use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShapeType {
    Empty,

    /// Solid square or rectangle.
    /// ````
    /// 1,
    /// ```
    Square,

    /// Solid rectangle.
    /// ````
    /// 1, 1,
    /// ```
    Rectangle,

    /// Rectangle with a rectangular hole.
    /// ````
    /// 1, 1, 1,
    /// 1, 0, 1,
    /// 1, 1, 1,
    /// ```
    Box,

    /// Shape `+`
    /// ````
    /// 0, 1, 0,
    /// 1, 1, 1,
    /// 0, 1, 0,
    /// ```
    Plus,

    /// Shape `O`
    /// ````
    /// 0, 1, 0,
    /// 1, 0, 1,
    /// 0, 1, 0,
    /// ```
    O,

    /// Shape `X`
    /// ````
    /// 1, 0, 1,
    /// 0, 1, 0,
    /// 1, 0, 1,
    /// ```
    X,

    /// Shape `L`
    /// ````
    /// 1, 0,
    /// 1, 1,
    /// ```
    L,

    // A shape like an upside down `T` symbol
    // https://en.wikipedia.org/wiki/Up_tack
    /// ````
    /// 0, 1, 0,
    /// 1, 1, 1,
    /// ```
    UpTack,

    /// A shape like an `U` symbol with 4 pixels. Asymmetric.
    /// ````
    /// 1, 0, 1
    /// 1, 1, 0,
    /// ```
    U4,

    /// A shape like an `U` symbol with 5 pixels. Symmetric.
    /// ````
    /// 1, 0, 1
    /// 1, 1, 1
    /// ```
    U5,

    /// Shape `I` or `H`
    /// ````
    /// 1, 1, 1,
    /// 0, 1, 0,
    /// 1, 1, 1,
    /// ```
    I,

    Unclassified,

    // Future experiments
    // V shape
    // tetris shape: [[1, 1, 0], [0, 1, 1]]
    // pyramid
    // diagonal cross
    // diagonal line
    // ◣ Lower Left Triangle
    // ◆ Diamond
}

impl ShapeType {
    fn name(&self) -> &str {
        match self {
            Self::Empty => "empty",
            Self::Square => "square",
            Self::Rectangle => "rectangle",
            Self::Box => "box",
            Self::Plus => "+",
            Self::O => "O",
            Self::L => "L",
            Self::UpTack => "⊥",
            Self::U4 => "U4",
            Self::U5 => "U5",
            Self::I => "I",
            Self::X => "X",
            Self::Unclassified => "unclassified",
        }
    }
}

impl Default for ShapeType {
    fn default() -> Self { ShapeType::Unclassified }
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ShapeIdentification {
    primary: ShapeType,
    secondary: Option<ShapeType>,
    width: Option<u8>,
    height: Option<u8>,
    rotated_cw_90: bool,
    rotated_cw_180: bool,
    rotated_cw_270: bool,
    flip_x: bool,
    flip_y: bool,
    flip_xy: bool,

    // Future experiments
    // is scaled down without losing information, apply scale factor to get original size
}

impl ShapeIdentification {
    #[allow(dead_code)]
    fn compute(mask: &Image) -> anyhow::Result<ShapeIdentification> {
        let mask2: Image = mask.trim_color(0)?;
        if mask2.is_empty() {
            let mut shape = ShapeIdentification::default();
            shape.primary = ShapeType::Empty;
            return Ok(shape);
        }
        if mask2.size() == ImageSize::new(1, 1) {
            let mut shape = ShapeIdentification::default();
            shape.primary = ShapeType::Square;
            shape.secondary = Some(ShapeType::Rectangle);
            shape.width = Some(1);
            shape.height = Some(1);
            shape.rotated_cw_90 = true;
            shape.rotated_cw_180 = true;
            shape.rotated_cw_270 = true;
            shape.flip_x = true;
            shape.flip_y = true;
            shape.flip_xy = true;
            return Ok(shape);
        }
        let mask3: Image = mask2.remove_duplicates()?;
        if mask3.size() == ImageSize::new(1, 1) {
            let is_square: bool = mask2.width() == mask2.height();
            if is_square {
                let scale: u8 = mask2.width();
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::Square;
                shape.secondary = Some(ShapeType::Rectangle);
                shape.width = Some(scale);
                shape.height = Some(scale);
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            } else {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::Rectangle;
                let size_min: u8 = mask2.width().min(mask2.height());
                let size_max: u8 = mask2.width().max(mask2.height());
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);    
            }
        }

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                1, 1, 1,
                1, 0, 1,
                1, 1, 1,
            ])?;

            if mask3 == shape_image {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::Box;
                let size_min: u8 = mask2.width().min(mask2.height());
                let size_max: u8 = mask2.width().max(mask2.height());
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }
        }

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                0, 1, 0,
                1, 1, 1,
                0, 1, 0,
            ])?;

            if mask3 == shape_image {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::Plus;
                let size_min: u8 = mask2.width().min(mask2.height());
                let size_max: u8 = mask2.width().max(mask2.height());
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }
        }

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                0, 1, 0,
                1, 0, 1,
                0, 1, 0,
            ])?;

            if mask3 == shape_image {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::O;
                let size_min: u8 = mask2.width().min(mask2.height());
                let size_max: u8 = mask2.width().max(mask2.height());
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }
        }

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                1, 0, 1,
                0, 1, 0,
                1, 0, 1,
            ])?;

            let is_same: bool = mask3 == shape_image;
            
            if is_same {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::X;
                let size_min: u8 = mask2.width().min(mask2.height());
                let size_max: u8 = mask2.width().max(mask2.height());
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }
        }

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                1, 1, 1,
                0, 1, 0,
                1, 1, 1,
            ])?;
            let rot_cw_90: Image = shape_image.rotate_cw()?;

            let is_same: bool = mask3 == shape_image;
            let is_rot_cw_90: bool = mask3 == rot_cw_90;
            
            if is_same || is_rot_cw_90 {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::I;
                let size_min: u8 = mask2.width().min(mask2.height());
                let size_max: u8 = mask2.width().max(mask2.height());
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.rotated_cw_90 = is_rot_cw_90;
                shape.rotated_cw_180 = is_same;
                shape.rotated_cw_270 = is_rot_cw_90;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }
        }

        if mask3.size() == ImageSize::new(2, 2) {
            let shape_image: Image = Image::try_create(2, 2, vec![
                1, 0,
                1, 1,
            ])?;
    
            let rot_cw_90: Image = shape_image.rotate_cw()?;
            let rot_cw_180: Image = rot_cw_90.rotate_cw()?;
            let rot_cw_270: Image = rot_cw_180.rotate_cw()?;

            let is_same: bool = mask3 == shape_image;
            let is_rot_cw_90: bool = mask3 == rot_cw_90;
            let is_rot_cw_180: bool = mask3 == rot_cw_180;
            let is_rot_cw_270: bool = mask3 == rot_cw_270;
            
            if is_same || is_rot_cw_90 || is_rot_cw_180 || is_rot_cw_270 {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::L;
                let size_min: u8 = mask2.width().min(mask2.height());
                let size_max: u8 = mask2.width().max(mask2.height());
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.rotated_cw_90 = is_rot_cw_90;
                shape.rotated_cw_180 = is_rot_cw_180;
                shape.rotated_cw_270 = is_rot_cw_270;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }
        }

        if mask3.size() == ImageSize::new(3, 2) || mask3.size().rotate() == ImageSize::new(3, 2) {
            let shape_image: Image = Image::try_create(3, 2, vec![
                0, 1, 0,
                1, 1, 1,
            ])?;
    
            let rot_cw_90: Image = shape_image.rotate_cw()?;
            let rot_cw_180: Image = rot_cw_90.rotate_cw()?;
            let rot_cw_270: Image = rot_cw_180.rotate_cw()?;

            let is_same: bool = mask3 == shape_image;
            let is_rot_cw_90: bool = mask3 == rot_cw_90;
            let is_rot_cw_180: bool = mask3 == rot_cw_180;
            let is_rot_cw_270: bool = mask3 == rot_cw_270;
            
            if is_same || is_rot_cw_90 || is_rot_cw_180 || is_rot_cw_270 {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::UpTack;
                let size_min: u8 = mask2.width().min(mask2.height());
                let size_max: u8 = mask2.width().max(mask2.height());
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.rotated_cw_90 = is_rot_cw_90;
                shape.rotated_cw_180 = is_rot_cw_180;
                shape.rotated_cw_270 = is_rot_cw_270;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }
        }

        if mask3.size() == ImageSize::new(3, 2) || mask3.size().rotate() == ImageSize::new(3, 2) {
            let shape_image: Image = Image::try_create(3, 2, vec![
                1, 0, 1,
                1, 1, 1,
            ])?;
    
            let rot_cw_90: Image = shape_image.rotate_cw()?;
            let rot_cw_180: Image = rot_cw_90.rotate_cw()?;
            let rot_cw_270: Image = rot_cw_180.rotate_cw()?;

            let is_same: bool = mask3 == shape_image;
            let is_rot_cw_90: bool = mask3 == rot_cw_90;
            let is_rot_cw_180: bool = mask3 == rot_cw_180;
            let is_rot_cw_270: bool = mask3 == rot_cw_270;
            
            if is_same || is_rot_cw_90 || is_rot_cw_180 || is_rot_cw_270 {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::U5;
                let size_min: u8 = mask2.width().min(mask2.height());
                let size_max: u8 = mask2.width().max(mask2.height());
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.rotated_cw_90 = is_rot_cw_90;
                shape.rotated_cw_180 = is_rot_cw_180;
                shape.rotated_cw_270 = is_rot_cw_270;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }
        }

        if mask3.size() == ImageSize::new(3, 2) || mask3.size().rotate() == ImageSize::new(3, 2) {
            let shape_image: Image = Image::try_create(3, 2, vec![
                1, 0, 1,
                1, 1, 0,
            ])?;
            let normal90: Image = shape_image.rotate_cw()?;
            let normal180: Image = normal90.rotate_cw()?;
            let normal270: Image = normal180.rotate_cw()?;
            
            let shape_image_flipped: Image = shape_image.flip_x()?;
            let flipped90: Image = shape_image_flipped.rotate_cw()?;
            let flipped180: Image = flipped90.rotate_cw()?;
            let flipped270: Image = flipped180.rotate_cw()?;

            let is_same: bool = mask3 == shape_image;
            let is_normal90: bool = mask3 == normal90;
            let is_normal180: bool = mask3 == normal180;
            let is_normal270: bool = mask3 == normal270;

            let is_flipped: bool = mask3 == shape_image_flipped;
            let is_flipped90: bool = mask3 == flipped90;
            let is_flipped180: bool = mask3 == flipped180;
            let is_flipped270: bool = mask3 == flipped270;
            
            if is_same || is_normal90 || is_normal180 || is_normal270 || is_flipped || is_flipped90 || is_flipped180 || is_flipped270 {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::U4;
                let size_min: u8 = mask2.width().min(mask2.height());
                let size_max: u8 = mask2.width().max(mask2.height());
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.rotated_cw_90 = is_normal90;
                shape.rotated_cw_180 = is_normal180;
                shape.rotated_cw_270 = is_normal270;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }
        }

        let shape = ShapeIdentification::default();
        Ok(shape)
    }
}

impl fmt::Display for ShapeIdentification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s: String = format!("{}", self.primary.name());
        if let Some(shape_type) = &self.secondary {
            s += &format!(",{}", shape_type.name());
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_empty() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0,
            0, 0,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "empty");
    }

    #[test]
    fn test_20000_square() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "square,rectangle");
    }

    #[test]
    fn test_20001_square() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1,
            1, 1,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "square,rectangle");
    }

    #[test]
    fn test_20002_rectangle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 1, 1,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "rectangle");
    }

    #[test]
    fn test_30000_box() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 1,
            1, 0, 1,
            1, 1, 1,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "box");
    }

    #[test]
    fn test_30001_box() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 0, 1,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "box");
    }

    #[test]
    fn test_40000_plus() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0, 0,
            0, 1, 1, 0, 0,
            1, 1, 1, 1, 1,
            0, 1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "+");
    }

    #[test]
    fn test_40001_plus() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0,
            0, 0, 1, 0,
            1, 1, 1, 1,
            1, 1, 1, 1,
            0, 0, 1, 0,
            0, 0, 1, 0,
        ];
        let input: Image = Image::try_create(4, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "+");
    }

    #[test]
    fn test_50000_o_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            0, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "O");
    }

    #[test]
    fn test_50001_o_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 0, 0,
            0, 0, 1, 1, 0, 0,
            1, 1, 0, 0, 1, 1,
            0, 0, 1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "O");
    }

    #[test]
    fn test_60000_l_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 0,
            1, 0, 0,
            1, 0, 0,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "L");
    }

    #[test]
    fn test_60001_l_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0,
            1, 1, 1,
            1, 1, 1,
            1, 1, 1,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "L");
    }

    #[test]
    fn test_60002_l_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1,
            0, 1,
            1, 1,
            1, 1,
        ];
        let input: Image = Image::try_create(2, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "L");
    }

    #[test]
    fn test_60003_l_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 1, 1,
            0, 1, 1,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "L");
    }

    #[test]
    fn test_70000_uptack() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 1, 1,
            0, 1, 0,
            0, 1, 0,
            0, 1, 0,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⊥");
    }

    #[test]
    fn test_70001_uptack() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            0, 1, 1, 0,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⊥");
    }

    #[test]
    fn test_70002_uptack() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0,
            1, 1, 1, 1,
            1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⊥");
    }

    #[test]
    fn test_70003_uptack() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1,
            1, 1,
            0, 1,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⊥");
    }

    #[test]
    fn test_80000_u5() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 1, 1,
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U5");
    }

    #[test]
    fn test_80001_u5() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U5");
    }

    #[test]
    fn test_80002_u5() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 0, 0,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U5");
    }

    #[test]
    fn test_80003_u5() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1,
            0, 1,
            1, 1,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U5");
    }

    #[test]
    fn test_90000_u4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0,
            1, 1, 0,
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U4");
    }

    #[test]
    fn test_90001_u4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1,
            0, 1, 1,
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U4");
    }

    #[test]
    fn test_90002_u4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U4");
    }

    #[test]
    fn test_90003_u4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 1,
            1, 0, 0, 1,
            0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U4");
    }

    #[test]
    fn test_90004_u4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 0, 0,
            0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U4");
    }

    #[test]
    fn test_90005_u4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1,
            1, 1, 0, 0,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U4");
    }

    #[test]
    fn test_90006_u4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0,
            0, 1,
            1, 1,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U4");
    }

    #[test]
    fn test_90007_u4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1,
            0, 1,
            1, 0,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U4");
    }

    #[test]
    fn test_100000_i() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            0, 1, 0, 0,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "I");
    }

    #[test]
    fn test_100001_i() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 1,
            1, 1, 1, 1, 1,
            1, 1, 0, 0, 1,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "I");
    }

    #[test]
    fn test_110000_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 1, 1,
            1, 0, 1, 1,
            0, 1, 0, 0,
            1, 0, 1, 1,
            1, 0, 1, 1,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "X");
    }

    #[test]
    fn test_110001_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 1,
            0, 0, 1, 1, 0,
            1, 1, 0, 0, 1,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "X");
    }
}
