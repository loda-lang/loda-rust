use super::{RandomImage, Image, ImageSize, ImageHistogram, Histogram};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::HashMap;

type SymbolNameCallback = fn(u8) -> String;

#[allow(dead_code)]
struct GenerateDataset;

impl GenerateDataset {
    #[allow(dead_code)]
    fn generate() -> anyhow::Result<()> {
        // let symbol_names: HashMap<u8, String> = Self::generate_symbol_names_with_callback(Self::symbol_name_special);
        // let symbol_names: HashMap<u8, String> = Self::generate_symbol_names_with_callback(Self::symbol_name_0_255);
        let symbol_names: HashMap<u8, String> = Self::generate_symbol_names_with_callback(Self::symbol_name_uppercase_a_z);

        let mut rng = StdRng::seed_from_u64(0);
        let size = ImageSize::new(20, 1);
        let image_left: Image = RandomImage::uniform_colors(&mut rng, size, 255)?;
        let image_right: Image = RandomImage::uniform_colors(&mut rng, size, 255)?;

        let histogram_left: Histogram = image_left.histogram_all();
        let image_histogram_left: Image = histogram_left.color_image()?;

        let histogram_right: Histogram = image_right.histogram_all();
        let image_histogram_right: Image = histogram_right.color_image()?;

        let mut histogram_left_only: Histogram = histogram_left.clone();
        histogram_left_only.subtract_histogram(&histogram_right);
        let image_histogram_left_only: Image = histogram_left_only.color_image()?;

        let mut histogram_right_only: Histogram = histogram_right.clone();
        histogram_right_only.subtract_histogram(&histogram_left);
        let image_histogram_right_only: Image = histogram_right_only.color_image()?;

        let mut histogram_union: Histogram = histogram_left.clone();
        histogram_union.add_histogram(&histogram_right);
        let image_histogram_union: Image = histogram_union.color_image()?;

        let mut histogram_intersection: Histogram = histogram_left.clone();
        histogram_intersection.intersection_histogram(&histogram_right);
        let image_histogram_intersection: Image = histogram_intersection.color_image()?;

        let body_left_data: String = Self::image_to_string(&image_left, &symbol_names, "missing");
        let body_right_data: String = Self::image_to_string(&image_right, &symbol_names, "missing");
        let body_union_left_right: String = Self::image_to_string(&image_histogram_union, &symbol_names, "missing");
        let body_intersection_left_right: String = Self::image_to_string(&image_histogram_intersection, &symbol_names, "missing");
        let body_left_only: String = Self::image_to_string(&image_histogram_left_only, &symbol_names, "missing");
        let body_right_only: String = Self::image_to_string(&image_histogram_right_only, &symbol_names, "missing");
        let body_left_histogram: String = Self::image_to_string(&image_histogram_left, &symbol_names, "missing");
        let body_right_histogram: String = Self::image_to_string(&image_histogram_right, &symbol_names, "missing");

        let mut markdown = String::new();
        markdown.push_str("# Histogram comparisons with summary\n\n");

        markdown.push_str("## Comparison A\n\n");

        markdown.push_str("### Left data\n\n");
        markdown.push_str(&Self::markdown_fenced_code_block(&body_left_data));
        markdown.push_str("\n\n");
        
        markdown.push_str("### Right data\n\n");
        markdown.push_str(&Self::markdown_fenced_code_block(&body_right_data));
        markdown.push_str("\n\n");

        markdown.push_str("### Compare\n\n");
        markdown.push_str("Left histogram: ");
        markdown.push_str(&Self::markdown_code(&body_left_histogram));
        markdown.push_str("\n\n");
        markdown.push_str("Right histogram: ");
        markdown.push_str(&Self::markdown_code(&body_right_histogram));
        markdown.push_str("\n\n");
        markdown.push_str("Union left right: ");
        markdown.push_str(&Self::markdown_code(&body_union_left_right));
        markdown.push_str("\n\n");
        markdown.push_str("Intersection left right: ");
        markdown.push_str(&Self::markdown_code(&body_intersection_left_right));
        markdown.push_str("\n\n");
        markdown.push_str("Left only: ");
        markdown.push_str(&Self::markdown_code(&body_left_only));
        markdown.push_str("\n\n");
        markdown.push_str("Right only: ");
        markdown.push_str(&Self::markdown_code(&body_right_only));
        markdown.push_str("\n\n");

        println!("{}", markdown);

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
                s.push_str("\n");
            }
            for x in 0..image.width() {
                if x > 0 {
                    s.push_str(",");
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
}

#[cfg(test)]
mod tests {
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

    // #[test]
    fn test_20000_generate() {
        // Arrange
        GenerateDataset::generate().expect("ok");
    }
}
