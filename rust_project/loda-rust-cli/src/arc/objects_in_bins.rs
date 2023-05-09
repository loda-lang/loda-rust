use super::{Histogram, Image, ImageMask, ImageMaskCount, ImageSize};

#[allow(dead_code)]
pub struct ObjectsInBins {
    image_size: ImageSize,
    items: Vec<Item>,
}

impl ObjectsInBins {
    /// The `enumerated_objects` must be 1x1 or bigger.
    /// 
    /// Measures the mass of each object.
    /// 
    /// An error is returned if there are zero objects.
    #[allow(dead_code)]
    pub fn analyze(enumerated_objects: &Image, ignore_colors: Option<&Histogram>) -> anyhow::Result<Self> {
        if enumerated_objects.is_empty() {
            return Err(anyhow::anyhow!("ObjectsInBins.analyze: image must be 1x1 or bigger"));
        }
        let mut items = Vec::<Item>::new();
        for color in 0..=255u8 {
            if let Some(other) = ignore_colors {
                if other.get(color) > 0 {
                    continue;
                }
            }
            let mask: Image = enumerated_objects.to_mask_where_color_is(color);
            let mass_of_object: u16 = mask.mask_count_one();
            if mass_of_object == 0 {
                continue;
            }
            let item = Item {
                object_id: color,
                mask,
                object_mass: mass_of_object,
            };
            items.push(item);
        }
        if items.is_empty() {
            return Err(anyhow::anyhow!("ObjectsInBins.analyze: found zero objects. There must be 1 or more objects"));
        }
        let instance = Self {
            image_size: enumerated_objects.size(),
            items
        };
        Ok(instance)
    }

    /// Assign new N object ids to the existing objects.
    /// 
    /// Usecase: Group the objects into 3 bins based on mass.
    /// - The smallest objects with same mass gets assigned `id=1`.
    /// - The in between objects gets assigned `id=2`.
    /// - The biggest objects with same mass gets assigned `id=3`.
    /// 
    /// Usecase: Group the objects into 2 bins based on mass.
    /// - The objects with exactly `mass=6` gets assigned `id=1`.
    /// - All other objects gets assigned `id=2`.
    /// 
    /// Returns an image with the same size as the input image.
    /// The pixel value is for non-objects.
    #[allow(dead_code)]
    pub fn group3_small_medium_big(&self, reverse: bool) -> anyhow::Result<Image> {
        let mut smallest_mass: u16 = u16::MAX;
        let mut biggest_mass: u16 = 0;
        for item in &self.items {
            smallest_mass = smallest_mass.min(item.object_mass);
            biggest_mass = biggest_mass.max(item.object_mass);
        }
        if smallest_mass == biggest_mass {
            return Err(anyhow::anyhow!("ObjectsInBins.group3_small_medium_big: it's ambiguous in what bin to place the objects. biggest and smallest is the same mass"));
        }

        let color_biggest: u8;
        let color_smallest: u8;
        if reverse {
            color_biggest = 1;
            color_smallest = 3;
        } else {
            color_biggest = 3;
            color_smallest = 1;
        }

        let mut result_image = Image::zero(self.image_size.width, self.image_size.height);
        for item in &self.items {
            let mut set_color: u8 = 2;
            if item.object_mass == biggest_mass {
                set_color = color_biggest;
            }
            if item.object_mass == smallest_mass {
                set_color = color_smallest;
            }
            result_image = item.mask.select_from_image_and_color(&result_image, set_color)?;
        }
        Ok(result_image)
    }
}

struct Item {
    object_id: u8,
    mask: Image,
    object_mass: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_group3_biggest_medium_smallest() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            1, 0, 2, 0, 3,
            0, 0, 0, 0, 0,
            4, 4, 0, 5, 5,
            4, 0, 6, 0, 5,
            6, 6, 6, 6, 6, 
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");
        let mut ignore_colors = Histogram::new();
        ignore_colors.increment(0);

        let oib: ObjectsInBins = ObjectsInBins::analyze(&enumerated_objects, Some(&ignore_colors)).expect("ok");

        // Act
        let actual: Image = oib.group3_small_medium_big(false).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 1, 0, 1,
            0, 0, 0, 0, 0,
            2, 2, 0, 2, 2,
            2, 0, 3, 0, 2,
            3, 3, 3, 3, 3, 
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
