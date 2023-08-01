use super::{Image, ImageRotate, ImageSymmetry};
use std::collections::HashMap;

type HistogramBigramKey = (u8,u8);
type HistogramTrigramKey = (u8,u8,u8);

#[derive(Clone, Debug, PartialEq)]
pub struct RecordBigram {
    pub count: u32,
    pub word0: u8,
    pub word1: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecordTrigram {
    pub count: u32,
    pub word0: u8,
    pub word1: u8,
    pub word2: u8,
}

pub trait ImageNgram {
    /// Horizontal bigrams from left to right
    fn bigram_x(&self) -> anyhow::Result<Vec<RecordBigram>>;

    /// Vertical bigrams from top to bottom
    fn bigram_y(&self) -> anyhow::Result<Vec<RecordBigram>>;

    /// Horizontal trigrams from left to right
    fn trigram_x(&self) -> anyhow::Result<Vec<RecordTrigram>>;

    /// Vertical trigrams from top to bottom
    fn trigram_y(&self) -> anyhow::Result<Vec<RecordTrigram>>;

    /// Diagonal trigrams from top-left to bottom-right
    /// 
    /// Known problem: This does not consider the corner pixels where there are not enough pixels to form a trigram.
    /// That is the bottom-left corner and the top-right corner.
    fn trigram_diagonal_a(&self) -> anyhow::Result<Vec<RecordTrigram>>;

    /// Diagonal trigrams from top-right to bottom-left
    /// 
    /// Known problem: This does not consider the corner pixels where there are not enough pixels to form a trigram.
    /// That is the top-left corner and the bottom-right corner.
    fn trigram_diagonal_b(&self) -> anyhow::Result<Vec<RecordTrigram>>;
}

impl ImageNgram for Image {
    
    fn bigram_x(&self) -> anyhow::Result<Vec<RecordBigram>> {
        let width: u8 = self.width();
        let height: u8 = self.height();
        if width < 2 || height < 1 {
            return Err(anyhow::anyhow!("too small bitmap, must be 2x1 or bigger"));
        }
        let width: i32 = self.width() as i32;
        let height: i32 = self.height() as i32;

        // Loop over rows
        let mut keys = Vec::<HistogramBigramKey>::new();
        for y in 0..height {
            let mut prev_word: u8 = 255;
            for x in 0..width {
                let word1: u8 = self.get(x, y).unwrap_or(255);
                let word0: u8 = prev_word;
                prev_word = word1.clone();
                if x == 0 {
                    continue;
                }
                // Construct bigram with two side-by-side pixels
                let key: HistogramBigramKey = (word0, word1.clone());
                keys.push(key);
            }
        }

        // Count the most frequent bigrams
        let mut histogram_bigram: HashMap<HistogramBigramKey,u32> = HashMap::new();
        for key in keys {
            let counter = histogram_bigram.entry(key).or_insert(0);
            *counter += 1;
        }

        // Convert from dictionary to array
        let mut records = Vec::<RecordBigram>::new();
        for (histogram_key, histogram_count) in &histogram_bigram {
            let record = RecordBigram {
                count: *histogram_count,
                word0: histogram_key.0,
                word1: histogram_key.1
            };
            records.push(record);
        }

        // Move the most frequently occurring items to the top
        // Move the lesser used items to the bottom
        records.sort_unstable_by_key(|item| (item.count, item.word0.clone(), item.word1.clone()));
        records.reverse();
        
        Ok(records)
    }

    fn bigram_y(&self) -> anyhow::Result<Vec<RecordBigram>> {
        let image: Image = self.rotate_cw()?;
        image.bigram_x()
    }

