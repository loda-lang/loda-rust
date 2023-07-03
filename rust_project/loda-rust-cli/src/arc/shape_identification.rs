use super::{Image, ImageSize, ImageTrim, ImageRemoveDuplicates, ImageTryCreate, ImageRotate, ImageSymmetry, ImageHistogram, Histogram, CenterOfMass};
use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShapeType {
    Empty,

    /// Solid square or rectangle.
    /// ````
    /// 1
    /// ```
    Square,

    /// Solid rectangle.
    /// ````
    /// 1, 1
    /// ```
    Rectangle,

    /// Rectangle with a rectangular hole.
    /// ````
    /// 1, 1, 1
    /// 1, 0, 1
    /// 1, 1, 1
    /// ```
    Box,

    /// Shape `+`
    /// ````
    /// 0, 1, 0
    /// 1, 1, 1
    /// 0, 1, 0
    /// ```
    Plus,

    /// Shape `O`
    /// ````
    /// 0, 1, 0
    /// 1, 0, 1
    /// 0, 1, 0
    /// ```
    O,

    /// Shape `X`
    /// ````
    /// 1, 0, 1
    /// 0, 1, 0
    /// 1, 0, 1
    /// ```
    X,

    /// Shape `L`
    /// ````
    /// 1, 0
    /// 1, 1
    /// ```
    L,

    /// Shape `⊥`, similar to an upside down `T` symbol
    /// 
    /// https://en.wikipedia.org/wiki/Up_tack
    /// 
    /// ````
    /// 0, 1, 0
    /// 1, 1, 1
    /// ```
    UpTack,

    /// A shape like an `U` symbol with 4 pixels. Asymmetric.
    /// ````
    /// 1, 0, 1
    /// 1, 1, 0
    /// ```
    U4,

    /// Shape `⊔`, similar to the `U` symbol with 5 pixels. Symmetric.
    /// 
    /// https://en.wikipedia.org/wiki/Disjoint_union
    /// 
    /// ````
    /// 1, 0, 1
    /// 1, 1, 1
    /// ```
    U5,

    /// Shape `H`, similar to an uppercase `H` symbol
    /// 
    /// The `H` symbol has more mass 5 pixels at the bottom 2 rows when compared to the `I` symbol that only has 4 pixels at the bottom 2 rows.
    /// 
    /// ````
    /// 1, 0, 1
    /// 1, 1, 1
    /// 1, 0, 1
    /// ```
    HUppercase,

    /// Shape `h`, similar to a lowercase `h` symbol
    /// 
    /// ````
    /// 1, 0, 0
    /// 1, 1, 1
    /// 1, 0, 1
    /// ```
    HLowercase,

    /// Shape `⅄`, similar to an uppercase `A` symbol or an upside down `Y` symbol
    /// 
    /// https://en.wikipedia.org/wiki/Voiced_palatal_lateral_approximant
    /// 
    /// ````
    /// 0, 1, 0
    /// 1, 1, 1
    /// 1, 0, 1
    /// ```
    TurnedY,

    /// Shape `⊻`, similar to an uppercase `K` symbol that have been rotated clockwise 90 degrees
    /// 
    /// https://en.wikipedia.org/wiki/Exclusive_or
    /// 
    /// ````
    /// 1, 0, 1
    /// 0, 1, 0
    /// 1, 1, 1
    /// ```
    RotatedK,

    /// Shape `⋀`, similar to an upside down `V` symbol
    /// 
    /// https://en.wikipedia.org/wiki/Turned_v
    /// 
    /// ````
    /// 0, 1, 0
    /// 1, 0, 1
    /// ```
    TurnedV,

    /// Shape `▞` with 2 pixels, similar to a forward slash `/` symbol
    /// 
    /// https://en.wikipedia.org/wiki/Slash_(punctuation)
    /// 
    /// ````
    /// 0, 1
    /// 1, 0
    /// ```
    Diagonal2,

    /// Shape `⋰` with 3 pixels, similar to the unicode `Up Right Diagonal Ellipsis` symbol or a forward slash `/` symbol
    /// 
    /// https://en.wikipedia.org/wiki/Slash_(punctuation)
    /// 
    /// ````
    /// 0, 0, 1
    /// 0, 1, 0
    /// 1, 0, 0
    /// ```
    Diagonal3,

    /// Tetris shape symbol that is skewed
    /// 
    /// https://en.wikipedia.org/wiki/Tetromino
    /// https://mathworld.wolfram.com/Tetromino.html
    /// 
    /// ````
    /// 0, 1, 1
    /// 1, 1, 0
    /// ```
    SkewTetromino,

    /// Shape `◣`
    /// 
    /// ````
    /// 1, 0, 0
    /// 1, 1, 0
    /// 1, 1, 1
    /// ```
    LowerLeftTriangle,

    /// Shape `𐐢`, similar to an flipped `J` symbol.
    /// 
    /// In between state between a `L` symbol and a `U5` symbol.
    /// 
    /// ````
    /// 1, 0, 0
    /// 1, 0, 1
    /// 1, 1, 1
    /// ```
    FlippedJ,

    Unclassified,

    // Future experiments
    // string representation of the shape
    // dashed line
    // checker board
    // ◆ Diamond
    // pyramid
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
            Self::U5 => "⊔",
            Self::HUppercase => "H",
            Self::HLowercase => "h",
            Self::X => "X",
            Self::TurnedY => "⅄",
            Self::RotatedK => "⊻",
            Self::TurnedV => "⋀",
            Self::Diagonal2 => "▞",
            Self::Diagonal3 => "⋰",
            Self::SkewTetromino => "skew",
            Self::LowerLeftTriangle => "◣",
            Self::FlippedJ => "𐐢",
            Self::Unclassified => "unclassified",
        }
    }
}

