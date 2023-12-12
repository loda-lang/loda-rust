//! Generate a dataset with basic trivial simple ARC tasks.
use super::{RandomImage, Image, ImageSize, ImageHistogram, Histogram, HtmlLog, ImageReplaceColor, ImageDenoise, ReverseColorPopularity, ImageRotate90, ImageTryCreate};
use rand::prelude::Distribution;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng, Rng};
use rand::distributions::WeightedIndex;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
enum Curriculum {
    Small,
    SmallMedium,
    SmallMediumBig,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct DatasetItem {
    curriculum: Curriculum,
    text: String,
}

#[allow(dead_code)]
pub struct GenerateDataset {
    dataset_items: Vec<DatasetItem>,
}

impl GenerateDataset {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            dataset_items: vec!(),
        }
    }

    #[allow(dead_code)]
    fn populate(&mut self, curriculum: Curriculum, number_of_items: u32, print_to_htmllog: bool) -> anyhow::Result<()> {

        for i in 0..number_of_items {
            if print_to_htmllog {
                HtmlLog::text(format!("iteration: {}", i));
            }
            if i % 100000 == 0 {
                println!("iteration: {} number_of_items: {} curriculum: {:?}", i, number_of_items, curriculum);
            }
            let random_seed: u64 = i as u64;
            let dataset_item: DatasetItem = Self::generate(curriculum, random_seed, print_to_htmllog)?;
            self.dataset_items.push(dataset_item);
        }

        Ok(())
    }

    fn generate(curriculum: Curriculum, random_seed: u64, print_to_htmllog: bool) -> anyhow::Result<DatasetItem> {
        let mut rng: StdRng = SeedableRng::seed_from_u64(random_seed);

        let mut pair_count_values: Vec<u8> = (3..=5).collect();
        pair_count_values.shuffle(&mut rng);
        let pair_count: u8 = pair_count_values[0];

        let mut available_color_values: Vec<u8> = (0..=9).collect();
        available_color_values.shuffle(&mut rng);

        HtmlLog::text(format!("pair_count: {}", pair_count));
        for _ in 0..pair_count {

            // Pick two random colors, different from each other
            let (color0, color1) = (available_color_values[0], available_color_values[1]);
            available_color_values.remove(0);
            available_color_values.remove(0);

            let mut input: Image = Image::try_create(2, 1, vec![color0, color1])?;
            input = input.rotate_cw()?;
    
            let mut output: Image = ReverseColorPopularity::apply_to_image(&input)?;
            output = output.rotate_cw()?;
            // HtmlLog::image(&output);
            HtmlLog::compare_images(vec![input.clone(), output.clone()]);
        }


        let mut dataset_item: DatasetItem = DatasetItem {
            curriculum,
            text: String::new(),
        };
        Ok(dataset_item)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use std::path::PathBuf;

    #[test]
    fn test_10000_generate() {
        // Arrange
        let mut generate_dataset = GenerateDataset::new();

        // Act
        generate_dataset.populate(Curriculum::Small, 10, true).expect("ok");

        // Assert
    }
}
