use super::{RandomImage, Image, ImageSize, ImageHistogram, Histogram, HtmlLog};
use rand::prelude::Distribution;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng, Rng};
use rand::distributions::WeightedIndex;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;

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
struct GenerateDataset {
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
            let random_seed: u64 = i as u64;
            let dataset_item: DatasetItem = Self::generate(curriculum, random_seed, print_to_htmllog)?;
            self.dataset_items.push(dataset_item);
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn generate(curriculum: Curriculum, random_seed: u64, print_to_htmllog: bool) -> anyhow::Result<DatasetItem> {
        let missing_symbol: &str = "missing";
        // let symbol_names: HashMap<u8, String> = Self::generate_symbol_names_with_callback(Self::symbol_name_special);
        let symbol_names: HashMap<u8, String> = Self::generate_symbol_names_with_callback(Self::symbol_name_0_255);
        // let symbol_names: HashMap<u8, String> = Self::generate_symbol_names_with_callback(Self::symbol_name_uppercase_a_z);

        let sizes: Vec<u8> = match curriculum {
            Curriculum::Small => vec![3, 4, 5, 6],
            Curriculum::SmallMedium => vec![3, 4, 5, 6, 7, 8, 9, 10],
            Curriculum::SmallMediumBig => vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
        };

        let mut rng = StdRng::seed_from_u64(random_seed);

        let randomize_newlines_in_images: bool = rng.gen_bool(0.5); // 50% chance

        let item_count: usize = Self::number_of_comparison_items_to_generate(&mut rng);
        let mut item_vec = Vec::<ComparisionItem>::new();
        for _ in 0..item_count {
            let width0: u8 = *sizes.choose(&mut rng).unwrap();
            let height0: u8 = *sizes.choose(&mut rng).unwrap();
            let width1: u8 = *sizes.choose(&mut rng).unwrap();
            let height1: u8 = *sizes.choose(&mut rng).unwrap();

            let size0 = ImageSize::new(width0, height0);
            let size1 = ImageSize::new(width1, height1);
            let image_left: Image = RandomImage::uniform_colors(&mut rng, size0, 9)?;
            let image_right: Image = RandomImage::uniform_colors(&mut rng, size1, 9)?;

            if print_to_htmllog {
                HtmlLog::compare_images(vec![image_left.clone(), image_right.clone()]);
            }
    
            let item: ComparisionItem = ComparisionItem::create(&image_left, &image_right)?;
            item_vec.push(item);
        }

        let mut markdown = String::new();
        markdown.push_str("# Histogram comparisons with summary\n\n");

        for (item_index, item) in item_vec.iter().enumerate() {
            let name: char = ('A' as u8 + item_index as u8) as char;
            markdown.push_str(&format!("## Comparison {}\n\n", name));
    
            Self::markdown_for_comparison_item(&mut rng, &mut markdown, &item, &symbol_names, missing_symbol, randomize_newlines_in_images)?;
    
            markdown.push_str("\n\n");
        }
        
        markdown.push_str("## Summary\n\n");

        let mut union_histogram_left: Histogram = Histogram::new();
        let mut union_histogram_right: Histogram = Histogram::new();
        let mut intersection_histogram_left: Histogram = Histogram::new();
        let mut intersection_histogram_right: Histogram = Histogram::new();
        let mut intersection_histogram_left_only: Histogram = Histogram::new();
        let mut intersection_histogram_right_only: Histogram = Histogram::new();

        for (item_index, item) in item_vec.iter().enumerate() {
            union_histogram_left.add_histogram(&item.histogram_left);
            union_histogram_right.add_histogram(&item.histogram_right);
            if item_index == 0 {
                intersection_histogram_left = item.histogram_left.clone();
                intersection_histogram_right = item.histogram_right.clone();
                intersection_histogram_left_only = item.histogram_left_only.clone();
                intersection_histogram_right_only = item.histogram_right_only.clone();
            } else {
                intersection_histogram_left.intersection_histogram(&item.histogram_left);
                intersection_histogram_right.intersection_histogram(&item.histogram_right);
                intersection_histogram_left_only.intersection_histogram(&item.histogram_left_only);
                intersection_histogram_right_only.intersection_histogram(&item.histogram_right_only);
            }
        }

        intersection_histogram_left.clamp01();
        intersection_histogram_left.multiply_histogram(&union_histogram_left);
        intersection_histogram_right.clamp01();
        intersection_histogram_right.multiply_histogram(&union_histogram_right);

        {
            let image_union_histogram_left: Image = union_histogram_left.color_image()?;
            let image_union_histogram_right: Image = union_histogram_right.color_image()?;
            let image_intersection_histogram_left: Image = intersection_histogram_left.color_image()?;
            let image_intersection_histogram_right: Image = intersection_histogram_right.color_image()?;
            let image_intersection_histogram_left_only: Image = intersection_histogram_left_only.color_image()?;
            let image_intersection_histogram_right_only: Image = intersection_histogram_right_only.color_image()?;
    
            let body_union_histogram_left: String = Self::image_to_string(&image_union_histogram_left, &symbol_names, missing_symbol);
            let body_union_histogram_right: String = Self::image_to_string(&image_union_histogram_right, &symbol_names, missing_symbol);
            let body_image_intersection_histogram_left: String = Self::image_to_string(&image_intersection_histogram_left, &symbol_names, missing_symbol);
            let body_image_intersection_histogram_right: String = Self::image_to_string(&image_intersection_histogram_right, &symbol_names, missing_symbol);
            let body_image_intersection_histogram_left_only: String = Self::image_to_string(&image_intersection_histogram_left_only, &symbol_names, missing_symbol);
            let body_image_intersection_histogram_right_only: String = Self::image_to_string(&image_intersection_histogram_right_only, &symbol_names, missing_symbol);
    
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
            markdown.push_str("Intersection left-only histograms: ");
            markdown.push_str(&Self::markdown_code(&body_image_intersection_histogram_left_only));
            markdown.push_str("\n\n");
            markdown.push_str("Intersection right-only histograms: ");
            markdown.push_str(&Self::markdown_code(&body_image_intersection_histogram_right_only));
        }

        if print_to_htmllog {
            println!("{}", markdown);
        }

        let dataset_item = DatasetItem {
            curriculum,
            text: markdown,
        };
        Ok(dataset_item)
    }

