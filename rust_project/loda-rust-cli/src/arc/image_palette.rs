use super::{Image, ImageHistogram, ImageExtractRowColumn, ImageStack, ImageTryCreate, ImageRotate};
use super::{ImageSymmetry, ImageDetectColorSymmetry, ImageDetectColorSymmetryMode};

pub trait ImageCreatePalette {
    /// Color mapping from `self` (the source image) to `target_image`, based on histogram data.
    /// 
    /// This is intended for scenarios where two image have the exact same number of unique colors.
    /// 
    /// If the number of unique colors are different, then an error is returned.
    /// 
    /// If the 2 or more colors have the same count, then their ordering is random.
    /// 
    /// Result image:
    /// - Top row is the source colors.
    /// - Bottom row is the destination colors.
    fn palette_using_histogram(&self, target_image: &Image, reverse: bool) -> anyhow::Result<Image>;
    
    /// Color mapping from `self` (the source image) to `target_image`, based on color symmetry data.
    /// 
    /// Result image:
    /// - Top row is the source colors.
    /// - Bottom row is the destination colors.
    fn palette_using_color_symmetry(&self, target_image: &Image, reverse: bool) -> anyhow::Result<Image>;
}

impl ImageCreatePalette for Image {
    fn palette_using_histogram(&self, target_image: &Image, reverse: bool) -> anyhow::Result<Image> {
        let histogram0 = self.histogram_all();
        let histogram1 = target_image.histogram_all();
        let count0: u32 = histogram0.number_of_counters_greater_than_zero();
        let count1: u32 = histogram1.number_of_counters_greater_than_zero();
        if count0 != count1 {
            return Err(anyhow::anyhow!("both images must have the same number of colors, cannot construct mapping. self has {} colors. target_image has {} colors.", count0, count1));
        }

        let histogram_image0: Image = histogram0.to_image()?;
        let histogram_image1: Image = histogram1.to_image()?;
        // The colors are stored in the bottom rows of the histogram image.

        // Extract the color rows
        let source_colors: Image = histogram_image0.bottom_rows(1)?;
        let mut target_colors: Image = histogram_image1.bottom_rows(1)?;

        if reverse {
            target_colors = target_colors.flip_x()?;
        }

        // Top row is the source colors 
        // Bottom row is the destination colors 
        let palette_image: Image = source_colors.vjoin(target_colors)?;
        Ok(palette_image)
    }

