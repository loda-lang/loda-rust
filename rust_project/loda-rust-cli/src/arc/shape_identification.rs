use super::{Image, ImageSize, ImageTrim, ImageRemoveDuplicates, ImageTryCreate, ImageRotate, ImageSymmetry, ImageHistogram, Histogram, CenterOfMass};
use std::fmt;
use std::collections::{HashMap, HashSet};
use lazy_static::lazy_static;

lazy_static! {
    static ref SHAPE_TYPE_IMAGE: ShapeTypeImage = ShapeTypeImage::new().expect("Unable to create shape type image");
}

struct ShapeTypeImage {
    image_box: Image,
    image_plus: Image,
    image_o: Image,
    image_x: Image,
    image_h_uppercase: Image,
    image_h_lowercase: Image,
    image_diagonal2: Image,
    image_diagonal3: Image,
    image_l: Image,
    image_uptack: Image,
    image_u5: Image,
    image_u4: Image,
    image_turned_v: Image,
    image_skew_tetromino: Image,
    image_flipped_j: Image,
    image_turned_y: Image,
    image_rotated_k: Image,
    image_lower_left_triangle: Image,
}

impl ShapeTypeImage {
    fn new() -> anyhow::Result<Self> {
        let image_box: Image = Image::try_create(3, 3, vec![
            1, 1, 1,
            1, 0, 1,
            1, 1, 1,
        ])?;

        let image_plus: Image = Image::try_create(3, 3, vec![
            0, 1, 0,
            1, 1, 1,
            0, 1, 0,
        ])?;

        let image_o: Image = Image::try_create(3, 3, vec![
            0, 1, 0,
            1, 0, 1,
            0, 1, 0,
        ])?;

        let image_x: Image = Image::try_create(3, 3, vec![
            1, 0, 1,
            0, 1, 0,
            1, 0, 1,
        ])?;

        let image_h_uppercase: Image = Image::try_create(3, 3, vec![
            1, 0, 1,
            1, 1, 1,
            1, 0, 1,
        ])?;

        let image_h_lowercase: Image = Image::try_create(3, 3, vec![
            1, 1, 0,
            0, 1, 0,
            1, 1, 1,
        ])?;

        let image_diagonal2: Image = Image::try_create(2, 2, vec![
            0, 1,
            1, 0,
        ])?;
    
        let image_diagonal3: Image = Image::try_create(3, 3, vec![
            0, 0, 1,
            0, 1, 0,
            1, 0, 0,
        ])?;

        let image_l: Image = Image::try_create(2, 2, vec![
            1, 0,
            1, 1,
        ])?;

        let image_uptack: Image = Image::try_create(3, 2, vec![
            0, 1, 0,
            1, 1, 1,
        ])?;

        let image_u5: Image = Image::try_create(3, 2, vec![
            1, 0, 1,
            1, 1, 1,
        ])?;

        let image_u4: Image = Image::try_create(3, 2, vec![
            1, 0, 1,
            1, 1, 0,
        ])?;

        let image_turned_v: Image = Image::try_create(3, 2, vec![
            0, 1, 0,
            1, 0, 1,
        ])?;

        let image_skew_tetromino: Image = Image::try_create(3, 2, vec![
            0, 1, 1,
            1, 1, 0,
        ])?;

        let image_flipped_j: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            1, 0, 1,
            1, 1, 1,
        ])?;

        let image_turned_y: Image = Image::try_create(3, 3, vec![
            0, 1, 0,
            1, 1, 1,
            1, 0, 1,
        ])?;

        let image_rotated_k: Image = Image::try_create(3, 3, vec![
            1, 0, 1,
            0, 1, 0,
            1, 1, 1,
        ])?;

        let image_lower_left_triangle: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            1, 1, 0,
            1, 1, 1,
        ])?;

        let instance = Self {
            image_box,
            image_plus,
            image_o,
            image_x,
            image_h_uppercase,
            image_h_lowercase,
            image_diagonal2,
            image_diagonal3,
            image_l,
            image_uptack,
            image_u5,
            image_u4,
            image_turned_v,
            image_skew_tetromino,
            image_flipped_j,
            image_turned_y,
            image_rotated_k,
            image_lower_left_triangle,
        };
        Ok(instance)
    }
}

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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum ShapeTransformation {
    Normal,
    RotateCw90,
    RotateCw180,
    RotateCw270,
    FlipX,
    FlipXRotateCw90,
    FlipXRotateCw180,
    FlipXRotateCw270,
}

