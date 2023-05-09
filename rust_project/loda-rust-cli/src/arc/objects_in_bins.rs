use super::{Histogram, Image, ImageMask, ImageMaskCount, ImageSize};
use std::collections::HashMap;
use num_integer::Integer;

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

    /// Group the objects into 3 bins based on mass.
    /// - The pixel value 0 is for non-objects.
    /// - The smallest objects with same mass gets assigned `id=1`.
    /// - The in between objects gets assigned `id=2`.
    /// - The biggest objects with same mass gets assigned `id=3`.
    /// 
    /// Returns an image with the same size as the input image.
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

    /// Object ids of the biggest objects.
    /// 
    /// Sets `object_id=0` for all other objects.
    /// The pixel value 0 is for non-objects.
    /// 
    /// Returns an image with the same size as the input image.
    #[allow(dead_code)]
    pub fn big_objects(&self) -> anyhow::Result<Image> {
        let mut biggest_mass: u16 = 0;
        for item in &self.items {
            biggest_mass = biggest_mass.max(item.object_mass);
        }
        if biggest_mass == 0 {
            return Err(anyhow::anyhow!("ObjectsInBins.big_objects: unable to find the biggest object"));
        }
        let mut result_image = Image::zero(self.image_size.width, self.image_size.height);
        for item in &self.items {
            let set_color: u8;
            if item.object_mass == biggest_mass {
                set_color = item.object_id;
            } else {
                set_color = 0;
            }
            result_image = item.mask.select_from_image_and_color(&result_image, set_color)?;
        }
        Ok(result_image)
    }

    /// Object ids of the smallest objects.
    /// 
    /// Sets `object_id=0` for all other objects.
    /// The pixel value 0 is for non-objects.
    /// 
    /// Returns an image with the same size as the input image.
    #[allow(dead_code)]
    pub fn small_objects(&self) -> anyhow::Result<Image> {
        let mut smallest_mass: u16 = u16::MAX;
        for item in &self.items {
            smallest_mass = smallest_mass.min(item.object_mass);
        }
        if smallest_mass == u16::MAX {
            return Err(anyhow::anyhow!("ObjectsInBins.small_objects: unable to find the smallest object"));
        }
        let mut result_image = Image::zero(self.image_size.width, self.image_size.height);
        for item in &self.items {
            let set_color: u8;
            if item.object_mass == smallest_mass {
                set_color = item.object_id;
            } else {
                set_color = 0;
            }
            result_image = item.mask.select_from_image_and_color(&result_image, set_color)?;
        }
        Ok(result_image)
    }

    /// Object ids for the objects that has the specified mass.
    /// 
    /// Sets `object_id=0` for all other objects.
    /// The pixel value 0 is for non-objects.
    /// 
    /// Returns an image with the same size as the input image.
    #[allow(dead_code)]
    pub fn objects_with_mass(&self, mass: u16) -> anyhow::Result<Image> {
        let mut result_image = Image::zero(self.image_size.width, self.image_size.height);
        for item in &self.items {
            let set_color: u8;
            if item.object_mass == mass {
                set_color = item.object_id;
            } else {
                set_color = 0;
            }
            result_image = item.mask.select_from_image_and_color(&result_image, set_color)?;
        }
        Ok(result_image)
    }

    fn mass_histogram(&self) -> HashMap<u16,u8> {
        let mut counters = HashMap::<u16,u8>::new();
        for item in &self.items {
            if let Some(counter) = counters.get_mut(&item.object_mass) {
                *counter += 1;
            } else {
                counters.insert(item.object_mass, 1);
            }
        }
        counters
    }

    /// Object ids for the objects with unique mass.
    /// 
    /// Sets `object_id=0` for all other objects.
    /// The pixel value 0 is for non-objects.
    /// 
    /// Returns an image with the same size as the input image.
    #[allow(dead_code)]
    pub fn unique_objects(&self) -> anyhow::Result<Image> {
        let histogram: HashMap::<u16,u8> = self.mass_histogram();
        let mut result_image = Image::zero(self.image_size.width, self.image_size.height);
        for item in &self.items {
            let mut set_color: u8 = 0;
            if let Some(count) = histogram.get(&item.object_mass) {
                if *count == 1 {
                    set_color = item.object_id;
                }
            }
            result_image = item.mask.select_from_image_and_color(&result_image, set_color)?;
        }
        Ok(result_image)
    }

    /// Object ids for the objects where the same mass occurs 2 or more times.
    /// 
    /// Sets `object_id=0` for all other objects.
    /// The pixel value 0 is for non-objects.
    /// 
    /// Returns an image with the same size as the input image.
    #[allow(dead_code)]
    pub fn duplicate_objects(&self) -> anyhow::Result<Image> {
        let histogram: HashMap::<u16,u8> = self.mass_histogram();
        let mut result_image = Image::zero(self.image_size.width, self.image_size.height);
        for item in &self.items {
            let mut set_color: u8 = 0;
            if let Some(count) = histogram.get(&item.object_mass) {
                if *count >= 2 {
                    set_color = item.object_id;
                }
            }
            result_image = item.mask.select_from_image_and_color(&result_image, set_color)?;
        }
        Ok(result_image)
    }

    /// Sort the objects by mass into 2 groups.
    /// - The pixel value 0 is for non-objects.
    /// - Assign half of the objects with smallest mass `id=1`.
    /// - Assign half of the objects with biggest mass `id=2`.
    /// 
    /// This only works when there are an even number of objects.
    /// 
    /// If there is an odd number of objects then an error is returned.
    /// since it's unclear what half to put the middle object inside.
    /// 
    /// If it's ambiguous how to sort the objects then an error is returned.
    /// 
    /// Returns an image with the same size as the input image.
    #[allow(dead_code)]
    pub fn sort2_small_big(&self, reverse: bool) -> anyhow::Result<Image> {
        let mut items = self.items.clone();
        if items.len() < 2 {
            return Err(anyhow::anyhow!("ObjectsInBins.sort2_small_big: There must be 2 or more objects."));
        }
        if items.len().is_odd() {
            return Err(anyhow::anyhow!("ObjectsInBins.sort2_small_big: The number of objects must be even. It's ambiguous how to split the objects into 2 parts."));
        }
        items.sort_unstable_by_key(|item| item.object_mass);

        // Check if the objects near the middle are in increasing order
        let half: usize = items.len() / 2;
        if items.len() >= 2 && half > 0 {
            let item0: &Item = &items[half - 1];
            let item1: &Item = &items[half];
            if item0.object_mass == item1.object_mass {
                return Err(anyhow::anyhow!("ObjectsInBins.sort2_small_big: The objects near the middle must be in increasing order. It's ambiguous how to sort the objects."));
            }
        }

        let color_small: u8;
        let color_big: u8;
        if reverse {
            color_small = 2;
            color_big = 1;
        } else {
            color_small = 1;
            color_big = 2;
        }

        let mut result_image = Image::zero(self.image_size.width, self.image_size.height);
        for (index, item) in items.iter().enumerate() {
            let set_color: u8;
            if index < half {
                set_color = color_small;
            } else {
                set_color = color_big;
            }
            result_image = item.mask.select_from_image_and_color(&result_image, set_color)?;
        }
        Ok(result_image)
    }

    // Future experiments
    // objects_mass_bigger_than(mass)
    // objects_mass_smaller_than(mass)
}

