use super::{Image, ImageExport, ImageOverlay, ImageStack, ImagePadding, Color, ImageSize, OverlayPositionId, ImageSymmetry, ImageRotate, ImageResize, ImageReplaceColor};
use super::arc_work_model::{Task, Pair, PairType};
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;

// Future experiments
// Order of training pairs: ascending, descending, permuted.
// Order of test pairs: ascending, descending, permuted.
// Positions across pairs: follows same position, no correspondence.
// Noise pixels in the input data. Can it still make correct predictions despite some noise.
// Image size. Don't always use 30x30 as the image size. Sometimes use a compact representation, such as 10x10.
// Combine 2 tasks into 1 task, separated with a splitview.
// Export the entire obfuscated task to a json file.
// Explain what settings the mutation index gets translated into.

#[derive(Debug, Clone)]
struct MutationConfig {
    // Flip the image.
    in_flipx: bool,
    out_flipx: bool,

    // Rotate the image.
    in_rotate: u8,
    out_rotate: u8,

    // Scale of the image.
    in_scalex: u8,
    in_scaley: u8,
    out_scalex: u8,
    out_scaley: u8,

    // Padding around the image.
    in_padding_count: u8,
    in_padding_color: u8,
    out_padding_count: u8,
    out_padding_color: u8,

    // Rotate the color palette.
    color_offset: u8,
    color_map: HashMap<u8, u8>,

    // Positioning of the images inside the template
    in_x: OverlayPositionId,
    in_y: OverlayPositionId,
    out_x: OverlayPositionId,
    out_y: OverlayPositionId,
}

impl MutationConfig {
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

    fn create(mutation_index: u64) -> Self {
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
        let color_offset = Self::take_u8(&mut current_mutation, 20);
        
        // Position of the image: top-left corner, centered, bottom-right.
        // Future experiment with position: random x, random y.
        let in_x = Self::take_position_id(&mut current_mutation);
        let in_y = Self::take_position_id(&mut current_mutation);
        let out_x = Self::take_position_id(&mut current_mutation);
        let out_y = Self::take_position_id(&mut current_mutation);

        Self {
            in_flipx,
            in_rotate,
            in_scalex,
            in_scaley,
            in_padding_count,
            in_padding_color,
            out_flipx,
            out_rotate,
            out_scalex,
            out_scaley,
            out_padding_count,
            out_padding_color,
            color_offset,
            in_x,
            in_y,
            out_x,
            out_y,
            color_map: Self::transformed_color_map(color_offset),
        }
    }

    /// When the `offset` is zero, then the color palette is unchanged.
    /// 
    /// When the `offset` is a value in the range [1..9]
    /// Then the color palette gets rotated forward by that amount.
    /// 
    /// When the `offset` is a value in the range [10..19]
    /// Then the color palette gets rotated backwards by that amount.
    fn transformed_color_map(offset: u8) -> HashMap<u8, u8> {
        let mut color_map = HashMap::<u8, u8>::new();
        if offset >= 10 {
            // Reverse the color palette and rotate
            for i in 0..10u8 {
                color_map.insert(i, (9 + offset - i) % 10);
            }
        } else {
            // Rotate the color palette
            for i in 0..10u8 {
                color_map.insert(i, (i + offset) % 10);
            }
        }
        color_map
    }
}

#[derive(Debug, Clone, Default)]
pub struct GenerateTrainingImageFiles {
    classification_counters: [u32; 10],
    accumulated_file_count: u32,
    accumulated_byte_count: u64,
}