impl ShapeTransformation {
    fn all() -> HashSet<ShapeTransformation> {
        let mut transformations = HashSet::<ShapeTransformation>::new();
        transformations.insert(ShapeTransformation::Normal);
        transformations.insert(ShapeTransformation::RotateCw90);
        transformations.insert(ShapeTransformation::RotateCw180);
        transformations.insert(ShapeTransformation::RotateCw270);
        transformations.insert(ShapeTransformation::FlipX);
        transformations.insert(ShapeTransformation::FlipXRotateCw90);
        transformations.insert(ShapeTransformation::FlipXRotateCw180);
        transformations.insert(ShapeTransformation::FlipXRotateCw270);
        transformations
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ShapeIdentification {
    shape_type: ShapeType,
    compacted_image: Option<Image>,
    width: Option<u8>,
    height: Option<u8>,
    transformations: HashSet<ShapeTransformation>,
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
            shape.shape_type = ShapeType::Empty;
            return Ok(shape);
        }
        let size: ImageSize = mask2.size();
        let size_min: u8 = size.width.min(size.height);
        let size_max: u8 = size.width.max(size.height);
        if mask2.size() == ImageSize::new(1, 1) {
            let mut shape = ShapeIdentification::default();
            shape.shape_type = ShapeType::Square;
            shape.width = Some(size_max);
            shape.height = Some(size_min);
            shape.transformations = ShapeTransformation::all();
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
                shape.shape_type = ShapeType::Square;
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.transformations = ShapeTransformation::all();
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            } else {
                let mut shape = ShapeIdentification::default();
                shape.shape_type = ShapeType::Rectangle;
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.transformations = ShapeTransformation::all();
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
            if mask3 == SHAPE_TYPE_IMAGE.image_box {
                let mut shape = ShapeIdentification::default();
                shape.shape_type = ShapeType::Box;
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.transformations = ShapeTransformation::all();
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }

            if mask3 == SHAPE_TYPE_IMAGE.image_plus {
                let mut shape = ShapeIdentification::default();
                shape.shape_type = ShapeType::Plus;
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.transformations = ShapeTransformation::all();
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }

            if mask3 == SHAPE_TYPE_IMAGE.image_o {
                let mut shape = ShapeIdentification::default();
                shape.shape_type = ShapeType::O;
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.transformations = ShapeTransformation::all();
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }

            if mask3 == SHAPE_TYPE_IMAGE.image_x {
                let mut shape = ShapeIdentification::default();
                shape.shape_type = ShapeType::X;
                shape.width = Some(size_max);
                shape.height = Some(size_min);
                shape.transformations = ShapeTransformation::all();
                shape.rotated_cw_90 = true;
                shape.rotated_cw_180 = true;
                shape.rotated_cw_270 = true;
                shape.flip_x = true;
                shape.flip_y = true;
                shape.flip_xy = true;
                return Ok(shape);
            }
        }

        let mut transformations = Vec::<(ShapeTransformation, Image)>::new();
        {
            let normal: Image = mask3.clone();
            let rot90: Image = normal.rotate_cw()?;
            let rot180: Image = rot90.rotate_cw()?;
            let rot270: Image = rot180.rotate_cw()?;
            transformations.push((ShapeTransformation::Normal, normal));
            transformations.push((ShapeTransformation::RotateCw90, rot90));
            transformations.push((ShapeTransformation::RotateCw180, rot180));
            transformations.push((ShapeTransformation::RotateCw270, rot270));
        }
        {
            let normal: Image = mask3.flip_x()?;
            let rot90: Image = normal.rotate_cw()?;
            let rot180: Image = rot90.rotate_cw()?;
            let rot270: Image = rot180.rotate_cw()?;
            transformations.push((ShapeTransformation::Normal, normal));
            transformations.push((ShapeTransformation::RotateCw90, rot90));
            transformations.push((ShapeTransformation::RotateCw180, rot180));
            transformations.push((ShapeTransformation::RotateCw270, rot270));
        }

        {
            let mut images_to_recognize = Vec::<(&Image, ShapeType)>::new();
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_h_uppercase, ShapeType::HUppercase));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_diagonal2, ShapeType::Diagonal2));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_diagonal3, ShapeType::Diagonal3));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_l, ShapeType::L));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_uptack, ShapeType::UpTack));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_u5, ShapeType::U5));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_turned_v, ShapeType::TurnedV));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_u4, ShapeType::U4));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_skew_tetromino, ShapeType::SkewTetromino));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_h_lowercase, ShapeType::HLowercase));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_flipped_j, ShapeType::FlippedJ));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_turned_y, ShapeType::TurnedY));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_rotated_k, ShapeType::RotatedK));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_lower_left_triangle, ShapeType::LowerLeftTriangle));

            let mut shape = ShapeIdentification::default();
            shape.shape_type = ShapeType::Unclassified;
            shape.width = Some(size_max);
            shape.height = Some(size_min);
            let mut found_transformations = HashSet::<ShapeTransformation>::new();
            for (image_to_recognize, recognized_shape_type) in &images_to_recognize {
                for (transformation_type, transformed_image) in &transformations {
                    if *transformed_image == **image_to_recognize {
                        found_transformations.insert(transformation_type.clone());
                    }
                }
                if !found_transformations.is_empty() {
                    shape.shape_type = *recognized_shape_type;
                    return Ok(shape);
                }
            }
        }

        let mask4: Image = Self::normalize(&mask3)?;
        let mut shape = ShapeIdentification::default();
        shape.shape_type = ShapeType::Unclassified;
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

        if y_x_image_vec.is_empty() {
            return Err(anyhow::anyhow!("Image vector is empty"));
        }
        // Pick the first image
        let image0: &Image = &y_x_image_vec[0].2;

        Ok(image0.clone())
    }

}

impl fmt::Display for ShapeIdentification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: String = format!("{}", self.shape_type.name());
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
        assert_eq!(actual.to_string(), "square");
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
        assert_eq!(actual.to_string(), "square");
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
    fn test_20003_rectangle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 1, pixels).expect("image");

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
