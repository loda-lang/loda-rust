use super::{Image, ImageExport, ImageOverlay, ImageStack, ImagePadding, Color, ImageSize, OverlayPositionId, ImageSymmetry, ImageRotate, ImageResize};
use super::arc_work_model::{Task, Pair, PairType};
use std::path::PathBuf;

// Future experiments
// Order of training pairs: ascending, descending, permuted.
// Order of test pairs: ascending, descending, permuted.
// Positions within a single pair between input image and output image: follows same position, no correspondence.
// Positions across pairs: follows same position, no correspondence.
// Colors of the image: normal histogram, reversed histogram, different colors, random colors.
// Amount of previously predicted data: none, pixels from input, all pixels from output, mix of input/output, random junk.
// Use mispredicted output from logistic regression as content in the prediction area.
// Noise pixels in the input data. Can it still make correct predictions despite some noise.
// Image size. Don't always use 30x30 as the image size. Sometimes use a compact representation, such as 10x10.
// Transformation: Double the size of the input images, as long as they stay below 30x30.
// Transformation: Double the size of the output images, as long as they stay below 30x30.
// Transformation: Pad the input image with 1..3 pixel wide border.
// Transformation: Pad the output image with 1..3 pixel wide border.

pub struct GenerateTrainingImageFiles;

impl GenerateTrainingImageFiles {
    fn take_position_id(index: &mut u64) -> OverlayPositionId {
        let variant: u64 = *index % 5;
        *index /= 5;
        match variant {
            0 => OverlayPositionId::Zero,
            1 => OverlayPositionId::OneThird,
            2 => OverlayPositionId::Half,
            3 => OverlayPositionId::TwoThird,
            _ => OverlayPositionId::One,
        }
    }

    fn take_bool(index: &mut u64) -> bool {
        let variant: u64 = *index % 2;
        *index /= 2;
        match variant {
            0 => false,
            _ => true,
        }
    }

    fn take_u8(index: &mut u64, count: u8) -> u8 {
        let variant: u64 = *index % (count as u64);
        *index /= count as u64;
        variant.min(255) as u8
    }

    fn generate_pair_image(pair: &Pair, test_index: u8, x: u8, y: u8, mutation_index: u64) -> anyhow::Result<Image> {
        let color_outside: u8 = Color::DarkGrey as u8;
        let color_padding: u8 = Color::LightGrey as u8;
        let color_padding_highlight: u8 = Color::White as u8;

        let mut in_x = OverlayPositionId::Half;
        let mut in_y = OverlayPositionId::Half;
        let mut out_x = OverlayPositionId::Half;
        let mut out_y = OverlayPositionId::Half;

        {
            let mut current_mutation: u64 = mutation_index;

            // Position of the image: top-left corner, centered, bottom-right.
            // Future experiment with position: random x, random y.
            in_x = Self::take_position_id(&mut current_mutation);
            in_y = Self::take_position_id(&mut current_mutation);
            out_x = Self::take_position_id(&mut current_mutation);
            out_y = Self::take_position_id(&mut current_mutation);
        }

        let mut input = Image::color(30, 30, color_outside);
        input = input.overlay_with_position_id(&pair.input.image, in_x, in_y)?;
        input = input.padding_with_color(1, color_padding)?;

        let the_output: Image;
        match pair.pair_type {
            PairType::Train => {
                let mut output = Image::color(30, 30, color_outside);
                output = output.overlay_with_position_id(&pair.output.image, out_x, out_y)?;
                output = output.padding_with_color(1, color_padding)?;
                the_output = output;
            },
            PairType::Test => {
                let output_size: ImageSize = pair.output.test_image.size();
                let mut image: Image = Image::color(output_size.width, output_size.height, color_outside);
                if pair.test_index == Some(test_index) {
                    _ = image.set(x as i32, y as i32, color_padding_highlight);
                }
                image = image.padding_with_color(1, color_padding_highlight)?;
                let mut output = Image::color(30, 30, color_outside);
                output = output.padding_with_color(1, color_padding)?;
                output = output.overlay_with_position_id(&image, out_x, out_y)?;
                the_output = output;
            }
        }
        let pair_image: Image = input.vjoin(the_output)?;
        Ok(pair_image)
    }

