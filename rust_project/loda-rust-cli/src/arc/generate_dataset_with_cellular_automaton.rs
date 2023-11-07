use super::{CellularAutomaton, cellular_automaton::rule};
use super::{Image, ImageSize, RandomImage, ImageMaskCount};
use super::HtmlLog;
use bloomfilter::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy)]
enum Strategy {
    DoNothing,
    ServiettesOneStep,
    ServiettesTwoSteps,
    HighLifeOneStep,
    HighLifeTwoSteps,
}

struct GenerateDataset;

impl GenerateDataset {
    fn curriculum_easy() -> anyhow::Result<()> {
        let sizes: [u8; 4] = [
            3, 4, 5, 6
            // 7, 8, 9, 10
        ];
        let temperatures: [u8; 9] = [
            10, 20, 30, 40, 50, 60, 70, 80, 90
        ];
        let strategies: [Strategy; 5] = [
            Strategy::DoNothing,
            Strategy::ServiettesOneStep,
            Strategy::ServiettesTwoSteps,
            Strategy::HighLifeOneStep,
            Strategy::HighLifeTwoSteps,
        ];

        let bloom_items_count = 1000;
        let false_positive_rate = 0.01;
        let mut bloom = Bloom::<Image>::new_for_fp_rate(bloom_items_count, false_positive_rate);

        let mut count_input_all_empty: usize = 0;
        let mut count_input_all_alive: usize = 0;
        let mut count_input_one_cell_empty: usize = 0;
        let mut count_input_one_cell_alive: usize = 0;

        for i in 0..100 {
            let mut rng = StdRng::seed_from_u64(i);
            let width: u8 = *sizes.choose(&mut rng).unwrap();
            let height: u8 = *sizes.choose(&mut rng).unwrap();
            let temperature: u8 = *temperatures.choose(&mut rng).unwrap();
            let strategy: Strategy = *strategies.choose(&mut rng).unwrap();

            let size = ImageSize::new(width, height);
            let mut input: Image = RandomImage::two_colors(&mut rng, size, 0, 1, temperature)?;

            // Mutate the input image, to get different distributions
            match strategy {
                Strategy::DoNothing => {},
                Strategy::HighLifeOneStep => {
                    let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::HighLife>::with_image(&input, None);
                    ca.step(1);
                    input = ca.image().clone();
                },
                Strategy::HighLifeTwoSteps => {
                    let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::HighLife>::with_image(&input, None);
                    ca.step(2);
                    input = ca.image().clone();
                },
                Strategy::ServiettesOneStep => {
                    let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::Serviettes>::with_image(&input, None);
                    ca.step(1);
                    input = ca.image().clone();
                },
                Strategy::ServiettesTwoSteps => {
                    let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::Serviettes>::with_image(&input, None);
                    ca.step(2);
                    input = ca.image().clone();
                },
            }

            let (input_count0, input_count1, _count_other) = input.mask_count();
            let is_input_all_empty: bool = input_count1 == 0;
            let is_input_all_alive: bool = input_count0 == 0;
            let is_input_one_cell_empty: bool = input_count0 == 1;
            let is_input_one_cell_alive: bool = input_count1 == 1;

            if is_input_all_empty {
                count_input_all_empty += 1;
                if count_input_all_empty > 3 {
                    debug!("ignoring input with all empty");
                    continue;
                }
            }
            if is_input_all_alive {
                count_input_all_alive += 1;
                if count_input_all_alive > 3 {
                    debug!("ignoring input with all alive");
                    continue;
                }
            }
            if is_input_one_cell_empty {
                count_input_one_cell_empty += 1;
                if count_input_one_cell_empty > 3 {
                    debug!("ignoring input with one cell empty");
                    continue;
                }
            }
            if is_input_one_cell_alive {
                count_input_one_cell_alive += 1;
                if count_input_one_cell_alive > 3 {
                    debug!("ignoring input with one cell alive");
                    continue;
                }
            }
            
            if bloom.check(&input) {
                debug!("skipping duplicate");
                continue;
            }
            bloom.set(&input);
            
            let mut ca_nowrap: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&input, Some(0));
            ca_nowrap.step_once();
            let output_without_wrap: Image = ca_nowrap.image().clone();

            let mut ca_wrap: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&input, None);
            ca_wrap.step_once();
            let output_with_wrap: Image = ca_wrap.image().clone();

            let compare_images: Vec<Image> = vec![
                input.clone(),
                output_without_wrap.clone(),
                output_with_wrap.clone(),
            ];
            let same_output_for_wrap_and_nowrap: bool = output_without_wrap == output_with_wrap;
            if same_output_for_wrap_and_nowrap {
                HtmlLog::text("wrap is identical to nowrap");
                HtmlLog::compare_images(compare_images);
            } else {
                HtmlLog::text("wrap is different than nowrap");
                HtmlLog::compare_images(compare_images);
            }

