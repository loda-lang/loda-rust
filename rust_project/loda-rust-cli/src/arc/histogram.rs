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
        let pixel_value: u8 = image.get(x, y).unwrap_or(255);
        self.increment(pixel_value);
    }

    pub fn increment(&mut self, index: u8) {
        let count: u32 = self.counters[index as usize];
        self.counters[index as usize] = count + 1;
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

    /// The most frequent occurring comes first.
    /// 
    /// The medium frequent occuring comes middle.
    /// 
    /// The least frequent are at the end.
    pub fn sorted_pairs(&self) -> Vec<(u32,u8)> {
        let mut pairs = Vec::<(u32, u8)>::with_capacity(256);
        for (index, number_of_occurences) in self.counters.iter().enumerate() {
            if *number_of_occurences > 0 {
                pairs.push((*number_of_occurences, (index & 255) as u8));
            }
        }
        pairs.sort();
        pairs.reverse();
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_30000_sorted_pairs() {
        // Arrange
        let mut h = Histogram::new();
        let values: [u8; 8] = [3, 42, 42, 3, 2, 3, 4, 5];
        for value in values {
            h.increment(value);
        }

        // Act
        let pairs: Vec<(u32, u8)> = h.sorted_pairs();

        // Assert
        let expected: Vec<(u32, u8)> = vec![(3, 3), (2, 42), (1, 5), (1, 4), (1, 2)];
        assert_eq!(pairs, expected);
    }
}
