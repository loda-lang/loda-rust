//! Identification of shape type and transformations.
//! 
//! Invariant to translation, scaling, flipping, rotation (90, 180, 270), horizontal/vertical-compression.
//! 
//! Similar to SIFT (Scale-invariant feature transform) but without using points.
//! https://en.wikipedia.org/wiki/Scale-invariant_feature_transform
//! 
//! Known problem: Cannot detect diagonal structures.
//! One diagonal line that is 11 pixels long, gets classified as `ShapeType::Unclassified`.
//! Another diagonal line that is 13 pixels long, also gets classified as `ShapeType::Unclassified`.
//! There is no way to tell that the diagonal lines are similar.
//! 
//! Future experiments:
//! Distinguish between `Rectangle` and `Line`. Currently a line gets compressed into a 1x1 rectangle.
//! It may be helpful identifying lines.
use super::{Image, ImageSize, ImageTrim, ImageRemoveDuplicates, ImageTryCreate, ImageRotate45, ImageRotate90, ImageSymmetry, CenterOfMass, Rectangle, ImageCrop, ImageResize, ImageMaskCount, ImagePadding, convolution3x3};
use std::fmt;
use std::collections::HashSet;
use lazy_static::lazy_static;

lazy_static! {
    static ref SHAPE_TYPE_IMAGE: ShapeTypeImage = ShapeTypeImage::new().expect("Unable to create ShapeTypeImage");
}

struct ShapeTypeImage {
    image_shapetype_vec: Vec::<(Image, ShapeType)>,
}