    fn number_of_comparison_items_to_generate(rng: &mut StdRng) -> usize {
        let items: [usize; 5] = [2, 3, 4, 5, 6];
        let weights: [u8; 5] = [1, 2, 2, 2, 2];
        // We don't want `2` to occur as often as the other values, so a lower weight is used.
        let dist = WeightedIndex::new(&weights).unwrap();
        items[dist.sample(rng)]
    }

    fn markdown_for_comparison_item(rng: &mut StdRng, markdown: &mut String, item: &ComparisionItem, symbol_names: &HashMap<u8, String>, missing_symbol: &str, randomize_newlines_in_images: bool) -> anyhow::Result<()> {
        let image_histogram_left: Image = item.histogram_left.color_image()?;
        let image_histogram_right: Image = item.histogram_right.color_image()?;
        let image_histogram_left_only: Image = item.histogram_left_only.color_image()?;
        let image_histogram_right_only: Image = item.histogram_right_only.color_image()?;
        let image_histogram_union: Image = item.histogram_union.color_image()?;
        let image_histogram_intersection: Image = item.histogram_intersection.color_image()?;

        let body_data_left: String;
        let body_data_right: String;
        if randomize_newlines_in_images {
            // Insert newlines random places
            body_data_left = Self::image_to_string_with_random_wrap(rng, &item.image_left, symbol_names, missing_symbol);
            body_data_right = Self::image_to_string_with_random_wrap(rng, &item.image_right, symbol_names, missing_symbol);
        } else {
            // Insert newlines after each row
            body_data_left = Self::image_to_string(&item.image_left, symbol_names, missing_symbol);
            body_data_right = Self::image_to_string(&item.image_right, symbol_names, missing_symbol);
        }
        let body_union_left_right: String = Self::image_to_string(&image_histogram_union, symbol_names, missing_symbol);
        let body_intersection_left_right: String = Self::image_to_string(&image_histogram_intersection, symbol_names, missing_symbol);
        let body_only_left: String = Self::image_to_string(&image_histogram_left_only, symbol_names, missing_symbol);
        let body_only_right: String = Self::image_to_string(&image_histogram_right_only, symbol_names, missing_symbol);
        let body_histogram_left: String = Self::image_to_string(&image_histogram_left, symbol_names, missing_symbol);
        let body_histogram_right: String = Self::image_to_string(&image_histogram_right, symbol_names, missing_symbol);

        markdown.push_str("### Data left\n\n");
        markdown.push_str(&Self::markdown_fenced_code_block(&body_data_left));
        markdown.push_str("\n\n");
        
        markdown.push_str("### Data right\n\n");
        markdown.push_str(&Self::markdown_fenced_code_block(&body_data_right));
        markdown.push_str("\n\n");

        markdown.push_str("### Compare\n\n");
        markdown.push_str("Histogram left: ");
        markdown.push_str(&Self::markdown_code(&body_histogram_left));
        markdown.push_str("\n\n");
        markdown.push_str("Histogram right: ");
        markdown.push_str(&Self::markdown_code(&body_histogram_right));
        markdown.push_str("\n\n");
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

    /// Two lowercase characters in the range a..z.
    fn symbol_name_lowercase_a_z(value: u8) -> String {
        let value0: u8 = value % 26;
        let value1: u8 = value / 26;
        let char0 = (b'a' + value0) as char;
        let char1 = (b'a' + value1) as char;
        format!("{}{}", char1, char0)
    }

    /// Two uppercase characters in the range A..Z.
    fn symbol_name_uppercase_a_z(value: u8) -> String {
        let value0: u8 = value % 26;
        let value1: u8 = value / 26;
        let char0 = (b'A' + value0) as char;
        let char1 = (b'A' + value1) as char;
        format!("{}{}", char1, char0)
    }

    /// Two digits with special symbols.
    fn symbol_name_special(value: u8) -> String {
        let strings_char0: [&str; 16] = [".", "*", "=", ":", ";", "@", "+", "-", "±", "$", "!", "?", "^", "|", "■", "□"];
        let strings_char1: [&str; 16] = ["◯", "▙", "▛", "▜", "▟", "░", "╬", "⛝", "←", "↑", "→", "↓", "⊕", "⊗", "⌦", "⌫"];
        let value0: u8 = value % 16;
        let value1: u8 = value / 16;
        let char0 = strings_char0[value0 as usize];
        let char1 = strings_char1[value1 as usize];
        format!("{}{}", char1, char0)
    }

    fn image_to_string(image: &Image, symbol_names: &HashMap<u8, String>, missing_symbol: &str) -> String {
        let mut s = String::new();
        for y in 0..image.height() {
            if y > 0 {
                s.push('\n');
            }
            for x in 0..image.width() {
                if x > 0 {
                    s.push(',');
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

    fn image_to_string_with_random_wrap(rng: &mut StdRng, image: &Image, symbol_names: &HashMap<u8, String>, missing_symbol: &str) -> String {
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
            let count: usize = rng.gen_range(5..=20);
            for _ in 0..count {
                if items.is_empty() {
                    break;
                }
                let item = items.remove(0);
                s.push_str(&item);
                if !items.is_empty() {
                    s.push(',');
                }
            }
            if items.is_empty() {
                break;
            }
            s.push('\n');
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
    fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let s: String = Self::dataset_to_jsonl(&self.dataset_items)?;
        println!("dataset number of rows: {}", self.dataset_items.len());
        println!("dataset jsonl bytes: {}", s.len());

        let mut file = std::fs::File::create(path)?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_image_to_string() {
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
        let actual: String = GenerateDataset::image_to_string(&image, &mapping, "?");

        // Assert
        let expected = "b1,c2,d3\na0,?,a0\nb1,c2,d3";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_image_to_string_with_random_wrap() {
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
            "?"
        );

        // Assert
        let expected = "255,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,8,\n9,1,2,3,4,\n5,6,7,8,9,1,2,3,4,5,6,7,8,254";
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
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(0), "aa");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(9), "aj");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(10), "ak");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(25), "az");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(26), "ba");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(254), "ju");
        assert_eq!(GenerateDataset::symbol_name_lowercase_a_z(255), "jv");
    }