impl Default for ShapeType {
    fn default() -> Self { ShapeType::Unclassified }
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ShapeIdentification {
    primary: ShapeType,
    secondary: Option<ShapeType>,
    compacted_image: Option<Image>,
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
        let size: ImageSize = mask2.size();
        let size_min: u8 = size.width.min(size.height);
        let size_max: u8 = size.width.max(size.height);
        if mask2.size() == ImageSize::new(1, 1) {
            let mut shape = ShapeIdentification::default();
            shape.primary = ShapeType::Square;
            shape.secondary = Some(ShapeType::Rectangle);
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
        let mask3: Image = mask2.remove_duplicates()?;
        if mask3.size() == ImageSize::new(1, 1) {
            let is_square: bool = mask2.width() == mask2.height();
            if is_square {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::Square;
                shape.secondary = Some(ShapeType::Rectangle);
                shape.width = Some(size_max);
                shape.height = Some(size_min);
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
                1, 1, 1,
                1, 0, 1,
            ])?;
            let rot_cw_90: Image = shape_image.rotate_cw()?;

            let is_same: bool = mask3 == shape_image;
            let is_rot_cw_90: bool = mask3 == rot_cw_90;
            
            if is_same || is_rot_cw_90 {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::HUppercase;
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
                0, 1,
                1, 0,
            ])?;
            let rot_cw_90: Image = shape_image.rotate_cw()?;
            let is_same: bool = mask3 == shape_image;
            let is_rot_cw_90: bool = mask3 == rot_cw_90;
            
            if is_same || is_rot_cw_90 {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::Diagonal2;
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

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                0, 0, 1,
                0, 1, 0,
                1, 0, 0,
            ])?;
            let rot_cw_90: Image = shape_image.rotate_cw()?;
            let is_same: bool = mask3 == shape_image;
            let is_rot_cw_90: bool = mask3 == rot_cw_90;
            
