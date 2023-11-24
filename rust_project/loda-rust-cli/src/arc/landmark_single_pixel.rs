use super::{Image, ImageMask, ImageMaskCount, MixMode, ImageMix, ImageCrop, Shape3x3};
use std::collections::HashSet;
use anyhow::bail;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LandmarkSinglePixel {
    pub x: u8,
    pub y: u8,
    pub color: u8,
}

impl LandmarkSinglePixel {
    #[allow(dead_code)]
    pub fn analyze(image: &Image, background_color: u8) -> anyhow::Result<Self> {
        let mask: Image = image.to_mask_where_color_is_different(background_color);
        let count: u16 = mask.mask_count_nonzero();
        if count == 0 {
            bail!("the image is entirely the background color. no landmark found");
        }
        if let Some(rectangle) = mask.bounding_box() {
            if rectangle.width() == 1 && rectangle.height() == 1 {
                let x_i32: i32 = rectangle.min_x();
                let y_i32: i32 = rectangle.min_y();
                if x_i32 < 0 || y_i32 < 0 || x_i32 >= 255 || y_i32 >= 255 {
                    bail!("the position is outside the image");
                }
                let x: u8 = x_i32 as u8;
                let y: u8 = y_i32 as u8;
                let color: u8 = image.get(x_i32, y_i32).unwrap_or(255);
                return Ok(LandmarkSinglePixel {
                    x,
                    y,
                    color,
                });
            }
        }

        let shapeid_set = HashSet::<u8>::from([
            4, // L-shape 45 degree 
            6, // L-shape
            8, // bended line
            14, // T-shape
            19, // T-shape 45 degree
            24, // 3 lines meeting in center
            25, // 3 lines meeting in center
            28, // one pixel attached on a 2x2 square
            37, // Plus shape
            43, // Plus with diagonal corners
            45, // Plus shape 45 degree
        ]);

        let mut corner_mask: Image = mask.clone_zero();
        for y in 0..mask.height() {
            for x in 0..mask.width() {
                let color: u8 = mask.get(x as i32, y as i32).unwrap_or(0);
                if color == 0 {
                    continue;
                }
                let area3x3: Image = mask.crop_outside(x as i32 - 1, y as i32 - 1, 3, 3, 0)?;
                let (shape_id, _transformations) = Shape3x3::catalog_without_trim().shapeid_and_transformations(&area3x3)?;

                if shapeid_set.contains(&shape_id) {
                    _ = corner_mask.set(x as i32, y as i32, 1);
                }
            }
        }

        let combined_mask: Image = mask.mix(&corner_mask, MixMode::Multiply)?;

        let count: u16 = combined_mask.mask_count_nonzero();
        if count == 0 {
            bail!("zero landmarks found in the corner mask");
        }
        if count >= 2 {
            bail!("2 or more landmarks found in the corner mask");
        }
        if let Some(rectangle) = combined_mask.bounding_box() {
            if rectangle.width() == 1 && rectangle.height() == 1 {
                let x_i32: i32 = rectangle.min_x();
                let y_i32: i32 = rectangle.min_y();
                if x_i32 < 0 || y_i32 < 0 || x_i32 >= 255 || y_i32 >= 255 {
                    bail!("the position is outside the image");
                }
                let x: u8 = x_i32 as u8;
                let y: u8 = y_i32 as u8;
                let color: u8 = image.get(x_i32, y_i32).unwrap_or(255);
                return Ok(LandmarkSinglePixel {
                    x,
                    y,
                    color,
                });
            }
        }

        bail!("didn't find a single pixel landmark")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_110000_one_pixel() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 6, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 1,
            y: 2,
            color: 6,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_120000_l_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 7, 3, 0,
            0, 6, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 1,
            y: 1,
            color: 7,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_120001_l_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 3, 0, 0,
            6, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 0,
            y: 0,
            color: 5,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_130000_t_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 3, 0,
            0, 6, 1, 4,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 2,
            y: 2,
            color: 1,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_130001_t_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 5, 0,
            0, 0, 5, 0,
            0, 0, 3, 0,
            0, 6, 1, 4,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 2,
            y: 3,
            color: 1,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_140000_plus_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 3, 0,
            0, 6, 9, 1,
            0, 0, 2, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 2,
            y: 2,
            color: 9,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_150000_x_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 0, 2, 0,
            0, 5, 0, 0,
            1, 0, 3, 0,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 1,
            y: 1,
            color: 5,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_160000_45degree_l_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 2, 0,
            0, 9, 0, 0,
            0, 0, 3, 0,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 1,
            y: 1,
            color: 9,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_170000_45degree_t_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 2, 0,
            0, 9, 0, 0,
            6, 0, 3, 0,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 1,
            y: 1,
            color: 9,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_180000_45degree_bending_line() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            3, 6, 9, 0, 0,
            0, 0, 0, 3, 0,
            0, 0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 2,
            y: 1,
            color: 9,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_190100_three_lines_meeting_in_center() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 5,
            0, 0, 0, 5, 0,
            8, 8, 6, 0, 0,
            0, 0, 0, 7, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 2,
            y: 2,
            color: 6,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_190200_three_lines_meeting_in_center() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 8, 0, 0,
            8, 8, 6, 0, 0,
            0, 0, 0, 5, 0,
            0, 0, 0, 0, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 2,
            y: 1,
            color: 6,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_190300_one_pixel_attached_on_a_2x2_square() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 7,
            0, 0, 0, 8, 0,
            0, 8, 1, 0, 0,
            0, 9, 3, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 2,
            y: 2,
            color: 1,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_190400_plus_with_diagonal_corners() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9, 8, 0,
            0, 8, 7, 9, 0,
            0, 9, 3, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 2,
            y: 1,
            color: 7,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_190401_plus_with_diagonal_corners() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9, 8, 8,
            8, 8, 7, 9, 9,
            9, 9, 3, 0, 0,
            5, 5, 3, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: LandmarkSinglePixel = LandmarkSinglePixel::analyze(&input, 0).expect("ok");

        // Assert
        let expected = LandmarkSinglePixel {
            x: 2,
            y: 1,
            color: 7,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_200000_reject_empty() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "the image is entirely the background color. no landmark found");
    }