    fn trigram_x(&self) -> anyhow::Result<Vec<RecordTrigram>> {
        let width: u8 = self.width();
        let height: u8 = self.height();
        if width < 3 || height < 1 {
            return Err(anyhow::anyhow!("too small bitmap, must be 3x1 or bigger"));
        }
        let width: i32 = self.width() as i32;
        let height: i32 = self.height() as i32;

        // Loop over rows
        let mut keys = Vec::<HistogramTrigramKey>::new();
        for y in 0..height {
            let mut prev_prev_word: u8 = 255;
            let mut prev_word: u8 = 255;
            for x in 0..width {
                let word0: u8 = prev_prev_word;
                let word1: u8 = prev_word;
                let word2: u8 = self.get(x, y).unwrap_or(255);
                prev_prev_word = prev_word;
                prev_word = word2;
                if x < 2 {
                    continue;
                }
                // Construct trigram with three side-by-side pixels
                let key: HistogramTrigramKey = (word0, word1, word2);
                keys.push(key);
            }
        }
        
        let records: Vec<RecordTrigram> = RecordTrigram::count_and_sort(keys);
        Ok(records)
    }

    fn trigram_y(&self) -> anyhow::Result<Vec<RecordTrigram>> {
        let image: Image = self.rotate_cw()?;
        image.trigram_x()
    }

    fn trigram_diagonal_a(&self) -> anyhow::Result<Vec<RecordTrigram>> {
        let width: u8 = self.width();
        let height: u8 = self.height();
        if width < 3 || height < 3 {
            return Err(anyhow::anyhow!("too small bitmap, must be 3x3 or bigger"));
        }
        let width: i32 = self.width() as i32;
        let height: i32 = self.height() as i32;

        // Loop over the pixels
        let mut keys = Vec::<HistogramTrigramKey>::new();
        for y in 0..height {
            for x in 0..width {
                if x < 2 || y < 2 {
                    continue;
                }
                let word0: u8 = self.get(x - 2, y - 2).unwrap_or(255);
                let word1: u8 = self.get(x - 1, y - 1).unwrap_or(255);
                let word2: u8 = self.get(x, y).unwrap_or(255);
                // Construct trigram with three diagonal pixels
                let key: HistogramTrigramKey = (word0, word1, word2);
                keys.push(key);
            }
        }
        
        let records: Vec<RecordTrigram> = RecordTrigram::count_and_sort(keys);
        Ok(records)
    }

    fn trigram_diagonal_b(&self) -> anyhow::Result<Vec<RecordTrigram>> {
        let image: Image = self.flip_x()?;
        image.trigram_diagonal_a()
    }
}

impl RecordTrigram {
    /// Count the frequency of each trigram, and sort by popularity.
    fn count_and_sort(keys: Vec<HistogramTrigramKey>) -> Vec<RecordTrigram> {
        // Count the most frequent trigrams
        let mut histogram_trigram: HashMap<HistogramTrigramKey,u32> = HashMap::new();
        for key in keys {
            let counter = histogram_trigram.entry(key).or_insert(0);
            *counter += 1;
        }

        // Convert from dictionary to array
        let mut records = Vec::<RecordTrigram>::new();
        for (histogram_key, histogram_count) in &histogram_trigram {
            let record = RecordTrigram {
                count: *histogram_count,
                word0: histogram_key.0,
                word1: histogram_key.1,
                word2: histogram_key.2,
            };
            records.push(record);
        }

        // Move the most frequently occurring items to the top
        // Move the lesser used items to the bottom
        records.sort_unstable_by_key(|item| (item.count, item.word0.clone(), item.word1.clone(), item.word2.clone()));
        records.reverse();
        
        records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_bigram_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 1,
            2, 1, 2,
            1, 2, 1,
            9, 1, 2,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");

        // Act
        let bigrams: Vec<RecordBigram> = input.bigram_x().expect("bigrams");

        // Assert
        let expected: Vec<RecordBigram> = vec![
            RecordBigram { count: 4, word0: 1, word1: 2 },
            RecordBigram { count: 3, word0: 2, word1: 1 },
            RecordBigram { count: 1, word0: 9, word1: 1 },
        ];
        assert_eq!(bigrams, expected);
    }

