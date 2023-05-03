use super::{Image, ImageCompare, Rectangle, ImageCrop, ImageColorProfile, ImagePadding, ImageReplaceSimple};
use std::collections::HashSet;

pub struct SubstitutionRule;

impl SubstitutionRule {
    pub fn find_rule(pairs: Vec<(Image, Image)>) -> anyhow::Result<(Image, Image)> {
        if pairs.is_empty() {
            return Err(anyhow::anyhow!("There must be 1 or more pairs. Cannot derive rule from zero pairs."));
        }

        // Find the positions where the `input` and `output` differs
        let mut items = Vec::<Item>::new();
        for (input, output) in pairs {
            if input.size() != output.size() || input.is_empty() {
                return Err(anyhow::anyhow!("Both input and output must have same size. And be 1x1 or bigger."));
            }
            let diff: Image = input.diff(&output)?;
            let mut diff_positions = HashSet::<(i32, i32)>::new();
            for y in 0..input.height() as i32 {
                for x in 0..input.width() as i32 {
                    if diff.get(x, y).unwrap_or(0) > 0 {
                        diff_positions.insert((x, y));
                    }
                }
            }
            let item = Item {
                input,
                output,
                diff_positions,
            };
            items.push(item);
        }

        // Ascending complexity
        // We prefer the simplest rules, so the simplest substitution rules comes at the top.
        // We try to avoid advanced rules, the more complex substitution rules comes at the bottom.
        let sizes: [(u8, u8); 12] = [
            (1, 1),
            (2, 1),
            (1, 2),
            (3, 1),
            (1, 3),
            (2, 2),
            (3, 2),
            (2, 3),
            (3, 3),
            (4, 3),
            (3, 4),
            (4, 4),
        ];
        for (width, height) in sizes {
            match Self::find_substitution_with_size(&items, width, height) {
                Ok((source, destination)) => {
                    return Ok((source, destination));
                },
                Err(error) => {
                    println!("area: {} {} error: {:?}", width, height, error);
                    continue;
                }
            }
        }
        Err(anyhow::anyhow!("analyze didn't find any replacement"))
    }

    fn find_substitution_with_size(items: &Vec<Item>, crop_width: u8, crop_height: u8) -> anyhow::Result<(Image, Image)> {
        println!("crop size: width {} height {}", crop_width, crop_height);
        let mut replacements = Vec::<(Image, Image)>::new();
        for item in items {
            let width: u8 = item.input.width();
            let height: u8 = item.input.height();

            let mut rects = Vec::<Rectangle>::new();
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
            println!("rects length: {} content: {:?}", rects.len(), rects);

            for rect in &rects {
                // println!("rect {:?}", rect);
                let replace_source: Image = match item.input.crop(*rect) {
                    Ok(value) => value,
                    Err(error) => {
                        // println!("crop is outside the input image. error: {:?}", error);
                        continue;
                    }
                };
                // println!("replace_source: {:?}", replace_source);
                let replace_target: Image = match item.output.crop(*rect) {
                    Ok(value) => value,
                    Err(error) => {
                        // println!("crop is outside the output image. error: {:?}", error);
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
        println!("number of replacements: {}", replacements.len());

        if replacements.is_empty() {
            return Err(anyhow::anyhow!("didn't find any replacements"));
        }

        // Find a single substitution rule that satisfy all the training pairs
        for (key, value) in &replacements {
            println!("replace key: {:?}", key);
            println!("replace value: {:?}", value);

            let mut encountered_problem: bool = false;
            for item in items {
                let background_color: u8 = item.input.most_popular_color().unwrap_or(255);

                let mut result_image: Image = item.input.padding_with_color(1, background_color)?;
                let count: u16 = result_image.replace_simple(&key, &value)?;
                if count == 0 {
                    println!("no replacements were performed. reject this replacement");
                    encountered_problem = true;
                    break;
                }
                let crop_rect = Rectangle::new(1, 1, item.input.width(), item.input.height());
                let result_image2: Image = result_image.crop(crop_rect)?;
                if result_image2 != item.output {
                    println!("the computed output does not match the expected output image. The substitution rules are incorrect.");
                    println!("computed_output: {:?}", result_image2);
                    encountered_problem = true;
                    break;
                }
                println!("found good substitutions");
            }
            if encountered_problem {
                continue;
            }

            return Ok((key.clone(), value.clone()));
        }

        Err(anyhow::anyhow!("Unable to find a single substitution rule that works for all pairs"))
    }
}

struct Item {
    input: Image,
    output: Image,

    /// Positions where `input` and `output` differs
    diff_positions: HashSet<(i32, i32)>,
}