impl GenerateTrainingImageFiles {
    fn generate_pair_image(pair: &Pair, test_index: u8, x: u8, y: u8, config: &MutationConfig) -> anyhow::Result<Image> {
        let color_outside: u8 = Color::DarkGrey as u8;
        let color_padding: u8 = Color::LightGrey as u8;
        let color_padding_highlight: u8 = Color::White as u8;

        let mut input = Image::color(30, 30, color_outside);
        input = input.overlay_with_position_id(&pair.input.image, config.in_x, config.in_y)?;
        input = input.padding_with_color(1, color_padding)?;

        let the_output: Image;
        match pair.pair_type {
            PairType::Train => {
                let mut output = Image::color(30, 30, color_outside);
                output = output.overlay_with_position_id(&pair.output.image, config.out_x, config.out_y)?;
                output = output.padding_with_color(1, color_padding)?;
                the_output = output;
            },
            PairType::Test => {
                let output_size: ImageSize = pair.output.test_image.size();
                // Future experiments
                // Partially copy pixel data from the expected output image.
                // Partially copy pixel data from the input image.
                // Mix between data from the input image and the expected output image.
                // Use mispredicted output from logistic regression as content in the prediction area.
                // Partially fill with junk data.
                // When copying, then copy with noise data to mimic bad predictions.
                let mut image: Image = Image::color(output_size.width, output_size.height, color_outside);
                if pair.test_index == Some(test_index) {
                    _ = image.set(x as i32, y as i32, color_padding_highlight);
                }
                image = image.padding_with_color(1, color_padding_highlight)?;
                let mut output = Image::color(30, 30, color_outside);
                output = output.padding_with_color(1, color_padding)?;
                output = output.overlay_with_position_id(&image, config.out_x, config.out_y)?;
                the_output = output;
            }
        }
        let pair_image: Image = input.vjoin(the_output)?;
        Ok(pair_image)
    }

    fn export_image(task: &Task, test_index: u8, x: u8, y: u8, config: &MutationConfig, path: &Path) -> anyhow::Result<()> {
        let mut images = Vec::<Image>::new();
        for (_pair_index, pair) in task.pairs.iter().enumerate() {
            let pair_image: Image = Self::generate_pair_image(pair, test_index, x, y, config)?;
            images.push(pair_image);
        }
        let task_image: Image = Image::hstack(images)?;
        task_image.save_as_file(&path)?;
        Ok(())
    }
    
