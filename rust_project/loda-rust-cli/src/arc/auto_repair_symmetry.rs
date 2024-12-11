use super::{Image, ImageHistogram, Histogram, Color, Symmetry, ImageMask, ImageRepairSymmetry};

pub struct AutoRepairSymmetry {}

impl AutoRepairSymmetry {
    pub fn execute(symmetry: &Symmetry, repair_mask: &Image, image_to_repair: &Image) -> anyhow::Result<Image> {
        if repair_mask.size() != image_to_repair.size() {
            return Err(anyhow::anyhow!("size must be the same"));
        }

        // Sometimes it's not possible to compute the entire output just by looking at the input pixels alone.
        // Fill the repair mask with `Color::CannotCompute`, so that it's clear there was a problem 
        // computing pixel data for these pixels.
        // This happens when the symmetric shape has an inset, and there is masked out an area
        // bigger than what is possible to recover just by looking at the input pixels alone.
        let mut result_image: Image = repair_mask.select_from_image_and_color(image_to_repair, Color::CannotCompute as u8)?;

        // horizontal
        if let Some(r) = symmetry.horizontal_rect {
            result_image.repair_symmetry_horizontal(r)?;
        }

        // vertical
        if let Some(r) = symmetry.vertical_rect {
            result_image.repair_symmetry_vertical(r)?;
        }
        
        // diagonal a
        if let Some(r) = symmetry.diagonal_a_rect {
            result_image.repair_symmetry_diagonal_a(r)?;
        }

        // diagonal b
        if let Some(r) = symmetry.diagonal_b_rect {
            result_image.repair_symmetry_diagonal_b(r)?;
        }

        let histogram: Histogram = result_image.histogram_all();
        if histogram.number_of_counters_greater_than_zero() < 2 {
            return Err(anyhow::anyhow!("Expected the repaired symmetric pattern to contain 2 or more unique colors"));
        }

        // Reject if more than 25% of the pixels could not be computed
        let problem_count: u32 = histogram.counters()[Color::CannotCompute as usize];
        if problem_count > (image_to_repair.width() as u32) * (image_to_repair.height() as u32) / 4 {
            return Err(anyhow::anyhow!("Too many pixels could not be computed. This may not be a symmetric image"));
        }

        // Most of the repaired images are junk that isn't symmetric.
        let sym = Symmetry::analyze(&result_image)?;
        let sym_horizontal: bool = sym.horizontal_found && sym.horizontal_mismatches == 0;
        let sym_vertical: bool = sym.vertical_found && sym.vertical_mismatches == 0;
        let sym_diagonal_a: bool = sym.diagonal_a_found && sym.diagonal_a_mismatches == 0;
        let sym_diagonal_b: bool = sym.diagonal_b_found && sym.diagonal_b_mismatches == 0;
        let is_symmetric: bool = sym_horizontal || sym_vertical || sym_diagonal_a || sym_diagonal_b;
        if !is_symmetric {
            return Err(anyhow::anyhow!("Unable to repair image. No symmetry after repair."));
        }
        
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_horizontal() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            1, 1, 2, 1, 1,
            2, a, 0, 1, 2,
            3, a, 3, 3, 3,
            4, 0, 0, a, a,
            1, 1, 0, a, a
        ];
        let image_to_repair: Image = Image::try_create(5, 5, pixels).expect("image");

        let symmetry: Symmetry = Symmetry::analyze(&image_to_repair).expect("image");
        