impl ShapeTypeImage {
    fn new() -> anyhow::Result<Self> {
        let image_rectangle: Image = Image::color(1, 1, 1);

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

        let image_crosshair: Image = Image::try_create(3, 3, vec![
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
            1, 0,
            0, 1,
        ])?;
    
        let image_diagonal3: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            0, 1, 0,
            0, 0, 1,
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
            1, 1, 0,
            0, 1, 1,
        ])?;

        let image_flipped_j: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            1, 0, 1,
            1, 1, 1,
        ])?;

        let image_inverted_fork: Image = Image::try_create(3, 3, vec![
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

        let image_left_plus: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            1, 1, 1,
            0, 1, 0,
        ])?;

        let image_leftwards_harpoon_with_barb_upwards: Image = Image::try_create(3, 3, vec![
            0, 1, 0,
            1, 0, 0,
            1, 1, 1,
        ])?;

        let image_box_without_one_corner: Image = Image::try_create(3, 3, vec![
            1, 1, 0,
            1, 0, 1,
            1, 1, 1,
        ])?;

        let image_rotated_d: Image = Image::try_create(3, 3, vec![
            0, 1, 0,
            1, 0, 1,
            1, 1, 1,
        ])?;

        let image_rotated_j_round: Image = Image::try_create(3, 3, vec![
            0, 1, 0,
            1, 0, 0,
            0, 1, 1,
        ])?;

        let image_box_without_diagonal: Image = Image::try_create(3, 3, vec![
            1, 1, 0,
            1, 0, 1,
            0, 1, 1,
        ])?;

        let image_rotated_s: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            1, 1, 1,
            0, 0, 1,
        ])?;

        let image_plus_with_one_corner: Image = Image::try_create(3, 3, vec![
            0, 1, 0,
            1, 1, 1,
            1, 1, 0,
        ])?;

        let image_square_without_diagonal_corners: Image = Image::try_create(3, 3, vec![
            1, 1, 0,
            1, 1, 1,
            0, 1, 1,
        ])?;

        let image_gameoflife_boat: Image = Image::try_create(3, 3, vec![
            0, 1, 0,
            1, 0, 1,
            1, 1, 0,
        ])?;

        let image_l_with_45degree_line: Image = Image::try_create(3, 3, vec![
            0, 1, 0,
            0, 1, 1,
            1, 0, 0,
        ])?;

        let image_x_without_one_corner: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            0, 1, 0,
            1, 0, 1,
        ])?;

        let image_l_skew: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            0, 1, 0,
            0, 1, 1,
        ])?;

        let image_uptack_skew: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            0, 1, 0,
            1, 1, 1,
        ])?;

        let image_lower_left_triangle_with_corner: Image = Image::try_create(3, 3, vec![
            1, 0, 1,
            1, 1, 0,
            1, 1, 1,
        ])?;

        let image_i_uppercase_moved_corner: Image = Image::try_create(3, 3, vec![
            1, 1, 0,
            0, 1, 1,
            1, 1, 1,
        ])?;

        let image_skew_tetromino_with_top_left_corner: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            0, 1, 1,
            1, 1, 0,
        ])?;

        let image_rotated_uppercase_e: Image = Image::try_create(5, 2, vec![
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
        ])?;

        let image_turned_w: Image = Image::try_create(5, 2, vec![
            0, 1, 0, 1, 0,
            1, 0, 1, 0, 1,
        ])?;

        let image_line_around_small_obstacle: Image = Image::try_create(5, 2, vec![
            0, 1, 1, 1, 0,
            1, 1, 0, 1, 1,
        ])?;

        let image_line_around_big_obstacle: Image = Image::try_create(5, 3, vec![
            0, 1, 1, 1, 0,
            0, 1, 0, 1, 0,
            1, 1, 0, 1, 1,
        ])?;

        let image_box_with_two_holes: Image = Image::try_create(5, 3, vec![
            1, 1, 1, 1, 1,
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
        ])?;

        let image_box_with_2x2_holes: Image = Image::try_create(5, 5, vec![
            1, 1, 1, 1, 1,
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
        ])?;

        let image_x_moved_corner: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            0, 1, 1,
            1, 0, 1,
        ])?;

        let image_lower_left_triangle_without_corner: Image = Image::try_create(3, 3, vec![
            1, 0, 0,
            1, 1, 0,
            0, 1, 1,
        ])?;

        let image_lower_left_triangle_moved_corner: Image = Image::try_create(3, 3, vec![
            1, 0, 1,
            1, 1, 0,
            0, 1, 1,
        ])?;

        let image_rotated_p: Image = Image::try_create(4, 3, vec![
            1, 1, 1, 0,
            1, 0, 1, 0,
            1, 1, 1, 1,
        ])?;

        let image_rotated_lowercase_f: Image = Image::try_create(4, 3, vec![
            0, 1, 0, 0,
            1, 1, 1, 1,
            0, 1, 0, 1,
        ])?;

        let image_box_with_rightwards_tick: Image = Image::try_create(4, 3, vec![
            1, 1, 1, 0,
            1, 0, 1, 1,
            1, 1, 1, 0,
        ])?;

        let image_open_box_with_hole_in_center_of_top_border: Image = Image::try_create(5, 3, vec![
            1, 1, 0, 1, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
        ])?;

        let image_open_box_with_hole_in_right_side_of_top_border: Image = Image::try_create(4, 3, vec![
            1, 1, 0, 1,
            1, 0, 0, 1,
            1, 1, 1, 1,
        ])?;

        let image_box_with_uptick: Image = Image::try_create(5, 4, vec![
            0, 0, 1, 0, 0,
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
        ])?;

        let image_grid3x2: Image = Image::try_create(5, 3, vec![
            0, 1, 0, 1, 0,
            1, 1, 1, 1, 1,
            0, 1, 0, 1, 0,
        ])?;

        let image_grid3x3: Image = Image::try_create(5, 5, vec![
            0, 1, 0, 1, 0,
            1, 1, 1, 1, 1,
            0, 1, 0, 1, 0,
            1, 1, 1, 1, 1,
            0, 1, 0, 1, 0,
        ])?;

        let image_grid4x2: Image = Image::try_create(7, 3, vec![
            0, 1, 0, 1, 0, 1, 0,
            1, 1, 1, 1, 1, 1, 1,
            0, 1, 0, 1, 0, 1, 0,
        ])?;

        let image_grid4x3: Image = Image::try_create(7, 5, vec![
            0, 1, 0, 1, 0, 1, 0,
            1, 1, 1, 1, 1, 1, 1,
            0, 1, 0, 1, 0, 1, 0,
            1, 1, 1, 1, 1, 1, 1,
            0, 1, 0, 1, 0, 1, 0,
        ])?;

        let mut items = Vec::<(Image, ShapeType)>::new();
        items.push((image_rectangle, ShapeType::Rectangle));
        items.push((image_box, ShapeType::Box));
        items.push((image_plus, ShapeType::Plus));
        items.push((image_crosshair, ShapeType::Crosshair));
        items.push((image_x, ShapeType::X));
        items.push((image_h_uppercase, ShapeType::HUppercase));
        items.push((image_diagonal2, ShapeType::Diagonal2));
        items.push((image_diagonal3, ShapeType::Diagonal3));
        items.push((image_l, ShapeType::L));
        items.push((image_uptack, ShapeType::UpTack));
        items.push((image_u5, ShapeType::U5));
        items.push((image_turned_v, ShapeType::TurnedV));
        items.push((image_u4, ShapeType::U4));
        items.push((image_skew_tetromino, ShapeType::SkewTetromino));
        items.push((image_h_lowercase, ShapeType::HLowercase));
        items.push((image_flipped_j, ShapeType::FlippedJ));
        items.push((image_inverted_fork, ShapeType::InvertedFork));
        items.push((image_rotated_k, ShapeType::RotatedK));
        items.push((image_lower_left_triangle, ShapeType::LowerLeftTriangle));
        items.push((image_left_plus, ShapeType::LeftPlus));
        items.push((image_leftwards_harpoon_with_barb_upwards, ShapeType::LeftwardsHarpoonWithBarbUpwards));
        items.push((image_box_without_one_corner, ShapeType::BoxWithoutOneCorner));
        items.push((image_rotated_d, ShapeType::RotatedD));
        items.push((image_rotated_j_round, ShapeType::RotatedJRound));
        items.push((image_box_without_diagonal, ShapeType::BoxWithoutDiagonal));
        items.push((image_rotated_s, ShapeType::RotatedS));
        items.push((image_plus_with_one_corner, ShapeType::PlusWithOneCorner));
        items.push((image_square_without_diagonal_corners, ShapeType::SquareWithoutDiagonalCorners));
        items.push((image_gameoflife_boat, ShapeType::GameOfLifeBoat));
        items.push((image_l_with_45degree_line, ShapeType::LWith45DegreeLine));
        items.push((image_x_without_one_corner, ShapeType::XWithoutOneCorner));
        items.push((image_l_skew, ShapeType::LSkew));
        items.push((image_uptack_skew, ShapeType::UpTackSkew));
        items.push((image_lower_left_triangle_with_corner, ShapeType::LowerLeftTriangleWithCorner));
        items.push((image_i_uppercase_moved_corner, ShapeType::IUppercaseMovedCorner));
        items.push((image_skew_tetromino_with_top_left_corner, ShapeType::SkewTetrominoWithTopLeftCorner));
        items.push((image_rotated_uppercase_e, ShapeType::RotatedUppercaseE));
        items.push((image_turned_w, ShapeType::TurnedW));
        items.push((image_line_around_small_obstacle, ShapeType::LineAroundSmallObstacle));
        items.push((image_line_around_big_obstacle, ShapeType::LineAroundBigObstacle));
        items.push((image_box_with_two_holes, ShapeType::BoxWithTwoHoles));
        items.push((image_box_with_2x2_holes, ShapeType::BoxWith2x2Holes));
        items.push((image_x_moved_corner, ShapeType::XMovedCorner));
        items.push((image_lower_left_triangle_without_corner, ShapeType::LowerLeftTriangleWithoutCorner));
        items.push((image_lower_left_triangle_moved_corner, ShapeType::LowerLeftTriangleMovedCorner));
        items.push((image_rotated_p, ShapeType::RotatedP));
        items.push((image_rotated_lowercase_f, ShapeType::RotatedLowercaseF));
        items.push((image_box_with_rightwards_tick, ShapeType::BoxWithRightwardsTick));
        items.push((image_open_box_with_hole_in_center_of_top_border, ShapeType::OpenBoxWithHoleInCenterOfTopBorder));
        items.push((image_open_box_with_hole_in_right_side_of_top_border, ShapeType::OpenBoxWithHoleInRightSideOfTopBorder));
        items.push((image_box_with_uptick, ShapeType::BoxWithUptick));
        items.push((image_grid3x2, ShapeType::Grid3x2));
        items.push((image_grid3x3, ShapeType::Grid3x3));
        items.push((image_grid4x2, ShapeType::Grid4x2));
        items.push((image_grid4x3, ShapeType::Grid4x3));

        let instance = Self {
            image_shapetype_vec: items,
        };
        Ok(instance)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ShapeType {
    /// Solid rectangle, such as a square, horizontal/vertical line.
    /// ````
    /// 1
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

    /// Shape `✜`, similar to a `+` symbol where the center is hollow.
    /// 
    /// Heavy Open Centre Cross
    /// 
    /// ````
    /// 0, 1, 0
    /// 1, 0, 1
    /// 0, 1, 0
    /// ```
    Crosshair,

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

    /// Shape `Ⴙ`, similar to a lowercase `h` symbol
    /// 
    /// U+10B9: GEORGIAN CAPITAL LETTER CHIN
    /// 
    /// ````
    /// 1, 0, 0
    /// 1, 1, 1
    /// 1, 0, 1
    /// ```
    HLowercase,

    /// Shape `⑃`, similar the shape `⅄` or an upside down `Y` symbol or an uppercase `A` symbol
    /// 
    /// U+2443: OCR INVERTED FORK
    /// 
    /// https://en.wikipedia.org/wiki/Voiced_palatal_lateral_approximant
    /// 
    /// ````
    /// 0, 1, 0
    /// 1, 1, 1
    /// 1, 0, 1
    /// ```
    InvertedFork,

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

    /// Shape `▚` with 2 pixels, similar to a backslash character
    /// 
    /// https://en.wikipedia.org/wiki/Backslash
    /// 
    /// ````
    /// 1, 0
    /// 0, 1
    /// ```
    Diagonal2,

    /// Shape `⋱` with 3 pixels, similar to the unicode `Down Right Diagonal Ellipsis` symbol or a backslash character
    /// 
    /// https://en.wikipedia.org/wiki/Backslash
    /// 
    /// ````
    /// 1, 0, 0
    /// 0, 1, 0
    /// 0, 0, 1
    /// ```
    Diagonal3,

    /// Tetris shape symbol that is skewed
    /// 
    /// https://en.wikipedia.org/wiki/Tetromino
    /// https://mathworld.wolfram.com/Tetromino.html
    /// 
    /// ````
    /// 1, 1, 0
    /// 0, 1, 1
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

    /// Shape `ഺ`, similar to a `+` where the top line is at the left side.
    /// 
    /// In between state between a `+` symbol and a rotated `T` symbol.
    /// 
    /// U+0D3A: MALAYALAM LETTER TTTA
    /// 
    /// ````
    /// 1, 0, 0
    /// 1, 1, 1
    /// 0, 1, 0
    /// ```
    LeftPlus,

    /// Shape `↼` corresponding to one of the 2 states in the game-of-life `glider`.
    /// 
    /// The `glider` is the smallest, most common, and first-discovered spaceship in Game of Life.
    /// 
    /// https://conwaylife.com/wiki/Glider
    /// 
    /// Similar to the Tetris `L` symbol with a pixel at the top middle.
    /// 
    /// In between state between a `L` symbol and a `⊥` (upside down `T` symbol).
    /// 
    /// A rotated version of the `ᒯ` symbol.
    /// 
    /// ````
    /// 0, 1, 0
    /// 1, 0, 0
    /// 1, 1, 1
    /// ```
    LeftwardsHarpoonWithBarbUpwards,

    /// A box that is hollow, that lack one of its corners.
    /// 
    /// Possible unicode character to represent this shape:
    /// `◳`, WHITE SQUARE WITH UPPER RIGHT QUADRANT
    /// 
    /// ````
    /// 1, 1, 0
    /// 1, 0, 1
    /// 1, 1, 1
    /// ```
    BoxWithoutOneCorner,

    /// Shape `⌓`, similar to an rotated `D` symbol.
    /// 
    /// Top half of a hollow circle.
    /// 
    /// ````
    /// 0, 1, 0
    /// 1, 0, 1
    /// 1, 1, 1
    /// ```
    RotatedD,

    /// Shape `ᓚ`, similar to an rotated lowercase `j` symbol with round corners.
    /// 
    /// Unicode name: Canadian syllabics la
    /// Codepoint (hexadecimal): 0x14DA
    /// 
    /// ````
    /// 0, 1, 0
    /// 1, 0, 0
    /// 0, 1, 1
    /// ```
    RotatedJRound,

    /// Two `L` symbols combined, or a hollow box without the diagonal corners.
    /// 
    /// ````
    /// 1, 1, 0
    /// 1, 0, 1
    /// 0, 1, 1
    /// ```
    BoxWithoutDiagonal,

    /// Shape `⥊`, similar to an rotated `S` symbol.
    /// 
    /// Unicode name: Left barb up right barb down harpoon
    /// 
    /// ````
    /// 1, 0, 0
    /// 1, 1, 1
    /// 0, 0, 1
    /// ```
    RotatedS,

    /// Shape `+` with 1 filled corner. Or a square without 3 corners.
    /// 
    /// ````
    /// 0, 1, 0
    /// 1, 1, 1
    /// 1, 1, 0
    /// ```
    PlusWithOneCorner,

    /// Solid square without two diagonal corners.
    /// 
    /// ````
    /// 0, 1, 1
    /// 1, 1, 1
    /// 1, 1, 0
    /// ```
    SquareWithoutDiagonalCorners,

    /// The game-of-life `boat` pattern, similar to the shape `⌂` rotated.
    /// 
    /// The boat is the only still life with 5 cells.
    /// 
    /// https://conwaylife.com/wiki/Boat
    /// 
    /// ````
    /// 0, 1, 0
    /// 1, 0, 1
    /// 1, 1, 0
    /// ```
    GameOfLifeBoat,

    /// The `L` shape where its corner has a line attached at a 45 degree angle.
    /// 
    /// ````
    /// 0, 1, 0
    /// 0, 1, 1
    /// 1, 0, 0
    /// ```
    LWith45DegreeLine,

    /// Shape `⋋` is similar to an `X` where the top-right corner has been removed.
    /// 
    /// Unicode: Left semidirect product
    /// 
    /// ````
    /// 1, 0, 0
    /// 0, 1, 0
    /// 1, 0, 1
    /// ```
    XWithoutOneCorner,

    /// Shape `Ꝇ` is similar to an `L` where the top-left corner has been skewed.
    /// 
    /// Unicode: Latin capital letter broken l
    /// 
    /// ````
    /// 1, 0, 0
    /// 0, 1, 0
    /// 0, 1, 1
    /// ```
    LSkew,

    /// Skewed variant of the shape `⊥` with a top-left corner.
    /// 
    /// https://en.wikipedia.org/wiki/Up_tack
    /// 
    /// ````
    /// 1, 0, 0
    /// 0, 1, 0
    /// 1, 1, 1
    /// ```
    UpTackSkew,

    /// Variant of shape `◣` with the top-right corner filled.
    /// 
    /// ````
    /// 1, 0, 1
    /// 1, 1, 0
    /// 1, 1, 1
    /// ```
    LowerLeftTriangleWithCorner,

    /// Variant of shape `I` with the top-right corner fallen down 1 pixel.
    /// 
    /// ````
    /// 1, 1, 0
    /// 0, 1, 1
    /// 1, 1, 1
    /// ```
    IUppercaseMovedCorner,

    /// Variant of the Tetris shape symbol that is skewed, with a top-left corner.
    /// 
    /// https://en.wikipedia.org/wiki/Tetromino
    /// https://mathworld.wolfram.com/Tetromino.html
    /// 
    /// ````
    /// 1, 0, 0
    /// 0, 1, 1
    /// 1, 1, 0
    /// ```
    SkewTetrominoWithTopLeftCorner,

    /// Shape `⧢` is similar to a rotated `E`.
    /// 
    /// Unicode: Shuffle product
    /// 
    /// ````
    /// 1, 0, 1, 0, 1
    /// 1, 1, 1, 1, 1
    /// ```
    RotatedUppercaseE,

    /// Shape `ʍ` is similar to a upside down `W`.
    /// 
    /// Unicode: Latin small letter turned w
    /// 
    /// ````
    /// 0, 1, 0, 1, 0
    /// 1, 0, 1, 0, 1
    /// ```
    TurnedW,

    /// A horizontal line shape `▟▀▙` around a small 1px obstacle.
    /// 
    /// ````
    /// 0, 1, 1, 1, 0
    /// 1, 1, 0, 1, 1
    /// ```
    LineAroundSmallObstacle,

    /// A horizontal line shape `⎍` around a big 2px obstacle.
    /// 
    /// Unicode: Monostable Symbol
    /// 
    /// ````
    /// 0, 1, 1, 1, 0
    /// 0, 1, 0, 1, 0
    /// 1, 1, 0, 1, 1
    /// ```
    LineAroundBigObstacle,

    /// Shape `◫` is similar to 2 boxes sharing the middle edge.
    /// 
    /// Unicode: White square with vertical bisecting line
    /// 
    /// ````
    /// 1, 1, 1, 1, 1
    /// 1, 0, 1, 0, 1
    /// 1, 1, 1, 1, 1
    /// ```
    BoxWithTwoHoles,

    /// Shape `𐌎` is similar to 2x2 boxes sharing the middle edges.
    /// 
    /// Unicode: Old italic letter esh
    /// 
    /// ````
    /// 1, 1, 1, 1, 1
    /// 1, 0, 1, 0, 1
    /// 1, 1, 1, 1, 1
    /// 1, 0, 1, 0, 1
    /// 1, 1, 1, 1, 1
    /// ```
    BoxWith2x2Holes,

    /// Variant of shape `X` with the top-right corner fallen down 1 pixel.
    /// 
    /// ````
    /// 1, 0, 0
    /// 0, 1, 1
    /// 1, 0, 1
    /// ```
    XMovedCorner,

    /// Variant of shape `◣` with the bottom-left corner missing.
    /// 
    /// ````
    /// 1, 0, 0
    /// 1, 1, 0
    /// 0, 1, 1
    /// ```
    LowerLeftTriangleWithoutCorner,

    /// Variant of shape `◣` where the bottom-left corner has been moved to the top-right corner.
    /// 
    /// ````
    /// 1, 0, 1
    /// 1, 1, 0
    /// 0, 1, 1
    /// ```
    LowerLeftTriangleMovedCorner,

    /// Shape `ᓇ`, a box with a pixel extending out at the bottom-right corner.
    /// 
    /// Unicode: Canadian syllabics na
    /// 
    /// ````
    /// 1, 1, 1, 0
    /// 1, 0, 1, 0
    /// 1, 1, 1, 1
    /// ```
    RotatedP,

    /// Shape `╋┓` or a `f` rotated 90 degrees.
    /// 
    /// Unicode: Canadian syllabics na
    /// 
    /// ````
    /// 0, 1, 0, 0
    /// 1, 1, 1, 1
    /// 0, 1, 0, 1
    /// ```
    RotatedLowercaseF,

    /// Shape `⟥` or a box with a pixel extending out at the center row.
    /// 
    /// Unicode: White square with rightwards tick
    /// 
    /// ````
    /// 1, 1, 1, 0
    /// 1, 0, 1, 1
    /// 1, 1, 1, 0
    /// ```
    BoxWithRightwardsTick,

    /// Shape `[_]` or a box with a pixel missing in the center of the top border.
    /// 
    /// ````
    /// 1, 1, 0, 1, 1
    /// 1, 0, 0, 0, 1
    /// 1, 1, 1, 1, 1
    /// ```
    OpenBoxWithHoleInCenterOfTopBorder,

    /// Shape `[_|` or a box with a pixel missing in the right side of the top border.
    /// 
    /// ````
    /// 1, 1, 0, 1
    /// 1, 0, 0, 1
    /// 1, 1, 1, 1
    /// ```
    OpenBoxWithHoleInRightSideOfTopBorder,

    /// Shape `box-with-uptick` or a box with a single pixel placed on top center of the box.
    /// 
    /// ````
    /// 0, 0, 1, 0, 0
    /// 1, 1, 1, 1, 1
    /// 1, 0, 0, 0, 1
    /// 1, 1, 1, 1, 1
    /// ```
    BoxWithUptick,

    /// Shape `++` similar 3x2 empty cells with a line in between.
    /// 
    /// ````
    /// 0, 1, 0, 1, 0
    /// 1, 1, 1, 1, 1
    /// 0, 1, 0, 1, 0
    /// ```
    Grid3x2,

    /// Shape `#` that is 3x3 empty cells with a line in between.
    /// 
    /// ````
    /// 0, 1, 0, 1, 0
    /// 1, 1, 1, 1, 1
    /// 0, 1, 0, 1, 0
    /// 1, 1, 1, 1, 1
    /// 0, 1, 0, 1, 0
    /// ```
    Grid3x3,

    /// Shape `+++` similar 4x2 empty cells with a line in between.
    /// 
    /// ````
    /// 0, 1, 0, 1, 0, 1, 0
    /// 1, 1, 1, 1, 1, 1, 1
    /// 0, 1, 0, 1, 0, 1, 0
    /// ```
    Grid4x2,

    /// Shape `‡‡‡` similar 4x3 empty cells with a line in between.
    /// 
    /// Unicode: Double dagger
    /// 
    /// ````
    /// 0, 1, 0, 1, 0, 1, 0
    /// 1, 1, 1, 1, 1, 1, 1
    /// 0, 1, 0, 1, 0, 1, 0
    /// 1, 1, 1, 1, 1, 1, 1
    /// 0, 1, 0, 1, 0, 1, 0
    /// ```
    Grid4x3,

    /// Shapes that could not be recognized.
    Unclassified,

    // Future experiments
    // dashed line
    // checker board
    // ◆ Diamond
    // pyramid
}

