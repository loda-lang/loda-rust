use super::{Image, ImageExtractRowColumn};

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
    pub fn counters(&self) -> &[u32; 256] {
        &self.counters
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

    pub fn increment(&mut self, index: u8) {
        let count: u32 = self.counters[index as usize];
        self.counters[index as usize] = count + 1;
    }

    pub fn set_counter_to_zero(&mut self, index: u8) {
        self.counters[index as usize] = 0;
    }

    pub fn add_histogram(&mut self, other: &Histogram) {
        for i in 0..256 {
            self.counters[i] += other.counters()[i];
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
            let b: u32 = other.counters()[i];
            let v: u32 = a.min(b).min(1);
            self.counters[i] = v;
        }
    }

    /// Clear counters where the other histogram has non-zero counters.
    /// 
    /// Performs an operation similar to: `self` AND NOT `other`.
    pub fn subtract_histogram(&mut self, other: &Histogram) {
        for i in 0..256 {
            if other.counters()[i] > 0 {
                self.counters[i] = 0;
            }
        }
    }

    #[allow(dead_code)]
    pub fn most_popular_pair(&self) -> Option<(u8, u32)> {
        let mut found_count: u32 = 0;
        let mut found_index: usize = 0;
        for (index, number_of_occurences) in self.counters.iter().enumerate() {
            if *number_of_occurences > found_count {
                found_count = *number_of_occurences;
                found_index = index;
            }
        }
        if found_count == 0 {
            return None;
        }
        let color_value: u8 = (found_index & 255) as u8;
        Some((color_value, found_count))
    }

    #[allow(dead_code)]
    pub fn most_popular_color(&self) -> Option<u8> {
        if let Some((color, _count)) = self.most_popular_pair() {
            return Some(color);
        }
        None
    }

    #[allow(dead_code)]
    pub fn most_popular_count(&self) -> Option<u32> {
        if let Some((_color, count)) = self.most_popular_pair() {
            return Some(count);
        }
        None
    }

    #[allow(dead_code)]
    pub fn least_popular_pair(&self) -> Option<(u8, u32)> {
        let mut found_count: u32 = u32::MAX;
        let mut found_index: usize = 0;
        for (index, number_of_occurences) in self.counters.iter().enumerate() {
            if *number_of_occurences == 0 {
                continue;
            }
            if *number_of_occurences < found_count {
                found_count = *number_of_occurences;
                found_index = index;
            }
        }
        if found_count == u32::MAX {
            return None;
        }
        let color_value: u8 = (found_index & 255) as u8;
        Some((color_value, found_count))
    }

    #[allow(dead_code)]
    pub fn least_popular_color(&self) -> Option<u8> {
        if let Some((color, _count)) = self.least_popular_pair() {
            return Some(color);
        }
        None
    }

    #[allow(dead_code)]
    pub fn least_popular_count(&self) -> Option<u32> {
        if let Some((_color, count)) = self.least_popular_pair() {
            return Some(count);
        }
        None
    }

    /// The least frequent occurring comes first.
    /// 
    /// The medium frequent occurring comes middle.
    /// 
    /// The most frequent occurring comes last.
    pub fn pairs_ascending(&self) -> Vec<(u32,u8)> {
        let mut pairs = Vec::<(u32, u8)>::with_capacity(256);
        for (index, number_of_occurences) in self.counters.iter().enumerate() {
            if *number_of_occurences > 0 {
                pairs.push((*number_of_occurences, (index & 255) as u8));
            }
        }
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
    pub fn number_of_counters_greater_than_zero(&self) -> u32 {
        let mut count: u32 = 0;
        for number_of_occurences in &self.counters {
            if *number_of_occurences > 0 {
                count += 1;
            }
        }
        count
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use std::collections::HashSet;

    #[test]
    fn test_10000_increment() {
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
    fn test_20000_most_popular_pair_some() {
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
    fn test_20001_most_popular_pair_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color_count: Option<(u8, u32)> = h.most_popular_pair();

        // Assert
        assert_eq!(color_count, None);
    }

    #[test]
    fn test_20002_most_popular_color_some() {
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
    fn test_20003_most_popular_color_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color: Option<u8> = h.most_popular_color();

        // Assert
        assert_eq!(color, None);
    }

    #[test]
    fn test_20004_most_popular_count_some() {
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
    fn test_20005_most_popular_count_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let count: Option<u32> = h.most_popular_count();

        // Assert
        assert_eq!(count, None);
    }

    #[test]
    fn test_30000_least_popular_pair_some() {
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
    fn test_30001_least_popular_pair_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color_count: Option<(u8, u32)> = h.least_popular_pair();

        // Assert
        assert_eq!(color_count, None);
    }

    #[test]
    fn test_30002_least_popular_color_some() {
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
    fn test_30003_least_popular_color_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color: Option<u8> = h.least_popular_color();

        // Assert
        assert_eq!(color, None);
    }

    #[test]
    fn test_30004_least_popular_count_some() {
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
    fn test_30005_least_popular_count_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let count: Option<u32> = h.least_popular_count();

        // Assert
        assert_eq!(count, None);
    }

    #[test]
    fn test_40000_pairs_descending() {
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
    fn test_40001_pairs_ascending() {
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
    fn test_50000_increment_pixel_out_of_bounds() {
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
    fn test_60000_number_of_counters_greater_than_zero_empty() {
        let h = Histogram::new();
        let actual: u32 = h.number_of_counters_greater_than_zero();
        assert_eq!(actual, 0);
    }

    #[test]
    fn test_60001_number_of_counters_greater_than_zero_some() {
        // Arrange
        let mut h = Histogram::new();
        let values: [u8; 8] = [3, 42, 42, 3, 2, 3, 4, 5];
        for value in values {
            h.increment(value);
        }

        // Act
        let actual: u32 = h.number_of_counters_greater_than_zero();

        // Assert
        assert_eq!(actual, 5);
    }

    #[test]
    fn test_60002_number_of_counters_greater_than_zero_all() {
        // Arrange
        let mut h = Histogram::new();
        for i in 0..=255 {
            h.increment(i);
        }

        // Act
        let actual: u32 = h.number_of_counters_greater_than_zero();

        // Assert
        assert_eq!(actual, 256);
    }

    #[test]
    fn test_70000_to_image() {
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
    fn test_70001_to_image_empty() {
        let h = Histogram::new();
        let actual: Image = h.to_image().expect("image");
        assert_eq!(actual, Image::empty());
    }

    #[test]
    fn test_80000_color_image() {
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
    fn test_80001_color_image_empty() {
        let h = Histogram::new();
        let actual: Image = h.color_image().expect("image");
        assert_eq!(actual, Image::empty());
    }

    #[test]
    fn test_90000_unused_color_some() {
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
    fn test_90001_unused_color_none() {
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
    fn test_100000_add_histogram() {
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
    fn test_110000_intersection_histogram() {
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
    fn test_120000_subtract_histogram() {
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
    fn test_130000_is_equal_yes() {
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
    fn test_130001_is_equal_yes() {
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
    fn test_140000_hash() {
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
    fn test_140001_hash() {
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
}