        let repair_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 0, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 0, 1, 1,
            0, 0, 0, 1, 1,
        ];
        let repair_mask: Image = Image::try_create(5, 5, repair_pixels).expect("image");

        // Act
        let actual: Image = AutoRepairSymmetry::execute(&symmetry, &repair_mask, &image_to_repair).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 2, 1, 1,
            2, 1, 0, 1, 2,
            3, 3, 3, 3, 3,
            4, 0, 0, 0, 4,
            1, 1, 0, 1, 1
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_vertical() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            1, 1, 5, 7, 3,
            2, 0, 5, 7, a,
            3, 1, 5, 0, 3,
            2, 0, a, a, 3,
            1, 1, a, a, 3,
        ];
        let image_to_repair: Image = Image::try_create(5, 5, pixels).expect("image");

        let symmetry: Symmetry = Symmetry::analyze(&image_to_repair).expect("image");
        
        let repair_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 0, 1, 1, 0,
            0, 0, 1, 1, 0,
        ];
        let repair_mask: Image = Image::try_create(5, 5, repair_pixels).expect("image");

        // Act
        let actual: Image = AutoRepairSymmetry::execute(&symmetry, &repair_mask, &image_to_repair).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 5, 7, 3,
            2, 0, 5, 7, 3,
            3, 1, 5, 0, 3,
            2, 0, 5, 7, 3,
            1, 1, 5, 7, 3,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_diagonal_a() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            1, a, a, 0, 0,
            1, 0, a, a, a,
            1, 1, 1, 0, a,
            0, 0, 0, 5, 5,
            0, 0, 0, 5, 5,
        ];
        let image_to_repair: Image = Image::try_create(5, 5, pixels).expect("image");

        let symmetry: Symmetry = Symmetry::analyze(&image_to_repair).expect("image");
        
        let repair_pixels: Vec<u8> = vec![
            0, 1, 1, 0, 0,
            0, 0, 1, 1, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let repair_mask: Image = Image::try_create(5, 5, repair_pixels).expect("image");

        // Act
        let actual: Image = AutoRepairSymmetry::execute(&symmetry, &repair_mask, &image_to_repair).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 0, 0,
            1, 0, 1, 0, 0,
            1, 1, 1, 0, 0,
            0, 0, 0, 5, 5,
            0, 0, 0, 5, 5,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_diagonal_b() {
        // Arrange
        let a = Color::CannotCompute as u8;
        let pixels: Vec<u8> = vec![
            0, 0, a, a, 1,
            a, a, a, 0, 1,
            a, 0, 1, 1, 1,
            5, 5, 0, 0, 0,
            5, 5, 0, 0, 0,
        ];
        let image_to_repair: Image = Image::try_create(5, 5, pixels).expect("image");

        let symmetry: Symmetry = Symmetry::analyze(&image_to_repair).expect("image");
        
        let repair_pixels: Vec<u8> = vec![
            0, 0, 1, 1, 0,
            1, 1, 1, 0, 0,
            1, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let repair_mask: Image = Image::try_create(5, 5, repair_pixels).expect("image");

        // Act
        let actual: Image = AutoRepairSymmetry::execute(&symmetry, &repair_mask, &image_to_repair).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1,
            0, 0, 1, 0, 1,
            0, 0, 1, 1, 1,
            5, 5, 0, 0, 0,
            5, 5, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_50000_nosymmetry() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5, 6,
            1, 2, 3, 4, 5, 6,
            8, 8, 8, 8, 8, 8,
            8, 1, 8, 1, 8, 1,
            0, 0, 1, 1, 2, 2,
            0, 0, 1, 1, 2, 2,
        ];
        let image_to_repair: Image = Image::try_create(6, 6, pixels).expect("image");

        let symmetry: Symmetry = Symmetry::analyze(&image_to_repair).expect("image");
        
        let repair_pixels: Vec<u8> = vec![
            0, 0, 1, 1, 0, 0,
            1, 1, 1, 0, 0, 0,
            1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let repair_mask: Image = Image::try_create(6, 6, repair_pixels).expect("image");

        // Act
        let error = AutoRepairSymmetry::execute(&symmetry, &repair_mask, &image_to_repair).expect_err("should fail");

        // Assert
        let message: String = format!("{:?}", error);
        assert_eq!(message.contains("No symmetry after repair"), true);
    }
}