    fn export_image(task: &Task, test_index: u8, x: u8, y: u8, mutation_index: u64, mutation_name: &str, classification: u8) -> anyhow::Result<()> {
        let mut images = Vec::<Image>::new();
        for (_pair_index, pair) in task.pairs.iter().enumerate() {
            let pair_image: Image = Self::generate_pair_image(pair, test_index, x, y, mutation_index)?;
            images.push(pair_image);
        }
        let task_image: Image = Image::hstack(images)?;

        let filename = format!("{}_test{}_x{}_y{}_color{}_mutation{}.png", task.id, test_index, x, y, classification, mutation_name);
        let basepath: PathBuf = PathBuf::from("/Users/neoneye/Downloads/image_save");
        let path: PathBuf = basepath.join(filename);
        task_image.save_as_file(&path)?;
        Ok(())
    }
    
    fn export_test_pairs(task: &Task, test_index: u8, mutation_index: u64, mutation_name: &str) -> anyhow::Result<()> {
        for (_pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.test_index != Some(test_index) {
                continue;
            }
            let output_image: Image = pair.output.test_image.clone();
            let output_size: ImageSize = output_image.size();
            for y in 0..output_size.height {
                for x in 0..output_size.width {
                    let classification: u8 = output_image.get(x as i32, y as i32).unwrap_or(255);
                    Self::export_image(task, test_index, x, y, mutation_index, mutation_name, classification)?;
                }
            }
        }
        Ok(())
    }

    fn transform_image(image: &Image, is_flipx: bool, rotate_count: u8, scalex: u8, scaley: u8) -> anyhow::Result<Image> {
        let mut image = image.clone();
        if scalex > 1 || scaley > 1 {
            let width: u16 = image.width() as u16 * scalex as u16;
            let height: u16 = image.height() as u16 * scaley as u16;
            if width > 30 || height > 30 {
                return Err(anyhow::anyhow!("Cannot create mutation, the image is too large. Width: {}, Height: {}", width, height));
            }
            image = image.resize(width as u8, height as u8)?;
        }
        if is_flipx {
            image = image.flip_x()?;
        }
        if rotate_count > 0 {
            image = image.rotate(rotate_count as i8)?;
        }
        Ok(image)
    }

    pub fn export_task_with_mutation(task: &Task, mutation_index: u64, mutation_name: &str) -> anyhow::Result<()> {
        let mut current_mutation: u64 = mutation_index;

        // Flip the image.
        let in_flipx = Self::take_bool(&mut current_mutation);
        let out_flipx = Self::take_bool(&mut current_mutation);

        // Rotate the image.
        let in_rotate = Self::take_u8(&mut current_mutation, 4);
        let out_rotate = Self::take_u8(&mut current_mutation, 4);

        // Scale the image.
        let in_scalex = Self::take_u8(&mut current_mutation, 3) + 1;
        let in_scaley = Self::take_u8(&mut current_mutation, 3) + 1;
        let out_scalex = Self::take_u8(&mut current_mutation, 3) + 1;
        let out_scaley = Self::take_u8(&mut current_mutation, 3) + 1;

        let mut task_copy: Task = task.clone();
        for pair in task_copy.pairs.iter_mut() {
            {
                let image: Image = Self::transform_image(&pair.input.image, in_flipx, in_rotate, in_scalex, in_scaley)?;
                pair.input.image = image;
            }
            match pair.pair_type {
                PairType::Train => {
                    let image: Image = Self::transform_image(&pair.output.image, out_flipx, out_rotate, out_scalex, out_scaley)?;
                    pair.output.image = image;
                },
                PairType::Test => {
                    let image: Image = Self::transform_image(&pair.output.test_image, out_flipx, out_rotate, out_scalex, out_scaley)?;
                    pair.output.test_image = image;
                }
            }
        }

        let count_test: u8 = task_copy.count_test().min(255) as u8;
        for test_index in 0..count_test {
            Self::export_test_pairs(&task_copy, test_index, mutation_index, mutation_name)?;
        }        
        Ok(())
    }

    pub fn export_task(task: &Task) -> anyhow::Result<()> {
        let mutation_indexes: [u64; 4] = [
            0,
            4 * 256 + 10 * 16,
            312 * 256 + 9 * 16 + 2,
            624 * 256 + 15 * 16 + 1,
        ];
        for mutation_index in mutation_indexes {
            let mutation_name: String = format!("{}", mutation_index);
            match Self::export_task_with_mutation(task, mutation_index, &mutation_name) {
                Ok(()) => {},
                Err(error) => {
                    println!("Failed to export task with mutation: {} error: {:?}", mutation_name, error);
                }
            }
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

    // #[test]
    fn test_90001_overlay_281123b4() {
        save_as_file("281123b4").expect("ok");
    }

    // #[test]
    fn test_90002_overlay_e98196ab() {
        save_as_file("e98196ab").expect("ok");
    }
}