    fn palette_using_color_symmetry(&self, target_image: &Image, reverse: bool) -> anyhow::Result<Image> {
        let source_colors: Image;
        match self.detect_color_symmetry() {
            ImageDetectColorSymmetryMode::Empty => {
                return Err(anyhow::anyhow!("Expected non-empty image, but self is empty"));
            },
            ImageDetectColorSymmetryMode::NoSymmetryDetected => {
                return Err(anyhow::anyhow!("Detected no symmetry in the source image"));
            },
            ImageDetectColorSymmetryMode::Same => {
                match target_image.detect_color_symmetry() {
                    ImageDetectColorSymmetryMode::Same => {
                        let source_color: u8 = self.get(0, 0).unwrap_or(255);
                        let target_color: u8 = target_image.get(0, 0).unwrap_or(255);
                        let image: Image = Image::try_create(1, 2, vec![source_color, target_color])?;
                        return Ok(image);
                    },
                    _ => {
                        return Err(anyhow::anyhow!("Detected full symmetry in the source image, but no full symmetry in the target image. Unclear what mapping to pick."));
                    }
                }
            },
            ImageDetectColorSymmetryMode::Rows => {
                let image: Image = self.left_columns(1)?;
                source_colors = image.rotate_ccw()?;
            },
            ImageDetectColorSymmetryMode::Columns => {
                source_colors = self.top_rows(1)?;
            }
        }

        let mut target_colors: Image;
        match target_image.detect_color_symmetry() {
            ImageDetectColorSymmetryMode::Empty => {
                return Err(anyhow::anyhow!("Expected non-empty image, but target_image is empty"));
            },
            ImageDetectColorSymmetryMode::NoSymmetryDetected => {
                return Err(anyhow::anyhow!("Detected no symmetry in the target image"));
            },
            ImageDetectColorSymmetryMode::Same => {
                let target_color: u8 = target_image.get(0, 0).unwrap_or(255);
                target_colors = Image::color(source_colors.width(), 1, target_color);
            },
            ImageDetectColorSymmetryMode::Rows => {
                let image: Image = target_image.left_columns(1)?;
                target_colors = image.rotate_ccw()?;
                if target_colors.width() != source_colors.width() {
                    return Err(anyhow::anyhow!("target_image.columns: Mismatch in number of symmetric colors. Cannot construct a meaningful palette."));
                }
            },
            ImageDetectColorSymmetryMode::Columns => {
                target_colors = target_image.top_rows(1)?;
                if target_colors.width() != source_colors.width() {
                    return Err(anyhow::anyhow!("target_image.rows: Mismatch in number of symmetric colors. Cannot construct a meaningful palette."));
                }
            }
        }

        if reverse {
            target_colors = target_colors.flip_x()?;
        }

        // Top row is the source colors 
        // Bottom row is the destination colors 
        let palette_image: Image = source_colors.vjoin(target_colors)?;
        Ok(palette_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_three_colors_forward() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            1, 1, 2, 2, 3,
            1, 1, 2, 2, 3,
            1, 1, 1, 2, 3,
        ];
        let input0: Image = Image::try_create(5, 3, input0_pixels).expect("image");

        let input1_pixels: Vec<u8> = vec![
            5, 5, 5, 6, 6, 7,
        ];
        let input1: Image = Image::try_create(6, 1, input1_pixels).expect("image");

        // Act
        let actual: Image = input0.palette_using_histogram(&input1, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3,  // top row is the unique colors in input0, sorted by popularity
            5, 6, 7,  // bottom row is the unique colors in input1, sorted by popularity
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_three_colors_reversed() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            1, 1, 2, 2, 3,
            1, 1, 2, 2, 3,
            1, 1, 1, 2, 3,
        ];
        let input0: Image = Image::try_create(5, 3, input0_pixels).expect("image");

        let input1_pixels: Vec<u8> = vec![
            5, 5, 5, 6, 6, 7,
        ];
        let input1: Image = Image::try_create(6, 1, input1_pixels).expect("image");

        // Act
        let actual: Image = input0.palette_using_histogram(&input1, true).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, // top row is the unique colors in input0, sorted by popularity
            7, 6, 5, // bottom row is the unique colors in input1, sorted by popularity, reversed
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_two_colors_forward() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 1, 9, 1, 1,
            1, 9, 9, 9, 1,
            1, 1, 9, 1, 1,
            1, 1, 1, 1, 1,
        ];
        let input0: Image = Image::try_create(5, 5, input0_pixels).expect("image");

        let input1_pixels: Vec<u8> = vec![
            2, 2, 3,
        ];
        let input1: Image = Image::try_create(3, 1, input1_pixels).expect("image");

        // Act
        let actual: Image = input0.palette_using_histogram(&input1, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 9, // top row is the unique colors in input0, sorted by popularity
            2, 3, // bottom row is the unique colors in input1, sorted by popularity
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_two_colors_reversed() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 1, 9, 1, 1,
            1, 9, 9, 9, 1,
            1, 1, 9, 1, 1,
            1, 1, 1, 1, 1,
        ];
        let input0: Image = Image::try_create(5, 5, input0_pixels).expect("image");

        let input1_pixels: Vec<u8> = vec![
            2, 2, 3,
        ];
        let input1: Image = Image::try_create(3, 1, input1_pixels).expect("image");

