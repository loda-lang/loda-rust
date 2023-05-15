use super::{Image, ImageExtractRowColumn};

#[derive(Clone, Copy, Debug)]
pub enum HistogramPair {
    None,
    Item { count: u32, color: u8, ambiguous_score: u8 }
}

impl HistogramPair {
    /// When there are +2 colors with same count. It's ambiguous which one to pick.
    pub fn color_count_disallow_ambiguous(&self) -> Option<(u8, u32)> {
        match self {
            HistogramPair::Item { count, color, ambiguous_score } => {
                if *ambiguous_score > 0 {
                    return None;
                } else {
                    return Some((*color, *count));
                }
            },
            HistogramPair::None => {
                return None;
            }
        }
    }

    /// This is a weak function and should be avoided, since it ignores the number of ambiguous colors.
    /// When multiple colors have the same count, then the color with the lowest color value is picked.
    /// 
    /// Instead the `color_count_disallow_ambiguous()` should be preferred, since it's more strict.
    pub fn color_count_allow_ambiguous(&self) -> Option<(u8, u32)> {
        match self {
            HistogramPair::Item { count, color, ambiguous_score: _ } => {
                return Some((*color, *count));
            },
            HistogramPair::None => {
                return None;
            }
        }
    }

    /// When there are +2 colors with same count. It's ambiguous which one to pick.
    pub fn color_disallow_ambiguous(&self) -> Option<u8> {
        match self {
            HistogramPair::Item { count: _, color, ambiguous_score } => {
                if *ambiguous_score > 0 {
                    return None;
                } else {
                    return Some(*color);
                }
            },
            HistogramPair::None => {
                return None;
            }
        }
    }

    /// This is a weak function and should be avoided, since it ignores the number of ambiguous colors.
    /// When multiple colors have the same count, then the color with the lowest color value is picked.
    /// 
    /// Instead the `color_disallow_ambiguous()` should be preferred, since it's more strict.
    pub fn color_allow_ambiguous(&self) -> Option<u8> {
        match self {
            HistogramPair::Item { count: _, color, ambiguous_score: _ } => {
                return Some(*color);
            },
            HistogramPair::None => {
                return None;
            }
        }
    }

    /// This is a weak function and should be avoided, since it ignores the number of ambiguous colors.
    /// When multiple colors have the same count, then the color with the lowest color value is picked.
    /// 
    /// Instead the `color_count_disallow_ambiguous()` should be preferred, since it's more strict.
    pub fn count_allow_ambiguous(&self) -> Option<u32> {
        match self {
            HistogramPair::Item { count, color: _, ambiguous_score: _ } => {
                return Some(*count);
            },
            HistogramPair::None => {
                return None;
            }
        }
    }
}

