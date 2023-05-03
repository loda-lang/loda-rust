use super::{Image, ImageCompare, Rectangle, ImageCrop, ImageColorProfile, ImagePadding, ImageReplaceSimple};
use std::collections::HashSet;

const SUBSTITUTION_RULE_VERBOSE: bool = false;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SubstitutionRule {
    pub source: Image,
    pub destination: Image,
}

impl SubstitutionRule {
    /// Determine the rule in the scenario when there is only one rule.
    /// 
    /// It cannot determine the rules when there are 2 or more substitution rules.
    /// 
    /// Parameter `pairs` is a vector of `input` and `output` images.
    /// 
    /// Returns `(source, destination)` when a replacement rule has been found.
    /// The min size is `1x1`. The max size is `4x4`.
    /// 
    /// Returns an error when no rule can be found.
    #[allow(dead_code)]
    pub fn find_rule(pairs: Vec<(Image, Image)>) -> anyhow::Result<Self> {
        if pairs.is_empty() {
            return Err(anyhow::anyhow!("There must be 1 or more pairs. Cannot derive rule from zero pairs."));
        }

        // Prepare as much data as possible upfront, so it doesn't have to be computed over and over.
        let mut items = Vec::<Item>::new();
        let mut count_positions: usize = 0;
        for (input, output) in pairs {
            if input.size() != output.size() || input.is_empty() {
                return Err(anyhow::anyhow!("Both input and output must have same size. And be 1x1 or bigger."));
            }

            // Find the positions where the `input` and `output` differs
            let diff: Image = input.diff(&output)?;
            let mut diff_positions = HashSet::<(i32, i32)>::new();
            for y in 0..input.height() as i32 {
                for x in 0..input.width() as i32 {
                    if diff.get(x, y).unwrap_or(0) > 0 {
                        diff_positions.insert((x, y));
                    }
                }
            }
            count_positions += diff_positions.len();

            // Add 1px border around the image with the most popular color
            let background_color: u8 = input.most_popular_color().unwrap_or(255);
            let input_with_1px_padding: Image = input.padding_with_color(1, background_color)?;

            let item = Item {
                input,
                input_with_1px_padding,
                output,
                diff_positions,
            };
            items.push(item);
        }

        if count_positions == 0 {
            return Err(anyhow::anyhow!("Without any differences, a rule cannot be derived."));
        }

        // Ordered by area (width x height) or ascending complexity.
        // We prefer the simplest rules, so the simplest substitution rules comes at the top.
        // We try to avoid advanced rules, the more complex substitution rules comes at the bottom.
        let sizes: [(u8, u8); 16] = [
            (1, 1),
            (2, 1),
            (1, 2),
            (3, 1),
            (1, 3),
            (4, 1),
            (1, 4),
            (2, 2),
            (3, 2),
            (2, 3),
            (4, 2),
            (2, 4),
            (3, 3),
            (4, 3),
            (3, 4),
            (4, 4),
        ];
        for (width, height) in sizes {
            match Self::find_substitution_with_size(&items, width, height) {
                Ok((source, destination)) => {
                    let instance = Self {
                        source,
                        destination,
                    };
                    return Ok(instance);
                },
                Err(error) => {
                    if SUBSTITUTION_RULE_VERBOSE {
                        println!("size: {} {} error: {:?}", width, height, error);
                    }
                    continue;
                }
            }
        }
        Err(anyhow::anyhow!("didn't find a replacement rule"))
    }

