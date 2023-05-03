use super::{Image, ImageCompare, Rectangle, ImageCrop, ImageColorProfile, ImagePadding, ImageReplaceSimple};
use std::collections::HashSet;

pub struct SubstitutionRule;

impl SubstitutionRule {
    pub fn find_rule(pairs: &Vec<(Image, Image)>) -> anyhow::Result<(Image, Image)> {
        // Ascending complexity
        // We prefer the simplest rules, so the simplest substitution rules comes at the top.
        // We try to avoid advanced rules, the more complex substitution rules comes at the bottom.
        let areas: [(u8, u8); 12] = [
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
        for (width, height) in areas {
            match Self::find_substitution_with_size(pairs, width, height) {
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

    fn find_substitution_with_size(pairs: &Vec<(Image, Image)>, crop_width: u8, crop_height: u8) -> anyhow::Result<(Image, Image)> {
        println!("crop size: width {} height {}", crop_width, crop_height);
        let mut replacements = Vec::<(Image, Image)>::new();
        for (input, output) in pairs {
            let diff: Image = input.diff(&output)?;
            let width: u8 = input.width();
            let height: u8 = input.height();
            let mut positions = Vec::<(u8, u8)>::new();
            let mut position_set = HashSet::<(i32, i32)>::new();
            for y in 0..height {
                for x in 0..width {
                    let get_x: i32 = x as i32;
                    let get_y: i32 = y as i32;
                    if diff.get(get_x, get_y).unwrap_or(0) > 0 {
                        positions.push((x, y));
                        position_set.insert((x as i32, y as i32));
                    }
                }
            }
            println!("positions: {:?}", positions);

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
                            if position_set.contains(&xy) {
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
                let replace_source: Image = match input.crop(*rect) {
                    Ok(value) => value,
                    Err(error) => {
                        // println!("crop is outside the input image. error: {:?}", error);
                        continue;
                    }
                };
                // println!("replace_source: {:?}", replace_source);
                let replace_target: Image = match output.crop(*rect) {
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
            for (input, output) in pairs {
                let background_color: u8 = input.most_popular_color().unwrap_or(255);

                let mut result_image: Image = input.padding_with_color(1, background_color)?;
                let count: u16 = result_image.replace_simple(&key, &value)?;
                if count == 0 {
                    println!("no replacements were performed. reject this replacement");
                    encountered_problem = true;
                    break;
                }
                let crop_rect = Rectangle::new(1, 1, input.width(), input.height());
                let result_image2: Image = result_image.crop(crop_rect)?;
                if result_image2 != *output {
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
