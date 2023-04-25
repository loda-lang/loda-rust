use crate::arc::ImageMaskCount;

use super::arc_work_model::{Task, PairType};
use super::{Image, ImageCompare, ImagePadding, ImageSize};
use anyhow::Context;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::distributions::{Distribution, Uniform};

#[derive(Clone, Copy, Debug)]
enum InputOutputType {
    Input,
    Output,
}

#[derive(Clone, Debug)]
struct Sample {
    convolution3x3: Image,
    position_x: u8,
    position_y: u8,
    image_width: u8,
    image_height: u8,
    input_output_type: InputOutputType,
}

pub struct ExperimentWithConvolution {
    tasks: Vec<Task>,
}

impl ExperimentWithConvolution {
    pub fn new(tasks: Vec<Task>) -> Self {
        Self {
            tasks
        }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        println!("will process {} tasks", self.tasks.len());

        let task0: &Task = self.tasks.first().context("one or more")?;
        println!("task: {}", task0.id);

        // Random weights
        let random_seed: u64 = 1;
        let mut rng: StdRng = StdRng::seed_from_u64(random_seed);
        let step = Uniform::<u16>::new(0, 1001);
        let mut weights = Vec::<f32>::new();
        for _ in 0..9 {
            let random_value: u16 = step.sample(&mut rng);
            let weight_between0and1: f32 = (random_value as f32) / 1000.0;
            let weight: f32 = weight_between0and1 + 1.0;
            weights.push(weight);
        }
        println!("weights: {:?}", weights);

        // Extract samples
        // for task in &self.tasks {

            // the global weights stays locked while training for a single task
            // local weights that gets updated while training with a single task
            for pair in &task0.pairs {
                if pair.pair_type == PairType::Test {
                    continue;
                }
                let samples_input: Vec<Sample> = Self::extract_samples(&pair.input.image, InputOutputType::Input)?;
                let samples_output: Vec<Sample> = Self::extract_samples(&pair.output.image, InputOutputType::Output)?;
                println!("pair: {} samples_input: {} samples_output: {}", pair.id, samples_input.len(), samples_output.len());

                // mutate the local weights
            }
        // }

        // train with the samples

        // query the model
        // for task in &self.tasks {
            for pair in &task0.pairs {
                let size: ImageSize;
                let expected_image: &Image;
                match pair.pair_type {
                    PairType::Test => {
                        // let image: &Image = &pair.output.test_image;
                        // expected_image = image;
                        // size = ImageSize { width: image.width(), height: image.height() };
                        continue;
                    },
                    PairType::Train => {
                        let image: &Image = &pair.output.image;
                        expected_image = image;
                        size = ImageSize { width: image.width(), height: image.height() };
                    }
                }
                let computed_image: Image = self.query(size)?;

                // measure difference from expected image
                let diff: Image = computed_image.diff(expected_image)?;
                let intersection: u16 = diff.mask_count_zero();
                let union: u16 = (size.width as u16) * (size.height as u16);
                if union == 0 {
                    return Err(anyhow::anyhow!("Encountered a task with an empty image. {}", pair.id));
                }
                let jaccard_index: f32 = (intersection as f32) / (union as f32);
                println!("pair: {} jaccard_index: {}", pair.id, jaccard_index);
            }
        // }

        // undo if the mutation was too poor

        // the global weights stays locked while training for a single task
        // local weights that gets updated while training with a single task
        // mutate the global weights.

        // repeat training

        Ok(())
    }

    fn query(&self, size: ImageSize) -> anyhow::Result<Image> {
        let mut result_image = Image::zero(size.width, size.height);
        for y in 0..size.height {
            for x in 0..size.width {
                let color: u8 = self.query_xy(x, y)?;
                _ = result_image.set(x as i32, y as i32, color);
            }
        }
        Ok(result_image)
    }

    fn query_xy(&self, x: u8, y: u8) -> anyhow::Result<u8> {
        let color: u8 = 0;
        Ok(color)
    }

    fn extract_samples(input: &Image, input_output_type: InputOutputType) -> anyhow::Result<Vec<Sample>> {
        let padded_image: Image = input.padding_with_color(1, 255)?;

        let width: u8 = padded_image.width();
        let height: u8 = padded_image.height();
        if width < 3 || height < 3 {
            return Err(anyhow::anyhow!("too small image, must be 3x3 or bigger"));
        }
        let mut samples = Vec::<Sample>::new();
        let mut conv_bitmap = Image::zero(3, 3);
        let image_width: u8 = input.width();
        let image_height: u8 = input.height();
        for self_y in 0..image_height {
            for self_x in 0..image_width {
                for conv_y in 0..3u8 {
                    for conv_x in 0..3u8 {
                        let get_x: i32 = (self_x as i32) + (conv_x as i32);
                        let get_y: i32 = (self_y as i32) + (conv_y as i32);
                        let pixel_value: u8 = padded_image.get(get_x, get_y)
                            .ok_or_else(|| anyhow::anyhow!("image.get({},{}) returned None", get_x, get_y))?;
                        conv_bitmap.set(conv_x as i32, conv_y as i32, pixel_value)
                            .ok_or_else(|| anyhow::anyhow!("conv_bitmap.set({},{}) returned None", conv_x, conv_y))?;
                    }
                }
                let sample = Sample {
                    convolution3x3: conv_bitmap.clone(),
                    position_x: self_x,
                    position_y: self_y,
                    image_width,
                    image_height,
                    input_output_type,
                };
                samples.push(sample);
            }
        }
        Ok(samples)
    }
}
