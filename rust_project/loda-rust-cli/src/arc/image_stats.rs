//! ImageStats - Measure `directionality` of an image.
//! 
//! measure noise and sameness of pixels in different directions:
//! * no particular direction
//! * horizontal
//! * vertical
//! 
//! There may be one direction with a low sigma, and another direction with a high sigma.
//! It may indicate that the image has a a particular orientation.
//! 
//! If the mean is near `1.0` with a low sigma, then the image is noisy.
//! 
//! If the mean is near `3.0` with a low sigma, then the image has larger clusters with pixels of the same color.
//! 
//! If the mean is near `3.0` with a high sigma, then the image has larger clusters with pixels of the same color,
//! and there may be lots of noisy pixels.
use super::{Image, ImageNgram, RecordTrigram};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Stats {
    pub n: u64,
    pub mean: f64,
    pub sigma: f64,
}

impl Stats {
    pub fn new(values: &Vec<u8>) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        let n: u64 = values.len() as u64;
        let n_f64: f64 = n as f64;
        let sum: usize = values.iter().map(|value| *value as usize).sum();
        let mean: f64 = sum as f64 / n_f64;
        let sigma_squared: f64 = values.iter().map(|value| (*value as f64 - mean).powi(2)).sum::<f64>() / n_f64;
        let sigma: f64 = sigma_squared.sqrt();
        let instance = Self {
            n,
            mean,
            sigma,
        };
        Some(instance)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageStatsMode {
    TrigramAll,
    TrigramHorizontal,
    TrigramVertical,

    // Future experiments
    // TrigramDiagonalA,
    // TrigramDiagonalB,
    // TrigramColor { color: u8 }, a particular color may be more noisy than other colors
    // Convolution2x2,
    // Convolution3x3,
}

#[derive(Debug, Clone)]
pub struct ImageStats {
    mode_to_stats: HashMap::<ImageStatsMode, Stats>,
}

impl ImageStats {
    pub fn new(image: &Image) -> anyhow::Result<Self> {
        let trigram_y: Vec<RecordTrigram> = match image.trigram_y() {
            Ok(value) => value,
            Err(_) => vec!()
        };
        let trigram_x: Vec<RecordTrigram> = match image.trigram_x() {
            Ok(value) => value,
            Err(_) => vec!()
        };
        let trigram_x_concat_y: Vec<RecordTrigram> = trigram_x.iter().chain(trigram_y.iter()).cloned().collect();

        let mut mode_to_stats = HashMap::<ImageStatsMode, Stats>::new();
        let modes = [
            ImageStatsMode::TrigramAll, 
            ImageStatsMode::TrigramHorizontal, 
            ImageStatsMode::TrigramVertical
        ];
        for mode in &modes {
            let trigram_vec: &Vec<RecordTrigram> = match mode {
                ImageStatsMode::TrigramAll => &trigram_x_concat_y,
                ImageStatsMode::TrigramHorizontal => &trigram_x,
                ImageStatsMode::TrigramVertical => &trigram_y,
            };
            if let Ok(stats) = Self::stats_from_trigrams(&trigram_vec) {
                mode_to_stats.insert(mode.clone(), stats);
            }
        }
        let instance = Self {
            mode_to_stats,
        };
        Ok(instance)
    }

    pub fn get(&self, mode: &ImageStatsMode) -> Option<&Stats> {
        self.mode_to_stats.get(mode)
    }

    fn stats_from_trigrams(trigram_vec: &Vec<RecordTrigram>) -> anyhow::Result<Stats> {
        if trigram_vec.is_empty() {
            return Err(anyhow::anyhow!("stats_from_trigrams, expected 1 or more trigram records"));
        }
        let mut values = Vec::<u8>::new();
        for record in trigram_vec {
            let same01 = record.word0 == record.word1;
            let same12 = record.word1 == record.word2;
            let number_of_adjacent_pixels_with_same_value: u8;
            if same01 && same12 {
                // 3 adjacent pixels with same color
                number_of_adjacent_pixels_with_same_value = 3;
            } else {
                if same01 || same12 {
                    // 2 adjacent pixels with same color, and 1 pixel with different color
                    number_of_adjacent_pixels_with_same_value = 2;
                } else {
                    // no adjacent pixels same color
                    number_of_adjacent_pixels_with_same_value = 1;
                }
            }
            for _ in 0..record.count {
                values.push(number_of_adjacent_pixels_with_same_value);
            }
        }
        let stats: Stats = match Stats::new(&values) {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("there must be 1 or more trigram records, cannot compute stats"));
            }
        };
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_standard_deviation() {
        // Arrange
        let v: Vec::<u8> = vec![2, 4, 4, 4, 5, 5, 7, 9];

        // Act
        let actual: Stats = Stats::new(&v).expect("ok");

        // Assert
        assert_eq!(actual.n, 8);
        assert_float_absolute_eq!(actual.mean, 5.0, 0.001);
        assert_float_absolute_eq!(actual.sigma, 2.0, 0.001);
    }

    #[test]
    fn test_20000_stats_from_trigrams() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 2, 2, 5, 2, 2, 5,
            5, 9, 2, 5, 2, 2, 5,
            3, 5, 5, 3, 5, 5, 3,
            5, 2, 2, 5, 2, 9, 9,
            5, 2, 2, 5, 2, 9, 9,
            3, 5, 5, 3, 5, 9, 9,
        ];
        let input: Image = Image::try_create(7, 6, pixels).expect("image");
        let trigram_vec: Vec<RecordTrigram> = input.trigram_y().expect("ok");

        // Act
        let actual: Stats = ImageStats::stats_from_trigrams(&trigram_vec).expect("ok");
        assert_eq!(actual.n, 28);
        assert_float_absolute_eq!(actual.mean, 1.78571, 0.001);
        assert_float_absolute_eq!(actual.sigma, 0.55787, 0.001);
    }

    #[test]
    fn test_30000_imagestats_new() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 2, 2, 5, 2, 2, 5,
            5, 9, 2, 5, 2, 2, 5,
            3, 5, 5, 3, 5, 5, 3,
            5, 2, 2, 5, 2, 9, 9,
            5, 2, 2, 5, 2, 9, 9,
            3, 5, 5, 3, 5, 9, 9,
        ];
        let input: Image = Image::try_create(7, 6, pixels).expect("image");

        // Act
        let actual: ImageStats = ImageStats::new(&input).expect("ok");

        // Assert
        assert_eq!(actual.get(&ImageStatsMode::TrigramAll).is_some(), true);
        assert_eq!(actual.get(&ImageStatsMode::TrigramHorizontal).is_some(), true);
        assert_eq!(actual.get(&ImageStatsMode::TrigramVertical).is_some(), true);
    }
}