    fn export_test_pairs(&mut self, task: &Task, test_index: u8, config: &MutationConfig, mutation_name: &str) -> anyhow::Result<()> {
        for (_pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.test_index != Some(test_index) {
                continue;
            }
            let output_image: Image = pair.output.test_image.clone();
            let output_size: ImageSize = output_image.size();
            for y in 0..output_size.height {
                for x in 0..output_size.width {
                    let classification: u8 = output_image.get(x as i32, y as i32).unwrap_or(255);
                    let counter_index: usize = (classification as usize) % 10;
                    let counter: u32 = self.classification_counters[counter_index];

                    // let filename = format!("{}_mutation{}_test{}_x{}_y{}_color{}.png", task.id, mutation_name, test_index, x, y, classification);
                    let filename = format!("color{}.{}.png", classification, counter);
                    // let filename = format!("{}.png", self.accumulated_file_count);
                    let basepath: PathBuf = PathBuf::from("/Users/neoneye/Downloads/image_save");
                    let path: PathBuf = basepath.join(filename);

                    Self::export_image(task, test_index, x, y, config, &path)?;
                    let filesize: u64 = path.metadata()?.len();

                    self.classification_counters[counter_index] += 1;
                    self.accumulated_file_count += 1;
                    self.accumulated_byte_count += filesize;
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
    /// Apply color map.
    fn transform_image(image: &Image, is_flipx: bool, rotate_count: u8, scalex: u8, scaley: u8, padding: u8, padding_color: u8, color_map: &HashMap<u8, u8>) -> anyhow::Result<Image> {
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

        // Future experiments:
        // Randomize colors.
        image = image.replace_colors_with_hashmap(color_map)?;
        Ok(image)
    }

    fn export_task_with_mutation(&mut self, task: &Task, config: &MutationConfig, mutation_name: &str) -> anyhow::Result<()> {
        if config.in_padding_count > 0 && task.input_histogram_intersection.get(config.in_padding_color) > 0 {
            return Err(anyhow::anyhow!("Cannot create mutation, the color cannot be used for padding"));
        }
        if config.out_padding_count > 0 && task.output_histogram_intersection.get(config.out_padding_color) > 0 {
            return Err(anyhow::anyhow!("Cannot create mutation, the color cannot be used for padding"));
        }

        let mut task_copy: Task = task.clone();

        // Transform the input images
        for pair in task_copy.pairs.iter_mut() {
            let transformed_image: Image = Self::transform_image(
                &pair.input.image, 
                config.in_flipx, 
                config.in_rotate, 
                config.in_scalex, 
                config.in_scaley, 
                config.in_padding_count, 
                config.in_padding_color, 
                &config.color_map,
            )?;
            pair.input.image = transformed_image;
        }

        // Transform the output images
        for pair in task_copy.pairs.iter_mut() {
            let original_image: Image = match pair.pair_type {
                PairType::Train => pair.output.image.clone(),
                PairType::Test => pair.output.test_image.clone(),
            };
            let transformed_image: Image = Self::transform_image(
                &original_image, 
                config.out_flipx, 
                config.out_rotate, 
                config.out_scalex, 
                config.out_scaley, 
                config.out_padding_count, 
                config.out_padding_color, 
                &config.color_map,
            )?;
            match pair.pair_type {
                PairType::Train => {
                    pair.output.image = transformed_image;
                },
                PairType::Test => {
                    pair.output.test_image = transformed_image;
                }
            }
        }

        let count_test: u8 = task_copy.count_test().min(255) as u8;
        for test_index in 0..count_test {
            self.export_test_pairs(&task_copy, test_index, &config, mutation_name)?;
        }        
        Ok(())
    }

    fn export_task_inner(&mut self, task: &Task, random_seed: u64, include_zero: bool) -> anyhow::Result<()> {
        let mut rng: StdRng = StdRng::seed_from_u64(random_seed);

        let limit_iteration_count: usize = 100000;
        let mut iteration_count: usize = 0;
        while iteration_count < limit_iteration_count { 
            let index: usize = iteration_count;
            iteration_count += 1;

            let mutation_index: u64;
            if index == 0 && include_zero {
                mutation_index = 0;
            } else {
                mutation_index = rng.gen();
            }
            let config = MutationConfig::create(mutation_index);
            let mutation_name: String = format!("{}", mutation_index);
            println!("Index {} accumulated_file_count: {} accumulated_byte_count: {} classification_counters: {:?}", index, self.accumulated_file_count, self.accumulated_byte_count, self.classification_counters);
            // println!("Index {} mutation: {} config: {:?}", index, mutation_name, config);
            match self.export_task_with_mutation(task, &config, &mutation_name) {
                Ok(()) => {},
                Err(error) => {
                    println!("Index {} Failed to export task with mutation: {} error: {:?}", index, mutation_name, error);
                }
            }

            let reached_limit_file_count: bool = self.accumulated_file_count > 10000;
            let reached_limit_byte_count: bool = self.accumulated_byte_count > 1024 * 1024 * 50;
            if reached_limit_file_count || reached_limit_byte_count {
                println!("Index {} reached_limit_file_count: {} reached_limit_byte_count: {}", index, reached_limit_file_count, reached_limit_byte_count);
                break;
            }
        }

        println!("iteration_count: {}", iteration_count);
        println!("accumulated_file_count: {}", self.accumulated_file_count);
        println!("accumulated_byte_count: {}", self.accumulated_byte_count);
        println!("number of files per classification: {:?}", self.classification_counters);
        Ok(())
    }

    pub fn export_task_train(task: &Task) -> anyhow::Result<()> {
        let random_seed: u64 = 0;
        let include_zero: bool = false;
        let mut instance = Self::default();
        instance.export_task_inner(task, random_seed, include_zero)?;
        Ok(())
    }

    pub fn export_task_test(task: &Task) -> anyhow::Result<()> {
        let random_seed: u64 = 42;
        let include_zero: bool = true;
        let mut instance = Self::default();
        instance.export_task_inner(task, random_seed, include_zero)?;
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
        GenerateTrainingImageFiles::export_task_train(&task)?;
        // GenerateTrainingImageFiles::export_task_test(&task)?;
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
