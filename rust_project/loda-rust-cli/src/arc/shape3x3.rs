//! Shape3x3 is a catalog of all 48 possible 3x3 shapes, in all their transformations (rotate, flip).
//! 
//! Beware that this catalog contains shapes smaller than 3x3. 
//! These have the sizes: 3x2, 2x3, 2x2, 1x3, 3x1, 1x2, 2x1, 1x1.
use super::{Image, ImageMask, ImageTrim, ShapeTransformation, ImageSize, ImageTryCreate, HtmlLog};
use std::collections::{HashSet, HashMap};
use lazy_static::lazy_static;

lazy_static! {
    static ref SHAPE3X3: Shape3x3 = Shape3x3::populate().expect("populate");
}

#[allow(dead_code)]
pub struct Shape3x3 {
    shape_set: HashSet<Image>,
    shape_vec: Vec<Image>,
    image_to_shapeid: HashMap<Image, u8>,
    index_to_transformation: HashMap<u8, HashSet<ShapeTransformation>>,
    index_to_shapeid: HashMap::<u8, u8>,
}

impl Shape3x3 {
    pub fn instance() -> &'static Self {
        &SHAPE3X3
    }

    pub fn number_of_shapes(&self) -> u8 {
        (self.shape_vec.len() & 255) as u8
    }
    
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
            let (transformations, output) = Shape3x3::analyze(&image).expect("image");
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

    /// Log all shapes to the html log, so it's possible to inspect all the 48 shapes.
    #[allow(dead_code)]
    pub fn htmllog_all_shapes(&self) {
        HtmlLog::text(format!("number of shapes: {}", self.shape_vec.len()));
        for (shape_id, image) in self.shape_vec.iter().enumerate() {
            HtmlLog::text(format!("shape_id: {}", shape_id));
            HtmlLog::image(&image);
        }
    }

    /// Create an 8bit representation from a 3x3 image.
    /// 
    /// If the pixels is the same as the center pixel, then the bit is 1.
    /// 
    /// If the pixels is the different than the center pixel, then the bit is 0.
    #[allow(dead_code)]
    pub fn id_from_3x3image(image: &Image) -> anyhow::Result<u8> {
        if image.width() != 3 || image.height() != 3 {
            return Err(anyhow::anyhow!("image size is not 3x3"));
        }
        let center: u8 = image.get(1, 1).unwrap_or(255);
        let mut current_bit: u16 = 1;
        let mut accumulated: u16 = 0;
        for y in 0..=2u8 {
            for x in 0..=2u8 {
                if x == 1 && y == 1 {
                    // Skip the center pixel. We are only interested in traversing the 8 pixels surrounding the center pixel.
                    continue;
                }
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                let is_same: bool = color == center;
                accumulated |= current_bit * is_same as u16;
                current_bit *= 2;
            }
        }
        Ok((accumulated & 255) as u8)
    }

    /// Find the shapeid and transformations for a 3x3 image.
    /// 
    /// Returns a tuple with the shapeid and the transformations.
    /// The shapeid is a value in the range `0..=47`.
    /// The set contains at least 1 transformation. And max 8 transformations `ShapeTransformation::all()`.
    #[allow(dead_code)]
    pub fn shapeid_and_transformations(&self, image: &Image) -> anyhow::Result<(u8, HashSet<ShapeTransformation>)> {
        if image.width() != 3 || image.height() != 3 {
            return Err(anyhow::anyhow!("image size is not 3x3"));
        }
        let image_id: u8 = Self::id_from_3x3image(image)?;
        let shape_id: u8 = match self.index_to_shapeid.get(&image_id) {
            Some(value) => *value,
            None => return Err(anyhow::anyhow!("shape not found")),
        };
        let transformations: HashSet<ShapeTransformation> = match self.index_to_transformation.get(&image_id) {
            Some(value) => value.clone(),
            None => return Err(anyhow::anyhow!("transformations not found")),
        };
        Ok((shape_id, transformations))
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
        let (transformations, output) = Shape3x3::analyze(&input).expect("image");

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
        let instance: Shape3x3 = Shape3x3::populate().expect("populate");
        // instance.htmllog_all_shapes();

        // Assert
        assert_eq!(instance.shape_set.len(), 48);
        assert_eq!(instance.shape_vec.len(), 48);
        assert_eq!(instance.image_to_shapeid.len(), 48);
        assert_eq!(instance.index_to_transformation.len(), 256);
        assert_eq!(instance.index_to_shapeid.len(), 256);
    }

    #[test]
    fn test_30000_id_from_3x3image() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7,
            7, 5, 7,
            7, 7, 7,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: u8 = Shape3x3::id_from_3x3image(&input).expect("id");

        // Assert
        assert_eq!(actual, 0);
    }

    #[test]
    fn test_30001_id_from_3x3image() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 7, 7,
            7, 5, 7,
            7, 7, 7,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: u8 = Shape3x3::id_from_3x3image(&input).expect("id");

        // Assert
        assert_eq!(actual, 1);
    }

    #[test]
    fn test_30002_id_from_3x3image() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 5,
            7, 5, 7,
            7, 7, 7,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: u8 = Shape3x3::id_from_3x3image(&input).expect("id");

        // Assert
        assert_eq!(actual, 4);
    }

    #[test]
    fn test_30003_id_from_3x3image() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7,
            7, 5, 7,
            5, 7, 7,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: u8 = Shape3x3::id_from_3x3image(&input).expect("id");

        // Assert
        assert_eq!(actual, 32);
    }

    #[test]
    fn test_30004_id_from_3x3image() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7,
            7, 5, 7,
            7, 7, 5,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: u8 = Shape3x3::id_from_3x3image(&input).expect("id");

        // Assert
        assert_eq!(actual, 128);
    }

    #[test]
    fn test_30005_id_from_3x3image() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5,
            5, 5, 8,
            5, 8, 8,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: u8 = Shape3x3::id_from_3x3image(&input).expect("id");

        // Assert
        assert_eq!(actual, 47);
    }

    #[test]
    fn test_30006_id_from_3x3image() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5,
            5, 5, 5,
            5, 5, 5,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: u8 = Shape3x3::id_from_3x3image(&input).expect("id");

        // Assert
        assert_eq!(actual, 255);
    }

    #[test]
    fn test_40000_shapeid_and_transformations_first() {
        // Arrange
        let pixels: Vec<u8> = vec![
            8, 8, 8,
            8, 3, 8,
            8, 8, 8,
        ];
        // The center pixel is surrounded with pixels of different color. This is the first shape id included in the catalog of shapes.
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        let find_shape: Shape3x3 = Shape3x3::populate().expect("ok");

        // Act
        let (shapeid, transformations) = find_shape.shapeid_and_transformations(&input).expect("ok");

        // Assert
        assert_eq!(shapeid, 0);
        assert_eq!(transformations, ShapeTransformation::all());
    }

    #[test]
    fn test_40001_shapeid_and_transformations_last() {
        // Arrange
        let pixels: Vec<u8> = vec![
            8, 8, 8,
            8, 8, 8,
            8, 8, 8,
        ];
        // The center pixel is surrounded with pixels of same color. This is the last shape id included in the catalog of shapes.
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        let find_shape: Shape3x3 = Shape3x3::populate().expect("ok");

        // Act
        let (shapeid, transformations) = find_shape.shapeid_and_transformations(&input).expect("ok");

        // Assert
        assert_eq!(shapeid, 47);
        assert_eq!(transformations, ShapeTransformation::all());
    }
}