            if is_same || is_rot_cw_90 {
                let mut shape = ShapeIdentification::default();
                shape.primary = ShapeType::Diagonal3;
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
                1, 0, 1,
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
                shape.primary = ShapeType::TurnedV;
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

        if mask3.size() == ImageSize::new(3, 2) || mask3.size().rotate() == ImageSize::new(3, 2) {
            let shape_image: Image = Image::try_create(3, 2, vec![
                0, 1, 1,
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
                shape.primary = ShapeType::SkewTetromino;
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

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                1, 1, 0,
                0, 1, 0,
                1, 1, 1,
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
                shape.primary = ShapeType::HLowercase;
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

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                1, 0, 0,
                1, 0, 1,
                1, 1, 1,
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
                shape.primary = ShapeType::FlippedJ;
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

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                0, 1, 0,
                1, 1, 1,
                1, 0, 1,
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
                shape.primary = ShapeType::TurnedY;
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

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                1, 0, 1,
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
                shape.primary = ShapeType::RotatedK;
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

        if mask3.size() == ImageSize::new(3, 3) {
            let shape_image: Image = Image::try_create(3, 3, vec![
                1, 0, 0,
                1, 1, 0,
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
                shape.primary = ShapeType::LowerLeftTriangle;
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

        let mask4: Image = Self::normalize(&mask3)?;
        let mut shape = ShapeIdentification::default();
        shape.primary = ShapeType::Unclassified;
        shape.width = Some(size_max);
        shape.height = Some(size_min);
        shape.compacted_image = Some(mask4);
        Ok(shape)
    }

    /// The intention is to always yield the same image, no matter if the input is rotated or flipped.
    /// 
    /// - First transform the image so it's always in landscape orientation.
    /// - The most massive side is resting on the floor.
    /// - If there is a tie, the prefer object towards the left side.
    /// - If there is a tie, then sort using the raw pixel data.
    fn normalize(image_with_unknown_orientation: &Image) -> anyhow::Result<Image> {
        // Correct orientation
        let landscape_image: Image;
        if image_with_unknown_orientation.width() < image_with_unknown_orientation.height() {
            landscape_image = image_with_unknown_orientation.rotate_cw()?;
        } else {
            landscape_image = image_with_unknown_orientation.clone();
        }

        // Transformations of the image
        let mut images: Vec<Image> = Vec::new();
        {
            let image: Image = landscape_image.flip_x()?;
            images.push(image);
        }
        {
            let image: Image = landscape_image.flip_y()?;
            images.push(image);
        }
        {
            let image: Image = landscape_image.flip_xy()?;
            images.push(image);
        }
        images.push(landscape_image);

        // Obtain center of mass for each image
        let mut y_x_image_vec: Vec<(i32, u32, Image)> = Vec::new();
        for image in images {
            let scale: u32 = 10000;
            if let Some((x, y)) = image.center_of_mass(scale) {
                // println!("x: {}, y: {} {:?}", x, y, image);
                let inverted_y: i32 = - (y.min(i32::MAX as u32) as i32);
                y_x_image_vec.push((inverted_y, x, image));
            }
        }

        // Sort by center of mass, y first, then x, then image data
        y_x_image_vec.sort();

        // println!("SORTED");
        // for (y, x, image) in &y_x_image_vec {
        //     println!("x: {}, y: {} {:?}", x, y, image);
        // }

        // Pick the first image
        let image0: &Image = &y_x_image_vec[0].2;

        Ok(image0.clone())
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
        assert_eq!(actual.to_string(), "⊔");
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
        assert_eq!(actual.to_string(), "⊔");
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
        assert_eq!(actual.to_string(), "⊔");
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
        assert_eq!(actual.to_string(), "⊔");
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
    fn test_100000_h_uppercase() {
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
        assert_eq!(actual.to_string(), "H");
    }

    #[test]
    fn test_100001_h_uppercase() {
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
        assert_eq!(actual.to_string(), "H");
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

    #[test]
    fn test_120000_turnedv() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            0, 0, 1, 1, 1,
            1, 1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⋀");
    }

    #[test]
    fn test_120001_turnedv() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1,
            1, 1, 0, 0, 0,
            0, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⋀");
    }

    #[test]
    fn test_120002_turnedv() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 1,
            1, 0, 0, 1,
            0, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⋀");
    }

    #[test]
    fn test_120003_turnedv() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0, 0,
            1, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(5, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⋀");
    }

    #[test]
    fn test_130000_diagonal2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            0, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "▞");
    }

    #[test]
    fn test_130001_diagonal2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1,
            1, 1, 0,
            1, 1, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "▞");
    }

    #[test]
    fn test_140000_diagonal3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⋰");
    }