    fn find_substitution_with_size(items: &Vec<Item>, crop_width: u8, crop_height: u8) -> anyhow::Result<(Image, Image)> {
        if SUBSTITUTION_RULE_VERBOSE {
            println!("crop size: width {} height {}", crop_width, crop_height);
        }
        let mut replacements = Vec::<(Image, Image)>::new();
        for item in items {
            let width: u8 = item.input.width();
            let height: u8 = item.input.height();

            let mut rects = Vec::<Rectangle>::new();
            // Generate rectangles for the crop size near areas that have differences.
            for y in 0..height {
                for x in 0..width {
                    let x0: i32 = x as i32;
                    let y0: i32 = y as i32;
                    let x1: i32 = x0 + (crop_width as i32) - 1;
                    let y1: i32 = y0 + (crop_height as i32) - 1;
                    if x1 >= (width as i32) {
                        continue;
                    }
                    if y1 >= (height as i32) {
                        continue;
                    }
                    let rect: Rectangle = match Rectangle::span(x0, y0, x1, y1) {
                        Some(value) => value,
                        None => {
                            continue;
                        }
                    };

                    // We are only interested in rectangles where there are differences between input/output.
                    // Reject areas that are identical between input/output.
                    let mut rect_intersects_with_positions: bool = false;
                    for yy in y0..=y1 {
                        for xx in x0..=x1 {
                            let xy: (i32, i32) = (xx, yy);
                            if item.diff_positions.contains(&xy) {
                                rect_intersects_with_positions = true;
                                break;
                            }
                        }
                        if rect_intersects_with_positions {
                            break;
                        }
                    }
                    if rect_intersects_with_positions {
                        rects.push(rect);
                    }
                }
            }
            if SUBSTITUTION_RULE_VERBOSE {
                println!("rects length: {} content: {:?}", rects.len(), rects);
            }

            // Accumulate candidates for replacing source with destination.
            // A candidate may work some places, but may not work for all the substitutions
            // We are interested in the simplest candidate that works across all the input/output pairs.
            for rect in &rects {
                let replace_source: Image = match item.input.crop(*rect) {
                    Ok(value) => value,
                    Err(error) => {
                        if SUBSTITUTION_RULE_VERBOSE {
                            println!("crop is outside the input image. error: {:?}", error);
                        }
                        continue;
                    }
                };
                // println!("replace_source: {:?}", replace_source);
                let replace_target: Image = match item.output.crop(*rect) {
                    Ok(value) => value,
                    Err(error) => {
                        if SUBSTITUTION_RULE_VERBOSE {
                            println!("crop is outside the output image. error: {:?}", error);
                        }
                        continue;
                    }
                };
                // println!("replace_target: {:?}", replace_target);

                let replacement: (Image, Image) = (replace_source, replace_target);
                if !replacements.contains(&replacement) {
                    replacements.push(replacement);
                }
            }
        }
        if SUBSTITUTION_RULE_VERBOSE {
            println!("number of replacements: {}", replacements.len());
        }

        if replacements.is_empty() {
            return Err(anyhow::anyhow!("didn't find any replacements"));
        }

        // Find a single substitution rule that satisfy all the input/output pairs
        for (source, destination) in replacements {
            if SUBSTITUTION_RULE_VERBOSE {
                println!("replace source: {:?}", source);
                println!("replace destination: {:?}", destination);
            }

            let mut encountered_problem: bool = false;
            let mut number_of_replacements_performed: usize = 0;
            for item in items {
                let mut result_image: Image = item.input_with_1px_padding.clone();
                let count: u16 = result_image.replace_simple(&source, &destination)?;
                number_of_replacements_performed += count as usize;
                let crop_rect = Rectangle::new(1, 1, item.input.width(), item.input.height());
                let cropped_image: Image = result_image.crop(crop_rect)?;
                if cropped_image != item.output {
                    if SUBSTITUTION_RULE_VERBOSE {
                        println!("the computed output does not match the expected output image. The substitution rules are incorrect.");
                        println!("computed_output: {:?}", cropped_image);
                    }
                    encountered_problem = true;
                    break;
                }
                if SUBSTITUTION_RULE_VERBOSE {
                    println!("found good substitutions");
                }
            }
            if number_of_replacements_performed == 0 {
                if SUBSTITUTION_RULE_VERBOSE {
                    println!("no replacements were performed. reject this replacement");
                }
                continue;
            }
            if encountered_problem {
                continue;
            }

            return Ok((source, destination));
        }

        Err(anyhow::anyhow!("Unable to find a single substitution rule that works for all pairs"))
    }