impl ShapeType {
    #[allow(dead_code)]
    fn name(&self) -> &str {
        match self {
            Self::Rectangle => "rectangle",
            Self::Box => "box",
            Self::Plus => "+",
            Self::Crosshair => "✜",
            Self::L => "L",
            Self::UpTack => "⊥",
            Self::U4 => "U4",
            Self::U5 => "⊔",
            Self::HUppercase => "H",
            Self::HLowercase => "Ⴙ",
            Self::X => "X",
            Self::InvertedFork => "⑃",
            Self::RotatedK => "⊻",
            Self::TurnedV => "⋀",
            Self::Diagonal2 => "▚",
            Self::Diagonal3 => "⋱",
            Self::SkewTetromino => "skew",
            Self::LowerLeftTriangle => "◣",
            Self::FlippedJ => "𐐢",
            Self::LeftPlus => "ഺ",
            Self::LeftwardsHarpoonWithBarbUpwards => "↼",
            Self::BoxWithoutOneCorner => "box1",
            Self::RotatedD => "⌓",
            Self::RotatedJRound => "ᓚ",
            Self::BoxWithoutDiagonal => "box2",
            Self::RotatedS => "⥊",
            Self::PlusWithOneCorner => "+1",
            Self::SquareWithoutDiagonalCorners => "square2",
            Self::GameOfLifeBoat => "boat",
            Self::LWith45DegreeLine => "L45",
            Self::XWithoutOneCorner => "⋋",
            Self::LSkew => "Ꝇ",
            Self::UpTackSkew => "⊥1",
            Self::LowerLeftTriangleWithCorner => "◣+1",
            Self::IUppercaseMovedCorner => "I1",
            Self::SkewTetrominoWithTopLeftCorner => "skew1",
            Self::RotatedUppercaseE => "⧢",
            Self::TurnedW => "ʍ",
            Self::LineAroundSmallObstacle => "▟▀▙",
            Self::LineAroundBigObstacle => "⎍",
            Self::BoxWithTwoHoles => "◫",
            Self::BoxWith2x2Holes => "𐌎",
            Self::XMovedCorner => "X1",
            Self::LowerLeftTriangleWithoutCorner => "◣-1",
            Self::LowerLeftTriangleMovedCorner => "◣move",
            Self::RotatedP => "ᓇ",
            Self::RotatedLowercaseF => "╋┓",
            Self::BoxWithRightwardsTick => "⟥",
            Self::OpenBoxWithHoleInCenterOfTopBorder => "[_]",
            Self::OpenBoxWithHoleInRightSideOfTopBorder => "[_|",
            Self::BoxWithUptick => "box-with-uptick",
            Self::Grid3x2 => "++",
            Self::Grid3x3 => "#",
            Self::Grid4x2 => "+++",
            Self::Grid4x3 => "‡‡‡",
            Self::Unclassified => "unclassified",
        }
    }

