//! Generate a dataset with basic trivial simple ARC tasks.
use super::{RandomImage, Image, ImageSize, ImageHistogram, Histogram, HtmlLog, ImageReplaceColor, ImageDenoise, ReverseColorPopularity, ImageRotate90, ImageTryCreate, ExportARCTaskJson};
use rand::prelude::Distribution;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng, Rng};
use rand::distributions::WeightedIndex;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

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
            let dataset_item: DatasetItem = Self::generate_twopixels(curriculum, random_seed, print_to_htmllog)?;
            self.dataset_items.push(dataset_item);
        }

        Ok(())
    }

    fn generate_twopixels(curriculum: Curriculum, random_seed: u64, print_to_htmllog: bool) -> anyhow::Result<DatasetItem> {
        let mut rng: StdRng = SeedableRng::seed_from_u64(random_seed);

        let mut pair_count_values: Vec<u8> = (3..=5).collect();
        pair_count_values.shuffle(&mut rng);
        let pair_count: u8 = pair_count_values[0];

        let mut available_color_values: Vec<u8> = (0..=9).collect();
        available_color_values.shuffle(&mut rng);

        if print_to_htmllog {
            HtmlLog::text(format!("pair_count: {}", pair_count));
        }
        let mut export = ExportARCTaskJson::new();
        let mut color_pairs = Vec::<String>::new();
        for i in 0..pair_count {
            let is_last_iteration: bool = i + 1 == pair_count;

            // Pick two random colors, different from each other
            let (color0, color1) = (available_color_values[0], available_color_values[1]);
            available_color_values.remove(0);
            available_color_values.remove(0);

            let mut input: Image = Image::try_create(2, 1, vec![color0, color1])?;
            input = input.rotate_cw()?;
    
            let mut output: Image = ReverseColorPopularity::apply_to_image(&input)?;
            output = output.rotate_cw()?;
            if print_to_htmllog {
                HtmlLog::compare_images(vec![input.clone(), output.clone()]);
            }
            if is_last_iteration {
                export.push_test(&input, &output);
            } else {
                export.push_train(&input, &output);
            }

            color_pairs.push(format!("{}{}", color0, color1));
        }

        let color_pairs_joined: String = color_pairs.join("_");
        let filename: String = format!("two_{}.json", color_pairs_joined);


        // let json: String = export.to_string()?;
        // println!("filename: {}", filename);
        // println!("{}", json);

        // filename = "twopixels_mixed_orientations_reverse_colors_53_91_72_08.json";
        // filename = "twopixels_rotate_53_91_72_08.json";
        // filename = "twopixels_flip_53_91_72_08.json";
        // filename = "twopixels_color0withsamesize_53_91_72_08.json";
        // filename = "twopixels_firstcolorwithsamesize_53_91_72_08.json";
        // filename = "twopixels_lastcolorwithsamesize_53_91_72_08.json";
        // filename = "twopixels_fixorientation_53_91_72_08.json";
        // Save task to file
        let basedir: PathBuf = PathBuf::from("/Users/neoneye/Downloads/output");
        let path: PathBuf = basedir.join(&filename);
        // println!("path: {}", path.display());
        export.save_json_file(&path)?;

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

    // #[test]
    fn test_10000_generate() {
        // Arrange
        let mut generate_dataset = GenerateDataset::new();

        // Act
        generate_dataset.populate(Curriculum::Small, 10, true).expect("ok");

        // Assert
    }
}
