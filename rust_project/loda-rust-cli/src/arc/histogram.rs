use super::Image;

/// Histogram with 256 counters
#[derive(Clone, Debug)]
pub struct Histogram {
    counters: [u32; 256],
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            counters: [0; 256],
        }
    }

    pub fn counters(&self) -> &[u32; 256] {
        &self.counters
    }

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

    pub fn most_popular(&self) -> Option<u8> {
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
        Some((found_index & 255) as u8)
    }

    pub fn least_popular(&self) -> Option<u8> {
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
        Some((found_index & 255) as u8)
    }

    /// The least frequent occuring comes first.
    /// 
    /// The medium frequent occuring comes middle.
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
    /// The medium frequent occuring comes middle.
    /// 
    /// The least frequent occurring are at the end.
    pub fn pairs_descending(&self) -> Vec<(u32,u8)> {
        let mut pairs = self.pairs_ascending();
        pairs.reverse();
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

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
    fn test_20000_most_popular_some() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(42);
        h.increment(3);
        h.increment(3);
        h.increment(3);
        h.increment(2);

        // Act
        let color: Option<u8> = h.most_popular();

        // Assert
        assert_eq!(color, Some(3));
    }

    #[test]
    fn test_20001_most_popular_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color: Option<u8> = h.most_popular();

        // Assert
        assert_eq!(color, None);
    }

    #[test]
    fn test_30000_least_popular_some() {
        // Arrange
        let mut h = Histogram::new();
        h.increment(42);
        h.increment(3);
        h.increment(2);
        h.increment(3);
        h.increment(42);
        h.increment(3);

        // Act
        let color: Option<u8> = h.least_popular();

        // Assert
        assert_eq!(color, Some(2));
    }

    #[test]
    fn test_30001_least_popular_none() {
        // Arrange
        let h = Histogram::new();

        // Act
        let color: Option<u8> = h.least_popular();

        // Assert
        assert_eq!(color, None);
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
}