    pub fn len() -> u8 {
        56
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShapeTransformation {
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
    #[allow(dead_code)]
    pub fn all() -> HashSet<ShapeTransformation> {
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

    #[allow(dead_code)]
    pub fn perform_all_transformations(image: &Image) -> anyhow::Result<Vec<(ShapeTransformation, Image)>> {
        let mut transformations = Vec::<(ShapeTransformation, Image)>::new();
        {
            let degree0: Image = image.clone();
            let degree90: Image = degree0.rotate_cw()?;
            let degree180: Image = degree90.rotate_cw()?;
            let degree270: Image = degree180.rotate_cw()?;
            transformations.push((ShapeTransformation::Normal, degree0));
            transformations.push((ShapeTransformation::RotateCw90, degree90));
            transformations.push((ShapeTransformation::RotateCw180, degree180));
            transformations.push((ShapeTransformation::RotateCw270, degree270));
        }
        {
            let degree0: Image = image.flip_x()?;
            let degree90: Image = degree0.rotate_cw()?;
            let degree180: Image = degree90.rotate_cw()?;
            let degree270: Image = degree180.rotate_cw()?;
            transformations.push((ShapeTransformation::FlipX, degree0));
            transformations.push((ShapeTransformation::FlipXRotateCw90, degree90));
            transformations.push((ShapeTransformation::FlipXRotateCw180, degree180));
            transformations.push((ShapeTransformation::FlipXRotateCw270, degree270));
        }
        Ok(transformations)
    }

    #[allow(dead_code)]
    fn forward(&self, image: &Image) -> anyhow::Result<Image> {
        let result_image: Image = match self {
            Self::Normal => image.clone(),
            Self::RotateCw90 => image.rotate_cw()?,
            Self::RotateCw180 => image.rotate(2)?,
            Self::RotateCw270 => image.rotate_ccw()?,
            Self::FlipX => image.flip_x()?,
            Self::FlipXRotateCw90 => image.flip_x()?.rotate_cw()?,
            Self::FlipXRotateCw180 => image.flip_x()?.rotate(2)?,
            Self::FlipXRotateCw270 => image.flip_x()?.rotate_ccw()?,
        };
        Ok(result_image)
    }

    #[allow(dead_code)]
    fn backward(&self, image: &Image) -> anyhow::Result<Image> {
        let result_image: Image = match self {
            Self::Normal => image.clone(),
            Self::RotateCw90 => image.rotate_ccw()?,
            Self::RotateCw180 => image.rotate(2)?,
            Self::RotateCw270 => image.rotate_cw()?,
            Self::FlipX => image.flip_x()?,
            Self::FlipXRotateCw90 => image.rotate_ccw()?.flip_x()?,
            Self::FlipXRotateCw180 => image.rotate(2)?.flip_x()?,
            Self::FlipXRotateCw270 => image.rotate_cw()?.flip_x()?,
        };
        Ok(result_image)
    }

    fn detect_scale(&self, trimmed_mask: &Image, shape_mask: &Image) -> anyhow::Result<Option<ScaleXY>> {
        if trimmed_mask.is_empty() || shape_mask.is_empty() {
            // println!("{:?} images must be 1x1 or bigger", self);
            return Ok(None);
        }

        let transformed_shape_mask: Image = self.backward(shape_mask)?;

        let size0: ImageSize = trimmed_mask.size();
        let size1: ImageSize = transformed_shape_mask.size();
        if size0.is_empty() || size1.is_empty() {
            // println!("{:?} images after transform must be 1x1 or bigger", self);
            return Ok(None);
        }

        let remainder_width: u8 = size0.width % size1.width;
        let remainder_height: u8 = size0.height % size1.height;
        if remainder_width != 0 || remainder_height != 0 {
            // println!("{:?} not divisible. {} {} {} {}", self, size0.width, size0.height, size1.width, size1.height);
            return Ok(None);
        }

        let scale_x: u8 = size0.width / size1.width;
        let scale_y: u8 = size0.height / size1.height;
        if scale_x == 0 || scale_y == 0 {
            // println!("{:?} scale factor is zero", self);
            return Ok(None);
        }

        let scaled_transformed_shape_mask: Image = transformed_shape_mask.resize(size0.width, size0.height)?;

        // println!("{:?} trimmed_mask: {:?}", self, trimmed_mask);
        // println!("{:?} shape_mask: {:?}", self, shape_mask);
        // println!("{:?} transformed_shape_mask: {:?}", self, transformed_shape_mask);
        // println!("{:?} scaled_transformed_shape_mask: {:?}", self, scaled_transformed_shape_mask);

        if *trimmed_mask != scaled_transformed_shape_mask {
            // println!("{:?} images does not match.", self);
            return Ok(None);
        }

        let scale: ScaleXY = ScaleXY { x: scale_x, y: scale_y };
        Ok(Some(scale))
    }

    /// The intention is to always yield the same image, no matter if the input is rotated or flipped.
    /// 
    /// - For a non-square image, ensure the image is in landscape orientation.
    /// - The most massive side is resting on the floor.
    /// - If there is a tie, the prefer object towards the left side.
    /// - If there is a tie, then sort using the raw pixel data.
    /// 
    /// Returns a set of transformations. At least one transformation is always returned.
    /// In case the image can be flipped/rotated then multiple transformations may be returned.
    pub fn normalize_advanced(size: ImageSize, transformation_image_vec: Vec<(ShapeTransformation, Image)>) -> anyhow::Result<(HashSet<ShapeTransformation>, Image)> {
        // Ensure the image is always in landscape orientation
        let width: u8 = size.width.max(size.height);
        let height: u8 = size.width.min(size.height);
        let landscape_size: ImageSize = ImageSize::new(width, height);

        // Obtain center of mass for each image
        type Record = (i32, u32, Image, ShapeTransformation);
        let mut y_x_image_transformation_vec: Vec<Record> = Vec::new();
        for (transformation, image) in &transformation_image_vec {
            if image.size() != landscape_size {
                // Ignore portrait images
                continue;
            }
            let scale: u32 = 10000;
            if let Some((x, y)) = image.center_of_mass(scale) {
                // println!("x: {}, y: {} {:?}", x, y, image);
                let inverted_y: i32 = - (y.min(i32::MAX as u32) as i32);
                y_x_image_transformation_vec.push((inverted_y, x, image.clone(), transformation.clone()));
            }
        }

        // Sort by center of mass, y first, then x, then image data
        // Prefer objects that are bottom heavy and leaning towards the left.
        y_x_image_transformation_vec.sort();

        // println!("SORTED");
        // for (y, x, image) in &y_x_image_vec {
        //     println!("x: {}, y: {} {:?}", x, y, image);
        // }

        if y_x_image_transformation_vec.is_empty() {
            return Err(anyhow::anyhow!("Image vector is empty"));
        }
        // Pick the first image and first transformation
        let record: &Record = &y_x_image_transformation_vec[0];
        let first_image: Image = record.2.clone();
        let first_transformation: ShapeTransformation = record.3.clone();

        // Identify multiple transformations that yield the same image, and remember the transformations
        let mut transformations = HashSet::<ShapeTransformation>::new();
        for (transformation, image) in &transformation_image_vec {
            if *transformation == first_transformation {
                // No need to do expensive image comparison
                continue;
            }
            if *image == first_image {
                transformations.insert(transformation.clone());
            }
        }
        transformations.insert(first_transformation);
        Ok((transformations, first_image))
    }

    #[allow(dead_code)]
    pub fn normalize(image_with_unknown_orientation: &Image) -> anyhow::Result<Image> {
        let size: ImageSize = image_with_unknown_orientation.size();
        let transformations: Vec<(ShapeTransformation, Image)> = Self::perform_all_transformations(&image_with_unknown_orientation)?;
        let (_transformation, output) = Self::normalize_advanced(size, transformations)?;
        Ok(output)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleXY {
    pub x: u8,
    pub y: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeIdentification {
    /// The original mask, without any cropping.
    /// No compression is applied. No transformation is applied.
    pub mask_uncropped: Image,

    /// The shape cropped out from the original mask. 
    /// No compression is applied. No transformation is applied.
    pub mask_cropped: Image,

    /// Bounding box of the shape, which is used for cropping.
    /// The size of the bounding box is the same as the size of the `mask_cropped`.
    pub rect: Rectangle,

    /// The number of solid pixels in the original mask. Transparent pixels does not count.
    /// No compression is applied. No transformation is applied.
    pub mass: u16,

    /// What shape type does it become when rotating the original shape by 90 degrees and doing horz/vert compression.
    /// The recognized shape type, or `Unclassified` if the shape is not recognized.
    pub shape_type: ShapeType,

    /// The transformations that converts from the original shape to the normalized shape.
    pub transformations: HashSet<ShapeTransformation>,

    /// The compacted mask, that is the smallest representation of the shape.
    pub normalized_mask: Option<Image>,

    /// is it scaled down without losing information, apply scale factor to get original size.
    pub scale: Option<ScaleXY>,

    /// Diagonal compression, so that a 10pixel long diagonal line and a 13pixel long diagonal line, gets the same representation.
    /// What shape type does it become when rotating the original shape by 45 degrees followed by horz/vert compression.
    /// The recognized shape type, or `Unclassified` if the shape is not recognized.
    pub shape_type45: ShapeType,
}

impl ShapeIdentification {

    #[allow(dead_code)]
    pub fn compute(mask: &Image) -> anyhow::Result<ShapeIdentification> {
        // Remove the empty space around the shape
        let color_to_be_trimmed: u8 = 0;
        let rect: Rectangle = mask.outer_bounding_box_after_trim_with_color(color_to_be_trimmed)?;
        if rect.is_empty() {
            return Err(anyhow::anyhow!("No shape was found in the image"));
        }
        let trimmed_mask: Image = mask.crop(rect)?;
        let shape_size: ImageSize = trimmed_mask.size();
        if shape_size.is_empty() {
            return Err(anyhow::anyhow!("Integrity error. Empty crop rect should have been rejected earlier"));
        }

        // Measure the mass of the shape
        let mass: u16 = trimmed_mask.mask_count_nonzero();
        if mass == 0 {
            return Err(anyhow::anyhow!("Integrity error. The trim should have rejected it earlier"));
        }

        // After trimming, if it's a 1x1 shape, then it's a square
        if shape_size == ImageSize::new(1, 1) {
            let scale = ScaleXY { x: 1, y: 1 };
            let shape = Self {
                shape_type: ShapeType::Rectangle,
                shape_type45: ShapeType::Rectangle,
                mask_uncropped: mask.clone(),
                mask_cropped: trimmed_mask.clone(),
                rect,
                transformations: ShapeTransformation::all(),
                normalized_mask: None,
                scale: Some(scale),
                mass,
            };
            return Ok(shape);
        }

        // Compact the shape even more by removing duplicate rows and columns
        let compact_mask: Image = trimmed_mask.remove_duplicates()?;

        // The color values in the `trimmed_mask` is either 0 or 1.
        // The color `0` is where the shape is transparent.
        // The color `1` is where the shape is solid.

        // The color values in the rotated mask is either 0 or 1 or 255.
        // The color `0` is where the shape is transparent.
        // The color `1` is where the shape is solid.
        // The color `255` are the gaps introduced by rotate45, and the surrounding space around the rotate45 image.

        let rotated45_raw: Image = trimmed_mask.rotate_ccw_45(255)?;
        let rotated45_padding: Image = rotated45_raw.padding_with_color(1, 255)?;

        // Repair gaps in the rotated image. Every pixel has 4 gaps surrounding it.
        // Which convolution 3x3 variant will behave nicest when processing diagonal data.
        // A: If there is no center pixel and it has top+bottom=2 or left+right=2, then fill the center pixel.
        // B: If there is no center pixel and have 2 or more neighbors, then fill the center pixel.
        // I choose A. And I have not tried out B.
        //
        // This gets rid of the color `255` and settles on either `0` or `1`.
        let repaired45: Image = convolution3x3(&rotated45_padding, |image| {
            let color_center: u8 = image.get(1, 1).unwrap_or(255);
            if color_center != 255 {
                // If the center pixel has a meaningful value, then keep the center pixel as it is.
                return Ok(color_center);
            }
            let color_top: u8 = image.get(1, 0).unwrap_or(255);
            let color_bottom: u8 = image.get(1, 2).unwrap_or(255);
            let color_left: u8 = image.get(0, 1).unwrap_or(255);
            let color_right: u8 = image.get(2, 1).unwrap_or(255);
            let same_top_bottom: bool = color_top == 1 && color_bottom == 1;
            let same_left_right: bool = color_left == 1 && color_right == 1;
            if same_top_bottom || same_left_right {
                return Ok(1);
            }
            Ok(0)
        })?;

        let trimmed_mask45: Image = repaired45.trim_color(0)?;

        // Compact the shape even more by removing duplicate rows and columns
        let compact_mask45: Image = trimmed_mask45.remove_duplicates()?;
        // println!("compact_mask45: {:?}", compact_mask45);

        let transformation_image_vec: Vec<(ShapeTransformation, Image)> = ShapeTransformation::perform_all_transformations(&compact_mask)?;
        let optional_sat0: Option<ShapeAndTransformations> = ShapeAndTransformations::find(&transformation_image_vec);

        let transformation_image_vec45: Vec<(ShapeTransformation, Image)> = ShapeTransformation::perform_all_transformations(&compact_mask45)?;
        let optional_sat45: Option<ShapeAndTransformations> = ShapeAndTransformations::find(&transformation_image_vec45);
        let shape_type45: ShapeType = optional_sat45.map(|sat| sat.shape_type).unwrap_or(ShapeType::Unclassified);

        // If we have a shape for the 0,90,180,270 transformations then stop here.
        // We don't care about the 45 degree transformations.
        // It's computationally cheaper to check for basic shapes, than analyzing an `Unclassified` shape.
        if let Some(sat) = optional_sat0 {
            let mut shape = Self {
                shape_type: sat.shape_type,
                shape_type45,
                mask_uncropped: mask.clone(),
                mask_cropped: trimmed_mask.clone(),
                rect,
                transformations: sat.transformations,
                normalized_mask: None,
                scale: None,
                mass,
            };
            shape.autodetect_scale(&trimmed_mask, &sat.recognized_image)?;
            return Ok(shape);
        }

        // The shape is more advanced than the basic ones we can recognize
        // apply even more computational expensive transformations.
        let (transformations, normalized_mask) = ShapeTransformation::normalize_advanced(compact_mask.size(), transformation_image_vec)?;
        let mut shape = Self {
            shape_type: ShapeType::Unclassified,
            shape_type45,
            mask_uncropped: mask.clone(),
            mask_cropped: trimmed_mask.clone(),
            rect,
            transformations,
            normalized_mask: Some(normalized_mask.clone()),
            scale: None,
            mass,
        };
        shape.autodetect_scale(&trimmed_mask, &normalized_mask)?;
        Ok(shape)
    }

    fn autodetect_scale(&mut self, trimmed_mask: &Image, shape_mask: &Image) -> anyhow::Result<()> {
        for transformation in &self.transformations {
            if let Some(scale) = transformation.detect_scale(trimmed_mask, shape_mask)? {
                self.scale = Some(scale);
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn transformations_sorted_vec(&self) -> Vec<ShapeTransformation> {
        let mut transformations = Vec::<ShapeTransformation>::new();
        for transformation in &self.transformations {
            transformations.push(transformation.clone());
        }
        transformations.sort();
        transformations
    }

    #[cfg(test)]
    fn scale_to_string(&self) -> String {
        match &self.scale {
            Some(scale) => format!("{}x{}", scale.x, scale.y),
            None => "none".to_string(),
        }
    }
}

impl fmt::Display for ShapeIdentification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: String = format!("{}", self.shape_type.name());
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
struct ShapeAndTransformations {
    shape_type: ShapeType, 
    transformations: HashSet<ShapeTransformation>,
    recognized_image: Image,
}

impl ShapeAndTransformations {
    /// Loop over all the basic shapes and see if any of them matches the transformed image
    fn find(transformation_image_vec: &Vec<(ShapeTransformation, Image)>) -> Option<Self> {
        let images_to_recognize: &Vec::<(Image, ShapeType)> = &SHAPE_TYPE_IMAGE.image_shapetype_vec;
        let mut found_transformations = HashSet::<ShapeTransformation>::new();
        for (image_to_recognize, recognized_shape_type) in images_to_recognize {
            for (transformation_type, transformed_image) in transformation_image_vec {
                if *transformed_image == *image_to_recognize {
                    found_transformations.insert(transformation_type.clone());
                }
            }
            if !found_transformations.is_empty() {
                let instance = Self {
                    shape_type: *recognized_shape_type,
                    transformations: found_transformations,
                    recognized_image: image_to_recognize.clone(),
                };
                return Some(instance)
            }
        }
        None
    }    
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_forward_backward() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 1,
            1, 0, 0, 1,
            1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");
        let transformations = ShapeTransformation::all();

        // Act & Assert
        for transformation in &transformations {
            let transformed_image: Image = transformation.forward(&input).expect("image");
            let restored_image: Image = transformation.backward(&transformed_image).expect("image");
            assert_eq!(restored_image, input);
        }
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
    fn test_20000_normalize() {
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
            ShapeTransformation::normalize(i).expect("ok")
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
    fn test_20001_normalize() {
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
            ShapeTransformation::normalize(i).expect("ok")
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
    fn test_20002_normalize() {
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
            ShapeTransformation::normalize(i).expect("ok")
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

    #[test]
    fn test_20003_normalize() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 1,
            0, 0, 1, 0,
            0, 1, 1, 0,
            1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let inputs: Vec<Image> = transformed_images(&input).expect("ok");

        // Act
        let actual_vec: Vec<Image> = inputs.iter().map(|i| 
            ShapeTransformation::normalize(i).expect("ok")
        ).collect();

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 1, 1, 0,
            0, 0, 0, 1,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        for actual in actual_vec {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_20004_normalize_incorrect_edgecase() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 1,
            0, 0, 1, 0,
            0, 1, 0, 0,
            1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let inputs: Vec<Image> = transformed_images(&input).expect("ok");

        // Act
        let actual_vec: Vec<Image> = inputs.iter().map(|i| 
            ShapeTransformation::normalize(i).expect("ok")
        ).collect();

        // Assert
        // Edge case that is handled incorrectly.
        // When the shape is a `diagonal line`, then this is what it should output.
        // let expected_pixels: Vec<u8> = vec![
        //     1, 0, 0, 0,
        //     0, 1, 0, 0,
        //     0, 0, 1, 0,
        //     0, 0, 0, 1,
        // ];
        // When the shape is a diagonal line, then this is what it actually outputs.
        // It's supposed to prefer transformations that causes its mass to be towards the bottom/left corner.
        // And supposed to avoid pixels in the top-right corner.
        // However the center-of-mass algorithm identifies it as being centered, and identifies nothing wrong with it.
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 1,
            0, 0, 1, 0,
            0, 1, 0, 0,
            1, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        for actual in actual_vec {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_110000_empty() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0,
            0, 0,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let error = ShapeIdentification::compute(&input).expect_err("is supposed to fail");

        // Assert
        let message: String = error.to_string();
        assert_eq!(message.contains("No shape was found in the image"), true);
    }

    #[test]
    fn test_120000_rectangle() {
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
        assert_eq!(actual.shape_type45.name(), "rectangle");
        assert_eq!(actual.to_string(), "rectangle");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "1x1");
        assert_eq!(actual.mass, 1);
        assert_eq!(actual.mask_cropped, Image::color(1, 1, 1));
    }

    #[test]
    fn test_120001_rectangle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1,
            1, 1,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "rectangle");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "2x2");
        assert_eq!(actual.mass, 4);
    }

    #[test]
    fn test_120002_rectangle() {
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "3x2");
        assert_eq!(actual.mass, 6);
    }

    #[test]
    fn test_120003_rectangle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 1, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "rectangle");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "6x1");
        assert_eq!(actual.mass, 6);
    }

    #[test]
    fn test_130000_box() {
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
        assert_eq!(actual.mass, 10);
    }

    #[test]
    fn test_130001_box() {
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_130002_box() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1,
            1, 1, 0, 0, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "box");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_140000_plus() {
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_140001_plus() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0,
            0, 1, 0,
            1, 1, 1,
            1, 1, 1,
            0, 1, 0,
            0, 1, 0,
        ];
        let input: Image = Image::try_create(3, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "+");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "1x2");
    }

    #[test]
    fn test_160000_crosshair_shape() {
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
        assert_eq!(actual.to_string(), "✜");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_160001_crosshair_shape() {
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
        assert_eq!(actual.to_string(), "✜");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_160000_l_shape() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_160001_l_shape() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_160002_l_shape() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "1x2");
    }

    #[test]
    fn test_160003_l_shape() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_160004_l_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 1, 0,
            0, 1, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "L");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "1x1");

        let expected_pixels: Vec<u8> = vec![
            1, 1,
            1, 0,
        ];
        let expected_mask_cropped: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual.mask_cropped, expected_mask_cropped);
    }

    #[test]
    fn test_170000_uptack() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_170001_uptack() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_170002_uptack() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_170003_uptack() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_180000_u5() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 1, 1,
            1, 1, 1,
            1, 0, 1,
            1, 0, 1,
            1, 0, 1,
        ];
        let input: Image = Image::try_create(3, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⊔");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "1x3");
    }

    #[test]
    fn test_180001_u5() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_180002_u5() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_180003_u5() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_190000_u4() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_190001_u4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1,
            0, 1, 1,
            1, 0, 1,
            1, 0, 1,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "U4");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180]));
        assert_eq!(actual.scale_to_string(), "1x2");
    }

    #[test]
    fn test_190002_u4() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_190003_u4() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_190004_u4() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_190005_u4() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_190006_u4() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_190007_u4() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_200000_h_uppercase() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_200001_h_uppercase() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::RotateCw180, ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_210000_x() {
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_210001_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 1, 1,
            0, 0, 1, 1, 0, 0,
            1, 1, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(6, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "X");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_220000_turnedv() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_220001_turnedv() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_220002_turnedv() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_220003_turnedv() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 0, 0,
            1, 1, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(6, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⋀");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_230000_diagonal2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1,
            1, 1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "▚");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_230001_diagonal2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0,
            1, 1, 0,
            0, 0, 1,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "▚");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_240000_diagonal3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 1, 1,
            0, 0, 1, 0, 0,
            1, 1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⋱");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_240001_diagonal3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⋱");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_250000_skew_tetramino() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::RotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_250001_skew_tetramino() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_250002_skew_tetramino() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_250003_skew_tetramino() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_260000_h_lowercase() {
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
        assert_eq!(actual.to_string(), "Ⴙ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_260001_h_lowercase() {
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
        assert_eq!(actual.to_string(), "Ⴙ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_260002_h_lowercase() {
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
        assert_eq!(actual.to_string(), "Ⴙ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_260003_h_lowercase() {
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
        assert_eq!(actual.to_string(), "Ⴙ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_270000_inverted_fork() {
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
        assert_eq!(actual.to_string(), "⑃");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_270001_inverted_fork() {
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
        assert_eq!(actual.to_string(), "⑃");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_270002_inverted_fork() {
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
        assert_eq!(actual.to_string(), "⑃");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_270003_inverted_fork() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 1,
            1, 0, 1,
            1, 1, 1,
            1, 1, 1,
            0, 1, 0,
            0, 1, 0,
        ];
        let input: Image = Image::try_create(3, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⑃");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "1x2");
    }

    #[test]
    fn test_280000_rotated_k() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_280001_rotated_k() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_280002_rotated_k() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_280003_rotated_k() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_290000_lower_left_triangle() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "2x2");
    }

    #[test]
    fn test_290001_lower_left_triangle() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_290002_lower_left_triangle() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_290003_lower_left_triangle() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_300000_flipped_j() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_300001_flipped_j() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_300002_flipped_j() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_300003_flipped_j() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_300000_left_plus() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0,
            1, 1, 1, 1,
            1, 1, 1, 1,
            0, 0, 1, 0,
            0, 0, 1, 0,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "ഺ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_310000_leftwards_harpoon_with_barb_upwards() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            0, 1, 1, 0,
            1, 0, 0, 0,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "↼");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_320000_image_box_without_one_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 1, 1, 0,
            1, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "box1");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_330000_image_rotated_d() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            0, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⌓");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_340000_image_rotated_j_round() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0,
            1, 0, 1, 1,
            1, 0, 1, 1,
            1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "ᓚ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_350000_image_box_without_diagonal() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 1,
            0, 1, 1, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 1, 1, 0,
            1, 1, 1, 0,
        ];
        let input: Image = Image::try_create(4, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "box2");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_360000_image_rotated_s() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1,
            0, 0, 1, 1, 1,
            0, 0, 1, 0, 0,
            1, 1, 1, 0, 0,
            1, 1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⥊");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_370000_image_plus_with_one_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0,
            1, 1, 1, 1, 1,
            1, 1, 1, 0, 0,
            1, 1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "+1");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_380000_image_square_without_two_diagonal_corners() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 0, 0,
            1, 1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "square2");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_390000_image_gameoflife_boat() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 0,
            0, 0, 1, 1, 0,
            1, 1, 0, 0, 1,
            1, 1, 1, 1, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "boat");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_400000_image_l_with_45degree_line() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 1, 1,
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "L45");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_410000_image_x_without_one_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            1, 1, 0, 1, 1,
            1, 1, 0, 1, 1,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⋋");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_420000_image_l_skew() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "Ꝇ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_430000_image_uptack_skew() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⊥1");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_440000_image_lower_left_triangle_with_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 1, 1,
            1, 1, 1, 0, 0,
            1, 1, 1, 0, 0,
            1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "◣+1");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_450000_image_i_uppercase_moved_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 0, 0,
            1, 1, 1, 0, 0,
            0, 0, 1, 1, 1,
            1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "I1");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_460000_image_skew_tetromino_with_top_left_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            0, 0, 1, 1, 1,
            0, 0, 1, 1, 1,
            1, 1, 1, 0, 0,
            1, 1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "skew1");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_470000_image_rotated_uppercase_e() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 1, 0, 0, 0,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 0, 0, 0,
            1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⧢");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_480000_image_turned_w() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1,
            1, 1, 0, 0, 0,
            0, 0, 1, 1, 1,
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "ʍ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_490000_image_line_around_small_obstacle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1,
            0, 0, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 0, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 1, 1, 1,
            0, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 7, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "▟▀▙");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_490001_image_line_around_big_obstacle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 1, 1,
            0, 0, 0, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 0, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 0, 1, 1,
            0, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(5, 7, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⎍");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_500000_image_box_with_two_holes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 1, 0, 0, 1,
            1, 1, 1, 1, 1,
            1, 1, 0, 0, 1,
            1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "◫");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_510000_image_box_with_2x2_holes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 0, 1, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 0, 1, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(8, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "𐌎");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_520000_image_x_moved_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0,
            1, 1, 0,
            0, 1, 0,
            0, 1, 0,
            1, 0, 1,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "X1");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_530000_image_lower_left_triangle_without_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0,
            1, 1, 0,
            1, 1, 0,
            0, 1, 1,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "◣-1");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_530001_image_lower_left_triangle_without_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1, 1,
            1, 1, 1, 1, 0, 0,
            1, 1, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "◣-1");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_540000_image_lower_left_triangle_moved_corner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 1,
            1, 0, 1,
            1, 1, 0,
            0, 1, 1,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "◣move");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_550000_image_rotated_p() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 1,
            1, 1, 1,
            1, 0, 0,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "ᓇ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_560000_image_rotated_lowercase_f() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1,
            0, 1, 0,
            1, 1, 1,
            0, 1, 0,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "╋┓");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_570000_image_box_with_rightwards_tick() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1,
            1, 1, 0, 0, 1, 1,
            1, 1, 1, 1, 1, 1,
            0, 0, 1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "⟥");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_570001_image_box_with_uptick() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1,
            1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1,
            0, 0, 1, 1, 0, 0,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "box-with-uptick");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_580000_image_open_box_with_hole_in_center_of_top_border() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 1,
            1, 0, 0,
            1, 0, 1,
            1, 1, 1,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "[_]");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_590000_image_open_box_with_hole_in_right_side_of_top_border() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 1,
            1, 0, 0,
            1, 0, 0,
            1, 1, 1,
            1, 1, 1,
        ];
        let input: Image = Image::try_create(3, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "[_|");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_591000_grid3x2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
        ];
        let input: Image = Image::try_create(5, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "++");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw180, ShapeTransformation::RotateCw180, ShapeTransformation::FlipX, ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_592000_grid3x3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            0, 1, 0, 1, 0,
            1, 1, 1, 1, 1,
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
        ];
        let input: Image = Image::try_create(5, 7, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "#");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_593000_grid4x2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0, 1, 0, 0,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            0, 1, 0, 1, 0, 1, 0, 0,
            0, 1, 0, 1, 0, 1, 0, 0,
        ];
        let input: Image = Image::try_create(8, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "+++");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipX, ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_594000_grid4x3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0, 1, 0,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
            0, 1, 0, 1, 0, 1, 0,
            1, 1, 1, 1, 1, 1, 1,
            0, 1, 0, 1, 0, 1, 0,
            0, 1, 0, 1, 0, 1, 0,
        ];
        let input: Image = Image::try_create(7, 7, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "‡‡‡");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipX, ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_600000_unclassified() {
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
        assert_eq!(actual.normalized_mask, Some(expected_compact));
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_600001_unclassified() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 0,
            1, 0, 0, 1,
            0, 0, 0, 1,
            1, 0, 0, 0,
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
        assert_eq!(actual.normalized_mask, Some(expected_compact));
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_600002_unclassified() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            1, 0, 0, 0,
            1, 0, 0, 0,
            1, 1, 0, 1,
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
        assert_eq!(actual.normalized_mask, Some(expected_compact));
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_600003_unclassified() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            1, 0, 0, 0,
            1, 1, 0, 1,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

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
        assert_eq!(actual.normalized_mask, Some(expected_compact));
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_600004_unclassified() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            0, 1, 1, 0,
            1, 0, 0, 0,
            1, 0, 0, 0,
            1, 1, 0, 1,
            1, 1, 0, 1,
        ];
        let input: Image = Image::try_create(4, 6, pixels).expect("image");

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
        assert_eq!(actual.normalized_mask, Some(expected_compact));
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "1x2");
    }

    #[test]
    fn test_600005_unclassified() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 0, 0,
            1, 1, 0, 0, 1, 1,
            0, 0, 0, 0, 1, 1,
            1, 1, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

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
        assert_eq!(actual.normalized_mask, Some(expected_compact));
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_600006_unclassified_multiple_transformations() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0,
            0, 1, 1, 1, 0,
            1, 1, 0, 1, 1,
            1, 0, 0, 0, 1,
            1, 1, 0, 1, 1,
            0, 1, 1, 1, 0,
        ];
        let input: Image = Image::try_create(5, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "unclassified");

        let expected_pixels: Vec<u8> = vec![
            0, 1, 1, 1, 0, 0,
            1, 1, 0, 1, 1, 0,
            1, 0, 0, 0, 1, 1,
            1, 1, 0, 1, 1, 0,
            0, 1, 1, 1, 0, 0,
        ];
        let expected_compact: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual.normalized_mask, Some(expected_compact));
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_600007_unclassified_scale2x1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 1, 1, 0, 0,
            0, 0, 1, 1, 0, 0, 1, 1,
            1, 1, 0, 0, 0, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(8, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.to_string(), "unclassified");

        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0,
            1, 0, 1, 0,
            1, 0, 0, 1,
            1, 1, 1, 1,
        ];
        let expected_compact: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual.normalized_mask, Some(expected_compact));
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "2x1");
    }

    #[test]
    fn test_610000_rotate45_uptack() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 1, 0, 1, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 1, 0,
            0, 0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.shape_type45.name(), "⊥");
        assert_eq!(actual.to_string(), "unclassified");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw270, ShapeTransformation::RotateCw180]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_610001_rotate45_box() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0,
            0, 1, 0, 1, 0,
            1, 0, 0, 0, 1,
            0, 1, 0, 1, 0,
            0, 0, 1, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.shape_type45.name(), "box");
        assert_eq!(actual.to_string(), "unclassified");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_610002_rotate45_tetris_tetromino() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 1, 0,
            0, 1, 0, 1,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.shape_type45.name(), "⥊");
        assert_eq!(actual.to_string(), "unclassified");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_610003_rotate45_uppercase_e() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0,
            0, 1, 0, 1, 0, 0,
            0, 0, 1, 0, 0, 0,
            0, 0, 0, 1, 0, 1,
            0, 0, 0, 0, 1, 0,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.shape_type45.name(), "⧢");
        assert_eq!(actual.to_string(), "unclassified");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_610004_rotate45_diagonal_line2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0,
            0, 1,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.shape_type45.name(), "rectangle");
        assert_eq!(actual.to_string(), "▚");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270, ShapeTransformation::RotateCw180]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_610005_rotate45_diagonal_line3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 1, 0,
            0, 0, 1,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.shape_type45.name(), "rectangle");
        assert_eq!(actual.to_string(), "⋱");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_610006_rotate45_diagonal_line4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.shape_type45.name(), "rectangle");
        assert_eq!(actual.to_string(), "unclassified");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_610007_rotate45_diagonal_line5() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 1, 0,
            0, 0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: ShapeIdentification = ShapeIdentification::compute(&input).expect("ok");

        // Assert
        assert_eq!(actual.shape_type45.name(), "rectangle");
        assert_eq!(actual.to_string(), "unclassified");
        // The following transformations are 90degrees wrong. It seems like the center-of-mass algorithm
        // cannot deal with this edge case. It must prefer the shape being as close to the bottom-left corner.
        // and avoid the top-right corner. A compromise for a diagonal line, is to go from the top-left corner 
        // to the bottom-right corner. This way the top-right corner is avoided, but the bottom-left corner is not reached.
        // Future experiment:
        // Tweak the center-of-mass algorithm so it can deal with this edge case.
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }
}