    #[test]
    fn test_20003_symbol_name_uppercase_a_z() {
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(0), "AA");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(9), "AJ");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(10), "AK");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(25), "AZ");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(26), "BA");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(254), "JU");
        assert_eq!(GenerateDataset::symbol_name_uppercase_a_z(255), "JV");
    }

    #[test]
    fn test_20004_symbol_name_special() {
        assert_eq!(GenerateDataset::symbol_name_special(0), "◯.");
        assert_eq!(GenerateDataset::symbol_name_special(16 + 1), "▙*");
        assert_eq!(GenerateDataset::symbol_name_special(2 * 16 + 2), "▛=");
        assert_eq!(GenerateDataset::symbol_name_special(3 * 16 + 3), "▜:");
        assert_eq!(GenerateDataset::symbol_name_special(4 * 16 + 4), "▟;");
        assert_eq!(GenerateDataset::symbol_name_special(5 * 16 + 5), "░@");
        assert_eq!(GenerateDataset::symbol_name_special(6 * 16 + 6), "╬+");
        assert_eq!(GenerateDataset::symbol_name_special(7 * 16 + 7), "⛝-");
        assert_eq!(GenerateDataset::symbol_name_special(8 * 16 + 8), "←±");
        assert_eq!(GenerateDataset::symbol_name_special(9 * 16 + 9), "↑$");
        assert_eq!(GenerateDataset::symbol_name_special(10 * 16 + 10), "→!");
        assert_eq!(GenerateDataset::symbol_name_special(11 * 16 + 11), "↓?");
        assert_eq!(GenerateDataset::symbol_name_special(12 * 16 + 12), "⊕^");
        assert_eq!(GenerateDataset::symbol_name_special(13 * 16 + 13), "⊗|");
        assert_eq!(GenerateDataset::symbol_name_special(14 * 16 + 14), "⌦■");
        assert_eq!(GenerateDataset::symbol_name_special(15 * 16 + 15), "⌫□");
    }

    #[test]
    fn test_30000_number_of_items_to_generate() {
        assert_eq!(GenerateDataset::number_of_comparison_items_to_generate(&mut StdRng::seed_from_u64(0)), 6);
        assert_eq!(GenerateDataset::number_of_comparison_items_to_generate(&mut StdRng::seed_from_u64(2)), 2);
    }

    // #[test]
    fn test_40000_generate() {
        let path: PathBuf = PathBuf::from("/Users/neoneye/Downloads/histograms.jsonl");
        let mut generator = GenerateDataset::new();
        let number_of_items: u32 = 8;
        generator.populate(Curriculum::Small, number_of_items, true).expect("ok");
        generator.populate(Curriculum::SmallMedium, number_of_items, false).expect("ok");
        generator.populate(Curriculum::SmallMediumBig, number_of_items, false).expect("ok");
        generator.shuffle();
        generator.save(&path).expect("ok");
    }
}
