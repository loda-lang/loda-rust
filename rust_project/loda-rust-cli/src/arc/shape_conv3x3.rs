use super::{Image, ImageMask, ImageTrim, ShapeTransformation, ImageSize};
use std::collections::HashSet;

#[allow(dead_code)]
struct FindShape;

impl FindShape {
    #[allow(dead_code)]
    fn analyze(image: &Image) -> anyhow::Result<(HashSet<ShapeTransformation>, Image)> {
        if image.width() != 3 || image.height() != 3 {
            return Err(anyhow::anyhow!("image size is not 3x3"));
        }
        let center: u8 = image.get(1, 1).unwrap_or(255);
        let mask0: Image = image.to_mask_where_color_is(center);
        let mask1: Image = mask0.trim_color(0)?;

        let size: ImageSize = mask1.size();
        let transformations: Vec<(ShapeTransformation, Image)> = ShapeTransformation::perform_all_transformations(&mask1)?;
        let (transformation_set, normalized_image) = ShapeTransformation::normalize_advanced(size, transformations)?;

        Ok((transformation_set, normalized_image))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_classify() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5,
            5, 5, 8,
            5, 8, 8,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let (transformations, output) = FindShape::analyze(&input).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0,
            1, 1, 0,
            1, 1, 1,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(output, expected);
        assert_eq!(transformations, HashSet::<ShapeTransformation>::from([ShapeTransformation::RotateCw270, ShapeTransformation::FlipXRotateCw180]));
    }
}
