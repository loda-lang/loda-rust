use super::{Image, ImageSegment, ImageSegmentAlgorithm, ImageColorProfile, ImageMask, ImageTrim};
use anyhow::Context;
use std::collections::HashMap;
#[allow(dead_code)]
struct Record {
    count: u32,
    image: Image,
}

pub struct PopularObjects;

impl PopularObjects {
    pub fn most_popular_object(input: &Image) -> anyhow::Result<Image> {
        let mut records: Vec<Record> = Self::find_objects(input)?;

        // Pick the item that is most popular
        let record: Record = records.pop()
            .with_context(|| "most_popular_object pop")?;
        Ok(record.image)
    }

    pub fn least_popular_object(input: &Image) -> anyhow::Result<Image> {
        let mut records: Vec<Record> = Self::find_objects(input)?;
        records.reverse();

        // Pick the item that is least popular
        let record: Record = records.pop()
            .with_context(|| "least_popular_object pop")?;
        Ok(record.image)
    }

    fn find_objects(input: &Image) -> anyhow::Result<Vec<Record>> {
        let object_mask_vec: Vec<Image> = input.find_objects(ImageSegmentAlgorithm::All)
            .with_context(|| "find_objects find_objects")?;

        // Preserve colors of original image where the mask is on
        let mut objects = Vec::<Image>::new();
        let background_color: u8 = input.most_popular_color()
            .with_context(|| "find_objects most_popular_color")?;
        for mask in &object_mask_vec {
            // If the mask is on, then preserve the pixel as it is.
            // If the mask is off, then clear the pixel.
            let image: Image = mask.select_from_color_and_image(background_color, &input)
                .with_context(|| "find_objects select_from_image")?;
            let object: Image = image.trim()
                .with_context(|| "find_objects trim")?;
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

        // Convert from dictionary to array
        let mut records = Vec::<Record>::new();
        for (histogram_key, histogram_count) in &histogram {
            let record = Record {
                count: *histogram_count,
                image: histogram_key.clone(),
            };
            records.push(record);
        }

        // Move the most frequently occurring items to the end
        // Move the lesser used items to the front
        records.sort_unstable_by_key(|item| item.count);
        Ok(records)
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

    #[test]
    fn test_20000_least_popular_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 0, 0, 0, 0, 5, 5,
            4, 0, 0, 4, 0, 5, 5,
            4, 4, 0, 4, 0, 0, 0,
            0, 0, 0, 4, 4, 0, 0,
        ];
        let input: Image = Image::try_create(7, 4, pixels).expect("image");

        // Act
        let actual: Image = PopularObjects::least_popular_object(&input).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5,
            5, 5,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_least_popular_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 0, 3, 0, 0, 5, 5,
            4, 0, 0, 0, 3, 5, 5,
            4, 4, 5, 5, 0, 0, 0,
            0, 0, 5, 5, 0, 0, 3,
        ];
        let input: Image = Image::try_create(7, 4, pixels).expect("image");

        // Act
        let actual: Image = PopularObjects::least_popular_object(&input).expect("image");

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
    fn test_20002_least_popular_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0, 0, 0, 0,
            3, 0, 3, 5, 5, 0, 0,
            0, 3, 0, 0, 0, 0, 0,
            3, 0, 3, 0, 0, 5, 5,
        ];
        let input: Image = Image::try_create(7, 4, pixels).expect("image");

        // Act
        let actual: Image = PopularObjects::least_popular_object(&input).expect("image");

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
