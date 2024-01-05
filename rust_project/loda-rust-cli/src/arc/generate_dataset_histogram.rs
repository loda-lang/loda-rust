//! Generate a dataset with histogram comparisons and a summary.
//! 
//! This full huge dataset is available here, 3m rows:
//! https://huggingface.co/datasets/neoneye/histogram-comparisons-v1
//! 
//! This small dataset is available here, 150k rows:
//! https://huggingface.co/datasets/neoneye/histogram-comparisons-small-v1
//! 
//! Given a prompt similar to ARC input/output raw pixel data.
//! The response is histogram for every image, comparisons of input/output histograms, and a summary of the histograms.
//! 
//! The primary goal is to deal with different symbols systems, and still be able to predict correct histograms.
//!
//! The goal is not to mimic ARC's patterns. Maybe a secondary goal in the future, but for now it's not a goal.
//! ARC tasks comes in so many variations with highly ordered 2d pixel data, so it's infeasible to mimic.
//! 
//! Mimic ARC ideas:
//! Same palette for all comparisons.
//! Two color images, where it's the most popular color, least popular color, that is the goal to identify.
//! Images with the same background color, where the goal is to identify the shared background color.
//! Images with the same foreground color, where the goal is to identify the shared foreground color.
//! Splitview where the goal is to identify the separator color.
//! Crop out an area from the input image, so that the output image is a subset of the input image.
use super::{RandomImage, Image, ImageSize, ImageHistogram, Histogram, HtmlLog, ImageReplaceColor, ImageDenoise, arc_json_model};
use rand::prelude::Distribution;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng, Rng};
use rand::distributions::WeightedIndex;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
enum SymbolNameId {
    Digit,
    LowercaseHex,
    LowercaseAZ,
    UppercaseAZ,
    SpecialAscii,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
enum Curriculum {
    Small,
    SmallMedium,
    SmallMediumBig,
}


type SymbolNameCallback = fn(u8) -> String;

struct ComparisionItem {
    image_left: Image,
    image_right: Image,
    histogram_left: Histogram,
    histogram_right: Histogram,
    histogram_left_only: Histogram,
    histogram_right_only: Histogram,
    histogram_union: Histogram,
    histogram_intersection: Histogram,
}

impl ComparisionItem {
    fn create(image_left: &Image, image_right: &Image) -> anyhow::Result<Self> {
        let histogram_left: Histogram = image_left.histogram_all();

        let histogram_right: Histogram = image_right.histogram_all();

        let mut histogram_left_only: Histogram = histogram_left.clone();
        histogram_left_only.subtract_histogram(&histogram_right);

        let mut histogram_right_only: Histogram = histogram_right.clone();
        histogram_right_only.subtract_histogram(&histogram_left);

        let mut histogram_union: Histogram = histogram_left.clone();
        histogram_union.add_histogram(&histogram_right);

        let mut histogram_intersection: Histogram = histogram_left.clone();
        histogram_intersection.intersection_histogram(&histogram_right);

        let instance = Self {
            image_left: image_left.clone(),
            image_right: image_right.clone(),
            histogram_left,
            histogram_right,
            histogram_left_only,
            histogram_right_only,
            histogram_union,
            histogram_intersection,
        };
        Ok(instance)
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct DatasetItem {
    curriculum: Curriculum,
    text: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct DatasetItemForTask {
    pub metadata_id: String,
    pub markdown: String,
}

struct GeneratorParameters {
    symbol_name_id: SymbolNameId,
    symbol_names: HashMap<u8, String>,
    data_separator_column: String,
    data_separator_row: String,
    separator_name: String,
    max_color_value: u8,
    shuffled_color_replacements: HashMap<u8, u8>,
    overall_max_color_value0: u8,
    overall_max_color_value1: u8,
    use_overall_max_color_value: bool,
    randomize_newlines_in_images: bool,
    same_left_right_histograms_with_shuffled_pixels: bool,
    color_strategy_id: usize,
    item_count: usize,
}

impl GeneratorParameters {
    #[allow(dead_code)]
    fn metadata_id(&self) -> String {
        let name: &str = match self.symbol_name_id {
            SymbolNameId::Digit => "digit",
            SymbolNameId::LowercaseHex => "hex",
            SymbolNameId::LowercaseAZ => "az",
            SymbolNameId::UppercaseAZ => "AZ",
            SymbolNameId::SpecialAscii => "special",
        };
        let mut id: String = format!("{}-{}", name, self.separator_name);
        if self.randomize_newlines_in_images {
            id.push_str("-randomnewline");
        }
        let mut symbols = Vec::<String>::new();
        for i in 0..=self.max_color_value {
            if let Some(shuffled_index) = self.shuffled_color_replacements.get(&i) {
                if let Some(value) = self.symbol_names.get(&shuffled_index) {
                    symbols.push(value.clone());
                    continue;
                }
            }
            symbols.push("unknown".to_string());
        }
        id.push_str(" ");
        id.push_str(&symbols.join(""));
        id
    }
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

    fn generator_parameters(rng: &mut StdRng, curriculum: Curriculum) -> GeneratorParameters {
        // Generate symbol names that fits with this curriculum
        let symbol_name_id: SymbolNameId = Self::choose_symbol_name_id(rng, curriculum);
        let symbol_names: HashMap<u8, String> = match symbol_name_id {
            SymbolNameId::Digit        => Self::generate_symbol_names_with_callback(Self::symbol_name_0_255),
            SymbolNameId::LowercaseHex => Self::generate_symbol_names_with_callback(Self::symbol_name_lowercase_hex),
            SymbolNameId::LowercaseAZ  => Self::generate_symbol_names_with_callback(Self::symbol_name_lowercase_a_z),
            SymbolNameId::UppercaseAZ  => Self::generate_symbol_names_with_callback(Self::symbol_name_uppercase_a_z),
            SymbolNameId::SpecialAscii => Self::generate_symbol_names_with_callback(Self::symbol_name_special_ascii),
        };

        let is_symbol_name_special_ascii: bool = symbol_name_id == SymbolNameId::SpecialAscii;
        let (data_separator_column, data_separator_row, separator_name) = Self::random_data_separator_column_and_row(rng, symbol_name_id);

        // println!("data_separator_column: {:?}", data_separator_column);
        // println!("data_separator_row: {:?}", data_separator_row);

        // The symbol names to pick from
        let symbols_available: Vec<u8> = match symbol_name_id {
            SymbolNameId::Digit        => (0..=9).collect(), // 0-9, only 1 digit
            SymbolNameId::LowercaseHex => (0..=255).collect(), // 00-ff, 2 digits
            SymbolNameId::LowercaseAZ  => (0..=25).collect(), // a-z, only 1 digit
            SymbolNameId::UppercaseAZ  => (0..=25).collect(), // A-Z, only 1 digit
            SymbolNameId::SpecialAscii => (0..=15).collect(), // Special ascii, only 1 digit
        };
        let mut shuffled_symbols_available: Vec<u8> = symbols_available.clone();
        shuffled_symbols_available.shuffle(rng);

        // Taken N symbols from the symbols to use
        let use_number_of_symbols: usize = match symbol_name_id {
            SymbolNameId::Digit        => 10, // 0-9, only 1 digit
            SymbolNameId::LowercaseHex => 14, // 00-ff, 2 digits
            SymbolNameId::LowercaseAZ  => 12, // a-z, only 1 digit
            SymbolNameId::UppercaseAZ  => 12, // A-Z, only 1 digit
            SymbolNameId::SpecialAscii => 12, // Special ascii, only 1 digit
        };

        // Take N random symbols from the available symbols.
        let mut shuffled_symbols_to_use: Vec<u8> = shuffled_symbols_available.clone();
        shuffled_symbols_to_use.truncate(use_number_of_symbols);

        let max_color_value: u8 = (shuffled_symbols_to_use.len().max(1) - 1).min(255) as u8;

        let mut shuffled_color_replacements = HashMap::<u8, u8>::new();
        for source_color in 0..=max_color_value {
            let destination_color: u8 = shuffled_symbols_to_use[source_color as usize];
            shuffled_color_replacements.insert(source_color, destination_color);
        }

        let overall_max_color_value0: u8 = rng.gen_range(2..=max_color_value);
        let overall_max_color_value1: u8 = rng.gen_range(2..=max_color_value);

        let use_overall_max_color_value: bool = rng.gen_bool(0.2); // 20% chance

        let mut randomize_newlines_in_images: bool = rng.gen_bool(0.5); // 50% chance
        if is_symbol_name_special_ascii {
            randomize_newlines_in_images = false;
        }
        let same_left_right_histograms_with_shuffled_pixels: bool = rng.gen_bool(0.05); // 5% chance

        let color_strategy_id: usize = Self::color_strategy_id(rng);

        let item_count: usize = Self::number_of_comparison_items_to_generate(rng);

        GeneratorParameters {
            symbol_name_id,
            symbol_names,
            data_separator_column,
            data_separator_row,
            separator_name,
            max_color_value,
            shuffled_color_replacements,
            overall_max_color_value0,
            overall_max_color_value1,
            use_overall_max_color_value,
            randomize_newlines_in_images,
            same_left_right_histograms_with_shuffled_pixels,
            color_strategy_id,
            item_count,
        }
    }

    #[allow(dead_code)]
    fn generate(curriculum: Curriculum, random_seed: u64, print_to_htmllog: bool) -> anyhow::Result<DatasetItem> {
        let sizes: Vec<u8> = match curriculum {
            Curriculum::Small => vec![3, 4, 5, 6],
            Curriculum::SmallMedium => vec![3, 4, 5, 6, 7, 8, 9, 10],
            Curriculum::SmallMediumBig => vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
        };

        let mut rng = StdRng::seed_from_u64(random_seed);

        let params: GeneratorParameters = Self::generator_parameters(&mut rng, curriculum);

        let mut item_vec = Vec::<ComparisionItem>::new();
        for _ in 0..params.item_count {
            // Size of the image
            let width0: u8 = *sizes.choose(&mut rng).unwrap();
            let height0: u8 = *sizes.choose(&mut rng).unwrap();
            let width1: u8 = *sizes.choose(&mut rng).unwrap();
            let height1: u8 = *sizes.choose(&mut rng).unwrap();
            let size0 = ImageSize::new(width0, height0);
            let size1 = ImageSize::new(width1, height1);

            let mut min_color_value0: u8 = 0;
            let mut min_color_value1: u8 = 0;
            let mut max_color_value0: u8 = rng.gen_range(2..=params.max_color_value);
            let mut max_color_value1: u8 = rng.gen_range(2..=params.max_color_value);

            if params.use_overall_max_color_value {
                max_color_value0 = params.overall_max_color_value0;
                max_color_value1 = params.overall_max_color_value1;
            }

            match params.color_strategy_id {
                1 => {
                    // same number of colors for left image and right image
                    max_color_value1 = max_color_value0;
                },
                2 => {
                    // one more color for the right image
                    if max_color_value0 < params.max_color_value {
                        max_color_value1 = max_color_value0 + 1;
                    }
                },
                3 => {
                    // one more color for the left image
                    if max_color_value1 < params.max_color_value {
                        max_color_value0 = max_color_value1 + 1;
                    }
                },
                4 => {
                    // split the color space into 2 parts, so there is no overlap between colors in left image and right image
                    if max_color_value1 > 4 {
                        min_color_value0 = 0;
                        max_color_value0 = max_color_value1 / 2;
                        min_color_value1 = max_color_value1 / 2 + 1;
                        max_color_value1 = max_color_value1;
                        if min_color_value0 == max_color_value0 || min_color_value1 == max_color_value1 {
                            error!("split. Identical colors");
                            continue;
                        }
                    }
                },
                5 => {
                    // split the color space into 2 parts, so there is 1 color overlap between colors in left image and right image
                    if max_color_value1 > 5 {
                        min_color_value0 = 0;
                        max_color_value0 = max_color_value1 / 2;
                        min_color_value1 = max_color_value1 / 2;
                        max_color_value1 = max_color_value1;
                        if min_color_value0 == max_color_value0 || min_color_value1 == max_color_value1 {
                            error!("split. Identical colors");
                            continue;
                        }
                    }
                },
                6 => {
                    // split the color space into 2 parts, so there are 2 colors overlap between colors in left image and right image
                    if max_color_value1 > 5 {
                        min_color_value0 = 0;
                        max_color_value0 = max_color_value1 / 2 + 1;
                        min_color_value1 = max_color_value1 / 2;
                        max_color_value1 = max_color_value1;
                        if min_color_value0 == max_color_value0 || min_color_value1 == max_color_value1 {
                            error!("split. Identical colors");
                            continue;
                        }
                    }
                },
                _ => {
                    // do nothing, use uniform colors for left image and right image
                },
            }

            // All noise
            let noise_image_left: Image = RandomImage::uniform_colors(&mut rng, size0, min_color_value0, max_color_value0)?;
            let noise_image_right: Image = RandomImage::uniform_colors(&mut rng, size1, min_color_value1, max_color_value1)?;

            // Denoised, and the images have some 2d structure, resembling ARC tasks
            let mut random_image_left: Image = noise_image_left.denoise_type5()?;
            let random_image_right: Image = noise_image_right.denoise_type5()?;

            if params.same_left_right_histograms_with_shuffled_pixels {
                random_image_left = RandomImage::shuffle_pixels(&mut rng, &random_image_right)?;
            }

            // Change color range from `0..color_count` to the shuffled colors
            let image_left: Image = random_image_left.replace_colors_with_hashmap(&params.shuffled_color_replacements)?;
            let image_right: Image = random_image_right.replace_colors_with_hashmap(&params.shuffled_color_replacements)?;

            if print_to_htmllog {
                HtmlLog::compare_images(vec![random_image_left.clone(), random_image_right.clone()]);
                // HtmlLog::compare_images(vec![image_left.clone(), image_right.clone()]);
            }
    
            let item: ComparisionItem = ComparisionItem::create(&image_left, &image_right)?;
            item_vec.push(item);
        }

        let markdown: String = Self::markdown_for_comparison_items(
            &mut rng, 
            &item_vec, 
            &params.symbol_names,
            &params.data_separator_column,
            &params.data_separator_row,
            params.randomize_newlines_in_images,
        )?;
        if print_to_htmllog {
            println!("{}", markdown);
        }

        let dataset_item = DatasetItem {
            curriculum,
            text: markdown,
        };
        Ok(dataset_item)
    }

    #[allow(dead_code)]
    pub fn generate_with_task(task: &arc_json_model::Task, random_seed: u64, print_to_htmllog: bool) -> anyhow::Result<DatasetItemForTask> {
        // Extract "train" images
        let image_train_pairs: Vec<arc_json_model::ImagePair> = match task.images_train() {
            Ok(value) => value,
            Err(error) => {
                anyhow::bail!("Unable to load 'train' images. error: {:?}", error);
            }
        };
        // debug!("train pairs: {}", image_train_pairs.len());

        // Extract "test" images
        let image_test_pairs: Vec<arc_json_model::ImagePair> = match task.images_test() {
            Ok(value) => value,
            Err(error) => {
                anyhow::bail!("Unable to load 'test' images. error: {:?}", error);
            }
        };
        // debug!("test pairs: {}", image_test_pairs.len());

        // Sanity check
        if image_train_pairs.is_empty() || image_test_pairs.is_empty() {
            anyhow::bail!("Either 'train' or 'test' have zero images.");
        }

        let train_count: usize = image_train_pairs.len();

        if print_to_htmllog {
            for image_pair in &image_train_pairs {
                HtmlLog::compare_images(vec![image_pair.input.clone(), image_pair.output.clone()]);
            }
            for image_pair in &image_test_pairs {
                HtmlLog::compare_images(vec![image_pair.input.clone(), image_pair.output.clone()]);
            }
        }

        let image_pairs: Vec<arc_json_model::ImagePair> = image_train_pairs.into_iter().chain(image_test_pairs.into_iter()).collect();

        let mut rng = StdRng::seed_from_u64(random_seed);

        let params: GeneratorParameters = Self::generator_parameters(&mut rng, Curriculum::SmallMediumBig);

        let mut item_vec = Vec::<ComparisionItem>::new();
        for (index, image_pair) in image_pairs.iter().enumerate() {
            if index >= train_count {
                // Skip test images
                // The hidden ARC dataset does not have any output image available,
                // so it's not possible to generate a histogram for the test images.
                continue;
            }

            let original_image_left: Image = image_pair.input.clone();
            let original_image_right: Image = image_pair.output.clone();

            // Change color range from `0..color_count` to the shuffled colors
            let image_left: Image = original_image_left.replace_colors_with_hashmap(&params.shuffled_color_replacements)?;
            let image_right: Image = original_image_right.replace_colors_with_hashmap(&params.shuffled_color_replacements)?;

            if print_to_htmllog {
                HtmlLog::compare_images(vec![original_image_left.clone(), original_image_right.clone()]);
                // HtmlLog::compare_images(vec![image_left.clone(), image_right.clone()]);
            }
    
            let item: ComparisionItem = ComparisionItem::create(&image_left, &image_right)?;
            item_vec.push(item);
        }

        let markdown: String = Self::markdown_for_comparison_items(
            &mut rng, 
            &item_vec, 
            &params.symbol_names,
            &params.data_separator_column,
            &params.data_separator_row,
            params.randomize_newlines_in_images,
        )?;

        let metadata_id: String = format!("histogram-{}", params.metadata_id());
        let dataset_item = DatasetItemForTask {
            metadata_id,
            markdown,
        };
        Ok(dataset_item)
    }

    fn markdown_for_comparison_items(
        rng: &mut StdRng, 
        item_vec: &Vec<ComparisionItem>, 
        symbol_names: &HashMap<u8, String>, 
        data_separator_column: &str,
        data_separator_row: &str,
        randomize_newlines_in_images: bool,
    ) -> anyhow::Result<String> {
        let missing_symbol: &str = "missing";

        let mut markdown = String::new();
        markdown.push_str("# Histogram comparisons with summary\n\n");

        for (item_index, item) in item_vec.iter().enumerate() {
            let name: char = ('A' as u8 + item_index as u8) as char;
            markdown.push_str(&format!("## Data {}\n\n", name));
    
            Self::markdown_for_data_item(rng, &mut markdown, &item, &symbol_names, missing_symbol, &data_separator_column, &data_separator_row, randomize_newlines_in_images)?;
    
            markdown.push_str("\n\n");
        }
        
        markdown.push_str("## Response\n\n");

        for (item_index, item) in item_vec.iter().enumerate() {
            let name: char = ('A' as u8 + item_index as u8) as char;
            markdown.push_str(&format!("## Compare {}\n\n", name));
    
            Self::markdown_for_comparison_item(&mut markdown, &item, &symbol_names, missing_symbol)?;
    
            markdown.push_str("\n\n");
        }
        
        markdown.push_str("## Summary\n\n");

        let mut union_all_histograms: Histogram = Histogram::new();
        let mut union_histogram_left: Histogram = Histogram::new();
        let mut union_histogram_right: Histogram = Histogram::new();
        let mut intersection_histogram_left: Histogram = Histogram::new();
        let mut intersection_histogram_right: Histogram = Histogram::new();
        let mut intersection_histogram_left_only: Histogram = Histogram::new();
        let mut intersection_histogram_right_only: Histogram = Histogram::new();
        let mut intersection_all_histograms: Histogram = Histogram::new();

        for (item_index, item) in item_vec.iter().enumerate() {
            union_all_histograms.add_histogram(&item.histogram_union);
            union_histogram_left.add_histogram(&item.histogram_left);
            union_histogram_right.add_histogram(&item.histogram_right);
            if item_index == 0 {
                intersection_histogram_left = item.histogram_left.clone();
                intersection_histogram_right = item.histogram_right.clone();
                intersection_histogram_left_only = item.histogram_left_only.clone();
                intersection_histogram_right_only = item.histogram_right_only.clone();
                intersection_all_histograms = item.histogram_intersection.clone();
            } else {
                intersection_histogram_left.intersection_histogram(&item.histogram_left);
                intersection_histogram_right.intersection_histogram(&item.histogram_right);
                intersection_histogram_left_only.intersection_histogram(&item.histogram_left_only);
                intersection_histogram_right_only.intersection_histogram(&item.histogram_right_only);
                intersection_all_histograms.intersection_histogram(&item.histogram_intersection);
            }
        }

        intersection_histogram_left.clamp01();
        intersection_histogram_left.multiply_histogram(&union_histogram_left);
        intersection_histogram_right.clamp01();
        intersection_histogram_right.multiply_histogram(&union_histogram_right);
        intersection_all_histograms.clamp01();
        intersection_all_histograms.multiply_histogram(&union_all_histograms);

        {
            let image_union_all_histograms: Image = union_all_histograms.color_image()?;
            let image_union_histogram_left: Image = union_histogram_left.color_image()?;
            let image_union_histogram_right: Image = union_histogram_right.color_image()?;
            let image_intersection_histogram_left: Image = intersection_histogram_left.color_image()?;
            let image_intersection_histogram_right: Image = intersection_histogram_right.color_image()?;
            let image_intersection_histogram_left_only: Image = intersection_histogram_left_only.color_image()?;
            let image_intersection_histogram_right_only: Image = intersection_histogram_right_only.color_image()?;
            let image_intersection_all_histograms: Image = intersection_all_histograms.color_image()?;
    
            let separator_column: &str = ",";
            let separator_row: &str = "\n";
            let body_image_union_all_histograms: String = Self::image_to_string(&image_union_all_histograms, &symbol_names, missing_symbol, separator_column, separator_row);
            let body_union_histogram_left: String = Self::image_to_string(&image_union_histogram_left, &symbol_names, missing_symbol, separator_column, separator_row);
            let body_union_histogram_right: String = Self::image_to_string(&image_union_histogram_right, &symbol_names, missing_symbol, separator_column, separator_row);
            let body_image_intersection_histogram_left: String = Self::image_to_string(&image_intersection_histogram_left, &symbol_names, missing_symbol, separator_column, separator_row);
            let body_image_intersection_histogram_right: String = Self::image_to_string(&image_intersection_histogram_right, &symbol_names, missing_symbol, separator_column, separator_row);
            let body_image_intersection_histogram_left_only: String = Self::image_to_string(&image_intersection_histogram_left_only, &symbol_names, missing_symbol, separator_column, separator_row);
            let body_image_intersection_histogram_right_only: String = Self::image_to_string(&image_intersection_histogram_right_only, &symbol_names, missing_symbol, separator_column, separator_row);
            let body_image_intersection_all_histograms: String = Self::image_to_string(&image_intersection_all_histograms, &symbol_names, missing_symbol, separator_column, separator_row);
    
            markdown.push_str("Union all histograms: ");
            markdown.push_str(&Self::markdown_code(&body_image_union_all_histograms));
            markdown.push_str("\n\n");
            markdown.push_str("Union left histograms: ");
            markdown.push_str(&Self::markdown_code(&body_union_histogram_left));
            markdown.push_str("\n\n");
            markdown.push_str("Union right histograms: ");
            markdown.push_str(&Self::markdown_code(&body_union_histogram_right));
            markdown.push_str("\n\n");
            markdown.push_str("Intersection left histograms: ");
            markdown.push_str(&Self::markdown_code(&body_image_intersection_histogram_left));
            markdown.push_str("\n\n");
            markdown.push_str("Intersection right histograms: ");
            markdown.push_str(&Self::markdown_code(&body_image_intersection_histogram_right));
            markdown.push_str("\n\n");
            if intersection_histogram_left.is_same_color_and_count(&intersection_histogram_right) {
                markdown.push_str("Intersection of left and right histograms are identical, same symbols and same counters.\n\n");
            } else {
                if intersection_histogram_left.is_same_color_but_ignore_count(&intersection_histogram_right) {
                    markdown.push_str("Intersection of left and right histograms have same symbols, but different counters\n\n");
                } else {
                    if intersection_histogram_left.is_same_count_but_ignore_color(&intersection_histogram_right) {
                        markdown.push_str("Intersection of left and right histograms have same counters, but different symbols\n\n");
                    }
                }
            }    
            markdown.push_str("Intersection left-only histograms: ");
            markdown.push_str(&Self::markdown_code(&body_image_intersection_histogram_left_only));
            markdown.push_str("\n\n");
            markdown.push_str("Intersection right-only histograms: ");
            markdown.push_str(&Self::markdown_code(&body_image_intersection_histogram_right_only));
            markdown.push_str("\n\n");
            markdown.push_str("Intersection all histograms: ");
            markdown.push_str(&Self::markdown_code(&body_image_intersection_all_histograms));
        }

        Ok(markdown)
    }

    fn number_of_comparison_items_to_generate(rng: &mut StdRng) -> usize {
        let items: [usize; 5] = [2, 3, 4, 5, 6];
        let weights: [u8; 5] = [1, 2, 2, 2, 2];
        // We don't want `2` to occur as often as the other values, so a lower weight is used.
        let dist = WeightedIndex::new(&weights).unwrap();
        items[dist.sample(rng)]
    }

    fn color_strategy_id(rng: &mut StdRng) -> usize {
        let items: [usize; 7] = [0, 1, 2, 3, 4, 5, 6];
        let weights: [u8; 7] = [1, 1, 1, 1, 1, 1, 1];
        let dist = WeightedIndex::new(&weights).unwrap();
        items[dist.sample(rng)]
    }

    fn random_data_separator_column_and_row(rng: &mut StdRng, symbol_name_id: SymbolNameId) -> (String, String, String) {
        let separator_id: u8 = Self::id_data_separator_column_and_row(rng, symbol_name_id);
        Self::data_separator_column_and_row(separator_id)
    }

    fn id_data_separator_column_and_row(rng: &mut StdRng, symbol_name_id: SymbolNameId) -> u8 {
        let indexes: Vec<u8> = match symbol_name_id {
            SymbolNameId::SpecialAscii => {
                // To prevent clashes when it's `symbol_name_special_ascii` then avoid using special characters for the separators.
                vec![0, 1, 2, 3, 4]
            },
            SymbolNameId::LowercaseHex => {
                // To ensure there is a separator between the columns, then avoid the empty separator.
                vec![0, 5, 6, 7, 8]
            },
            _ => vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        };
        let separator_id: u8 = *indexes.choose(rng).unwrap();
        separator_id
    }

    fn data_separator_column_and_row(separator_id: u8) -> (String, String, String) {
        let separator_column: &str;
        let separator_row: &str;
        let separator_name: &str;
        match separator_id {
            1 => {
                // No separator between columns. Space for separating rows.
                separator_column = "";
                separator_row = " ";
                separator_name = "none-space";
            },
            2 => {
                // No separators. It's a single line of pixels.
                separator_column = "";
                separator_row = "";
                separator_name = "none-none";
            },
            3 => {
                // No separator between columns. Comma for separating rows.
                separator_column = "";
                separator_row = ",";
                separator_name = "none-comma";
            },
            4 => {
                // No separator between columns. Newline for separating rows.
                separator_column = "";
                separator_row = "\n";
                separator_name = "none-newline";
            },
            5 => {
                // Space separator between columns. Semicolon for separating rows.
                separator_column = " ";
                separator_row = ";";
                separator_name = "space-semicolon";
            },
            6 => {
                // Colon separator between columns. Newline for separating rows.
                separator_column = ":";
                separator_row = "\n";
                separator_name = "colon-newline";
            },
            7 => {
                // Comma separator between columns. Newline for separating rows.
                separator_column = ",";
                separator_row = "\n";
                separator_name = "comma-newline";
            },
            8 => {
                // Comma separator between columns. Comma newline for separating rows.
                separator_column = ",";
                separator_row = ",\n";
                separator_name = "comma-commanewline";
            },
            _ => {
                // Space separator between columns. Newline for separating rows.
                separator_column = " ";
                separator_row = "\n";
                separator_name = "space-newline";
            },
        }
        (separator_column.to_string(), separator_row.to_string(), separator_name.to_string())
    }

    fn choose_symbol_name_id(rng: &mut StdRng, curriculum: Curriculum) -> SymbolNameId {
        let items: [SymbolNameId; 5] = [
            SymbolNameId::Digit,
            SymbolNameId::LowercaseHex,
            SymbolNameId::LowercaseAZ,
            SymbolNameId::UppercaseAZ,
            SymbolNameId::SpecialAscii
        ];
        let weights: [u8; 5] = match curriculum {
            Curriculum::Small => [1, 0, 0, 0, 0],
            Curriculum::SmallMedium => [1, 1, 1, 0, 0],
            Curriculum::SmallMediumBig => [1, 1, 1, 1, 1],
        };
        let dist = WeightedIndex::new(&weights).unwrap();
        items[dist.sample(rng)]
    }

    fn markdown_for_data_item(rng: &mut StdRng, markdown: &mut String, item: &ComparisionItem, symbol_names: &HashMap<u8, String>, missing_symbol: &str, separator_column: &str, separator_row: &str, randomize_newlines_in_images: bool) -> anyhow::Result<()> {
        let body_data_left: String;
        let body_data_right: String;
        if randomize_newlines_in_images {
            // Insert newlines random places
            body_data_left = Self::image_to_string_with_random_wrap(rng, &item.image_left, symbol_names, missing_symbol, separator_column, separator_row);
            body_data_right = Self::image_to_string_with_random_wrap(rng, &item.image_right, symbol_names, missing_symbol, separator_column, separator_row);
        } else {
            // Insert newlines after each row
            body_data_left = Self::image_to_string(&item.image_left, symbol_names, missing_symbol, separator_column, separator_row);
            body_data_right = Self::image_to_string(&item.image_right, symbol_names, missing_symbol, separator_column, separator_row);
        }

        markdown.push_str("### Data left\n\n");
        markdown.push_str(&Self::markdown_fenced_code_block(&body_data_left));
        markdown.push_str("\n\n");
        
        markdown.push_str("### Data right\n\n");
        markdown.push_str(&Self::markdown_fenced_code_block(&body_data_right));
        Ok(())
    }

    fn markdown_for_comparison_item(markdown: &mut String, item: &ComparisionItem, symbol_names: &HashMap<u8, String>, missing_symbol: &str) -> anyhow::Result<()> {
        let separator_column: &str = ",";
        let separator_row: &str = "\n";

        let image_histogram_left: Image = item.histogram_left.color_image()?;
        let image_histogram_right: Image = item.histogram_right.color_image()?;
        let image_histogram_left_only: Image = item.histogram_left_only.color_image()?;
        let image_histogram_right_only: Image = item.histogram_right_only.color_image()?;
        let image_histogram_union: Image = item.histogram_union.color_image()?;
        let image_histogram_intersection: Image = item.histogram_intersection.color_image()?;

        let body_union_left_right: String = Self::image_to_string(&image_histogram_union, symbol_names, missing_symbol, separator_column, separator_row);
        let body_intersection_left_right: String = Self::image_to_string(&image_histogram_intersection, symbol_names, missing_symbol, separator_column, separator_row);
        let body_only_left: String = Self::image_to_string(&image_histogram_left_only, symbol_names, missing_symbol, separator_column, separator_row);
        let body_only_right: String = Self::image_to_string(&image_histogram_right_only, symbol_names, missing_symbol, separator_column, separator_row);
        let body_histogram_left: String = Self::image_to_string(&image_histogram_left, symbol_names, missing_symbol, separator_column, separator_row);
        let body_histogram_right: String = Self::image_to_string(&image_histogram_right, symbol_names, missing_symbol, separator_column, separator_row);

        markdown.push_str("Histogram left: ");
        markdown.push_str(&Self::markdown_code(&body_histogram_left));
        markdown.push_str("\n\n");
        markdown.push_str("Histogram right: ");
        markdown.push_str(&Self::markdown_code(&body_histogram_right));
        markdown.push_str("\n\n");
        if item.histogram_left.is_same_color_and_count(&item.histogram_right) {
            markdown.push_str("Histogram left and right are identical, same symbols and same counters.\n\n");
        } else {
            if item.histogram_left.is_same_color_but_ignore_count(&item.histogram_right) {
                markdown.push_str("Histogram left and right have same symbols, but different counters\n\n");
            } else {
                if item.histogram_left.is_same_count_but_ignore_color(&item.histogram_right) {
                    markdown.push_str("Histogram left and right have same counters, but different symbols\n\n");
                }
            }
        }
        markdown.push_str("Union left right: ");
        markdown.push_str(&Self::markdown_code(&body_union_left_right));
        markdown.push_str("\n\n");
        markdown.push_str("Intersection left right: ");
        markdown.push_str(&Self::markdown_code(&body_intersection_left_right));
        markdown.push_str("\n\n");
        markdown.push_str("Only left: ");
        markdown.push_str(&Self::markdown_code(&body_only_left));
        markdown.push_str("\n\n");
        markdown.push_str("Only right: ");
        markdown.push_str(&Self::markdown_code(&body_only_right));

        Ok(())
    }
    
    fn markdown_code(body: &String) -> String {
        format!("`{}`", body)
    }

    fn markdown_fenced_code_block(body: &String) -> String {
        format!("```\n{}\n```", body)
    }

    fn generate_symbol_names_with_callback(callback: SymbolNameCallback) -> HashMap<u8, String> {
        let mut names = HashMap::<u8, String>::new();
        for i in 0..=255 {
            let name: String = callback(i);
            names.insert(i, name);
        }
        names
    }

    /// Variable number of digits in the range 0-9.
    fn symbol_name_0_255(value: u8) -> String {
        format!("{}", value)
    }

    /// Two hex digits in the range 0-9a-f.
    fn symbol_name_lowercase_hex(value: u8) -> String {
        format!("{:02x}", value)
    }

    /// 1 or 2 digits with lowercase characters in the range a..z.
    /// 
    /// If the value is between `0..=25`, then it's yield 1 digit.
    /// If the value is between `26..=255``, then it's yields 2 digits.
    fn symbol_name_lowercase_a_z(value: u8) -> String {
        let value0: u8 = value % 26;
        let value1: u8 = value / 26;
        let char0 = (b'a' + value0) as char;
        let char1 = (b'a' + value1) as char;
        if value1 == 0 {
            format!("{}", char0)
        } else {
            format!("{}{}", char1, char0)
        }
    }

    /// 1 or 2 digits with uppercase characters in the range A..Z.
    /// 
    /// If the value is between `0..=25`, then it's yield 1 digit.
    /// If the value is between `26..=255``, then it's yields 2 digits.
    fn symbol_name_uppercase_a_z(value: u8) -> String {
        let value0: u8 = value % 26;
        let value1: u8 = value / 26;
        let char0 = (b'A' + value0) as char;
        let char1 = (b'A' + value1) as char;
        if value1 == 0 {
            format!("{}", char0)
        } else {
            format!("{}{}", char1, char0)
        }
    }

    /// 1 or 2 digits with special ascii symbols. The characters are sorted by their ascii value.
    /// 
    /// Cherry picked characters, so there is no interfering with the markdown syntax.
    /// And aren't any backslashes.
    /// 
    /// If the value is between `0..=15`, then it's yield 1 digit.
    /// If the value is between `16..=255``, then it's yields 2 digits.
    fn symbol_name_special_ascii(value: u8) -> String {
        let strings_char: [&str; 16] = ["!", "$", "%", "&", "*", "+", "-", ".", "/", ":", ";", "?", "@", "_", "|", "~"];
        let value0: u8 = value % 16;
        let value1: u8 = value / 16;
        let char0 = strings_char[value0 as usize];
        let char1 = strings_char[value1 as usize];
        if value1 == 0 {
            format!("{}", char0)
        } else {
            format!("{}{}", char1, char0)
        }
    }

    /// 1 or 2 digits with special unicode symbols. The characters are not sorted by their unicode value.
    /// 
    /// I'm no fan about this one. The characters are not sorted by their unicode value.
    /// 
    /// If the value is between `0..=15`, then it's yield 1 digit.
    /// If the value is between `16..=255``, then it's yields 2 digits.
    #[allow(dead_code)]
    fn symbol_name_special_unicode(value: u8) -> String {
        let strings_char0: [&str; 16] = [".", "*", "=", ":", ";", "@", "+", "-", "±", "$", "!", "?", "^", "|", "■", "□"];
        let strings_char1: [&str; 16] = ["", "▙", "▛", "▜", "▟", "░", "╬", "⛝", "←", "↑", "→", "↓", "⊕", "⊗", "⌦", "⌫"];
        let value0: u8 = value % 16;
        let value1: u8 = value / 16;
        let char0 = strings_char0[value0 as usize];
        let char1 = strings_char1[value1 as usize];
        format!("{}{}", char1, char0)
    }

    fn image_to_string(image: &Image, symbol_names: &HashMap<u8, String>, missing_symbol: &str, separator_column: &str, separator_row: &str) -> String {
        let mut s = String::new();
        for y in 0..image.height() {
            if y > 0 {
                s.push_str(separator_row);
            }
            for x in 0..image.width() {
                if x > 0 {
                    s.push_str(separator_column);
                }
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                if let Some(name) = symbol_names.get(&color) {
                    s.push_str(name);
                } else {
                    s.push_str(missing_symbol);
                }
            }
        }
        s
    }

    fn image_to_string_with_random_wrap(rng: &mut StdRng, image: &Image, symbol_names: &HashMap<u8, String>, missing_symbol: &str, separator_column: &str, separator_row: &str) -> String {
        let mut items = Vec::<String>::new();
        for y in 0..image.height() {
            for x in 0..image.width() {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                if let Some(name) = symbol_names.get(&color) {
                    items.push(name.clone());
                } else {
                    items.push(missing_symbol.to_string());
                }
            }
        }
        let mut s = String::new();
        for _ in 0..items.len() {
            if !s.is_empty() {
                s.push_str(separator_row);
            }
            let count: usize = rng.gen_range(5..=20);
            let mut s2 = String::new();
            for _ in 0..count {
                if items.is_empty() {
                    break;
                }
                if !s2.is_empty() {
                    s2.push_str(separator_column);
                }
                let item = items.remove(0);
                s2.push_str(&item);
            }
            s.push_str(&s2);
            if items.is_empty() {
                break;
            }
        }
        s
    }

    fn dataset_to_jsonl(dataset_items: &Vec<DatasetItem>) -> anyhow::Result<String> {
        let mut jsonl_rows = Vec::<String>::new();
        for dataset_item in dataset_items {
            let jsonl_row: String = serde_json::to_string(dataset_item)?;
            jsonl_rows.push(jsonl_row);
        }
        let jsonl_data: String = jsonl_rows.join("\n");
        Ok(jsonl_data)
    }

    #[allow(dead_code)]
    fn shuffle(&mut self) {
        let mut rng = StdRng::seed_from_u64(0);
        self.dataset_items.shuffle(&mut rng);
    }

    #[allow(dead_code)]
    fn save(&self, path: &Path) -> anyhow::Result<()> {
        let s: String = Self::dataset_to_jsonl(&self.dataset_items)?;
        println!("dataset number of rows: {}", self.dataset_items.len());
        println!("dataset jsonl bytes: {}", s.len());

        let mut file = std::fs::File::create(path)?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn generate_dataset_huge(path: &Path) -> anyhow::Result<()> {
        let mut generator = GenerateDataset::new();
        let number_of_items: u32 = 1000000;
        generator.populate(Curriculum::Small, number_of_items, false)?;
        generator.populate(Curriculum::SmallMedium, number_of_items, false)?;
        generator.populate(Curriculum::SmallMediumBig, number_of_items, false)?;
        generator.shuffle();
        generator.save(&path)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn generate_dataset_small(path: &Path) -> anyhow::Result<()> {
        let mut generator = GenerateDataset::new();
        let number_of_items: u32 = 50000;
        generator.populate(Curriculum::Small, number_of_items, false)?;
        generator.populate(Curriculum::SmallMedium, number_of_items, false)?;
        generator.populate(Curriculum::SmallMediumBig, number_of_items, false)?;
        generator.shuffle();
        generator.save(&path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use std::path::PathBuf;

    #[test]
    fn test_10000_image_to_string_separator_comma_newline() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            0, 255, 0,
            1, 2, 3,
        ];
        let image: Image = Image::try_create(3, 3, pixels).expect("image");

        let mapping: HashMap<u8, String> = [
            (0, String::from("a0")),
            (1, String::from("b1")),
            (2, String::from("c2")),
            (3, String::from("d3")),
        ].iter().cloned().collect();

        // Act
        let actual: String = GenerateDataset::image_to_string(
            &image, 
            &mapping, 
            "?",
            ",",
            ",\n",
        );

        // Assert
        let expected = "b1,c2,d3,\na0,?,a0,\nb1,c2,d3";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_image_to_string_separator_pipe_dash() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            0, 255, 0,
            1, 2, 3,
        ];
        let image: Image = Image::try_create(3, 3, pixels).expect("image");

        let mapping: HashMap<u8, String> = [
            (0, String::from("a0")),
            (1, String::from("b1")),
            (2, String::from("c2")),
            (3, String::from("d3")),
        ].iter().cloned().collect();

        // Act
        let actual: String = GenerateDataset::image_to_string(
            &image, 
            &mapping, 
            "?",
            "|",
            " - ",
        );

        // Assert
        let expected = "b1|c2|d3 - a0|?|a0 - b1|c2|d3";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_image_to_string_separator_none_space() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            0, 255, 0,
            1, 2, 3,
        ];
        let image: Image = Image::try_create(3, 3, pixels).expect("image");

        let mapping: HashMap<u8, String> = [
            (0, String::from("a")),
            (1, String::from("b")),
            (2, String::from("c")),
            (3, String::from("d")),
        ].iter().cloned().collect();

        // Act
        let actual: String = GenerateDataset::image_to_string(
            &image, 
            &mapping, 
            "?",
            "",
            " ",
        );

        // Assert
        let expected = "bcd a?a bcd";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_image_to_string_oneliner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            0, 255, 0,
            1, 2, 3,
        ];
        let image: Image = Image::try_create(3, 3, pixels).expect("image");

        let mapping: HashMap<u8, String> = [
            (0, String::from("a")),
            (1, String::from("b")),
            (2, String::from("c")),
            (3, String::from("d")),
        ].iter().cloned().collect();

        // Act
        let actual: String = GenerateDataset::image_to_string(
            &image, 
            &mapping, 
            "?",
            "",
            "",
        );

        // Assert
        let expected = "bcda?abcd";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_image_to_string_with_random_wrap_separator_comma() {
        // Arrange
        let pixels: Vec<u8> = vec![
            255, 2, 3, 4, 5, 6, 7, 8, 9,
            1, 2, 3, 4, 5, 6, 7, 8, 9,
            1, 2, 3, 4, 5, 6, 7, 8, 9,
            1, 2, 3, 4, 5, 6, 7, 8, 254,
        ];
        let image: Image = Image::try_create(9, 4, pixels).expect("image");

        let symbol_names: HashMap<u8, String> = GenerateDataset::generate_symbol_names_with_callback(GenerateDataset::symbol_name_0_255);

        // Act
        let actual: String = GenerateDataset::image_to_string_with_random_wrap(
            &mut StdRng::seed_from_u64(0), 
            &image, 
            &symbol_names, 
            "?",
            ",",
            ",\n",
        );

        // Assert
        let expected = "255,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,8,\n9,1,2,3,4,\n5,6,7,8,9,1,2,3,4,5,6,7,8,254";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10005_image_to_string_with_random_wrap_separator_pipe() {
        // Arrange
        let pixels: Vec<u8> = vec![
            255, 2, 3, 4, 5, 6, 7, 8, 9,
            1, 2, 3, 4, 5, 6, 7, 8, 9,
            1, 2, 3, 4, 5, 6, 7, 8, 9,
            1, 2, 3, 4, 5, 6, 7, 8, 254,
        ];
        let image: Image = Image::try_create(9, 4, pixels).expect("image");

        let symbol_names: HashMap<u8, String> = GenerateDataset::generate_symbol_names_with_callback(GenerateDataset::symbol_name_0_255);

        // Act
        let actual: String = GenerateDataset::image_to_string_with_random_wrap(
            &mut StdRng::seed_from_u64(0), 
            &image, 
            &symbol_names, 
            "?",
            "|",
            "\n",
        );

        // Assert
        let expected = "255|2|3|4|5|6|7|8|9|1|2|3|4|5|6|7|8\n9|1|2|3|4\n5|6|7|8|9|1|2|3|4|5|6|7|8|254";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10006_image_to_string_with_random_wrap_separator_none() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 2, 3, 4, 5, 6, 7, 8, 9,
            1, 2, 3, 4, 5, 6, 7, 8, 9,
            1, 2, 3, 4, 5, 6, 7, 8, 9,
            1, 2, 3, 4, 5, 6, 7, 8, 0,
        ];
        let image: Image = Image::try_create(9, 4, pixels).expect("image");

        let symbol_names: HashMap<u8, String> = GenerateDataset::generate_symbol_names_with_callback(GenerateDataset::symbol_name_0_255);

        // Act
        let actual: String = GenerateDataset::image_to_string_with_random_wrap(
            &mut StdRng::seed_from_u64(0), 
            &image, 
            &symbol_names, 
            "?",
            "",
            ",",
        );

        // Assert
        let expected = "02345678912345678,91234,56789123456780";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_symbol_name_0_255() {
        assert_eq!(GenerateDataset::symbol_name_0_255(0), "0");
        assert_eq!(GenerateDataset::symbol_name_0_255(9), "9");
        assert_eq!(GenerateDataset::symbol_name_0_255(10), "10");
        assert_eq!(GenerateDataset::symbol_name_0_255(255), "255");
    }

    #[test]
    fn test_20001_symbol_name_lowercase_hex() {
        assert_eq!(GenerateDataset::symbol_name_lowercase_hex(0), "00");
        assert_eq!(GenerateDataset::symbol_name_lowercase_hex(9), "09");
        assert_eq!(GenerateDataset::symbol_name_lowercase_hex(10), "0a");
        assert_eq!(GenerateDataset::symbol_name_lowercase_hex(16), "10");
        assert_eq!(GenerateDataset::symbol_name_lowercase_hex(255), "ff");
    }

    #[test]
    fn test_20002_symbol_name_lowercase_a_z() {
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(0), "a");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(9), "j");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(10), "k");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(25), "z");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(26), "ba");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(254), "ju");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(255), "jv");
    }

    #[test]
    fn test_20003_symbol_name_uppercase_a_z() {
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(0), "A");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(9), "J");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(10), "K");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(25), "Z");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(26), "BA");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(254), "JU");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(255), "JV");
    }

    #[test]
    fn test_20004_symbol_name_special_ascii() {
        assert_eq!(GenerateDataset::symbol_name_special_ascii(0), "!");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(15), "~");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(16), "$!");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(16 + 1), "$$");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(2 * 16 + 2), "%%");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(3 * 16 + 3), "&&");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(4 * 16 + 4), "**");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(5 * 16 + 5), "++");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(6 * 16 + 6), "--");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(7 * 16 + 7), "..");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(8 * 16 + 8), "//");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(9 * 16 + 9), "::");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(10 * 16 + 10), ";;");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(11 * 16 + 11), "??");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(12 * 16 + 12), "@@");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(13 * 16 + 13), "__");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(14 * 16 + 14), "||");
        assert_eq!(GenerateDataset::symbol_name_special_ascii(15 * 16 + 15), "~~");
    }

    #[test]
    fn test_20005_symbol_name_special_unicode() {
        assert_eq!(GenerateDataset::symbol_name_special_unicode(0), ".");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(15), "□");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(16), "▙.");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(16 + 1), "▙*");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(2 * 16 + 2), "▛=");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(3 * 16 + 3), "▜:");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(4 * 16 + 4), "▟;");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(5 * 16 + 5), "░@");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(6 * 16 + 6), "╬+");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(7 * 16 + 7), "⛝-");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(8 * 16 + 8), "←±");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(9 * 16 + 9), "↑$");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(10 * 16 + 10), "→!");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(11 * 16 + 11), "↓?");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(12 * 16 + 12), "⊕^");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(13 * 16 + 13), "⊗|");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(14 * 16 + 14), "⌦■");
        assert_eq!(GenerateDataset::symbol_name_special_unicode(15 * 16 + 15), "⌫□");
    }

    #[test]
    fn test_30000_number_of_items_to_generate() {
        assert_eq!(GenerateDataset::number_of_comparison_items_to_generate(&mut StdRng::seed_from_u64(0)), 6);
        assert_eq!(GenerateDataset::number_of_comparison_items_to_generate(&mut StdRng::seed_from_u64(2)), 2);
    }

    #[test]
    fn test_40000_id_data_separator_column_and_row_all() {
        let mut min_value: u8 = 255;
        let mut max_value: u8 = 0;
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..20 {
            let value: u8 = GenerateDataset::id_data_separator_column_and_row(&mut rng, SymbolNameId::Digit);
            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }
        assert_eq!(min_value, 0);
        assert_eq!(max_value, 8);
    }

    #[test]
    fn test_40001_id_data_separator_column_and_row_special_ascii() {
        let mut min_value: u8 = 255;
        let mut max_value: u8 = 0;
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..20 {
            let value: u8 = GenerateDataset::id_data_separator_column_and_row(&mut rng, SymbolNameId::SpecialAscii);
            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }
        assert_eq!(min_value, 0);
        assert_eq!(max_value, 4);
    }

    #[test]
    fn test_40002_id_data_separator_column_and_row_lowercase_hex() {
        let mut min_value: u8 = 255;
        let mut max_value: u8 = 0;
        let mut histogram = Histogram::new();
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..20 {
            let value: u8 = GenerateDataset::id_data_separator_column_and_row(&mut rng, SymbolNameId::LowercaseHex);
            histogram.increment(value);
            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }
        assert_eq!(min_value, 0);
        assert_eq!(max_value, 8);
        assert_eq!(histogram.get(1), 0);
        assert_eq!(histogram.get(2), 0);
        assert_eq!(histogram.get(3), 0);
        assert_eq!(histogram.get(4), 0);
    }

    #[test]
    fn test_40003_data_separator_column_row_name() {
        {
            let (col, row, name) = GenerateDataset::data_separator_column_and_row(0);
            assert_eq!(col, " ");
            assert_eq!(row, "\n");
            assert_eq!(name, "space-newline");
        }
        {
            let (col, row, name) = GenerateDataset::data_separator_column_and_row(1);
            assert_eq!(col, "");
            assert_eq!(row, " ");
            assert_eq!(name, "none-space");
        }
        {
            let (col, row, name) = GenerateDataset::data_separator_column_and_row(2);
            assert_eq!(col, "");
            assert_eq!(row, "");
            assert_eq!(name, "none-none");
        }
        {
            let (col, row, name) = GenerateDataset::data_separator_column_and_row(3);
            assert_eq!(col, "");
            assert_eq!(row, ",");
            assert_eq!(name, "none-comma");
        }
        {
            let (col, row, name) = GenerateDataset::data_separator_column_and_row(4);
            assert_eq!(col, "");
            assert_eq!(row, "\n");
            assert_eq!(name, "none-newline");
        }
        {
            let (col, row, name) = GenerateDataset::data_separator_column_and_row(5);
            assert_eq!(col, " ");
            assert_eq!(row, ";");
            assert_eq!(name, "space-semicolon");
        }
        {
            let (col, row, name) = GenerateDataset::data_separator_column_and_row(6);
            assert_eq!(col, ":");
            assert_eq!(row, "\n");
            assert_eq!(name, "colon-newline");
        }
        {
            let (col, row, name) = GenerateDataset::data_separator_column_and_row(7);
            assert_eq!(col, ",");
            assert_eq!(row, "\n");
            assert_eq!(name, "comma-newline");
        }
        {
            let (col, row, name) = GenerateDataset::data_separator_column_and_row(8);
            assert_eq!(col, ",");
            assert_eq!(row, ",\n");
            assert_eq!(name, "comma-commanewline");
        }
    }

    #[test]
    fn test_50000_metadata_id() {
        // Arrange
        let seeds: Vec<u64> = vec![
            0,
            1,
            2,
            3,
            4,
            5,
            6,
            7,
            8,
            9,
            10,
        ];

        // Act
        let mut actual = Vec::<String>::new();
        for seed in seeds {
            let mut rng = StdRng::seed_from_u64(seed);
            let params: GeneratorParameters = GenerateDataset::generator_parameters(&mut rng, Curriculum::SmallMediumBig);
            actual.push(params.metadata_id());
        }

        // Assert
        let expected_str = [
            "special-none-newline @:~|+_*-.%$&", 
            "special-none-comma ?_+!|@/*$.;:", 
            "digit-comma-commanewline 5971683420",
            "AZ-none-comma RWCYSNVJQBXL", 
            "AZ-colon-newline SJBRKECIUDLV", 
            "digit-none-comma 7936281540", 
            "digit-none-comma 4301285967",
            "az-space-newline-randomnewline triksupcwavg", 
            "AZ-none-comma ULRQKZVADCES",
            "az-comma-newline-randomnewline ryknlsbgzfic", 
            "hex-comma-newline 5fd7cd60b958dc076fd3e22f9c57"
        ];
        let expected: Vec<String> = expected_str.iter().map(|s| s.to_string()).collect();
        assert_eq!(actual, expected);
    }

    #[allow(dead_code)]
    // #[test]
    fn test_60000_generate() {
        let path: PathBuf = PathBuf::from("/Users/neoneye/Downloads/histograms.jsonl");
        let mut generator = GenerateDataset::new();
        let number_of_items: u32 = 100;
        // generator.populate(Curriculum::Small, number_of_items, false).expect("ok");
        // generator.populate(Curriculum::SmallMedium, number_of_items, false).expect("ok");
        generator.populate(Curriculum::SmallMediumBig, number_of_items, true).expect("ok");
        generator.shuffle();
        generator.save(&path).expect("ok");
    }
}