/// Histogram with 256 counters
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Histogram {
    counters: [u32; 256],
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            counters: [0; 256],
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.counters = [0; 256];
    } 

    #[allow(dead_code)]
    pub fn counters(&self) -> &[u32; 256] {
        &self.counters
    }

    /// Get the counter value for a color.
    /// 
    /// Example: Get the number of times that `color 42` occur in an image.
    pub fn get(&self, index: u8) -> u32 {
        self.counters[index as usize]
    }

    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<u32> {
        self.counters.to_vec()
    }

    pub fn increment_pixel(&mut self, image: &Image, x: i32, y: i32) {
        if let Some(pixel_value) = image.get(x, y) {
            self.increment(pixel_value);
        }
    }

    pub fn increment_by(&mut self, index: u8, increment: u32) {
        let count: u32 = self.counters[index as usize];
        self.counters[index as usize] = count + increment;
    }

    pub fn increment(&mut self, index: u8) {
        self.increment_by(index, 1);
    }

    pub fn set_counter_to_zero(&mut self, index: u8) {
        self.counters[index as usize] = 0;
    }

    #[allow(dead_code)]
    pub fn set_counter_to_zero_where_count_is_below(&mut self, limit: u32) {
        for i in 0..256 {
            if self.counters[i] < limit {
                self.counters[i] = 0;
            }
        }
    }

    pub fn add_histogram(&mut self, other: &Histogram) {
        for i in 0..256 {
            self.counters[i] += other.counters[i];
        }
    }

    /// Finds the `intersection` between two histograms, similar to performing an `AND` operation.
    /// 
    /// The counter is `1` when the color is present in both histograms.
    /// 
    /// Otherwise the counter is `0`.
    pub fn intersection_histogram(&mut self, other: &Histogram) {
        for i in 0..256 {
            let a: u32 = self.counters[i];
            let b: u32 = other.counters[i];
            let v: u32 = a.min(b).min(1);
            self.counters[i] = v;
        }
    }

    /// Clear counters where the other histogram has non-zero counters.
    /// 
    /// Performs an operation similar to: `self` AND NOT `other`.
    pub fn subtract_histogram(&mut self, other: &Histogram) {
        for i in 0..256 {
            if other.counters[i] > 0 {
                self.counters[i] = 0;
            }
        }
    }

    pub fn most_popular(&self) -> HistogramPair {
        let mut found_count: u32 = 0;
        let mut found_index: usize = 0;
        let mut ambiguous_count: usize = 0;
        for (index, number_of_occurrences) in self.counters.iter().enumerate() {
            if *number_of_occurrences == found_count {
                ambiguous_count += 1;
            }
            if *number_of_occurrences > found_count {
                found_count = *number_of_occurrences;
                found_index = index;
                ambiguous_count = 0;
            }
        }
        if found_count == 0 {
            return HistogramPair::None;
        }
        let ambiguous_score: u8 = (ambiguous_count & 255) as u8;
        let color_value: u8 = (found_index & 255) as u8;
        HistogramPair::Item {
            count: found_count,
            color: color_value,
            ambiguous_score
        }
    }

    #[allow(dead_code)]
    pub fn most_popular_pair_disallow_ambiguous(&self) -> Option<(u8, u32)> {
        self.most_popular().color_count_disallow_ambiguous()
    }

    #[allow(dead_code)]
    pub fn most_popular_pair(&self) -> Option<(u8, u32)> {
        self.most_popular().color_count_allow_ambiguous()
    }
    
    #[allow(dead_code)]
    pub fn most_popular_color(&self) -> Option<u8> {
        self.most_popular().color_allow_ambiguous()
    }
    
    #[allow(dead_code)]
    pub fn most_popular_color_disallow_ambiguous(&self) -> Option<u8> {
        self.most_popular().color_disallow_ambiguous()
    }
    
    #[allow(dead_code)]
    pub fn most_popular_count(&self) -> Option<u32> {
        self.most_popular().count_allow_ambiguous()
    }

    pub fn least_popular(&self) -> HistogramPair {
        let mut found_count: u32 = u32::MAX;
        let mut found_index: usize = 0;
        let mut ambiguous_count: usize = 0;
        for (index, number_of_occurrences) in self.counters.iter().enumerate() {
            if *number_of_occurrences == 0 {
                continue;
            }
            if *number_of_occurrences == found_count {
                ambiguous_count += 1;
            }
            if *number_of_occurrences < found_count {
                found_count = *number_of_occurrences;
                found_index = index;
                ambiguous_count = 0;
            }
        }
        if found_count == u32::MAX {
            return HistogramPair::None;
        }
        let ambiguous_score: u8 = (ambiguous_count & 255) as u8;
        let color_value: u8 = (found_index & 255) as u8;
        HistogramPair::Item {
            count: found_count,
            color: color_value,
            ambiguous_score
        }
    }

    #[allow(dead_code)]
    pub fn least_popular_pair_disallow_ambiguous(&self) -> Option<(u8, u32)> {
        self.least_popular().color_count_disallow_ambiguous()
    }

    #[allow(dead_code)]
    pub fn least_popular_pair(&self) -> Option<(u8, u32)> {
        self.least_popular().color_count_allow_ambiguous()
    }

    #[allow(dead_code)]
    pub fn least_popular_color_disallow_ambiguous(&self) -> Option<u8> {
        self.least_popular().color_disallow_ambiguous()
    }
    
    #[allow(dead_code)]
    pub fn least_popular_color(&self) -> Option<u8> {
        self.least_popular().color_allow_ambiguous()
    }

    #[allow(dead_code)]
    pub fn least_popular_count(&self) -> Option<u32> {
        self.least_popular().count_allow_ambiguous()
    }

    /// The pairs ordered by their color value.
    /// 
    /// The lowest color value comes first.
    /// 
    /// The highest color value comes last.
    pub fn pairs_ordered_by_color(&self) -> Vec<(u32,u8)> {
        let mut pairs = Vec::<(u32, u8)>::with_capacity(256);
        for (index, number_of_occurences) in self.counters.iter().enumerate() {
            if *number_of_occurences > 0 {
                pairs.push((*number_of_occurences, (index & 255) as u8));
            }
        }
        pairs
    }

    /// The least frequent occurring comes first.
    /// 
    /// The medium frequent occurring comes middle.
    /// 
    /// The most frequent occurring comes last.
    pub fn pairs_ascending(&self) -> Vec<(u32,u8)> {
        let mut pairs = self.pairs_ordered_by_color();
        pairs.sort();
        pairs
    }

    /// The most frequent occurring comes first.
    /// 
    /// The medium frequent occurring comes middle.
    /// 
    /// The least frequent occurring are at the end.
    pub fn pairs_descending(&self) -> Vec<(u32,u8)> {
        let mut pairs = self.pairs_ascending();
        pairs.reverse();
        pairs
    }

    /// Number of counters that are greater than zero.
    /// 
    /// The returned value is in the range `[0..256]`.
    /// - Returns `0` when all of the 256 counters are zero.
    /// - Returns `256` when all of the 256 counters are non-zero.
    pub fn number_of_counters_greater_than_zero(&self) -> u16 {
        let mut count: u32 = 0;
        for number_of_occurences in &self.counters {
            if *number_of_occurences > 0 {
                count += 1;
            }
        }
        count.min(256) as u16
    }

    /// Returns an `Image` where `width` is the number of counters greater than zero, and `height=2`.
    /// 
    /// The top row is the counter values, clamped to 255. None of the counters are zero.
    /// 
    /// The bottom row is the color values.
    /// 
    /// The most popular colors are to the left side.
    /// 
    /// The least popular colors are to the right side.
    pub fn to_image(&self) -> anyhow::Result<Image> {
        let pairs: Vec<(u32, u8)> = self.pairs_descending();
        let mut image = Image::zero(pairs.len() as u8, 2);
        for (index, pair) in pairs.iter().enumerate() {
            let clamped_count: u8 = u32::min(pair.0, u8::max as u32) as u8;
            let color: u8 = pair.1;
            match image.set(index as i32, 0, clamped_count) {
                Some(()) => {},
                None => {
                    return Err(anyhow::anyhow!("Histogram.to_image: Unable to set pixel ({}, {})", index, 0));
                }
            }
            match image.set(index as i32, 1, color) {
                Some(()) => {},
                None => {
                    return Err(anyhow::anyhow!("Histogram.to_image: Unable to set pixel ({}, {})", index, 1));
                }
            }
        }
        Ok(image)
    }

    /// Returns an `Image` where `width` is the number of counters greater than zero, and `height=1`.
    /// 
    /// The most popular colors are to the left side.
    /// 
    /// The least popular colors are to the right side.
    pub fn color_image(&self) -> anyhow::Result<Image> {
        let image: Image = self.to_image()?;
        let image: Image = image.bottom_rows(1)?;
        Ok(image)
    }

    /// Find the lowest available color, that is not used in the histogram
    pub fn unused_color(&self) -> Option<u8> {
        for index in 0..=255u8 {
            if self.counters[index as usize] == 0 {
                return Some(index);
            }
        }
        None
    }

    /// Add all the counters together.
    /// 
    /// Usecase: Compute the mass of a multicolored object.
    pub fn sum(&self) -> u32 {
        self.counters.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use std::collections::HashSet;

    #[test]
    fn test_11000_increment() {
        // Arrange
        let mut h = Histogram::new();

        // Act
        h.increment(42);
        h.increment(42);
        h.increment(3);

        // Assert
        let counters = h.counters();
        assert_eq!(counters[42], 2);
        assert_eq!(counters[3], 1);
    }

    #[test]
    fn test_20000_most_popular_pair_disallow_ambiguous_some() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(42);
        h.increment(3);
        h.increment(3);
        h.increment(3);
        h.increment(2);

        // Act
        let color_count: Option<(u8, u32)> = h.most_popular_pair_disallow_ambiguous();

        // Assert
        assert_eq!(color_count, Some((3,3)));
    }

    #[test]
    fn test_20001_most_popular_pair_disallow_ambiguous_none_due_to_ambiguous() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(1);
        h.increment(42);
        h.increment(42);
        h.increment(3);
        h.increment(3);

        // Act
        let color_count: Option<(u8, u32)> = h.most_popular_pair_disallow_ambiguous();

        // Assert
        assert_eq!(color_count, None);
    }

    #[test]
    fn test_20002_most_popular_pair_disallow_ambiguous_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color_count: Option<(u8, u32)> = h.most_popular_pair_disallow_ambiguous();

        // Assert
        assert_eq!(color_count, None);
    }

    #[test]
    fn test_30000_most_popular_pair_some() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(42);
        h.increment(3);
        h.increment(3);
        h.increment(3);
        h.increment(2);

        // Act
        let color_count: Option<(u8, u32)> = h.most_popular_pair();

        // Assert
        assert_eq!(color_count, Some((3,3)));
    }

    #[test]
    fn test_30001_most_popular_pair_some_lowest_value_due_to_ambiguous_colors() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(1);
        h.increment(42);
        h.increment(42);
        h.increment(3);
        h.increment(3);

        // Act
        let color_count: Option<(u8, u32)> = h.most_popular_pair();

        // Assert
        assert_eq!(color_count, Some((3,2)));
    }

    #[test]
    fn test_30002_most_popular_pair_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color_count: Option<(u8, u32)> = h.most_popular_pair();

        // Assert
        assert_eq!(color_count, None);
    }

    #[test]
    fn test_30003_most_popular_color_some() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(42);
        h.increment(9);
        h.increment(9);
        h.increment(9);
        h.increment(2);

        // Act
        let color: Option<u8> = h.most_popular_color();

        // Assert
        assert_eq!(color, Some(9));
    }

    #[test]
    fn test_30004_most_popular_color_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color: Option<u8> = h.most_popular_color();

        // Assert
        assert_eq!(color, None);
    }

    #[test]
    fn test_30005_most_popular_count_some() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(42);
        h.increment(9);
        h.increment(9);
        h.increment(9);
        h.increment(2);

        // Act
        let count: Option<u32> = h.most_popular_count();

        // Assert
        assert_eq!(count, Some(3));
    }

    #[test]
    fn test_30006_most_popular_count_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let count: Option<u32> = h.most_popular_count();

        // Assert
        assert_eq!(count, None);
    }

    #[test]
    fn test_50000_least_popular_pair_disallow_ambiguous_some() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(3);
        h.increment(2);
        h.increment(3);
        h.increment(42);
        h.increment(3);

        // Act
        let color_count: Option<(u8, u32)> = h.least_popular_pair_disallow_ambiguous();

        // Assert
        assert_eq!(color_count, Some((2,1)));
    }

    #[test]
    fn test_50001_least_popular_pair_disallow_ambiguous_none_due_to_ambiguous() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(3);
        h.increment(3);
        h.increment(42);

        // Act
        let color_count: Option<(u8, u32)> = h.least_popular_pair_disallow_ambiguous();

        // Assert
        assert_eq!(color_count, None);
    }

    #[test]
    fn test_50002_least_popular_pair_disallow_ambiguous_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color_count: Option<(u8, u32)> = h.least_popular_pair_disallow_ambiguous();

        // Assert
        assert_eq!(color_count, None);
    }

    #[test]
    fn test_50003_least_popular_pair_some() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(3);
        h.increment(2);
        h.increment(3);
        h.increment(42);
        h.increment(3);

        // Act
        let color_count: Option<(u8, u32)> = h.least_popular_pair();

        // Assert
        assert_eq!(color_count, Some((2,1)));
    }

    #[test]
    fn test_50004_least_popular_pair_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color_count: Option<(u8, u32)> = h.least_popular_pair();

        // Assert
        assert_eq!(color_count, None);
    }

    #[test]
    fn test_50005_least_popular_color_some() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(3);
        h.increment(2);
        h.increment(3);
        h.increment(42);
        h.increment(3);

        // Act
        let color: Option<u8> = h.least_popular_color();

        // Assert
        assert_eq!(color, Some(2));
    }

    #[test]
    fn test_50006_least_popular_color_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color: Option<u8> = h.least_popular_color();

        // Assert
        assert_eq!(color, None);
    }

    #[test]
    fn test_50007_least_popular_count_some() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(3);
        h.increment(2);
        h.increment(3);
        h.increment(42);
        h.increment(3);

        // Act
        let count: Option<u32> = h.least_popular_count();

        // Assert
        assert_eq!(count, Some(1));
    }

    #[test]
    fn test_50008_least_popular_count_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let count: Option<u32> = h.least_popular_count();

        // Assert
        assert_eq!(count, None);
    }

    #[test]
    fn test_50000_pairs_ordered_by_color() {
        // Arrange
        let mut h = Histogram::new();
        let values: [u8; 8] = [3, 42, 42, 3, 2, 3, 4, 5];
        for value in values {
            h.increment(value);
        }

        // Act
        let pairs: Vec<(u32, u8)> = h.pairs_ordered_by_color();

        // Assert
        let expected: Vec<(u32, u8)> = vec![(1, 2), (3, 3), (1, 4), (1, 5), (2, 42)];
        assert_eq!(pairs, expected);
    }

    #[test]
    fn test_50001_pairs_descending() {
        // Arrange
        let mut h = Histogram::new();
        let values: [u8; 8] = [3, 42, 42, 3, 2, 3, 4, 5];
        for value in values {
            h.increment(value);
        }

        // Act
        let pairs: Vec<(u32, u8)> = h.pairs_descending();

        // Assert
        let expected: Vec<(u32, u8)> = vec![(3, 3), (2, 42), (1, 5), (1, 4), (1, 2)];
        assert_eq!(pairs, expected);
    }

    #[test]
    fn test_50002_pairs_ascending() {
        // Arrange
        let mut h = Histogram::new();
        let values: [u8; 8] = [3, 42, 42, 3, 2, 3, 4, 5];
        for value in values {
            h.increment(value);
        }

        // Act
        let pairs: Vec<(u32, u8)> = h.pairs_ascending();

        // Assert
        let expected: Vec<(u32, u8)> = vec![(1, 2), (1, 4), (1, 5), (2, 42), (3, 3)];
        assert_eq!(pairs, expected);
    }

    #[test]
    fn test_60000_increment_pixel_out_of_bounds() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let image: Image = Image::try_create(2, 2, pixels).expect("image");
        let mut h = Histogram::new();

        // Arrange
        h.increment_pixel(&image, -1, -1);
        h.increment_pixel(&image, 2, 0);
        h.increment_pixel(&image, 0, 2);
        h.increment_pixel(&image, 50, 50);

        // Assert
        let pairs = h.pairs_descending();
        assert_eq!(pairs.is_empty(), true);
    }

    #[test]
    fn test_70000_number_of_counters_greater_than_zero_empty() {
        let h = Histogram::new();
        let actual: u16 = h.number_of_counters_greater_than_zero();
        assert_eq!(actual, 0);
    }

    #[test]
    fn test_70001_number_of_counters_greater_than_zero_some() {
        // Arrange
        let mut h = Histogram::new();
        let values: [u8; 8] = [3, 42, 42, 3, 2, 3, 4, 5];
        for value in values {
            h.increment(value);
        }

        // Act
        let actual: u16 = h.number_of_counters_greater_than_zero();

        // Assert
        assert_eq!(actual, 5);
    }

    #[test]
    fn test_70002_number_of_counters_greater_than_zero_all() {
        // Arrange
        let mut h = Histogram::new();
        for i in 0..=255 {
            h.increment(i);
        }

        // Act
        let actual: u16 = h.number_of_counters_greater_than_zero();

        // Assert
        assert_eq!(actual, 256);
    }

    #[test]
    fn test_80000_to_image() {
        // Arrange
        let mut h = Histogram::new();
        let values: [u8; 10] = [0, 0, 1, 1, 1, 9, 9, 5, 3, 3];
        for value in values {
            h.increment(value);
        }

        // Act
        let actual: Image = h.to_image().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 2, 2, 2, 1,
            1, 9, 3, 0, 5,
        ];
        let expected_image: Image = Image::try_create(5, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected_image);
    }

    #[test]
    fn test_80001_to_image_empty() {
        let h = Histogram::new();
        let actual: Image = h.to_image().expect("image");
        assert_eq!(actual, Image::empty());
    }

    #[test]
    fn test_90000_color_image() {
        // Arrange
        let mut h = Histogram::new();
        let values: [u8; 10] = [0, 0, 1, 1, 1, 9, 9, 5, 3, 3];
        for value in values {
            h.increment(value);
        }

        // Act
        let actual: Image = h.color_image().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 9, 3, 0, 5,
        ];
        let expected_image: Image = Image::try_create(5, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected_image);
    }

    #[test]
    fn test_90001_color_image_empty() {
        let h = Histogram::new();
        let actual: Image = h.color_image().expect("image");
        assert_eq!(actual, Image::empty());
    }

    #[test]
    fn test_100000_unused_color_some() {
        // Arrange
        let mut h = Histogram::new();
        let values: [u8; 11] = [0, 0, 2, 1, 1, 1, 9, 9, 5, 3, 3];
        for value in values {
            h.increment(value);
        }

        // Act
        let actual: Option<u8> = h.unused_color();

        // Assert
        assert_eq!(actual, Some(4));
    }

    #[test]
    fn test_100001_unused_color_none() {
        // Arrange
        let mut h = Histogram::new();
        for value in 0..=255u8 {
            h.increment(value);
        }

        // Act
        let actual: Option<u8> = h.unused_color();

        // Assert
        assert_eq!(actual, None);
    }

    #[test]
    fn test_110000_add_histogram() {
        // Arrange
        let mut h0 = Histogram::new();
        h0.increment(42);
        h0.increment(42);
        h0.increment(9);
        let mut h1 = Histogram::new();
        h1.increment(42);
        h1.increment(42);
        h1.increment(42);
        h1.increment(11);
        h1.increment(11);

        // Act
        let mut h: Histogram = h0.clone();
        h.add_histogram(&h1);
        
        // Assert
        let pairs: Vec<(u32, u8)> = h.pairs_descending();
        let expected: Vec<(u32, u8)> = vec![(5, 42), (2, 11), (1, 9)];
        assert_eq!(pairs, expected);
    }

    #[test]
    fn test_120000_intersection_histogram() {
        // Arrange
        let mut h0 = Histogram::new();
        h0.increment(42);
        h0.increment(42);
        h0.increment(5);
        h0.increment(9);
        let mut h1 = Histogram::new();
        h1.increment(42);
        h1.increment(42);
        h1.increment(42);
        h1.increment(5);
        h1.increment(0);

        // Act
        let mut h: Histogram = h0.clone();
        h.intersection_histogram(&h1);
        
        // Assert
        let pairs: Vec<(u32, u8)> = h.pairs_descending();
        let expected: Vec<(u32, u8)> = vec![(1, 42), (1, 5)];
        assert_eq!(pairs, expected);
    }

    #[test]
    fn test_130000_subtract_histogram() {
        // Arrange
        let mut h0 = Histogram::new();
        h0.increment(42);
        h0.increment(42);
        h0.increment(5);
        h0.increment(9);
        h0.increment(13);
        let mut h1 = Histogram::new();
        h1.increment(42);
        h1.increment(42);
        h1.increment(42);
        h1.increment(5);
        h1.increment(0);

        // Act
        let mut h: Histogram = h0.clone();
        h.subtract_histogram(&h1);
        
        // Assert
        let pairs: Vec<(u32, u8)> = h.pairs_descending();
        let expected: Vec<(u32, u8)> = vec![(1, 13), (1, 9)];
        assert_eq!(pairs, expected);
    }

    #[test]
    fn test_140000_is_equal_yes() {
        // Arrange
        let mut h0 = Histogram::new();
        h0.increment(42);
        h0.increment(42);
        h0.increment(3);

        let mut h1 = Histogram::new();
        h1.increment(42);
        h1.increment(42);
        h1.increment(3);

        // Act Assert
        assert_eq!(h0, h1);
    }

    #[test]
    fn test_140001_is_equal_yes() {
        // Arrange
        let mut h0 = Histogram::new();
        h0.increment(42);
        h0.increment(42);
        h0.increment(3);
        h0.increment(3);

        let mut h1 = Histogram::new();
        h1.increment(42);
        h1.increment(42);
        h1.increment(3);

        // Act Assert
        assert_ne!(h0, h1);
    }

    #[test]
    fn test_150000_hash() {
        // Arrange
        let mut h0 = Histogram::new();
        h0.increment(42);
        h0.increment(42);
        h0.increment(3);
        h0.increment(3);

        let mut h1 = Histogram::new();
        h1.increment(42);
        h1.increment(42);
        h1.increment(3);

        // Act
        let mut set = HashSet::<Histogram>::new();
        set.insert(h0);
        set.insert(h1);

        // Assert
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_150001_hash() {
        // Arrange
        let mut h0 = Histogram::new();
        h0.increment(42);

        let mut h1 = Histogram::new();
        h1.increment(42);

        // Act
        let mut set = HashSet::<Histogram>::new();
        set.insert(h0);
        set.insert(h1);

        // Assert
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_160000_sum() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(42);
        h.increment(3);
        h.increment(3);
        h.increment(3);

        // Act
        let actual: u32 = h.sum();

        // Assert
        assert_eq!(actual, 5);
    }
}
