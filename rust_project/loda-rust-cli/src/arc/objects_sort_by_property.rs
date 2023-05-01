use crate::arc::{Image, ImageMaskCount};

#[allow(dead_code)]
pub struct ObjectsSortByProperty;

impl ObjectsSortByProperty {
    /// Smallest objects first, biggest objects last.
    #[allow(dead_code)]
    pub fn sort_by_mass_ascending(images: &Vec<Image>) -> anyhow::Result<Vec<Image>> {
        Self::sort_by_mass_inner(images, false)
    }

    /// Biggest objects first, smallest objects last.
    #[allow(dead_code)]
    pub fn sort_by_mass_descending(images: &Vec<Image>) -> anyhow::Result<Vec<Image>> {
        Self::sort_by_mass_inner(images, true)
    }

    fn sort_by_mass_inner(images: &Vec<Image>, reverse: bool) -> anyhow::Result<Vec<Image>> {
        let mut items: Vec<Item> = images.iter().map(|image| Item { mass: image.mask_count_one(), image: image.clone() }).collect();
        items.sort();
        if reverse {
            items.reverse();
        }
        let sorted_image: Vec<Image> = items.iter().map(|item| item.image.clone()).collect();
        Ok(sorted_image)
    }
}

#[derive(Clone, Hash, Eq, Ord, PartialEq, PartialOrd)]
struct Item {
    mass: u16,
    image: Image,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{ImageTryCreate, ImageSegment, ImageSegmentAlgorithm, ImageObjectEnumerate, ImageMask};

    #[test]
    fn test_10000_sort_by_mass() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 4, 0, 0, 0,
            4, 4, 0, 5, 0,
            0, 0, 0, 5, 0,
            7, 0, 5, 5, 5,
            7, 0, 0, 0, 5,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let background_color: u8 = 0;

        let image_mask: Image = input.to_mask_where_color_is_different(background_color);
        let ignore_mask: Image = image_mask.to_mask_where_color_is(0);
        let objects: Vec<Image> = image_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &ignore_mask).expect("objects");
        assert_eq!(objects.len(), 3);

        // Act
        let actual: Vec<Image> = ObjectsSortByProperty::sort_by_mass_ascending(&objects).expect("objects");

        // Assert
        let enumerated_objects: Image = Image::object_enumerate(&actual).expect("image");
        let expected_pixels: Vec<u8> = vec![
            2, 2, 0, 0, 0,
            2, 2, 0, 3, 0,
            0, 0, 0, 3, 0,
            1, 0, 3, 3, 3,
            1, 0, 0, 0, 3,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(enumerated_objects, expected);
    }
}
