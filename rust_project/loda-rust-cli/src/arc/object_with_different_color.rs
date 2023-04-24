use crate::arc::{Image, Histogram, ImageHistogram, ImageMask};

#[allow(dead_code)]
pub struct ObjectWithDifferentColor;

impl ObjectWithDifferentColor {
    /// Find the single object that has different colors than the other objects.
    /// 
    /// Returns an error when there it's ambiguous what object to pick.
    /// 
    /// The `image` and the `enumerated_objects` must have same size. And the size must be 1x1 or bigger.
    /// 
    /// Returns an image mask with the same size as the input image.
    #[allow(dead_code)]
    pub fn run(image: &Image, enumerated_objects: &Image, ignore_colors: Option<&Histogram>) -> anyhow::Result<Image> {
        if image.size() != enumerated_objects.size() {
            return Err(anyhow::anyhow!("ObjectWithDifferentColor: images must have same size"));
        }
        if image.is_empty() {
            return Err(anyhow::anyhow!("ObjectWithDifferentColor: image must be 1x1 or bigger"));
        }

        // Determine the most popular color of each cell
        let mut object_color_and_object_index = Vec::<(u8, u8)>::new();
        for object_index in 1..=255u8 {
            let mask: Image = enumerated_objects.to_mask_where_color_is(object_index);
            let mut histogram: Histogram = image.histogram_with_mask(&mask)?;
            if let Some(other) = ignore_colors {
                histogram.subtract_histogram(other);
            }
            if histogram.number_of_counters_greater_than_zero() == 0 {
                continue;
            }
            // println!("object: {} histogram: {:?}", object_index, histogram);
            let object_color: u8 = match histogram.most_popular_color_disallow_ambiguous() {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Ambiguous what is the most popular color"));
                }
            };
            object_color_and_object_index.push((object_color, object_index));
        }
        // println!("object_colors: {:?}", object_color_and_object_index);

        // histogram of all object colors
        let mut object_histogram = Histogram::new();
        for (color, _index) in &object_color_and_object_index {
            object_histogram.increment(*color);
        }

        // pick the object with the least popular color
        let least_popular_object_color: u8 = match object_histogram.least_popular_color_disallow_ambiguous() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Ambiguous what is the least popular color"));
            }
        };
        // println!("least_popular_object_color: {}", least_popular_object_color);
        let mut found_object_index: Option<u8> = None;
        for (object_color, object_index) in &object_color_and_object_index {
            if *object_color == least_popular_object_color {
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

        // mask for the found object
        let mask: Image = enumerated_objects.to_mask_where_color_is(object_index);
        Ok(mask)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_found_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 3, 3, 3, 5,
            5, 5, 5, 5, 5,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        let enumerated_object_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 2, 2, 2, 3,
            3, 3, 3, 3, 3,
        ];
        let enumerated_objects: Image = Image::try_create(5, 3, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectWithDifferentColor::run(&input, &enumerated_objects, None).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_found_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 7, 2, 7,
            5, 5, 7, 2, 7,
            7, 7, 7, 7, 7,
            5, 5, 7, 5, 7,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        let enumerated_object_pixels: Vec<u8> = vec![
            1, 1, 0, 2, 0,
            1, 1, 0, 2, 0,
            0, 0, 0, 0, 0,
            3, 3, 0, 4, 0,
        ];
        let enumerated_objects: Image = Image::try_create(5, 4, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectWithDifferentColor::run(&input, &enumerated_objects, None).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 1, 0,
            0, 0, 0, 1, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
