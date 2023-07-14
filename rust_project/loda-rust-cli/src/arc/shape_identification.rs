//! Identification of shape type and transformations.
//! 
//! Invariant to translation, scaling, flipping, rotation (90, 180, 270), horizontal/vertical-compression.
//! 
//! Similar to SIFT (Scale-invariant feature transform) but without using points.
//! https://en.wikipedia.org/wiki/Scale-invariant_feature_transform
use super::{Image, ImageSize, ImageTrim, ImageRemoveDuplicates, ImageTryCreate, ImageRotate, ImageSymmetry, CenterOfMass, Rectangle, ImageCrop, ImageResize};
use std::fmt;
use std::collections::HashSet;
use lazy_static::lazy_static;

lazy_static! {
    static ref SHAPE_TYPE_IMAGE: ShapeTypeImage = ShapeTypeImage::new().expect("Unable to create shape type image");
}

struct ShapeTypeImage {
    image_box: Image,
    image_plus: Image,
    image_crosshair: Image,
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
    image_inverted_fork: Image,
    image_rotated_k: Image,
    image_lower_left_triangle: Image,
    image_left_plus: Image,
    image_leftwards_harpoon_with_barb_upwards: Image,
    image_box_without_one_corner: Image,
    image_rotated_d: Image,
    image_rotated_j_round: Image,
    image_box_without_diagonal: Image,
    image_rotated_s: Image,
    image_plus_with_one_corner: Image,
    image_square_without_diagonal_corners: Image,
    image_gameoflife_boat: Image,
    image_l_with_45degree_line: Image,
    image_x_without_one_corner: Image,
    image_l_skew: Image,
    image_uptack_skew: Image,
    image_lower_left_triangle_with_corner: Image,
    image_i_uppercase_moved_corner: Image,
    image_skew_tetromino_with_top_left_corner: Image,
    image_rotated_uppercase_e: Image,
    image_turned_w: Image,
    image_line_around_obstacle: Image,
    image_box_with_two_holes: Image,
    image_box_with_2x2_holes: Image,
    image_x_moved_corner: Image,
    image_lower_left_triangle_without_corner: Image,
    image_lower_left_triangle_moved_corner: Image,
    image_rotated_p: Image,
    image_rotated_lowercase_f: Image,
    image_box_with_rightwards_tick: Image,
    image_open_box_with_hole_in_border: Image,
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

        let image_line_around_obstacle: Image = Image::try_create(5, 2, vec![
            0, 1, 1, 1, 0,
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

        let image_open_box_with_hole_in_border: Image = Image::try_create(5, 3, vec![
            1, 1, 0, 1, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
        ])?;

        let instance = Self {
            image_box,
            image_plus,
            image_crosshair,
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
            image_inverted_fork,
            image_rotated_k,
            image_lower_left_triangle,
            image_left_plus,
            image_leftwards_harpoon_with_barb_upwards,
            image_box_without_one_corner,
            image_rotated_d,
            image_rotated_j_round,
            image_box_without_diagonal,
            image_rotated_s,
            image_plus_with_one_corner,
            image_square_without_diagonal_corners,
            image_gameoflife_boat,
            image_l_with_45degree_line,
            image_x_without_one_corner,
            image_l_skew,
            image_uptack_skew,
            image_lower_left_triangle_with_corner,
            image_i_uppercase_moved_corner,
            image_skew_tetromino_with_top_left_corner,
            image_rotated_uppercase_e,
            image_turned_w,
            image_line_around_obstacle,
            image_box_with_two_holes,
            image_box_with_2x2_holes,
            image_x_moved_corner,
            image_lower_left_triangle_without_corner,
            image_lower_left_triangle_moved_corner,
            image_rotated_p,
            image_rotated_lowercase_f,
            image_box_with_rightwards_tick,
            image_open_box_with_hole_in_border,
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

    /// A horizontal line shape `▟▀▙` around an obstacle.
    /// 
    /// ````
    /// 0, 1, 1, 1, 0
    /// 1, 1, 0, 1, 1
    /// ```
    LineAroundObstacle,

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
    OpenBoxWithHoleInBorder,

    /// Shapes that could not be recognized.
    Unclassified,

    // Future experiments
    // dashed line
    // checker board
    // ◆ Diamond
    // pyramid
}

impl ShapeType {
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
            Self::Diagonal2 => "▞",
            Self::Diagonal3 => "⋰",
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
            Self::LineAroundObstacle => "▟▀▙",
            Self::BoxWithTwoHoles => "◫",
            Self::BoxWith2x2Holes => "𐌎",
            Self::XMovedCorner => "X1",
            Self::LowerLeftTriangleWithoutCorner => "◣-1",
            Self::LowerLeftTriangleMovedCorner => "◣move",
            Self::RotatedP => "ᓇ",
            Self::RotatedLowercaseF => "╋┓",
            Self::BoxWithRightwardsTick => "⟥",
            Self::OpenBoxWithHoleInBorder => "[_]",
            Self::Unclassified => "unclassified",
        }
    }
}

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

