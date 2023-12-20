//! Generate a dataset with basic trivial simple ARC tasks.
use super::{Image, HtmlLog, ReverseColorPopularity, ImageRotate90, ImageTryCreate, ExportARCTaskJson};
use rand::Rng;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};
use serde::Serialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
enum Curriculum {
    Small,
    SmallMedium,
    SmallMediumBig,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
enum TwoPixelTransformation {
    LandscapeFlip,
    LandscapeRotateCW,
    LandscapeRotateCCW,
    PortraitFlip,
    PortraitRotateCW,
    PortraitRotateCCW,

    // Ideas for more transformations
    // MixedOrientationFlip,
    // MixedOrientationRotateCW,
    // MixedOrientationRotateCCW,
    // MixedOrientationOutputSolidColor,
    // MixedOrientationRotateOutputSolidColor,
    // LandscapeInputIsOneSolidColorButOutputIsTwoDifferentColors, // needs more than 5 training pairs.
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

            let transformation: TwoPixelTransformation = match i % 6 {
                0 => TwoPixelTransformation::LandscapeFlip,
                1 => TwoPixelTransformation::LandscapeRotateCW,
                2 => TwoPixelTransformation::LandscapeRotateCCW,
                3 => TwoPixelTransformation::PortraitFlip,
                4 => TwoPixelTransformation::PortraitRotateCW,
                5 => TwoPixelTransformation::PortraitRotateCCW,
                _ => unreachable!(),
            };

            let dataset_item: DatasetItem = Self::generate_twopixels(transformation, random_seed, print_to_htmllog)?;
            self.dataset_items.push(dataset_item);
        }

        Ok(())
    }

    /// The two colors inside each pair are always different.
    /// 
    /// The pairs are always different from each other.
    /// 
    /// Each color is only used once.
    fn five_unique_color_pairs(rng: &mut StdRng) -> Vec<(u8, u8)> {
        let mut colors: Vec<u8> = (0..=9).collect();
        colors.shuffle(rng);
        let mut pairs = Vec::<(u8, u8)>::new();
        while colors.len() >= 2 {
            let color0: u8 = colors.remove(0);
            let color1: u8 = colors.remove(0);
            pairs.push((color0, color1));
        }
        assert!(pairs.len() == 5);
        pairs
    }

    fn generate_twopixels(transformation: TwoPixelTransformation, random_seed: u64, print_to_htmllog: bool) -> anyhow::Result<DatasetItem> {
        let mut rng: StdRng = SeedableRng::seed_from_u64(random_seed);

        let pair_count_values: Vec<(u8, u8)> = vec![
            (2, 1), (2, 2), (2, 3), (3, 1), (3, 2), (3, 3), (4, 1), (4, 2), (4, 3)
        ];
        let (train_count, test_count) = *pair_count_values.choose(&mut rng).unwrap();
        let pair_count: u8 = train_count + test_count;

        // There are max 4 `train` pairs. Since there are 5 unique color pairs, we can be 
        // certain that the `train` pairs have different colors from each other.
        let mut available_color_pairs: Vec<(u8, u8)> = Self::five_unique_color_pairs(&mut rng);

        // Fill up with more random colors until there are enough color pairs.
        // The `test` pairs comes last, so it's ok if they are not as unique as the `train` pairs.
        while available_color_pairs.len() < pair_count as usize {
            let color0: u8 = rng.gen_range(0..=9);
            let color1: u8 = rng.gen_range(0..=9);
            if color0 == color1 {
                continue;
            }
            if available_color_pairs.contains(&(color0, color1)) {
                continue;
            }
            available_color_pairs.push((color0, color1));
        }

        if print_to_htmllog {
            HtmlLog::text(format!("pair_count: {}", pair_count));
        }
        let mut export = ExportARCTaskJson::new();
        let mut color_pairs = Vec::<String>::new();
        for i in 0..pair_count {
            let is_train: bool = i < train_count;

            // Pick two random colors, different from each other
            let (color0, color1) = available_color_pairs.remove(0);

            // Future experiments
            // If it's a test pair, then pick 2 colors that are the same, so it's ambiguous.

            let input_landscape: Image = Image::try_create(2, 1, vec![color0, color1])?;
            let input_portrait: Image = input_landscape.rotate_cw()?;

            let input: &Image = match transformation {
                TwoPixelTransformation::LandscapeFlip => &input_landscape,
                TwoPixelTransformation::LandscapeRotateCW => &input_landscape,
                TwoPixelTransformation::LandscapeRotateCCW => &input_landscape,
                TwoPixelTransformation::PortraitFlip => &input_portrait,
                TwoPixelTransformation::PortraitRotateCW => &input_portrait,
                TwoPixelTransformation::PortraitRotateCCW => &input_portrait,
            };

            let output_reversed: Image = ReverseColorPopularity::apply_to_image(input)?;
            let output_rotate_ccw: Image = input.rotate_ccw()?;
            let output_rotate_cw: Image = input.rotate_cw()?;

            let output: &Image = match transformation {
                TwoPixelTransformation::LandscapeFlip => &output_reversed,
                TwoPixelTransformation::PortraitFlip => &output_reversed,
                TwoPixelTransformation::LandscapeRotateCW => &output_rotate_cw,
                TwoPixelTransformation::PortraitRotateCW => &output_rotate_cw,
                TwoPixelTransformation::LandscapeRotateCCW => &output_rotate_ccw,
                TwoPixelTransformation::PortraitRotateCCW => &output_rotate_ccw,
            };

            if print_to_htmllog {
                HtmlLog::compare_images(vec![input.clone(), output.clone()]);
            }
            assert!(input != output, "input and output must be different");
            if is_train {
                export.push_train(&input, &output);
            } else {
                export.push_test(&input, &output);
            }

            color_pairs.push(format!("{}{}", color0, color1));
        }

        let transformation_name: &str = match transformation {
            TwoPixelTransformation::LandscapeFlip => "landscape_flip",
            TwoPixelTransformation::LandscapeRotateCW => "landscape_cw",
            TwoPixelTransformation::LandscapeRotateCCW => "landscape_ccw",
            TwoPixelTransformation::PortraitFlip => "portrait_flip",
            TwoPixelTransformation::PortraitRotateCW => "portrait_cw",
            TwoPixelTransformation::PortraitRotateCCW => "portrait_ccw",
        };

        let color_pairs_joined: String = color_pairs.join("_");
        let filename: String = format!("two_{}_{}.json", transformation_name, color_pairs_joined);


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

        let dataset_item: DatasetItem = DatasetItem {
            curriculum: Curriculum::Small,
            text: String::new(),
        };
        Ok(dataset_item)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_five_unique_color_pairs() {
        let actual: Vec<(u8, u8)> = GenerateDataset::five_unique_color_pairs(&mut StdRng::seed_from_u64(0));
        assert_eq!(actual, vec![(5, 2), (9, 1), (6, 3), (4, 0), (7, 8)]);
    }

    #[allow(dead_code)]
    // #[test]
    fn test_20000_generate() {
        // Arrange
        let mut generate_dataset = GenerateDataset::new();

        // Act
        generate_dataset.populate(Curriculum::Small, 24, false).expect("ok");

        // Assert
    }
}
