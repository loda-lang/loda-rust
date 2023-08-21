use super::{Image, ImageExport, ImageOverlay, ImageStack, ImagePadding, Color, ImageSize, OverlayPositionId, ImageSymmetry, ImageRotate, ImageResize, ImageReplaceColor};
use super::arc_work_model::{Task, Pair, PairType};
use std::collections::HashMap;
use std::path::PathBuf;

// Future experiments
// Order of training pairs: ascending, descending, permuted.
// Order of test pairs: ascending, descending, permuted.
// Positions across pairs: follows same position, no correspondence.
// Amount of previously predicted data: none, pixels from input, all pixels from output, mix of input/output, random junk.
// Use mispredicted output from logistic regression as content in the prediction area.
// Noise pixels in the input data. Can it still make correct predictions despite some noise.
// Image size. Don't always use 30x30 as the image size. Sometimes use a compact representation, such as 10x10.
// Combine 2 tasks into 1 task, separated with a splitview.

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

        let filename = format!("{}_mutation{}_test{}_x{}_y{}_color{}.png", task.id, mutation_name, test_index, x, y, classification);
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

    /// Transform the original images.
    /// 
    /// Double/triple the size of the input images, as long as they stay below 30x30.
    /// 
    /// Flip x.
    /// 
    /// Rotate 90, 180, 270.
    /// 
    /// Padding with 1..3 pixel wide border.
    /// 
    /// When the `color_offset` is zero, then the color palette is unchanged.
    /// 
    /// When the `color_offset` is a value in the range [1..9]
    /// Then the color palette gets rotated forward by that amount.
    /// 
    /// When the `color_offset` is a value in the range [10..19]
    /// Then the color palette gets rotated backwards by that amount.
    fn transform_image(image: &Image, is_flipx: bool, rotate_count: u8, scalex: u8, scaley: u8, padding: u8, padding_color: u8, color_offset: u8) -> anyhow::Result<Image> {
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
        if padding > 0 {
            image = image.padding_with_color(padding, padding_color)?;
            if image.width() > 30 || image.height() > 30 {
                return Err(anyhow::anyhow!("Cannot create mutation, the image is too large. Width: {}, Height: {}", image.width(), image.height()));
            }
        }
        if color_offset > 0 {
            // Future experiments:
            // Randomize colors.

            let mut color_map = HashMap::<u8, u8>::new();
            if color_offset >= 10 {
                // Reverse the color palette and rotate
                for i in 0..10u8 {
                    color_map.insert(i, (9 + color_offset - i) % 10);
                }
            } else {
                // Rotate the color palette
                for i in 0..10u8 {
                    color_map.insert(i, (i + color_offset) % 10);
                }
            }
            image = image.replace_colors_with_hashmap(&color_map)?;
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

        // Pad the image.
        let in_padding_count = Self::take_u8(&mut current_mutation, 3);
        let in_padding_color = Self::take_u8(&mut current_mutation, 10);
        let out_padding_count = Self::take_u8(&mut current_mutation, 3);
        let out_padding_color = Self::take_u8(&mut current_mutation, 10);

        // Rotate the color palette.
        let in_out_color_offset = Self::take_u8(&mut current_mutation, 20);

        if in_padding_count > 0 && task.input_histogram_intersection.get(in_padding_color) > 0 {
            return Err(anyhow::anyhow!("Cannot create mutation, the color cannot be used for padding"));
        }
        if out_padding_count > 0 && task.output_histogram_intersection.get(out_padding_color) > 0 {
            return Err(anyhow::anyhow!("Cannot create mutation, the color cannot be used for padding"));
        }

        let mut task_copy: Task = task.clone();
        for pair in task_copy.pairs.iter_mut() {
            {
                let image: Image = Self::transform_image(&pair.input.image, in_flipx, in_rotate, in_scalex, in_scaley, in_padding_count, in_padding_color, in_out_color_offset)?;
                pair.input.image = image;
            }
            match pair.pair_type {
                PairType::Train => {
                    let image: Image = Self::transform_image(&pair.output.image, out_flipx, out_rotate, out_scalex, out_scaley, out_padding_count, out_padding_color, in_out_color_offset)?;
                    pair.output.image = image;
                },
                PairType::Test => {
                    let image: Image = Self::transform_image(&pair.output.test_image, out_flipx, out_rotate, out_scalex, out_scaley, out_padding_count, out_padding_color, in_out_color_offset)?;
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
            259818103,
            480205127,
            523106332,
        ];
        for (index, mutation_index) in mutation_indexes.iter().enumerate() {
            let mutation_name: String = format!("{}", mutation_index);
            println!("Index {} mutation: {}", index, mutation_name);
            match Self::export_task_with_mutation(task, *mutation_index, &mutation_name) {
                Ok(()) => {},
                Err(error) => {
                    println!("Index {} Failed to export task with mutation: {} error: {:?}", index, mutation_name, error);
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