    fn perform_all_transformations(image: &Image) -> anyhow::Result<Vec<(ShapeTransformation, Image)>> {
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

    fn apply_forward(&self, image: &Image) -> anyhow::Result<Image> {
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

    fn apply_backward(&self, image: &Image) -> anyhow::Result<Image> {
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

        let transformed_shape_mask: Image = self.apply_backward(shape_mask)?;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleXY {
    pub x: u8,
    pub y: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeIdentification {
    /// The recognized shape type, or `Unclassified` if the shape is not recognized.
    pub shape_type: ShapeType,

    /// The original mask, without any cropping.
    pub mask_uncropped: Image,

    /// Bounding box of the shape, which is used for cropping.
    pub rect: Rectangle,

    /// The transformations that converts from the original shape to the normalized shape.
    pub transformations: HashSet<ShapeTransformation>,

    /// The compacted mask, that is the smallest representation of the shape.
    pub normalized_mask: Option<Image>,

    /// is it scaled down without losing information, apply scale factor to get original size.
    pub scale: Option<ScaleXY>,

    // Future experiments
    // diagonal compression, so that a 10pixel long diagonal line and a 13pixel long diagonal line, gets the same representation.
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

        // After trimming, if it's a 1x1 shape, then it's a square
        if shape_size == ImageSize::new(1, 1) {
            let scale = ScaleXY { x: 1, y: 1 };
            let shape = Self {
                shape_type: ShapeType::Rectangle,
                mask_uncropped: mask.clone(),
                rect,
                transformations: ShapeTransformation::all(),
                normalized_mask: None,
                scale: Some(scale),
            };
            return Ok(shape);
        }

        // Compact the shape even more by removing duplicate rows and columns
        let compact_mask: Image = trimmed_mask.remove_duplicates()?;

        // If it compacts into a 1x1 pixel then it's a square or rectangle
        if compact_mask.size() == ImageSize::new(1, 1) {
            let scale = ScaleXY { x: trimmed_mask.width(), y: trimmed_mask.height() };
            let shape = Self {
                shape_type: ShapeType::Rectangle,
                mask_uncropped: mask.clone(),
                rect,
                transformations: ShapeTransformation::all(),
                normalized_mask: None,
                scale: Some(scale),
            };
            return Ok(shape);    
        }

        let transformations: Vec<(ShapeTransformation, Image)> = ShapeTransformation::perform_all_transformations(&compact_mask)?;

        {
            let mut images_to_recognize = Vec::<(&Image, ShapeType)>::new();
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_box, ShapeType::Box));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_plus, ShapeType::Plus));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_crosshair, ShapeType::Crosshair));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_x, ShapeType::X));
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
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_inverted_fork, ShapeType::InvertedFork));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_rotated_k, ShapeType::RotatedK));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_lower_left_triangle, ShapeType::LowerLeftTriangle));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_left_plus, ShapeType::LeftPlus));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_leftwards_harpoon_with_barb_upwards, ShapeType::LeftwardsHarpoonWithBarbUpwards));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_box_without_one_corner, ShapeType::BoxWithoutOneCorner));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_rotated_d, ShapeType::RotatedD));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_rotated_j_round, ShapeType::RotatedJRound));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_box_without_diagonal, ShapeType::BoxWithoutDiagonal));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_rotated_s, ShapeType::RotatedS));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_plus_with_one_corner, ShapeType::PlusWithOneCorner));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_square_without_diagonal_corners, ShapeType::SquareWithoutDiagonalCorners));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_gameoflife_boat, ShapeType::GameOfLifeBoat));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_l_with_45degree_line, ShapeType::LWith45DegreeLine));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_x_without_one_corner, ShapeType::XWithoutOneCorner));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_l_skew, ShapeType::LSkew));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_uptack_skew, ShapeType::UpTackSkew));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_lower_left_triangle_with_corner, ShapeType::LowerLeftTriangleWithCorner));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_i_uppercase_moved_corner, ShapeType::IUppercaseMovedCorner));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_skew_tetromino_with_top_left_corner, ShapeType::SkewTetrominoWithTopLeftCorner));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_rotated_uppercase_e, ShapeType::RotatedUppercaseE));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_turned_w, ShapeType::TurnedW));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_line_around_obstacle, ShapeType::LineAroundObstacle));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_box_with_two_holes, ShapeType::BoxWithTwoHoles));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_box_with_2x2_holes, ShapeType::BoxWith2x2Holes));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_x_moved_corner, ShapeType::XMovedCorner));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_lower_left_triangle_without_corner, ShapeType::LowerLeftTriangleWithoutCorner));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_lower_left_triangle_moved_corner, ShapeType::LowerLeftTriangleMovedCorner));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_rotated_p, ShapeType::RotatedP));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_rotated_lowercase_f, ShapeType::RotatedLowercaseF));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_box_with_rightwards_tick, ShapeType::BoxWithRightwardsTick));
            images_to_recognize.push((&SHAPE_TYPE_IMAGE.image_open_box_with_hole_in_border, ShapeType::OpenBoxWithHoleInBorder));

            let mut found_transformations = HashSet::<ShapeTransformation>::new();
            for (image_to_recognize, recognized_shape_type) in &images_to_recognize {
                for (transformation_type, transformed_image) in &transformations {
                    if *transformed_image == **image_to_recognize {
                        found_transformations.insert(transformation_type.clone());
                    }
                }
                if !found_transformations.is_empty() {
                    let mut shape = Self {
                        shape_type: *recognized_shape_type,
                        mask_uncropped: mask.clone(),
                        rect,
                        transformations: found_transformations,
                        normalized_mask: None,
                        scale: None,
                    };
                    shape.autodetect_scale(&trimmed_mask, &image_to_recognize)?;
                    return Ok(shape);
                }
            }
        }

        // The shape is more advanced than the basic ones we can recognize
        // apply even more expensive transformations to recognize it.
        let (transformation, normalized_mask) = Self::normalize(compact_mask.size(), transformations)?;
        let mut shape = Self {
            shape_type: ShapeType::Unclassified,
            mask_uncropped: mask.clone(),
            rect,
            transformations: HashSet::<ShapeTransformation>::from([transformation]),
            normalized_mask: Some(normalized_mask.clone()),
            scale: None,
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

    /// The intention is to always yield the same image, no matter if the input is rotated or flipped.
    /// 
    /// - For a non-square image, ensure the image is in landscape orientation.
    /// - The most massive side is resting on the floor.
    /// - If there is a tie, the prefer object towards the left side.
    /// - If there is a tie, then sort using the raw pixel data.
    fn normalize(size: ImageSize, transformations: Vec<(ShapeTransformation, Image)>) -> anyhow::Result<(ShapeTransformation, Image)> {
        // Ensure the image is always in landscape orientation
        let width: u8 = size.width.max(size.height);
        let height: u8 = size.width.min(size.height);
        let landscape_size: ImageSize = ImageSize::new(width, height);

        // Obtain center of mass for each image
        type Record = (i32, u32, Image, ShapeTransformation);
        let mut y_x_image_transformation_vec: Vec<Record> = Vec::new();
        for (transformation, image) in &transformations {
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
        // Pick the first image
        let record: &Record = &y_x_image_transformation_vec[0];
        let image: Image = record.2.clone();
        let transformation: ShapeTransformation = record.3.clone();
        Ok((transformation, image))
    }

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
        let error = ShapeIdentification::compute(&input).expect_err("is supposed to fail");

        // Assert
        let message: String = error.to_string();
        assert_eq!(message.contains("No shape was found in the image"), true);
    }

    #[test]
    fn test_20000_rectangle() {
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
        assert_eq!(actual.to_string(), "rectangle");
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_20001_rectangle() {
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "3x2");
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "6x1");
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_30002_box() {
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_40001_plus() {
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
    fn test_50000_crosshair_shape() {
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
    fn test_50001_crosshair_shape() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "1x2");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "2x1");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "1x1");
    }

    #[test]
    fn test_80000_u5() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "2x1");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "1x1");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_90001_u4() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270]));
        assert_eq!(actual.scale_to_string(), "2x1");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "2x1");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90]));
        assert_eq!(actual.scale_to_string(), "1x1");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "1x1");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::RotateCw180, ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, ShapeTransformation::all());
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_110001_x() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_120003_turnedv() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270, ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::RotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipX, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::RotateCw270]));
        assert_eq!(actual.scale_to_string(), "2x1");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "2x1");
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
        assert_eq!(actual.to_string(), "Ⴙ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.to_string(), "Ⴙ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.to_string(), "Ⴙ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.to_string(), "Ⴙ");
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_170000_inverted_fork() {
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
    fn test_170001_inverted_fork() {
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
    fn test_170002_inverted_fork() {
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
    fn test_170003_inverted_fork() {
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal, ShapeTransformation::FlipXRotateCw90]));
        assert_eq!(actual.scale_to_string(), "2x2");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180, ShapeTransformation::FlipXRotateCw270]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw90, ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::Normal]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipX]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
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
        assert_eq!(actual.transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::FlipXRotateCw180]));
        assert_eq!(actual.scale_to_string(), "none");
    }

    #[test]
    fn test_200000_left_plus() {
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
    fn test_210000_leftwards_harpoon_with_barb_upwards() {
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
    fn test_220000_image_box_without_one_corner() {
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
    fn test_230000_image_rotated_d() {
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
    fn test_240000_image_rotated_j_round() {
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
    fn test_250000_image_box_without_diagonal() {
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
    fn test_260000_image_rotated_s() {
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
    fn test_270000_image_plus_with_one_corner() {
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
    fn test_280000_image_square_without_two_diagonal_corners() {
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
    fn test_290000_image_gameoflife_boat() {
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
    fn test_300000_image_l_with_45degree_line() {
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
    fn test_310000_image_x_without_one_corner() {
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
    fn test_320000_image_l_skew() {
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
    fn test_330000_image_uptack_skew() {
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
    fn test_340000_image_lower_left_triangle_with_corner() {
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
    fn test_350000_image_i_uppercase_moved_corner() {
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
    fn test_360000_image_skew_tetromino_with_top_left_corner() {
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
    fn test_370000_image_rotated_uppercase_e() {
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
    fn test_380000_image_turned_w() {
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
    fn test_390000_image_line_around_obstacle() {
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
    fn test_400000_image_box_with_two_holes() {
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
    fn test_410000_image_box_with_2x2_holes() {
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
    fn test_420000_image_x_moved_corner() {
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
    fn test_430000_image_lower_left_triangle_without_corner() {
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
    fn test_430001_image_lower_left_triangle_without_corner() {
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
    fn test_440000_image_lower_left_triangle_moved_corner() {
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
    fn test_450000_image_rotated_p() {
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
    fn test_460000_image_rotated_lowercase_f() {
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
    fn test_470000_image_box_with_rightwards_tick() {
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
    fn test_480000_image_open_box_with_hole_in_border() {
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
    fn test_500000_unclassified() {
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
    fn test_500001_unclassified() {
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
    fn test_500002_unclassified() {
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
    fn test_500003_unclassified() {
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
    fn test_500004_unclassified() {
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
    fn test_500005_unclassified() {
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

    fn normalize(image_with_unknown_orientation: &Image) -> anyhow::Result<Image> {
        let size: ImageSize = image_with_unknown_orientation.size();
        let transformations: Vec<(ShapeTransformation, Image)> = ShapeTransformation::perform_all_transformations(&image_with_unknown_orientation)?;
        let (_transformation, output) = ShapeIdentification::normalize(size, transformations)?;
        Ok(output)
    }

    #[test]
    fn test_600000_normalize() {
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
            normalize(i).expect("ok")
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
    fn test_600001_normalize() {
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
            normalize(i).expect("ok")
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
    fn test_600002_normalize() {
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
            normalize(i).expect("ok")
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
