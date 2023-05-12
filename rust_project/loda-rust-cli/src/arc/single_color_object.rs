use super::{Histogram, Image, ImageHistogram, ImageMask, Rectangle, ImageMix, ImageSize, MixMode, ImageMaskCount, ImageCrop};

/// A rectangle filled with a single solid color and no other colors are present inside the object.
#[derive(Clone, Debug)]
pub struct SingleColorObjectRectangle {
    pub color: u8,
    pub mask: Image,
    pub bounding_box: Rectangle,
    pub mass: u16,
    pub is_square: bool,
}

/// A mask of pixels that have the same color, but isn't fully connected.
/// 
/// It may be separate objects.
/// 
/// It may be a diagonal line that has one color, but the pixels that aren't on the diagonal
/// having a different color.
/// 
/// The rectangle that contains the object also contains 1 or more pixels of different colors.
#[derive(Clone, Debug)]
pub struct SingleColorObjectSparse {
    pub color: u8,
    pub mask: Image,

    /// Bounding box of the mask
    pub bounding_box: Rectangle,

    /// Number of pixels with same value as `color`
    pub mass_object: u16,

    /// Number of pixels different than `color`
    pub mass_non_object: u16,

    /// Histogram of the non-object pixels 
    pub histogram_non_object: Histogram,

    // Future experiments:
    // vector with clusters, number of clusters, enumerated clusters
    // number of holes in each cluster
    // shape type: L shape, T shape, + shape, diagonal shape, other shape
    // symmetry
    // is a box
    // outermost pixels have same color
    // number of holes
    // are the non-object pixels a single color
    // child objects
    // surrounding objects
    // If there is only a single color that isn't ObjectWithOneColor
    // then it may be because it's the background color.
    // compare the background color across all the single objects if it's the same.
    // 
    // Segment the mask into objects.
    // Identify each object.
    //
    // Detect objects with multiple colors
}

impl SingleColorObjectSparse {
    fn create(color: u8, image: &Image, mask: Image, rect: Rectangle) -> anyhow::Result<Self> {
        let cropped_object: Image = image.crop(rect)?;
        let mut histogram: Histogram = cropped_object.histogram_all();
        let mass_object: u16 = histogram.get(color).min(u16::MAX as u32) as u16;
        histogram.set_counter_to_zero(color);
        let mass_non_object: u16 = histogram.sum().min(u16::MAX as u32) as u16;
        let instance = SingleColorObjectSparse {
            color,
            mask,
            bounding_box: rect,
            mass_object,
            mass_non_object,
            histogram_non_object: histogram,
        };
        Ok(instance)
    }
}

#[derive(Clone, Debug)]
pub struct SingleColorObjects {
    pub image_size: ImageSize,
    pub rectangle_vec: Vec<SingleColorObjectRectangle>,
    pub sparse_vec: Vec<SingleColorObjectSparse>,
}

impl SingleColorObjects {
    pub fn find_objects(image: &Image) -> anyhow::Result<Self> {
        if image.is_empty() {
            return Err(anyhow::anyhow!("The image must be 1x1 or bigger"));
        }
        let image_histogram: Histogram = image.histogram_all();
        let mut rectangle_vec = Vec::<SingleColorObjectRectangle>::new();
        let mut sparse_vec = Vec::<SingleColorObjectSparse>::new();
        for (count, color) in image_histogram.pairs_ordered_by_color() {
            let mask: Image = image.to_mask_where_color_is(color);
            let rect: Rectangle = match mask.bounding_box() {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let mass: u16 = (rect.width() as u16) * (rect.height() as u16);
            if count != (mass as u32) {
                let item: SingleColorObjectSparse = SingleColorObjectSparse::create(color, image, mask, rect)?;
                sparse_vec.push(item);
                continue;
            }

            let is_square: bool = rect.width() == rect.height();
            let item = SingleColorObjectRectangle {
                color,
                mask,
                bounding_box: rect,
                mass,
                is_square,
            };
            rectangle_vec.push(item);
        }
        let instance = Self {
            image_size: image.size(),
            rectangle_vec,
            sparse_vec,
        };
        instance.verify_all_pixels_are_accounted_for()?;
        Ok(instance)
    }

    /// Verify that the every pixel in the image are fully explained by the 
    /// `rectangle_vec` and the `sparse_vec`. If one or more pixels isn't accounted for,
    /// then something must have gone wrong while analyzing the pixels.
    fn verify_all_pixels_are_accounted_for(&self) -> anyhow::Result<()> {
        let mut result_mask = Image::zero(self.image_size.width, self.image_size.height);
        for object in &self.rectangle_vec {
            result_mask = result_mask.mix(&object.mask, MixMode::Plus)?;
        }
        for object in &self.sparse_vec {
            result_mask = result_mask.mix(&object.mask, MixMode::Plus)?;
        }
        let actual_mass: u16 = result_mask.mask_count_one();
        let expected_mass: u16 = (self.image_size.width as u16) * (self.image_size.height as u16);
        if actual_mass != expected_mass {
            return Err(anyhow::anyhow!("The objects doesn't cover the image correctly. Each pixel is supposed to be counted once, but was either not counted at all, or counted multiple times. Cannot explain all the pixels in the image."));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_object_rectangle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");

        // Assert
        assert_eq!(actual.rectangle_vec.len(), 6);
        assert_eq!(actual.sparse_vec.len(), 0);
    }

    #[test]
    fn test_20000_object_sparse() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 6,
            6, 3, 3,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");

        // Assert
        assert_eq!(actual.rectangle_vec.len(), 2);
        assert_eq!(actual.sparse_vec.len(), 1);

        let object: &SingleColorObjectSparse = actual.sparse_vec.first().expect("1 instance");
        assert_eq!(object.bounding_box, Rectangle::new(0, 0, 3, 2));

        let expected_pixels: Vec<u8> = vec![
            0, 0, 1,
            1, 0, 0,
        ];
        let expected_mask: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(object.mask, expected_mask);

        assert_eq!(object.mass_object, 2);
        assert_eq!(object.mass_non_object, 4);

        {
            let mut histogram = Histogram::new();
            histogram.increment(5);
            histogram.increment(5);
            histogram.increment(3);
            histogram.increment(3);
            assert_eq!(object.histogram_non_object, histogram);
        }
    }
}
