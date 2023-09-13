use super::{Image, ImageMask, ImageTrim, ShapeTransformation, ImageSize, ImageTryCreate};
use std::collections::{HashSet, HashMap};

#[allow(dead_code)]
struct FindShape {
    shape_set: HashSet<Image>,
    shape_vec: Vec<Image>,
    image_to_shapeid: HashMap<Image, u8>,
    index_to_transformation: HashMap<u8, HashSet<ShapeTransformation>>,
    index_to_shapeid: HashMap::<u8, u8>,
}

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

    #[allow(dead_code)]
    fn populate() -> anyhow::Result<Self> {
        let mut shape_set = HashSet::<Image>::new();
        let mut shape_vec = Vec::<Image>::new();
        let mut image_to_shapeid = HashMap::<Image, u8>::new();
        let mut index_to_transformation = HashMap::<u8, HashSet<ShapeTransformation>>::new();
        let mut index_to_shapeid = HashMap::<u8, u8>::new();

        // Traverse all possible 3x3 images, where the center pixel is 1.
        // The center pixel is surrounded by 8 pixels. So there are 2^8 = 256 possible combinations.
        for i in 0..=255u8 {
            let values: [u8; 9] = [
                i & 1, (i >> 1) & 1, (i >> 2) & 1,
                (i >> 3) & 1, 1, (i >> 4) & 1,
                (i >> 5) & 1, (i >> 6) & 1, (i >> 7) & 1,
            ];
            let image: Image = Image::try_create(3, 3, values.to_vec())?;
            let (transformations, output) = FindShape::analyze(&image).expect("image");
            if !shape_set.contains(&output) {
                let shapeid: u8 = (shape_set.len() & 255) as u8;
                image_to_shapeid.insert(output.clone(), shapeid);
                shape_set.insert(output.clone());
                shape_vec.push(output.clone());
            }
            if let Some(shapeid) = image_to_shapeid.get(&output) {
                index_to_shapeid.insert(i, *shapeid);
            }
            index_to_transformation.insert(i, transformations);
        }
        // println!("shape_set: {}", shape_set.len());
        // println!("shape_vec: {}", shape_vec.len());
        // println!("image_to_shapeid: {}", image_to_shapeid.len());
        // println!("index_to_transformation: {}", index_to_transformation.len());
        // println!("index_to_shapeid: {}", index_to_shapeid.len());
        let instance = Self {
            shape_set,
            shape_vec,
            image_to_shapeid,
            index_to_transformation,
            index_to_shapeid,
        };
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_analyze() {
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

    #[test]
    fn test_20000_populate() {
        // Arrange
        
        // Act
        let instance: FindShape = FindShape::populate().expect("populate");

        // Assert
        assert_eq!(instance.shape_set.len(), 48);
        assert_eq!(instance.shape_vec.len(), 48);
        assert_eq!(instance.image_to_shapeid.len(), 48);
        assert_eq!(instance.index_to_transformation.len(), 256);
        assert_eq!(instance.index_to_shapeid.len(), 256);
    }
}
