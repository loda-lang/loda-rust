//! Generate a dataset with basic trivial simple ARC tasks.
//! https://github.com/neoneye/arc-dataset-diva
//! 
//! Ideas for more task types:
//! MixedOrientationOutputSolidColor,
//! MixedOrientationRotateOutputSolidColor,
//! LandscapeInputIsOneSolidColorButOutputIsTwoDifferentColors, // needs more than 5 training pairs.
use super::{Image, HtmlLog, ReverseColorPopularity, ImageRotate90, ImageTryCreate, ExportARCTaskJson};
use rand::Rng;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};
use serde::Serialize;
use std::fs;
use std::path::{PathBuf, Path};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
enum TwoPixelBasicTransformation {
    LandscapeOrientationReverse,
    LandscapeOrientationRotateCW,
    LandscapeOrientationRotateCCW,
    PortraitOrientationReverse,
    PortraitOrientationRotateCW,
    PortraitOrientationRotateCCW,
    MixedOrientationReverse,
    MixedOrientationRotateCW,
    MixedOrientationRotateCCW,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
enum TwoPixelSpecialTransformation {
    LandscapeOrientation,
    PortraitOrientation,
    MixedOrientation,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
enum TwoPixelTransformation {
    Basic { basic: TwoPixelBasicTransformation },
    Special { special: TwoPixelSpecialTransformation },
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct DatasetItem {
    dirname: String,
    filename: String,
    json: String,
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
    fn populate(&mut self, number_of_items: u32, print_to_htmllog: bool) -> anyhow::Result<()> {
        let transformations: Vec<TwoPixelTransformation> = vec![
            TwoPixelTransformation::Basic { basic: TwoPixelBasicTransformation::LandscapeOrientationReverse },
            TwoPixelTransformation::Basic { basic: TwoPixelBasicTransformation::LandscapeOrientationRotateCW },
            TwoPixelTransformation::Basic { basic: TwoPixelBasicTransformation::LandscapeOrientationRotateCCW },
            TwoPixelTransformation::Basic { basic: TwoPixelBasicTransformation::PortraitOrientationReverse },
            TwoPixelTransformation::Basic { basic: TwoPixelBasicTransformation::PortraitOrientationRotateCW },
            TwoPixelTransformation::Basic { basic: TwoPixelBasicTransformation::PortraitOrientationRotateCCW },
            TwoPixelTransformation::Basic { basic: TwoPixelBasicTransformation::MixedOrientationReverse },
            TwoPixelTransformation::Basic { basic: TwoPixelBasicTransformation::MixedOrientationRotateCW },
            TwoPixelTransformation::Basic { basic: TwoPixelBasicTransformation::MixedOrientationRotateCCW },
            TwoPixelTransformation::Special { special: TwoPixelSpecialTransformation::LandscapeOrientation },
            TwoPixelTransformation::Special { special: TwoPixelSpecialTransformation::PortraitOrientation },
            TwoPixelTransformation::Special { special: TwoPixelSpecialTransformation::MixedOrientation },
        ];

        for i in 0..number_of_items {
            if print_to_htmllog {
                HtmlLog::text(format!("iteration: {}", i));
            }
            if i % 100000 == 0 {
                println!("iteration: {} number_of_items: {}", i, number_of_items);
            }
            let transform_index: usize = (i as usize) % transformations.len();
            let transformation: TwoPixelTransformation = transformations[transform_index].clone();
            // Introducing a new transformation does not impact the random seed.
            let random_seed: u64 = ((i as usize / transformations.len()) + (transform_index * 1000000)) as u64;

            match transformation {
                TwoPixelTransformation::Basic { basic } => {
                    let dataset_item: DatasetItem = Self::generate_twopixels_basic(basic.clone(), random_seed, print_to_htmllog)?;
                    self.dataset_items.push(dataset_item);
                },
                TwoPixelTransformation::Special { special } => {
                    let dataset_item: DatasetItem = Self::generate_twopixels_special(special.clone(), random_seed, print_to_htmllog)?;
                    self.dataset_items.push(dataset_item);
                },
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn save(&self, save_dir: &Path) -> anyhow::Result<()> {
        if !save_dir.is_dir() {
            fs::create_dir(save_dir)?;
        }
        assert!(save_dir.is_dir());

        for dataset_item in &self.dataset_items {
            let task_type_dir: PathBuf = save_dir.join(&dataset_item.dirname);
            if !task_type_dir.is_dir() {
                fs::create_dir(&task_type_dir)?;
            }
            assert!(task_type_dir.is_dir());

            let path: PathBuf = task_type_dir.join(&dataset_item.filename);
            fs::write(&path, &dataset_item.json)?;
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

    fn alternate(count: usize, values: Vec<u8>) -> Vec<u8> {
        assert!(values.len() >= 2);
        let mut result = Vec::<u8>::new();
        for i in 0..count {
            let index: usize = i % values.len();
            result.push(values[index]);
        }
        result
    }

    /// Example with two values: `[0, 1]`, then roughly half of the items are `0` and the other half are `1`. 
    /// 
    /// When `N` is even there is half 0 and half 1.
    /// 
    /// When `N` is odd then considering `N-1` have half 0 and half 1. And the last one is either 0 or 1.
    fn round_robin_shuffled(rng: &mut StdRng, count: usize, values: &Vec<u8>) -> Vec<u8> {
        assert!(values.len() >= 2);
        // In case there is an even number of items, then both value0 and value1 gets used equally. Good.
        // In case there are an odd number of items, then one of the values is used one more time than the other value. Bad.
        // Shuffle to prevent bias.
        let mut values: Vec<u8> = values.clone();
        values.shuffle(rng);
        let mut items: Vec<u8> = Self::alternate(count, values);
        // Now the items are alternating. Bad.
        // Shuffle to prevent bias.
        items.shuffle(rng);
        items
    }

    fn generate_twopixels_basic(transformation: TwoPixelBasicTransformation, random_seed: u64, print_to_htmllog: bool) -> anyhow::Result<DatasetItem> {
        let mut rng: StdRng = SeedableRng::seed_from_u64(random_seed);

        let insert_same_color_when_reaching_this_limit: u8 = 50;
        let insert_same_value: u8 = rng.gen_range(0..=100);

        let pair_count_values: Vec<(u8, u8)> = vec![
            (2, 1), (2, 2), (2, 3), (3, 1), (3, 2), (3, 3), (4, 1), (4, 2), (4, 3)
        ];
        let (train_count, test_count) = *pair_count_values.choose(&mut rng).unwrap();
        let pair_count: u8 = train_count + test_count;

        let zero_one: Vec<u8> = vec![0, 1];
        let mut mixed_orientation_vec = Vec::<u8>::new();
        mixed_orientation_vec.extend(Self::round_robin_shuffled(&mut rng, train_count as usize, &zero_one));
        mixed_orientation_vec.extend(Self::round_robin_shuffled(&mut rng, test_count as usize, &zero_one));
        assert!(mixed_orientation_vec.len() == pair_count as usize);

        // There are max 4 `train` pairs. Since there are 5 unique color pairs, we can be 
        // certain that the `train` pairs have different colors from each other.
        let mut color_pairs: Vec<(u8, u8)> = Self::five_unique_color_pairs(&mut rng);
        color_pairs.truncate(pair_count as usize);

        // Fill up with more random colors until there are enough color pairs.
        // The `test` pairs comes last, so it's ok if they are not as unique as the `train` pairs.
        while color_pairs.len() < pair_count as usize {
            let color0: u8 = rng.gen_range(0..=9);
            let color1: u8 = rng.gen_range(0..=9);
            if color0 == color1 {
                continue;
            }
            if color_pairs.contains(&(color0, color1)) {
                continue;
            }
            color_pairs.push((color0, color1));
        }
        assert!(color_pairs.len() == pair_count as usize);

        // Make one of the `test` pairs slightly ambiguous so it's more tricky to solve.
        // It doesn't make sense when it's a `flip` operation where the two pixels exchange places.
        // then don't make it ambiguous, because it would cause input and output to be identical.
        // we want input and output to always be different.
        let allow_same_color: bool = match transformation {
            TwoPixelBasicTransformation::LandscapeOrientationReverse => false,
            TwoPixelBasicTransformation::LandscapeOrientationRotateCW => true,
            TwoPixelBasicTransformation::LandscapeOrientationRotateCCW => true,
            TwoPixelBasicTransformation::PortraitOrientationReverse => false,
            TwoPixelBasicTransformation::PortraitOrientationRotateCW => true,
            TwoPixelBasicTransformation::PortraitOrientationRotateCCW => true,
            TwoPixelBasicTransformation::MixedOrientationReverse => false,
            TwoPixelBasicTransformation::MixedOrientationRotateCW => true,
            TwoPixelBasicTransformation::MixedOrientationRotateCCW => true,
        };
        if allow_same_color && train_count >= 2 && test_count >= 1 && insert_same_value >= insert_same_color_when_reaching_this_limit {
            // Replace a color_pair so it uses the same color for both its colors, so it's ambiguous and more tricky to solve.
            let index: usize = rng.gen_range(train_count..pair_count) as usize;
            let color: u8 = rng.gen_range(0..=9);
            color_pairs[index] = (color, color);
        }

        if print_to_htmllog {
            HtmlLog::text(format!("pair_count: {}", pair_count));
        }
        let mut export = ExportARCTaskJson::new();
        let mut color_pair_strings = Vec::<String>::new();
        for i in 0..pair_count {
            let is_train: bool = i < train_count;

            // Pick two random colors
            // The colors are always different from each other for the `train` pairs.
            // The colors are sometimes the same and sometimes different for the `test` pairs.
            let (color0, color1) = color_pairs.remove(0);

            let input_landscape: Image = Image::try_create(2, 1, vec![color0, color1])?;
            let input_portrait: Image = input_landscape.rotate_cw()?;

            // Pick either input_landscape or input_portrait based on a random number
            // Make sure that both landscape and portrait orientations are used for the training pairs, so 2 or more train pairs.
            // Make sure that both landscape and portrait orientations are used for the test pairs, so 2 or more test pairs.
            let input_mixed: Image = match mixed_orientation_vec[i as usize] {
                0 => input_landscape.clone(),
                1 => input_portrait.clone(),
                _ => unreachable!(),
            };

            let input: &Image = match transformation {
                TwoPixelBasicTransformation::LandscapeOrientationReverse => &input_landscape,
                TwoPixelBasicTransformation::LandscapeOrientationRotateCW => &input_landscape,
                TwoPixelBasicTransformation::LandscapeOrientationRotateCCW => &input_landscape,
                TwoPixelBasicTransformation::PortraitOrientationReverse => &input_portrait,
                TwoPixelBasicTransformation::PortraitOrientationRotateCW => &input_portrait,
                TwoPixelBasicTransformation::PortraitOrientationRotateCCW => &input_portrait,
                TwoPixelBasicTransformation::MixedOrientationReverse => &input_mixed,
                TwoPixelBasicTransformation::MixedOrientationRotateCW => &input_mixed,
                TwoPixelBasicTransformation::MixedOrientationRotateCCW => &input_mixed,
            };

            let output_reversed: Image = ReverseColorPopularity::apply_to_image(input)?;
            let output_rotate_ccw: Image = input.rotate_ccw()?;
            let output_rotate_cw: Image = input.rotate_cw()?;

            let output: &Image = match transformation {
                TwoPixelBasicTransformation::LandscapeOrientationReverse => &output_reversed,
                TwoPixelBasicTransformation::LandscapeOrientationRotateCW => &output_rotate_cw,
                TwoPixelBasicTransformation::LandscapeOrientationRotateCCW => &output_rotate_ccw,
                TwoPixelBasicTransformation::PortraitOrientationReverse => &output_reversed,
                TwoPixelBasicTransformation::PortraitOrientationRotateCW => &output_rotate_cw,
                TwoPixelBasicTransformation::PortraitOrientationRotateCCW => &output_rotate_ccw,
                TwoPixelBasicTransformation::MixedOrientationReverse => &output_reversed,
                TwoPixelBasicTransformation::MixedOrientationRotateCW => &output_rotate_cw,
                TwoPixelBasicTransformation::MixedOrientationRotateCCW => &output_rotate_ccw,
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

            color_pair_strings.push(format!("{}{}", color0, color1));
        }

        let transformation_name: &str = match transformation {
            TwoPixelBasicTransformation::LandscapeOrientationReverse => "land_rev",
            TwoPixelBasicTransformation::LandscapeOrientationRotateCW => "land_cw",
            TwoPixelBasicTransformation::LandscapeOrientationRotateCCW => "land_ccw",
            TwoPixelBasicTransformation::PortraitOrientationReverse => "port_rev",
            TwoPixelBasicTransformation::PortraitOrientationRotateCW => "port_cw",
            TwoPixelBasicTransformation::PortraitOrientationRotateCCW => "port_ccw",
            TwoPixelBasicTransformation::MixedOrientationReverse => "landport_rev",
            TwoPixelBasicTransformation::MixedOrientationRotateCW => "landport_cw",
            TwoPixelBasicTransformation::MixedOrientationRotateCCW => "landport_ccw",
        };

        let color_pair_strings_joined: String = color_pair_strings.join("_");
        let filename: String = format!("{}_{}.json", transformation_name, color_pair_strings_joined);

        let dataset_item: DatasetItem = DatasetItem {
            json: export.to_string()?,
            dirname: transformation_name.to_string(),
            filename,
        };
        Ok(dataset_item)
    }

    fn generate_twopixels_special(transformation: TwoPixelSpecialTransformation, random_seed: u64, print_to_htmllog: bool) -> anyhow::Result<DatasetItem> {
        let mut rng: StdRng = SeedableRng::seed_from_u64(random_seed);

        let pair_count_values: Vec<(u8, u8)> = match transformation {
            TwoPixelSpecialTransformation::LandscapeOrientation | TwoPixelSpecialTransformation::PortraitOrientation => vec![
                // minimum 2 `train` pairs are needed to make sense of the rules
                (2, 2), (2, 3), (3, 2), (3, 3)
            ],
            TwoPixelSpecialTransformation::MixedOrientation => vec![
                // minimum 4 `train` pairs are needed to make sense of the rules
                (4, 2), (4, 3), (5, 2), (5, 3)
            ],
        };

        let (train_count, test_count) = *pair_count_values.choose(&mut rng).unwrap();
        let pair_count: u8 = train_count + test_count;

        let mut values: Vec<u8> = (0..pair_count).collect();
        values.shuffle(&mut rng);

        // Determine which orientation to use and when to assign the same color to both pixels.        
        let zero_one_two_tree: Vec<u8> = match transformation {
            TwoPixelSpecialTransformation::LandscapeOrientation | TwoPixelSpecialTransformation::PortraitOrientation => {
                // bit[1] = unused, bit[0] = same color. In to total 2 combinations.
                // 0 = same color
                // 1 = different color
                vec![0, 1]
            },
            TwoPixelSpecialTransformation::MixedOrientation => {
                // bit[1] = orientation, bit[0] = same color. In to total 4 combinations.
                // 0 = same color, landscape
                // 1 = different color, landscape
                // 2 = same color, portrait
                // 3 = different color, portrait
                vec![0, 1, 2, 3]
            }
        };
        let mut mode_vec = Vec::<u8>::new();
        mode_vec.extend(Self::round_robin_shuffled(&mut rng, train_count as usize, &zero_one_two_tree));
        mode_vec.extend(Self::round_robin_shuffled(&mut rng, test_count as usize, &zero_one_two_tree));
        assert!(mode_vec.len() == pair_count as usize);

        // Assign colors to each pair.
        let mut color_pairs: Vec<(u8, u8)> = Self::five_unique_color_pairs(&mut rng);
        color_pairs.truncate(pair_count as usize);
        while color_pairs.len() < pair_count as usize {
            let color0: u8 = rng.gen_range(0..=9);
            let color1: u8 = rng.gen_range(0..=9);
            if color0 == color1 {
                continue;
            }
            if color_pairs.contains(&(color0, color1)) {
                continue;
            }
            color_pairs.push((color0, color1));
        }
        assert!(color_pairs.len() == pair_count as usize);

        let mut available_colors: Vec<u8> = (0..=9).collect();
        available_colors.shuffle(&mut rng);

        // Assign same color to both pixels in roughly half of the pairs.
        for i in 0..pair_count {
            let value: u8 = mode_vec[i as usize];
            let assign_same_color: bool = (value & 1) == 0;
            if !assign_same_color {
                continue;
            }
            let color: u8 = available_colors.remove(0);
            color_pairs[i as usize] = (color, color);
            // println!("assigning same color: {} to index: {}", color, i);
        }

        if print_to_htmllog {
            HtmlLog::text(format!("pair_count: {}", pair_count));
        }
        let mut export = ExportARCTaskJson::new();
        let mut color_pair_strings = Vec::<String>::new();
        for i in 0..pair_count {
            let is_train: bool = i < train_count;

            // Pick two random colors
            // The colors are always different from each other for the `train` pairs.
            // The colors are sometimes the same and sometimes different for the `test` pairs.
            let (color0, color1) = color_pairs.remove(0);

            let input_landscape: Image = Image::try_create(2, 1, vec![color0, color1])?;
            let input_portrait: Image = input_landscape.rotate_cw()?;

            // Pick either input_landscape or input_portrait based on a random number
            // Make sure that both landscape and portrait orientations are used for the training pairs, so 2 or more train pairs.
            // Make sure that both landscape and portrait orientations are used for the test pairs, so 2 or more test pairs.
            let orientation: bool = mode_vec[i as usize] & 2 == 0;
            let input_mixed: Image = match orientation {
                false => input_landscape.clone(),
                true => input_portrait.clone(),
            };

            let input: &Image = match transformation {
                TwoPixelSpecialTransformation::LandscapeOrientation => &input_landscape,
                TwoPixelSpecialTransformation::PortraitOrientation => &input_portrait,
                TwoPixelSpecialTransformation::MixedOrientation => &input_mixed,
            };

            let output_reversed: Image = ReverseColorPopularity::apply_to_image(input)?;
            let output_rotate_ccw: Image = input.rotate_ccw()?;

            let output: &Image = if color0 == color1 {
                &output_rotate_ccw
            } else {
                &output_reversed
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

            color_pair_strings.push(format!("{}{}", color0, color1));
        }

        let transformation_name: &str = match transformation {
            TwoPixelSpecialTransformation::LandscapeOrientation => "land_rotrev",
            TwoPixelSpecialTransformation::PortraitOrientation => "port_rotrev",
            TwoPixelSpecialTransformation::MixedOrientation => "landport_rotrev",
        };

        let color_pair_strings_joined: String = color_pair_strings.join("_");
        let filename: String = format!("{}_{}.json", transformation_name, color_pair_strings_joined);

        let dataset_item: DatasetItem = DatasetItem {
            json: export.to_string()?,
            dirname: transformation_name.to_string(),
            filename,
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

    #[test]
    fn test_20000_alternate() {
        assert_eq!(GenerateDataset::alternate(2, vec![0, 1]), vec![0, 1]);
        assert_eq!(GenerateDataset::alternate(3, vec![0, 1]), vec![0, 1, 0]);
        assert_eq!(GenerateDataset::alternate(4, vec![0, 1]), vec![0, 1, 0, 1]);
        assert_eq!(GenerateDataset::alternate(3, vec![4, 5]), vec![4, 5, 4]);
        assert_eq!(GenerateDataset::alternate(6, vec![1, 2, 3]), vec![1, 2, 3, 1, 2, 3]);
    }

    #[test]
    fn test_30000_round_robin_shuffled() {
        let a: Vec<u8> = vec![0, 1];
        let b: Vec<u8> = vec![5, 6, 7];
        assert_eq!(GenerateDataset::round_robin_shuffled(&mut StdRng::seed_from_u64(0), 5, &a), vec![1, 1, 0, 0, 0]);
        assert_eq!(GenerateDataset::round_robin_shuffled(&mut StdRng::seed_from_u64(0), 6, &a), vec![1, 1, 0, 0, 0, 1]);
        assert_eq!(GenerateDataset::round_robin_shuffled(&mut StdRng::seed_from_u64(0), 7, &a), vec![1, 0, 0, 0, 1, 0, 1]);
        assert_eq!(GenerateDataset::round_robin_shuffled(&mut StdRng::seed_from_u64(1), 5, &a), vec![0, 0, 1, 1, 0]);
        assert_eq!(GenerateDataset::round_robin_shuffled(&mut StdRng::seed_from_u64(1), 6, &a), vec![0, 0, 1, 1, 0, 1]);
        assert_eq!(GenerateDataset::round_robin_shuffled(&mut StdRng::seed_from_u64(1), 7, &a), vec![0, 1, 0, 1, 0, 0, 1]);
        assert_eq!(GenerateDataset::round_robin_shuffled(&mut StdRng::seed_from_u64(1), 8, &a), vec![0, 0, 1, 0, 1, 0, 1, 1]);
        assert_eq!(GenerateDataset::round_robin_shuffled(&mut StdRng::seed_from_u64(0), 3, &b), vec![7, 5, 6]);
    }

    #[allow(dead_code)]
    // #[test]
    fn test_40000_generate() {
        // Arrange
        let mut generate_dataset = GenerateDataset::new();

        // Act
        // generate_dataset.populate(60, true).expect("ok");
        generate_dataset.populate(1200, false).expect("ok");

        generate_dataset.save(&PathBuf::from("/Users/neoneye/Downloads/output")).expect("ok");
    
        // Assert
    }
}
