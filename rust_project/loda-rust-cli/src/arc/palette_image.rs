use super::{Image, ImageHistogram, ImageGetRowColumn, ImageStack, ImageSymmetry};

pub struct PaletteImage;

impl PaletteImage {
    /// This is intended for scenarios where two image have the exact same number of unique colors.
    /// 
    /// If the number of unique colors are different, then an error is returned.
    /// 
    /// If the 2 or more colors have the same count, then their ordering is random.
    pub fn palette_image(image0: &Image, image1: &Image, reverse: bool) -> anyhow::Result<Image> {
        let histogram0 = image0.histogram_all();
        let histogram1 = image1.histogram_all();
        let count0: u32 = histogram0.number_of_counters_greater_than_zero();
        let count1: u32 = histogram1.number_of_counters_greater_than_zero();
        if count0 != count1 {
            return Err(anyhow::anyhow!("both images must have the same number of colors, cannot construct mapping. image0 has {} colors. image1 has {} colors.", count0, count1));
        }

        let histogram_image0: Image = histogram0.to_image()?;
        let histogram_image1: Image = histogram1.to_image()?;
        // The colors are stored in the bottom rows of the histogram image.

        // Extract the color rows
        let row_with_colors0: Image = histogram_image0.bottom_rows(1)?;
        let mut row_with_colors1: Image = histogram_image1.bottom_rows(1)?;

        if reverse {
            row_with_colors1 = row_with_colors1.flip_x()?;
        }

        // Top row is the source colors 
        // Bottom row is the destination colors 
        let palette_image: Image = row_with_colors0.vjoin(row_with_colors1)?;
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
        let actual: Image = PaletteImage::palette_image(&input0, &input1, false).expect("image");

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
        let actual: Image = PaletteImage::palette_image(&input0, &input1, true).expect("image");

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
        let actual: Image = PaletteImage::palette_image(&input0, &input1, false).expect("image");

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
        let actual: Image = PaletteImage::palette_image(&input0, &input1, true).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 9, // top row is the unique colors in input0, sorted by popularity
            3, 2, // bottom row is the unique colors in input1, sorted by popularity, reversed
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_error_mismatch_in_number_of_unique_colors() {
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
        PaletteImage::palette_image(&input0, &input1, true).expect_err("mismatch in number of unique colors");
    }
}
