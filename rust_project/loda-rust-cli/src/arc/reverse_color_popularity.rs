use super::{Image, Histogram, ImageHistogram, ImageMask, ImageReplaceColor};
use std::collections::HashMap;

#[allow(dead_code)]
pub struct ReverseColorPopularity;

impl ReverseColorPopularity {
    #[allow(dead_code)]
    pub fn apply_to_image(image: &Image) -> anyhow::Result<Image> {
        if image.is_empty() {
            return Err(anyhow::anyhow!("ReverseColorPopularity.apply_to_image: image must be 1x1 or bigger"));
        }

        let object_histogram: Histogram = image.histogram_all();
        let replacements: HashMap<u8, u8> = ReverseColorPopularity::reverse_popularity(&object_histogram)?;
        let mut result_image: Image = image.clone();
        result_image = result_image.replace_colors_with_hashmap(&replacements)?;
        Ok(result_image)
    }

    #[allow(dead_code)]
    pub fn apply_to_objects(image: &Image, enumerated_objects: &Image) -> anyhow::Result<Image> {
        if image.size() != enumerated_objects.size() {
            return Err(anyhow::anyhow!("ReverseColorPopularity.apply_to_objects: images must have same size"));
        }
        if image.is_empty() {
            return Err(anyhow::anyhow!("ReverseColorPopularity.apply_to_objects: image must be 1x1 or bigger"));
        }

        let mut histogram_all: Histogram = enumerated_objects.histogram_all();
        // Ignore `object id=0` which is the background
        histogram_all.set_counter_to_zero(0);

        let mut result_image: Image = image.clone();

        for object_index in 1..=255u8 {
            let count: u32 = histogram_all.get(object_index);
            if count == 0 {
                continue;
            }

            let object_mask: Image = enumerated_objects.to_mask_where_color_is(object_index);
            let object_histogram: Histogram = image.histogram_with_mask(&object_mask)?;
            let replacements: HashMap<u8, u8> = ReverseColorPopularity::reverse_popularity(&object_histogram)?;

            // Replace colors inside the object using the object_mask
            result_image = result_image.replace_colors_with_mask_and_hashmap(&object_mask, &replacements)?;
        }

        Ok(result_image)
    }

    /// Reorder the color palette, so that the `most popular color` changes place with the `least popular color`.
    /// 
    /// The reordering is possible when the colors popularity is descending in an unambiguous way.
    /// 
    /// The reordering is not possible when there are 3 or more colors with the same frequency.
    /// 
    /// Returns a `HashMap` with color replacements.
    fn reverse_popularity(histogram: &Histogram) -> anyhow::Result<HashMap<u8, u8>> {
        let pairs_ascending: Vec<(u32, u8)> = histogram.pairs_ascending();
        let pairs_descending: Vec<(u32, u8)> = histogram.pairs_descending();
        if pairs_ascending.len() != pairs_descending.len() {
            return Err(anyhow::anyhow!("Integrity error. Supposed to have same length"));
        }
        let mut ambiguity_counters = HashMap::<u32, u8>::new();
        let mut dict = HashMap::<u8, u8>::new();
        for ((count0, color0), (_count1, color1)) in pairs_ascending.iter().zip(pairs_descending.iter()) {
            if let Some(counter) = ambiguity_counters.get_mut(count0) {
                *counter += 1;
            } else {
                ambiguity_counters.insert(*count0, 1);
            }

            if color0 == color1 {
                continue;
            }
            dict.insert(*color0, *color1);
        }

        // There is an ambiguous scenario, when there are 3 or more colors with the same count, 
        // then it's unclear what should happen.
        //
        // Interestingly this ambiguous scenario is not a problem 2 colors.
        // When there are 2 colors with the same popularity then, then swap the colors.
        for (_key, count) in ambiguity_counters {
            if count >= 3 {
                return Err(anyhow::anyhow!("Detected 3 or more colors with the same frequency. Unclear how to reverse the colors by popularity."));
            }
        }

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_reverse_popularity_one_color_do_nothing() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(5);

        // Act
        let actual: HashMap<u8, u8> = ReverseColorPopularity::reverse_popularity(&h).expect("dict");

        // Assert
        let expected = HashMap::<u8, u8>::new();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_reverse_popularity_two_colors_ordering_unambiguous() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(5);
        h.increment(7);
        h.increment(7);

        // Act
        let actual: HashMap<u8, u8> = ReverseColorPopularity::reverse_popularity(&h).expect("dict");

        // Assert
        let mut expected = HashMap::<u8, u8>::new();
        expected.insert(5, 7);
        expected.insert(7, 5);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_reverse_popularity_ordering_is_somewhat_ambiguous() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(5);
        h.increment(5);
        h.increment(7);
        h.increment(7);

        // Act
        let actual: HashMap<u8, u8> = ReverseColorPopularity::reverse_popularity(&h).expect("dict");

        // Assert
        let mut expected = HashMap::<u8, u8>::new();
        expected.insert(5, 7);
        expected.insert(7, 5);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_reverse_popularity_ordering_is_ambiguous() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(5);
        h.increment(5);
        h.increment(3);
        h.increment(3);
        h.increment(7);
        h.increment(7);

        // Act
        let error = ReverseColorPopularity::reverse_popularity(&h).expect_err("should fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("Detected 3 or more colors with the same frequency"), true);
    }

    #[test]
    fn test_20000_apply_to_image() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 2, 2, 2, 1,
            1, 2, 3, 2, 1,
            1, 2, 2, 2, 1,
            1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 5, input_pixels).expect("image");

        // Act
        let actual: Image = ReverseColorPopularity::apply_to_image(&input).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3, 3,
            3, 2, 2, 2, 3,
            3, 2, 1, 2, 3,
            3, 2, 2, 2, 3,
            3, 3, 3, 3, 3,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_apply_to_objects() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 1, 1, 0, 9,
            1, 0, 1, 0, 0,
            1, 1, 1, 0, 0,
            0, 0, 3, 3, 3,
            1, 0, 2, 2, 1,
        ];
        let input: Image = Image::try_create(5, 5, input_pixels).expect("image");

        let enumerated_objects_pixels: Vec<u8> = vec![
            4, 4, 4, 5, 5,
            4, 4, 4, 5, 5,
            4, 4, 4, 5, 5,
            7, 7, 2, 2, 2,
            7, 7, 2, 2, 2,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_objects_pixels).expect("image");

        // Act
        let actual: Image = ReverseColorPopularity::apply_to_objects(&input, &enumerated_objects).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 9, 0,
            0, 1, 0, 9, 9,
            0, 0, 0, 9, 9,
            1, 1, 1, 1, 1,
            0, 1, 2, 2, 3,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
