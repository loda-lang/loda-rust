use super::{Image, ImageExport, ImageOverlay, ImageStack, ImagePadding, Color, ImageSize};
use super::arc_work_model::{Task, PairType};
use std::path::PathBuf;

// Future experiments
// Order of training pairs: ascending, descending, permuted.
// Order of test pairs: ascending, descending, permuted.
// Position of the image: top-left corner, centered, bottom-right, random x, random y.
// Positions within a single pair between input image and output image: follows same position, no correspondence.
// Positions across pairs: follows same position, no correspondence.
// Colors of the image: normal histogram, reversed histogram, different colors, random colors.
// Amount of previously predicted data: none, pixels from input, all pixels from output, mix of input/output, random junk.
// Noise pixels in the input data. Can it still make correct predictions despite some noise.
// Transformation: Double the size of the input images, as long as they stay below 30x30.
// Transformation: Double the size of the output images, as long as they stay below 30x30.
// Transformation: Flip x, flip y, rotate 90, rotate 180, rotate 270.
// Transformation: Pad the input image with 1..3 pixel wide border.
// Transformation: Pad the output image with 1..3 pixel wide border.

pub struct GenerateTrainingImageFiles;

impl GenerateTrainingImageFiles {
    fn export_pixel(task: &Task, test_index: u8, x: u8, y: u8, classification: u8) -> anyhow::Result<()> {
        let color_outside: u8 = Color::DarkGrey as u8;
        let color_padding: u8 = Color::LightGrey as u8;
        let color_padding_highlight: u8 = Color::White as u8;

        let mut images = Vec::<Image>::new();
        for (_pair_index, pair) in task.pairs.iter().enumerate() {
            let mut input = Image::color(30, 30, color_outside);
            input = input.overlay_with_position(&pair.input.image, 0, 0)?;
            input = input.padding_with_color(1, color_padding)?;

            let mut output = Image::color(30, 30, color_outside);
            output = output.padding_with_color(1, color_padding)?;
            match pair.pair_type {
                PairType::Train => {
                    output = output.overlay_with_position(&pair.output.image, 1, 1)?;
                },
                PairType::Test => {
                    if pair.test_index == Some(test_index) {
                        let output_size: ImageSize = pair.output.test_image.size();
                        let mut image: Image = Image::color(output_size.width, output_size.height, color_outside);
                        _ = image.set(x as i32, y as i32, color_padding_highlight);
                        // let mut image: Image = pair.output.test_image.clone();
                        image = image.padding_with_color(1, color_padding_highlight)?;
                        output = output.overlay_with_position(&image, 0, 0)?;
                    } else {
                        let output_size: ImageSize = pair.output.test_image.size();
                        let image: Image = Image::color(output_size.width, output_size.height, color_outside);
                        // let image: Image = pair.output.test_image.clone();
                        output = output.overlay_with_position(&image, 1, 1)?;
                    }
                },
            }
            let pair_image: Image = input.vjoin(output)?;

            images.push(pair_image);
        }
        let task_image: Image = Image::hstack(images)?;

        let filename = format!("color{}_test{}_x{}_y{}.png", classification, test_index, x, y);
        let basepath: PathBuf = PathBuf::from("/Users/neoneye/Downloads/image_save");
        let path: PathBuf = basepath.join(filename);
        task_image.save_as_file(&path)?;
        Ok(())
    }
    
    fn export_pair(task: &Task, test_index: u8) -> anyhow::Result<()> {
        for (_pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.test_index != Some(test_index) {
                continue;
            }
            let output_image: Image = pair.output.test_image.clone();
            let output_size: ImageSize = output_image.size();
            for y in 0..output_size.height {
                for x in 0..output_size.width {
                    let classification: u8 = output_image.get(x as i32, y as i32).unwrap_or(255);
                    Self::export_pixel(task, test_index, x, y, classification)?;
                }
            }
        }
        Ok(())
    }

    pub fn export_task(task: &Task) -> anyhow::Result<()> {
        let count_test: u8 = task.count_test().min(255) as u8;
        for test_index in 0..count_test {
            Self::export_pair(task, test_index)?;
        }        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::arc_json_model;
    use crate::arc::arc_work_model::Task;

    fn save_as_file(name: &str) -> anyhow::Result<()> {
        let json_task: arc_json_model::Task = arc_json_model::Task::load_testdata(name)?;
        let task: Task = Task::try_from(&json_task)?;
        GenerateTrainingImageFiles::export_task(&task)?;
        Ok(())
    }

    // #[test]
    fn test_90000_overlay_cf98881b() {
        save_as_file("cf98881b").expect("ok");
    }
}
