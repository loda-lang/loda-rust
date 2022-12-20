use std::collections::HashMap;

use crate::arc::{ImageColorProfile, ImageMask, ImageTrim};

use super::{Image, ImageSegment, ImageSegmentAlgorithm};

pub struct PopularObjects;

impl PopularObjects {

    pub fn most_popular_object(input: &Image) -> anyhow::Result<Image> {
        let object_mask_vec: Vec<Image> = input.find_objects(ImageSegmentAlgorithm::All).expect("image");

        // Preserve colors of original image where the mask is on
        let mut objects = Vec::<Image>::new();
        let background_color: u8 = input.most_popular_color().expect("color");
        for mask in &object_mask_vec {
            // If the mask is on, then preserve the pixel as it is.
            // If the mask is off, then clear the pixel.
            let image: Image = mask.select_from_image(&input, background_color).expect("image");
            let object: Image = image.trim().expect("image");
            if object.is_empty() {
                continue;
            }
            objects.push(object);
        }

        // Build histogram of objects
        let mut histogram = HashMap::<Image,u32>::new();
        for object in objects {
            let counter = histogram.entry(object).or_insert(0);
            *counter += 1;
        }

        #[allow(dead_code)]
        struct Record {
            count: u32,
            image: Image,
        }

        // Convert from dictionary to array
        let mut records = Vec::<Record>::new();
        for (histogram_key, histogram_count) in &histogram {
            let record = Record {
                count: *histogram_count,
                image: histogram_key.clone(),
            };
            records.push(record);
        }

        // Move the most frequently occuring items to the top
        // Move the lesser used items to the bottom
        records.sort_unstable_by_key(|item| item.count);
        records.reverse();
        
        // Pick the item that is most popular
        let record0: &Record = records.first().expect("record");
        Ok(record0.image.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_most_popular_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 0, 0, 0, 0, 5, 5,
            4, 0, 0, 4, 0, 5, 5,
            4, 4, 0, 4, 0, 0, 0,
            0, 0, 0, 4, 4, 0, 0,
        ];
        let input: Image = Image::try_create(7, 4, pixels).expect("image");

        // Act
        let actual: Image = PopularObjects::most_popular_object(&input).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            4, 0,
            4, 0,
            4, 4,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_most_popular_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 0, 0, 0, 0, 5, 5,
            4, 0, 0, 0, 0, 5, 5,
            4, 4, 5, 5, 0, 0, 0,
            0, 0, 5, 5, 0, 0, 3,
        ];
        let input: Image = Image::try_create(7, 4, pixels).expect("image");

        // Act
        let actual: Image = PopularObjects::most_popular_object(&input).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5,
            5, 5,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_most_popular_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 0, 0, 0, 3, 0, 3,
            3, 0, 3, 5, 5, 3, 0,
            0, 3, 0, 0, 3, 0, 3,
            3, 0, 3, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(7, 4, pixels).expect("image");

        // Act
        let actual: Image = PopularObjects::most_popular_object(&input).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 0, 3,
            0, 3, 0,
            3, 0, 3,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