#[derive(Clone, Debug)]
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

    #[test]
    fn test_20000_big_objects() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            1, 0, 2, 0, 3,
            0, 0, 0, 0, 0,
            4, 4, 0, 5, 5,
            4, 0, 0, 0, 5,
            6, 6, 7, 7, 7, 
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");
        let mut ignore_colors = Histogram::new();
        ignore_colors.increment(0);

        let oib: ObjectsInBins = ObjectsInBins::analyze(&enumerated_objects, Some(&ignore_colors)).expect("ok");

        // Act
        let actual: Image = oib.big_objects().expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            4, 4, 0, 5, 5,
            4, 0, 0, 0, 5,
            0, 0, 7, 7, 7, 
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_small_objects() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            1, 0, 2, 0, 3,
            0, 0, 0, 0, 0,
            4, 4, 0, 5, 5,
            4, 0, 0, 0, 5,
            6, 6, 7, 7, 7, 
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");
        let mut ignore_colors = Histogram::new();
        ignore_colors.increment(0);

        let oib: ObjectsInBins = ObjectsInBins::analyze(&enumerated_objects, Some(&ignore_colors)).expect("ok");

        // Act
        let actual: Image = oib.small_objects().expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 2, 0, 3,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_objects_with_mass() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            1, 0, 2, 0, 3,
            0, 0, 0, 0, 3,
            4, 4, 0, 5, 5,
            4, 0, 0, 0, 5,
            6, 6, 7, 7, 7, 
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");
        let mut ignore_colors = Histogram::new();
        ignore_colors.increment(0);

        let oib: ObjectsInBins = ObjectsInBins::analyze(&enumerated_objects, Some(&ignore_colors)).expect("ok");

        // Act
        let actual: Image = oib.objects_with_mass(2).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 3,
            0, 0, 0, 0, 3,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            6, 6, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_50000_unique_objects() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            1, 0, 2, 0, 3,
            0, 0, 2, 0, 3,
            4, 4, 0, 5, 5,
            4, 0, 0, 5, 5,
            6, 6, 7, 7, 7, 
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");
        let mut ignore_colors = Histogram::new();
        ignore_colors.increment(0);

        let oib: ObjectsInBins = ObjectsInBins::analyze(&enumerated_objects, Some(&ignore_colors)).expect("ok");

        // Act
        let actual: Image = oib.unique_objects().expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 5, 5,
            0, 0, 0, 5, 5,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_60000_duplicate_objects() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            1, 0, 2, 0, 3,
            0, 0, 2, 0, 3,
            4, 4, 0, 5, 5,
            4, 0, 0, 5, 5,
            6, 6, 7, 7, 7, 
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");
        let mut ignore_colors = Histogram::new();
        ignore_colors.increment(0);

        let oib: ObjectsInBins = ObjectsInBins::analyze(&enumerated_objects, Some(&ignore_colors)).expect("ok");

        // Act
        let actual: Image = oib.duplicate_objects().expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 2, 0, 3,
            0, 0, 2, 0, 3,
            4, 4, 0, 0, 0,
            4, 0, 0, 0, 0,
            6, 6, 7, 7, 7, 
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_70000_sort2_small_big() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            1, 0, 2, 0, 3,
            0, 0, 2, 0, 3,
            4, 4, 0, 5, 5,
            4, 0, 0, 5, 5,
            6, 6, 6, 0, 0,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");
        let mut ignore_colors = Histogram::new();
        ignore_colors.increment(0);

        let oib: ObjectsInBins = ObjectsInBins::analyze(&enumerated_objects, Some(&ignore_colors)).expect("ok");

        // Act
        let actual: Image = oib.sort2_small_big(false).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 1, 0, 1,
            0, 0, 1, 0, 1,
            2, 2, 0, 2, 2,
            2, 0, 0, 2, 2,
            2, 2, 2, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_70001_sort2_small_big_ambiguous_how_to_sort() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            1, 0, 2, 0, 3,
            0, 0, 2, 0, 3,
            4, 0, 0, 0, 5,
            4, 0, 0, 0, 5,
            6, 6, 6, 0, 0,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");
        let mut ignore_colors = Histogram::new();
        ignore_colors.increment(0);

        let oib: ObjectsInBins = ObjectsInBins::analyze(&enumerated_objects, Some(&ignore_colors)).expect("ok");

        // Act
        let error = oib.sort2_small_big(false).expect_err("should fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("objects near the middle must be in increasing order"), true);
    }
}