    /// Apply the substitution rule.
    /// 
    /// Replaces the `source` image with the `destination` image.
    #[allow(dead_code)]
    pub fn apply(&self, input: &Image) -> anyhow::Result<Image> {
        let background_color: u8 = input.most_popular_color().unwrap_or(255);
        let mut result_image: Image = input.padding_with_color(1, background_color)?;
        _ = result_image.replace_simple(&self.source, &self.destination)?;
        let crop_rect = Rectangle::new(1, 1, input.width(), input.height());
        let result_image2: Image = result_image.crop(crop_rect)?;
        Ok(result_image2)
    }
}

struct Item {
    input: Image,
    input_with_1px_padding: Image,
    output: Image,

    /// Positions where `input` and `output` differs
    diff_positions: HashSet<(i32, i32)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_one_pair_replace1x1() {
        // Arrange
        let pair0_input_pixels: Vec<u8> = vec![
            0, 6, 0, 0,
            0, 0, 6, 0,
            6, 0, 0, 0,
            0, 0, 0, 6,
        ];
        let pair0_input: Image = Image::try_create(4, 4, pair0_input_pixels).expect("image");

        let pair0_output_pixels: Vec<u8> = vec![
            0, 3, 0, 0,
            0, 0, 3, 0,
            3, 0, 0, 0,
            0, 0, 0, 3,
        ];
        let pair0_output: Image = Image::try_create(4, 4, pair0_output_pixels).expect("image");

        let pairs: Vec<(Image, Image)> = vec![(pair0_input, pair0_output)];
        
        // Act
        let rule: SubstitutionRule = SubstitutionRule::find_rule(pairs).expect("rule");

        // Assert
        let expected_source_pixels: Vec<u8> = vec![
            6,
        ];
        let expected_source: Image = Image::try_create(1, 1, expected_source_pixels).expect("image");
        assert_eq!(rule.source, expected_source);
        let expected_destination_pixels: Vec<u8> = vec![
            3,
        ];
        let expected_destination: Image = Image::try_create(1, 1, expected_destination_pixels).expect("image");
        assert_eq!(rule.destination, expected_destination);
    }

    #[test]
    fn test_10001_one_pair_replace2x1() {
        // Arrange
        let pair0_input_pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 5, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let pair0_input: Image = Image::try_create(4, 5, pair0_input_pixels).expect("image");

        let pair0_output_pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 5, 5, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let pair0_output: Image = Image::try_create(4, 5, pair0_output_pixels).expect("image");

        let pairs: Vec<(Image, Image)> = vec![(pair0_input, pair0_output)];
        
        // Act
        let rule: SubstitutionRule = SubstitutionRule::find_rule(pairs).expect("rule");

        // Assert
        let expected_source_pixels: Vec<u8> = vec![
            5, 0,
        ];
        let expected_source: Image = Image::try_create(2, 1, expected_source_pixels).expect("image");
        assert_eq!(rule.source, expected_source);
        let expected_destination_pixels: Vec<u8> = vec![
            5, 5,
        ];
        let expected_destination: Image = Image::try_create(2, 1, expected_destination_pixels).expect("image");
        assert_eq!(rule.destination, expected_destination);
    }

    #[test]
    fn test_10002_one_pair_replace3x3() {
        // Arrange
        let pair0_input_pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let pair0_input: Image = Image::try_create(4, 5, pair0_input_pixels).expect("image");

        let pair0_output_pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            1, 2, 3, 0,
            4, 5, 6, 0,
            7, 8, 9, 0,
            0, 0, 0, 0,
        ];
        let pair0_output: Image = Image::try_create(4, 5, pair0_output_pixels).expect("image");

        let pairs: Vec<(Image, Image)> = vec![(pair0_input, pair0_output)];
        
        // Act
        let rule: SubstitutionRule = SubstitutionRule::find_rule(pairs).expect("rule");