    #[test]
    fn test_200001_reject_empty() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5,
            5, 5, 5, 5,
            5, 5, 5, 5,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_200002_reject_half_and_half() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 3, 3,
            5, 5, 3, 3,
            5, 5, 3, 3,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_210000_reject_two_single_pixels() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 3,
            0, 6, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_220000_reject_box() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 3, 8, 3,
            0, 6, 0, 3,
            0, 5, 7, 9,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "2 or more landmarks found in the corner mask");
    }

    #[test]
    fn test_230000_reject_two_l_shapes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            3, 3, 8, 0, 0,
            0, 0, 0, 0, 8,
            0, 5, 7, 9, 7,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "2 or more landmarks found in the corner mask");
    }

    #[test]
    fn test_240000_reject_two_plus_shapes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            3, 3, 8, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 0, 9, 0,
            0, 0, 1, 9, 7,
            0, 0, 0, 9, 0,
        ];
        let input: Image = Image::try_create(5, 6, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "2 or more landmarks found in the corner mask");
    }

    #[test]
    fn test_250000_reject_two_t_shapes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            3, 3, 8, 0, 0,
            1, 0, 0, 0, 0,
            0, 0, 0, 9, 0,
            0, 0, 1, 9, 7,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "2 or more landmarks found in the corner mask");
    }

    #[test]
    fn test_260000_reject_rectangle_2x2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 5, 6, 0, 0,
            0, 7, 8, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_260001_reject_rectangle_2x2_attached_to_border() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 6, 0,
            7, 8, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_260002_reject_rectangle_3x2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 4, 5, 6, 0,
            0, 7, 8, 9, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_270000_reject_line() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 7, 8, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_270001_reject_line() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 7, 0,
            0, 8, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_270002_reject_line() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 7, 0,
            0, 0, 8,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_270003_reject_line() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 8,
            0, 7, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_280000_reject_skew_tetris() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 7, 8, 0, 0,
            0, 0, 4, 5, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_280001_reject_skew_tetris_attached_to_border() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 7, 8, 0, 0,
            0, 0, 4, 5, 0,
        ];
        let input: Image = Image::try_create(5, 2, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_280002_reject_45degree_bending_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            3, 6, 9, 0, 0,
            5, 5, 5, 3, 0,
            5, 5, 5, 5, 1,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "zero landmarks found in the corner mask");
    }

    #[test]
    fn test_280003_reject_c_shape() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            3, 6, 9, 0, 0,
            7, 0, 0, 0, 0,
            9, 9, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let error = LandmarkSinglePixel::analyze(&input, 0).expect_err("should fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "2 or more landmarks found in the corner mask");
    }
}
