use super::{RandomImage, Image, ImageSize, ImageHistogram, Histogram};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::HashMap;

#[allow(dead_code)]
struct GenerateDataset;

impl GenerateDataset {
    #[allow(dead_code)]
    fn generate() -> anyhow::Result<()> {
        let mut rng = StdRng::seed_from_u64(0);
        let size = ImageSize::new(3, 1);
        let image_left: Image = RandomImage::uniform_colors(&mut rng, size, 255)?;
        let image_right: Image = RandomImage::uniform_colors(&mut rng, size, 255)?;

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

        let body_left_data: String = image_left.human_readable();
        let body_right_data: String = image_right.human_readable();
        let body_union_left_right: String = histogram_union.color_image()?.human_readable();
        let body_intersection_left_right: String = histogram_intersection.color_image()?.human_readable();
        let body_left_only: String = histogram_left_only.color_image()?.human_readable();
        let body_right_only: String = histogram_right_only.color_image()?.human_readable();
        let body_left_histogram: String = histogram_left.color_image()?.human_readable();
        let body_right_histogram: String = histogram_right.color_image()?.human_readable();

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

    // #[test]
    fn test_20000_generate() {
        // Arrange
        GenerateDataset::generate().expect("ok");
    }
}
