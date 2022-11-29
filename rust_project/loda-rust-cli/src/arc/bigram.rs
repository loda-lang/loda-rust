use super::{Bitmap, BitmapRotate};
use std::collections::HashMap;

type HistogramBigramKey = (u8,u8);

#[derive(Clone, Debug, PartialEq)]
pub struct RecordBigram {
    pub count: u32,
    pub word0: u8,
    pub word1: u8,
}

pub trait BitmapBigram {
    fn bigram_x(&self) -> anyhow::Result<Vec<RecordBigram>>;
    fn bigram_y(&self) -> anyhow::Result<Vec<RecordBigram>>;
}

impl BitmapBigram for Bitmap {
    
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
            // Bigrams considering all the pixels in a row
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

        // Move the most frequently occuring items to the top
        // Move the lesser used items to the bottom
        records.sort_unstable_by_key(|item| (item.count, item.word0.clone(), item.word1.clone()));
        records.reverse();
        
        Ok(records)
    }

    fn bigram_y(&self) -> anyhow::Result<Vec<RecordBigram>> {
        let bitmap: Bitmap = self.rotate_cw()?;
        bitmap.bigram_x()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::BitmapTryCreate;

    #[test]
    fn test_10000_bigram_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 1,
            2, 1, 2,
            1, 2, 1,
            9, 1, 2,
        ];
        let input: Bitmap = Bitmap::try_create(3, 4, pixels).expect("bitmap");

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
    fn test_20000_bigram_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 1, 9,
            2, 1, 2, 1,
            1, 2, 1, 2,
        ];
        let input: Bitmap = Bitmap::try_create(4, 3, pixels).expect("bitmap");

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
}