            let mut markdown = String::new();
            markdown.push_str("# Conway's Game of Life\n\n");
            markdown.push_str("Perform 1 step.\n\n");
            markdown.push_str("## Input\n\n");
            markdown.push_str(&Self::image_to_markdown_fenced_code_block(&input));
            markdown.push_str("\n\n");
            Self::caption_for_input_output_image(&mut markdown, &input);
            markdown.push_str("\n");
            markdown.push_str("## Output without wrap\n\n");
            markdown.push_str(&Self::image_to_markdown_fenced_code_block(&output_without_wrap));
            markdown.push_str("\n\n");
            Self::caption_for_input_output_image(&mut markdown, &output_without_wrap);
            Self::caption_for_output_compared_to_input(&mut markdown, &output_without_wrap, &input);
            markdown.push_str("\n");
            markdown.push_str("## Output with wrap\n\n");
            markdown.push_str(&Self::image_to_markdown_fenced_code_block(&output_with_wrap));
            markdown.push_str("\n\n");
            Self::caption_for_input_output_image(&mut markdown, &output_with_wrap);
            Self::caption_for_output_compared_to_input(&mut markdown, &output_with_wrap, &input);
            markdown.push_str("\n");
            markdown.push_str("## Status\n\n");
            if same_output_for_wrap_and_nowrap {
                markdown.push_str("The outputs are identical.\n");
            } else {
                markdown.push_str("The outputs are different.\n");
            }
            markdown.push_str("\n\n");
            println!("{}---\n\n", markdown);
        }
        Ok(())
    }

    fn caption_for_input_output_image(markdown: &mut String, image: &Image) {
        let (count_empty, count_alive, _count_other) = image.mask_count();
        if count_alive == 0 {
            markdown.push_str("All cells are empty.\n");
        }
        if count_empty == 0 {
            markdown.push_str("All cells are alive.\n");
        }
        if count_alive == 1 {
            HtmlLog::text("Only one cell is alive.");
            markdown.push_str("Only one cell is alive.\n");
        }
        if count_empty == 1 {
            HtmlLog::text("Only one cell is empty.");
            markdown.push_str("Only one cell is empty.\n");
        }
    }

    fn caption_for_output_compared_to_input(markdown: &mut String, output: &Image, input: &Image) {
        if output == input {
            markdown.push_str("This output is identical to the input.\n");
            let (count0, count1, _count_other) = input.mask_count();
            if count0 > 0 && count1 > 0 {
                markdown.push_str("Still life.\n");
                HtmlLog::text("Still life.");
            }
        } else {
            markdown.push_str("This output is different than the input.\n");
        }
    }

    fn image_to_markdown_fenced_code_block(image: &Image) -> String {
        format!("```\n{}\n```", GenerateDataset::image_to_string(image))
    }

    fn image_to_string(image: &Image) -> String {
        let mut result = String::new();
        for y in 0..image.height() {
            if y > 0 {
                result.push('\n');
            }
            for x in 0..image.width() {
                let value: u8 = image.get(x as i32, y as i32).unwrap_or(0);
                let character: char = match value {
                    0 => '.',
                    1 => '*',
                    _ => '?',
                };
                result.push(character);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_image_to_string() {
        // Act
        let pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 1, 1,
            1, 1, 255,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: String = GenerateDataset::image_to_string(&input);
        
        // Assert
        assert_eq!(actual, "*..\n.**\n**?");
    }

    // #[test]
    fn test_20000_do_something() {
        for i in 0..10u64 {
            let size = ImageSize::new(6, 5);
            let step0: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(i), size, 0, 1, 25).expect("ok");
            let mut images = Vec::<Image>::new();
            images.push(step0.clone());
            let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&step0, None);
            for _ in 0..2 {
                ca.step_once();
                images.push(ca.image().clone());
            }
            HtmlLog::compare_images(images);
        }
    }

    // #[test]
    fn test_20001_do_something() {
        for i in 0..20u64 {
            let size = ImageSize::new(10, 8);
            let step0: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(i), size, 0, 1, 35).expect("ok");
            let step1: Image = RandomImage::draw_dots(&mut StdRng::seed_from_u64(i+5), &step0, 2, 5).expect("ok");
            let step2: Image = RandomImage::draw_dots(&mut StdRng::seed_from_u64(i+8), &step1, 3, 5).expect("ok");
            let mut images = Vec::<Image>::new();
            images.push(step2.clone());
            let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLifeExtra>::with_image(&step0, None);
            for _ in 0..4 {
                ca.step_once();
                images.push(ca.image().clone());
            }
            HtmlLog::compare_images(images);
        }
    }

    // #[test]
    fn test_20002_do_something() {
        GenerateDataset::curriculum_easy().expect("ok");
    }
}