        // Act
        let actual: Image = input0.palette_using_histogram(&input1, true).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 9, // top row is the unique colors in input0, sorted by popularity
            3, 2, // bottom row is the unique colors in input1, sorted by popularity, reversed
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_error_mismatch_in_number_of_unique_colors() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 1, 9, 1, 1,
            1, 9, 9, 9, 1,
            1, 1, 9, 1, 1,
            1, 1, 1, 1, 1,
        ];
        let input0: Image = Image::try_create(5, 5, input0_pixels).expect("image");
        // input0 has 2 unique colors

        let input1_pixels: Vec<u8> = vec![
            2, 2, 3, 4,
        ];
        let input1: Image = Image::try_create(4, 1, input1_pixels).expect("image");
        // input1 has 3 unique colors

        // Act
        input0.palette_using_histogram(&input1, true).expect_err("mismatch in number of unique colors");
    }

    #[test]
    fn test_20000_color_symmetry_same_to_same() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            3, 3,
            3, 3,
            3, 3,
        ];
        let input0: Image = Image::try_create(2, 3, input0_pixels).expect("image");

        let input1_pixels: Vec<u8> = vec![
            4, 4, 4, 4,
            4, 4, 4, 4,
        ];
        let input1: Image = Image::try_create(4, 2, input1_pixels).expect("image");

        // Act
        let actual: Image = input0.palette_using_color_symmetry(&input1, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, // top row is source color
            4, // bottom row is the target color
        ];
        let expected: Image = Image::try_create(1, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_color_symmetry_same_to_error() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            3, 3,
            3, 3,
            3, 3,
        ];
        let input0: Image = Image::try_create(2, 3, input0_pixels).expect("image");

        let input1_pixels: Vec<u8> = vec![
            4, 4, 4, 4,
            4, 4, 4, 5,
        ];
        let input1: Image = Image::try_create(4, 2, input1_pixels).expect("image");

        // Act
        input0.palette_using_color_symmetry(&input1, false).expect_err("should fail");
    }

    #[test]
    fn test_20002_color_symmetry_rows_to_rows() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            3, 1, 2,
            3, 1, 2,
            3, 1, 2,
            3, 1, 2,
        ];
        let input0: Image = Image::try_create(3, 4, input0_pixels).expect("image");

        let input1_pixels: Vec<u8> = vec![
            4, 5, 6,
            4, 5, 6,
            4, 5, 6,
        ];
        let input1: Image = Image::try_create(3, 3, input1_pixels).expect("image");

        // Act
        let actual: Image = input0.palette_using_color_symmetry(&input1, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 1, 2, // top row is a copy of a row in input0
            4, 5, 6, // bottom row a copy of a row in input1
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20003_color_symmetry_columns_to_columns() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            3, 3, 3, 3,
            1, 1, 1, 1,
            2, 2, 2, 2,
        ];
        let input0: Image = Image::try_create(4, 3, input0_pixels).expect("image");

        let input1_pixels: Vec<u8> = vec![
            4, 4,
            5, 5,
            6, 6,
        ];
        let input1: Image = Image::try_create(2, 3, input1_pixels).expect("image");

        // Act
        let actual: Image = input0.palette_using_color_symmetry(&input1, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 1, 2, // top row is a copy of a row in input0
            4, 5, 6, // bottom row a copy of a row in input1
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20004_color_symmetry_row_to_columns() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            3, 3, 3, 3,
            1, 1, 1, 1,
            2, 2, 2, 2,
        ];
        let input0: Image = Image::try_create(4, 3, input0_pixels).expect("image");

        let input1_pixels: Vec<u8> = vec![
            4, 5, 6,
            4, 5, 6,
        ];
        let input1: Image = Image::try_create(3, 2, input1_pixels).expect("image");

        // Act
        let actual: Image = input0.palette_using_color_symmetry(&input1, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 1, 2, // top row is a copy of a row in input0
            4, 5, 6, // bottom row a copy of a row in input1
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20005_color_symmetry_columns_to_rows() {
        // Arrange
        let input0_pixels: Vec<u8> = vec![
            3, 1, 2,
            3, 1, 2,
            3, 1, 2,
            3, 1, 2,
        ];
        let input0: Image = Image::try_create(3, 4, input0_pixels).expect("image");

        let input1_pixels: Vec<u8> = vec![
            4, 4,
            5, 5,
            6, 6,
        ];
        let input1: Image = Image::try_create(2, 3, input1_pixels).expect("image");

        // Act
        let actual: Image = input0.palette_using_color_symmetry(&input1, false).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 1, 2, // top row is a copy of a row in input0
            4, 5, 6, // bottom row a copy of a row in input1
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