    #[test]
    fn test_140001_diagonal3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⋰");
    }

    #[test]
    fn test_150000_skew_tetramino() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1, 1,
            1, 1, 1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "skew");
    }

    #[test]
    fn test_150001_skew_tetramino() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 0, 0, 0,
            0, 0, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "skew");
    }

    #[test]
    fn test_150002_skew_tetramino() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0,
            1, 1, 1, 1,
            0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "skew");
    }

    #[test]
    fn test_150003_skew_tetramino() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1,
            1, 1, 1, 1,
            1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "skew");
    }

    #[test]
    fn test_160000_h_lowercase() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0,
            1, 1, 1, 1,
            1, 0, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "h");
    }

    #[test]
    fn test_160001_h_lowercase() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1,
            1, 1, 1, 1,
            1, 0, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "h");
    }

    #[test]
    fn test_160002_h_lowercase() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            0, 0, 1, 0,
            1, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "h");
    }

    #[test]
    fn test_160003_h_lowercase() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 0,
            0, 0, 1, 0,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "h");
    }

    #[test]
    fn test_170000_turned_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 0,
            0, 0, 1, 1,
            1, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⅄");
    }

    #[test]
    fn test_170001_turned_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 1,
            1, 1, 0, 0, 
            0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⅄");
    }

    #[test]
    fn test_170002_turned_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⅄");
    }

    #[test]
    fn test_170003_turned_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
            0, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⅄");
    }

    #[test]
    fn test_180000_rotated_k() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 1,
            1, 0, 0, 1,
            0, 1, 1, 0,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⊻");
    }

    #[test]
    fn test_180001_rotated_k() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            0, 1, 1, 0,
            1, 0, 0, 1,
            1, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⊻");
    }

    #[test]
    fn test_180002_rotated_k() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 1,
            1, 1, 0,
            1, 1, 0,
            1, 0, 1,
            1, 0, 1,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⊻");
    }

    #[test]
    fn test_180003_rotated_k() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 1,
            0, 1, 1,
            0, 1, 1,
            0, 1, 1,
            1, 0, 1,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⊻");
    }

    #[test]
    fn test_190000_lower_left_triangle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0, 0,
            1, 1, 0, 0, 0, 0,
            1, 1, 1, 1, 0, 0,
            1, 1, 1, 1, 0, 0,
            1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "◣");
    }

    #[test]
    fn test_190001_lower_left_triangle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 0,
            1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "◣");
    }

    #[test]
    fn test_190002_lower_left_triangle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            0, 1, 1, 1,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "◣");
    }

    #[test]
    fn test_190003_lower_left_triangle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 1,
            0, 1, 1, 1,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "◣");
    }

    #[test]
    fn test_200000_flipped_j() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0,
            1, 0, 0, 0,
            1, 0, 0, 1,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "𐐢");
    }

    #[test]
    fn test_200001_flipped_j() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "𐐢");
    }

    #[test]
    fn test_200002_flipped_j() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "𐐢");
    }

    #[test]
    fn test_200003_flipped_j() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 0, 0, 1,
            1, 0, 0, 0,
            1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "𐐢");
    }

    #[test]
    fn test_210000_unclassified() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 1,
            1, 0, 0, 1,
            1, 0, 0, 0,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "unclassified");

        let expected_pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            1, 0, 0, 0,
            1, 1, 0, 1,
        ];
        let expected_compact: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual.compacted_image, Some(expected_compact));
    }

    fn transform(input: &Image, mode: u8) -> anyhow::Result<Image> {
        let output: Image = match mode {
            0 => input.clone(),
            1 => input.flip_x()?,
            2 => input.flip_y()?,
            3 => input.rotate_cw()?,
            4 => input.rotate_ccw()?,
            _ => return Err(anyhow::anyhow!("invalid mode")),
        };
        Ok(output)
    }

    fn transformed_images(input: &Image) -> anyhow::Result<Vec<Image>> {
        let mut images: Vec<Image> = Vec::new();
        for mode in 0..=4 {
            images.push(transform(&input, mode)?);
        }
        Ok(images)
    }

    #[test]
    fn test_300000_normalize() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 1,
            1, 0, 0, 1,
            1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");
        let inputs: Vec<Image> = transformed_images(&input).expect("ok");

        // Act
        let actual_vec: Vec<Image> = inputs.iter().map(|i| 
            ShapeIdentification::normalize(i).expect("ok")
        ).collect();

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 1,
            1, 0, 0, 1,
            1, 1, 1, 0,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        for actual in actual_vec {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_300001_normalize() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1,
            1, 1, 0,
            0, 0, 1,
            0, 1, 0,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");
        let inputs: Vec<Image> = transformed_images(&input).expect("ok");

        // Act
        let actual_vec: Vec<Image> = inputs.iter().map(|i| 
            ShapeIdentification::normalize(i).expect("ok")
        ).collect();

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0,
            1, 1, 0, 1,
            1, 0, 1, 0,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        for actual in actual_vec {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_300002_normalize() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1,
            1, 1, 0,
            0, 0, 1,
            0, 1, 0,
            1, 0, 1,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");
        let inputs: Vec<Image> = transformed_images(&input).expect("ok");

        // Act
        let actual_vec: Vec<Image> = inputs.iter().map(|i| 
            ShapeIdentification::normalize(i).expect("ok")
        ).collect();

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0, 1,
            1, 1, 0, 1, 0,
            1, 0, 1, 0, 1,
        ];
        let expected: Image = Image::try_create(5, 3, expected_pixels).expect("image");
        for actual in actual_vec {
            assert_eq!(actual, expected);
        }
    }
}