    #[test]
    fn test_10001_bigram_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 9,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let bigrams: Vec<RecordBigram> = input.bigram_y().expect("bigrams");

        // Assert
        let expected: Vec<RecordBigram> = vec![
            RecordBigram { count: 4, word0: 1, word1: 2 },
            RecordBigram { count: 3, word0: 2, word1: 1 },
            RecordBigram { count: 1, word0: 9, word1: 1 },
        ];
        assert_eq!(bigrams, expected);
    }

    #[test]
    fn test_20000_trigram_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 2,
            9, 1, 2, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let trigrams: Vec<RecordTrigram> = input.trigram_x().expect("trigrams");

        // Assert
        let expected: Vec<RecordTrigram> = vec![
            RecordTrigram { count: 4, word0: 1, word1: 2, word2: 1 },
            RecordTrigram { count: 3, word0: 2, word1: 1, word2: 2 },
            RecordTrigram { count: 1, word0: 9, word1: 1, word2: 2 },
        ];
        assert_eq!(trigrams, expected);
    }

    #[test]
    fn test_20001_trigram_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            2, 1, 2, 1,
            1, 2, 1, 2,
            2, 1, 2, 1,
            1, 2, 1, 9,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let trigrams: Vec<RecordTrigram> = input.trigram_y().expect("trigrams");

        // Assert
        let expected: Vec<RecordTrigram> = vec![
            RecordTrigram { count: 4, word0: 1, word1: 2, word2: 1 },
            RecordTrigram { count: 3, word0: 2, word1: 1, word2: 2 },
            RecordTrigram { count: 1, word0: 9, word1: 1, word2: 2 },
        ];
        assert_eq!(trigrams, expected);
    }

    #[test]
    fn test_30000_trigram_diagonal_a() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 5, 6, 7,
            3, 4, 5, 6,
            2, 3, 4, 5,
            1, 2, 3, 4,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let trigrams: Vec<RecordTrigram> = input.trigram_diagonal_a().expect("trigrams");

        // Assert
        let expected: Vec<RecordTrigram> = vec![
            RecordTrigram { count: 2, word0: 4, word1: 4, word2: 4 },
            RecordTrigram { count: 1, word0: 5, word1: 5, word2: 5 },
            RecordTrigram { count: 1, word0: 3, word1: 3, word2: 3 },
        ];
        assert_eq!(trigrams, expected);
    }

    #[test]
    fn test_30001_trigram_diagonal_a() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 5, 6, 7, 8,
            3, 4, 5, 6, 7,
            2, 3, 4, 5, 6,
            1, 2, 3, 4, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let trigrams: Vec<RecordTrigram> = input.trigram_diagonal_a().expect("trigrams");

        // Assert
        let expected: Vec<RecordTrigram> = vec![
            RecordTrigram { count: 2, word0: 5, word1: 5, word2: 5 },
            RecordTrigram { count: 2, word0: 4, word1: 4, word2: 4 },
            RecordTrigram { count: 1, word0: 6, word1: 6, word2: 6 },
            RecordTrigram { count: 1, word0: 3, word1: 3, word2: 3 },
        ];
        assert_eq!(trigrams, expected);
    }

    #[test]
    fn test_30002_trigram_diagonal_b() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5,
            2, 3, 4, 5, 6,
            3, 4, 5, 6, 7,
            4, 5, 6, 7, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let trigrams: Vec<RecordTrigram> = input.trigram_diagonal_b().expect("trigrams");

        // Assert
        let expected: Vec<RecordTrigram> = vec![
            RecordTrigram { count: 2, word0: 5, word1: 5, word2: 5 },
            RecordTrigram { count: 2, word0: 4, word1: 4, word2: 4 },
            RecordTrigram { count: 1, word0: 6, word1: 6, word2: 6 },
            RecordTrigram { count: 1, word0: 3, word1: 3, word2: 3 },
        ];
        assert_eq!(trigrams, expected);
    }
}