        // Assert
        let expected_source_pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ];
        let expected_source: Image = Image::try_create(3, 3, expected_source_pixels).expect("image");
        assert_eq!(rule.source, expected_source);
        let expected_destination_pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
        ];
        let expected_destination: Image = Image::try_create(3, 3, expected_destination_pixels).expect("image");
        assert_eq!(rule.destination, expected_destination);
    }

    #[test]
    fn test_10003_one_pair_replace1x3() {
        // Arrange
        let pair0_input_pixels: Vec<u8> = vec![
            0, 0, 0, 1,
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 1,
        ];
        let pair0_input: Image = Image::try_create(4, 5, pair0_input_pixels).expect("image");

        let pair0_output_pixels: Vec<u8> = vec![
            0, 2, 0, 3,
            0, 3, 0, 4,
            0, 4, 0, 0,
            0, 0, 0, 2,
            0, 0, 0, 3,
        ];
        let pair0_output: Image = Image::try_create(4, 5, pair0_output_pixels).expect("image");

        let pairs: Vec<(Image, Image)> = vec![(pair0_input, pair0_output)];
        
        // Act
        let rule: SubstitutionRule = SubstitutionRule::find_rule(pairs).expect("rule");

        // Assert
        let expected_source_pixels: Vec<u8> = vec![
            0,
            1,
            0,
        ];
        let expected_source: Image = Image::try_create(1, 3, expected_source_pixels).expect("image");
        assert_eq!(rule.source, expected_source);
        let expected_destination_pixels: Vec<u8> = vec![
            2,
            3,
            4,
        ];
        let expected_destination: Image = Image::try_create(1, 3, expected_destination_pixels).expect("image");
        assert_eq!(rule.destination, expected_destination);
    }

    #[test]
    fn test_20000_two_pairs_replace1x1() {
        // Arrange
        let pair0_input_pixels: Vec<u8> = vec![
            0, 6, 0, 0,
            0, 0, 6, 6,
            6, 0, 0, 0,
        ];
        let pair0_input: Image = Image::try_create(4, 3, pair0_input_pixels).expect("image");

        let pair0_output_pixels: Vec<u8> = vec![
            0, 3, 0, 0,
            0, 0, 3, 3,
            3, 0, 0, 0,
        ];
        let pair0_output: Image = Image::try_create(4, 3, pair0_output_pixels).expect("image");

        let pair1_input_pixels: Vec<u8> = vec![
            0, 0, 6,
            6, 0, 0,
            0, 6, 0,
        ];
        let pair1_input: Image = Image::try_create(3, 3, pair1_input_pixels).expect("image");

        let pair1_output_pixels: Vec<u8> = vec![
            0, 0, 3,
            3, 0, 0,
            0, 3, 0,
        ];
        let pair1_output: Image = Image::try_create(3, 3, pair1_output_pixels).expect("image");

        let pairs: Vec<(Image, Image)> = vec![(pair0_input, pair0_output), (pair1_input, pair1_output)];
        
        // Act
        let rule: SubstitutionRule = SubstitutionRule::find_rule(pairs).expect("rule");

        // Assert
        let expected_source_pixels: Vec<u8> = vec![
            6,
        ];
        let expected_source: Image = Image::try_create(1, 1, expected_source_pixels).expect("image");
        assert_eq!(rule.source, expected_source);
        let expected_destination_pixels: Vec<u8> = vec![
            3,
        ];
        let expected_destination: Image = Image::try_create(1, 1, expected_destination_pixels).expect("image");
        assert_eq!(rule.destination, expected_destination);
    }

    #[test]
    fn test_20001_two_pairs_replace1x1() {
        // Arrange
        let pair0_input_pixels: Vec<u8> = vec![
            0, 6, 0, 0,
            0, 0, 6, 6,
            6, 0, 0, 0,
        ];
        let pair0_input: Image = Image::try_create(4, 3, pair0_input_pixels).expect("image");

        let pair0_output_pixels: Vec<u8> = vec![
            0, 3, 0, 0,
            0, 0, 3, 3,
            3, 0, 0, 0,
        ];
        let pair0_output: Image = Image::try_create(4, 3, pair0_output_pixels).expect("image");

        let pair1_input_pixels: Vec<u8> = vec![
            0, 0, 3,
            3, 0, 0,
            0, 3, 0,
        ];
        let pair1_input: Image = Image::try_create(3, 3, pair1_input_pixels).expect("image");

        let pair1_output_pixels: Vec<u8> = vec![
            0, 0, 3,
            3, 0, 0,
            0, 3, 0,
        ];
        let pair1_output: Image = Image::try_create(3, 3, pair1_output_pixels).expect("image");

        let pairs: Vec<(Image, Image)> = vec![(pair0_input, pair0_output), (pair1_input, pair1_output)];
        
        // Act
        let rule: SubstitutionRule = SubstitutionRule::find_rule(pairs).expect("rule");

        // Assert
        let expected_source_pixels: Vec<u8> = vec![
            6,
        ];
        let expected_source: Image = Image::try_create(1, 1, expected_source_pixels).expect("image");
        assert_eq!(rule.source, expected_source);
        let expected_destination_pixels: Vec<u8> = vec![
            3,
        ];
        let expected_destination: Image = Image::try_create(1, 1, expected_destination_pixels).expect("image");
        assert_eq!(rule.destination, expected_destination);
    }

    #[test]
    fn test_20002_two_pairs_replace3x2() {
        // Arrange
        let pair0_input_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0,
            0, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let pair0_input: Image = Image::try_create(7, 4, pair0_input_pixels).expect("image");

        let pair0_output_pixels: Vec<u8> = vec![
            1, 2, 3, 0, 0, 0, 0,
            4, 5, 6, 0, 1, 2, 3,
            0, 1, 2, 3, 4, 5, 6,
            0, 4, 5, 6, 0, 0, 0,
        ];
        let pair0_output: Image = Image::try_create(7, 4, pair0_output_pixels).expect("image");

        let pair1_input_pixels: Vec<u8> = vec![
            0, 0, 0,
            1, 0, 0,
            0, 0, 0,
            0, 0, 0,
        ];
        let pair1_input: Image = Image::try_create(3, 4, pair1_input_pixels).expect("image");

        let pair1_output_pixels: Vec<u8> = vec![
            0, 0, 0,
            1, 2, 3,
            4, 5, 6,
            0, 0, 0,
        ];
        let pair1_output: Image = Image::try_create(3, 4, pair1_output_pixels).expect("image");

        let pairs: Vec<(Image, Image)> = vec![(pair0_input, pair0_output), (pair1_input, pair1_output)];
        
        // Act
        let rule: SubstitutionRule = SubstitutionRule::find_rule(pairs).expect("rule");

        // Assert
        let expected_source_pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 0, 0,
        ];
        let expected_source: Image = Image::try_create(3, 2, expected_source_pixels).expect("image");
        assert_eq!(rule.source, expected_source);
        let expected_destination_pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let expected_destination: Image = Image::try_create(3, 2, expected_destination_pixels).expect("image");
        assert_eq!(rule.destination, expected_destination);
    }

    #[test]
    fn test_30000_one_pair_no_differences() {
        // Arrange
        let pair0_input: Image = Image::try_create(1, 1, vec![5]).expect("image");

        let pair0_output: Image = Image::try_create(1, 1, vec![5]).expect("image");

        let pairs: Vec<(Image, Image)> = vec![(pair0_input, pair0_output)];
        
        // Act
        let error = SubstitutionRule::find_rule(pairs).expect_err("should fail");

        // Assert
        let message: String = format!("{:?}", error);
        assert_eq!(message.contains("Without any differences, a rule cannot be derived."), true);
    }

    #[test]
    fn test_40000_apply() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(7, 4, input_pixels).expect("image");

        let source_pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ];
        let source: Image = Image::try_create(3, 3, source_pixels).expect("image");

        let destination_pixels: Vec<u8> = vec![
            2, 0, 2,
            0, 1, 0,
            2, 0, 2,
        ];
        let destination: Image = Image::try_create(3, 3, destination_pixels).expect("image");
        let rule = SubstitutionRule { source, destination };
       
        // Act
        let actual: Image = rule.apply(&input).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 2, 0, 2,
            0, 2, 0, 0, 0, 1, 0,
            0, 2, 0, 2, 2, 0, 2,
            0, 0, 1, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(7, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
