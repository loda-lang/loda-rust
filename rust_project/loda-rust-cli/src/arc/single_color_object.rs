use super::{Histogram, Image, ImageHistogram, ImageMask, Rectangle};

/// A rectangle filled with a single solid color and no other colors are present inside the object.
#[derive(Clone, Debug)]
pub struct SingleColorObject {
    pub color: u8,
    pub mask: Image,
    pub bounding_box: Rectangle,
    pub mass: u16,
    pub is_square: bool,
}

#[derive(Clone, Debug)]
pub struct SingleColorObjects {
    pub single_color_object_vec: Vec<SingleColorObject>,
}

impl SingleColorObjects {
    pub fn find_objects(image: &Image) -> anyhow::Result<Self> {
        if image.is_empty() {
            return Err(anyhow::anyhow!("The image must be 1x1 or bigger"));
        }
        let histogram: Histogram = image.histogram_all();
        let mut items = Vec::<SingleColorObject>::new();
        for (count, color) in histogram.pairs_ordered_by_color() {
            let mask: Image = image.to_mask_where_color_is(color);
            let rect: Rectangle = match mask.bounding_box() {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let mass: u16 = (rect.width() as u16) * (rect.height() as u16);
            if count != (mass as u32) {
                // Future experiments:
                // If there is only a single color that isn't ObjectWithOneColor
                // then it may be because it's the background color.
                // compare the background color across all the single objects if it's the same.
                // 
                // Is the input image fully explained by the ObjectWithOneColor's and a background color.
                //
                // Segment the mask into objects.
                // Identify each object.
                //
                // Detect objects with multiple colors
                continue;
            }

            let is_square: bool = rect.width() == rect.height();
            let item = SingleColorObject {
                color,
                mask,
                bounding_box: rect,
                mass,
                is_square,
            };
            items.push(item);
        }
        if items.is_empty() {
            return Err(anyhow::anyhow!("Unable to find any objects of single color"));
        }
        let instance = Self { single_color_object_vec: items };
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_find_objects() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");

        // Assert
        assert_eq!(actual.single_color_object_vec.len(), 6);
    }

    #[test]
    fn test_10001_find_objects() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 1,
            1, 3, 3,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");

        // Assert
        assert_eq!(actual.single_color_object_vec.len(), 2);
    }
}
