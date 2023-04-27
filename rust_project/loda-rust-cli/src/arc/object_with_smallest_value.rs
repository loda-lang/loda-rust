use crate::arc::{Image, Histogram, ImageHistogram, ImageMask};

#[allow(dead_code)]
pub struct ObjectWithSmallestValue;

impl ObjectWithSmallestValue {
    /// Find the object with the smallest pixel value.
    /// 
    /// The `image` and the `enumerated_objects` must have same size. And the size must be 1x1 or bigger.
    /// 
    /// Returns an error when there are multiple objects with the same smallest value, then it's ambiguous which object to pick. 
    /// 
    /// Returns an image mask with the same size as the input image.
    /// The pixel values are `1` where the found object is located.
    #[allow(dead_code)]
    pub fn run(image: &Image, enumerated_objects: &Image) -> anyhow::Result<Image> {
        if image.size() != enumerated_objects.size() {
            return Err(anyhow::anyhow!("images must have same size"));
        }
        if image.is_empty() {
            return Err(anyhow::anyhow!("image must be 1x1 or bigger"));
        }
        let mut object_color_and_object_index = Vec::<(u8, u8)>::new();
        // Traverse the objects and extract the most popular color of each object
        // Skip object_index=0, because it's not considered an object.
        for object_index in 1..=255u8 {
            let mask: Image = enumerated_objects.to_mask_where_color_is(object_index);
            let histogram: Histogram = image.histogram_with_mask(&mask)?;
            if histogram.number_of_counters_greater_than_zero() == 0 {
                continue;
            }
            let unique_color_count: u8 = match histogram.most_popular_color_disallow_ambiguous() {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Cannot decide what color is the most popular. ambiguous"));
                }
            };
            object_color_and_object_index.push((unique_color_count, object_index));
        }

        object_color_and_object_index.sort();
        // println!("object_color_and_object_index: {:?}", object_color_and_object_index);

        // Find the object with lowest value. Disallow ambiguous results.
        let mut histogram = Histogram::new();
        for (unique_color_count, _object_index) in &object_color_and_object_index {
            histogram.increment(*unique_color_count);
        }
        let mut found: Option<u8> = None;
        for i in 0..=255u8 {
            let count: u32 = histogram.get(i);
            if count == 0 {
                continue;
            }
            if count == 1 {
                found = Some(i);
                break;
            }
            return Err(anyhow::anyhow!("Multiple objects with the same number of unique colors. Ambiguous which object to pick."));
        }
        let color_count: u8 = match found {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Did not found any color. Cannot decide on which color to pick."));
            }
        };
        let mut found_object_index: Option<u8> = None;
        for (unique_color_count, object_index) in &object_color_and_object_index {
            if *unique_color_count == color_count {
                found_object_index = Some(*object_index);
                break;
            }
        }
        let object_index: u8 = match found_object_index {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Unable to identify the object of interest"));
            }
        };

        // mask for the object
        let mask: Image = enumerated_objects.to_mask_where_color_is(object_index);
        Ok(mask)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_two_objects_interwoven() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 5, 5, 5,
            7, 7, 5, 5, 5,
            5, 5, 7, 7, 7,
            5, 5, 7, 7, 7,
            5, 5, 7, 7, 7,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let enumerated_object_pixels: Vec<u8> = vec![
            1, 1, 2, 2, 2,
            1, 1, 2, 2, 2,
            2, 2, 1, 1, 1,
            2, 2, 1, 1, 1,
            2, 2, 1, 1, 1,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectWithSmallestValue::run(&input, &enumerated_objects).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1,
            0, 0, 1, 1, 1,
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_boxes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 7, 7,
            7, 0, 0, 0, 7,
            7, 0, 7, 0, 7,
            7, 0, 0, 0, 7,
            7, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let enumerated_object_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 1, 2, 1, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectWithSmallestValue::run(&input, &enumerated_objects).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 1, 0, 1, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
