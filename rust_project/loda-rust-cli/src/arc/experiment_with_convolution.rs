use super::arc_work_model::{Task, PairType};
use super::{Image, ImageCompare, ImagePadding, ImageSize, ImageMaskCount};
use anyhow::Context;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::distributions::{Distribution, Uniform};

#[allow(unused_imports)]
use super::{HtmlLog, ImageToHTML};

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
    global_weights: Vec::<f32>,
    local_weights: Vec::<f32>,
}

impl ExperimentWithConvolution {
    pub fn new(tasks: Vec<Task>) -> Self {
        Self {
            tasks,
            global_weights: vec!(),
            local_weights: vec!(),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        println!("will process {} tasks", self.tasks.len());

        let task0: Task;
        {
            let first_task: &Task = self.tasks.first().context("one or more")?;
            task0 = first_task.clone();
        }
        println!("task: {}", task0.id);

        // Random weights
        let random_seed: u64 = 1;
        let mut rng: StdRng = StdRng::seed_from_u64(random_seed);
        let step = Uniform::<u16>::new(0, 1001);
        {
            let mut weights = Vec::<f32>::new();
            for _ in 0..5 {
                let random_value: u16 = step.sample(&mut rng);
                let weight_between0and1: f32 = (random_value as f32) / 1000.0;
                let weight: f32 = weight_between0and1 + 1.0;
                weights.push(weight);
            }
            self.global_weights = weights;
        }
        {
            let mut weights = Vec::<f32>::new();
            for _ in 0..19 {
                let random_value: u16 = step.sample(&mut rng);
                let weight_between0and1: f32 = (random_value as f32) / 1000.0;
                let weight: f32 = weight_between0and1 + 1.0;
                weights.push(weight);
            }
            self.local_weights = weights;
        }
        println!("global_weights: {}", self.global_weights.len());
        println!("local_weights: {}", self.local_weights.len());

        // Add some junk to the initial weights
        for _ in 0..100 {
            self.mutate_global_weights(&mut rng);
            self.mutate_local_weights(&mut rng);
        }


        // Extract samples
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
            // train with the samples
            for sample in &samples_input {
                self.mutate_local_weights_with_sample(sample, &mut rng);
            }
            for sample in &samples_output {
                self.mutate_local_weights_with_sample(sample, &mut rng);
            }
        }


        // query the model
        let pair_count: usize = task0.pairs.len();
        for (pair_index, pair) in task0.pairs.iter().enumerate() {
            let pair_id: f32 = ((pair_index as f32) + 1.0) / ((pair_count as f32) + 1.0);
            // println!("pair_id: {}", pair_id);

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
            let computed_image: Image = self.query(pair_id, size)?;

            // measure difference from expected image
            let diff: Image = computed_image.diff(expected_image)?;
            let intersection: u16 = diff.mask_count_zero();
            let union: u16 = (size.width as u16) * (size.height as u16);
            if union == 0 {
                return Err(anyhow::anyhow!("Encountered a task with an empty image. {}", pair.id));
            }
            let jaccard_index: f32 = (intersection as f32) / (union as f32);
            println!("pair: {} jaccard_index: {}", pair.id, jaccard_index);

            HtmlLog::text(format!("pair: {}", pair.id));
            HtmlLog::image(&expected_image);
            HtmlLog::image(&computed_image);
            HtmlLog::image(&diff);
        }

        // undo if the mutation was too poor

        // the global weights stays locked while training for a single task
        // local weights that gets updated while training with a single task
        // mutate the global weights.

        self.mutate_global_weights(&mut rng);

        // repeat training

        Ok(())
    }

    fn mutate_local_weights_with_sample(&mut self, sample: &Sample, rng: &mut StdRng) {
        for y in 0..3 {
            for x in 0..3 {
                let pixel: u8 = sample.convolution3x3.get(x as i32, y as i32).unwrap_or(255);

                // propagate pixel value to all local weights, based on position, pair_id, input/output
                // update local weights
            }
        }
        self.mutate_local_weights(rng);
    }

    fn mutate_global_weights(&mut self, rng: &mut StdRng) {
        let step = Uniform::<u16>::new(0, 1001);
        for weight in self.global_weights.iter_mut() {
            let random_value: u16 = step.sample(rng);
            let weight_between0and1: f32 = (random_value as f32) / 1000.0;
            let adjustment: f32 = (weight_between0and1 - 0.5) / 100.0;
            *weight = (*weight + adjustment).max(2.0).min(1.0);
        }
    }

    fn mutate_local_weights(&mut self, rng: &mut StdRng) {
        let step = Uniform::<u16>::new(0, 1001);
        for weight in self.local_weights.iter_mut() {
            let random_value: u16 = step.sample(rng);
            let weight_between0and1: f32 = (random_value as f32) / 1000.0;
            let adjustment: f32 = (weight_between0and1 - 0.5) / 100.0;
            *weight = (*weight + adjustment).max(2.0).min(1.0);
        }
    }

    fn query(&self, pair_id: f32, size: ImageSize) -> anyhow::Result<Image> {
        let mut result_image = Image::zero(size.width, size.height);
        for y in 0..size.height {
            let yy: f32 = ((y as f32) + 1.0) / ((size.height as f32) + 1.0);
            for x in 0..size.width {
                let xx: f32 = ((x as f32) + 1.0) / ((size.width as f32) + 1.0);
                let color: u8 = self.query_xy(pair_id, xx, yy)?;
                _ = result_image.set(x as i32, y as i32, color);
            }
        }
        Ok(result_image)
    }

    fn query_xy(&self, pair_id: f32, x: f32, y: f32) -> anyhow::Result<u8> {
        // println!("pair_id: {} x: {} y: {}", pair_id, x, y);

        // Experiments with multiple iterations and passing data to the next iteration
        for i in 0..1 {
            let global_address: usize = 3 * i;
            let g0: f32 = self.global_weights[global_address + 0] * pair_id;
            let g1: f32 = self.global_weights[global_address + 1] * x;
            let g2: f32 = (2.0 - self.global_weights[global_address + 1]) * x;
            let g3: f32 = self.global_weights[global_address + 2] * y;
            let g4: f32 = (2.0 - self.global_weights[global_address + 2]) * y;

            let local_address: usize = 19 * i;
            let l0: f32 = self.local_weights[local_address + 0] * g0;
            let l1: f32 = self.local_weights[local_address + 1] * g1;
            let l2: f32 = self.local_weights[local_address + 2] * g2;
            let l3: f32 = self.local_weights[local_address + 3] * g3;
            let l4: f32 = self.local_weights[local_address + 4] * g4;

            let sum: f32 = l0 + l1 + l2 + l3 + l4;
            let product: f32 = l0 * l1 * l2 * l3 * l4;
            let min: f32 = l0.min(l1).min(l2).min(l3).min(l4);
            let max: f32 = l0.max(l1).max(l2).max(l3).max(l4);
            let minus01: f32 = l0 - l1;
            let minus12: f32 = l1 - l2;
            let minus23: f32 = l2 - l3;
            let minus34: f32 = l3 - l4;

            let values: [f32; 13] = [
                self.local_weights[local_address + 5] * l0,
                self.local_weights[local_address + 6] * l1,
                self.local_weights[local_address + 7] * l2,
                self.local_weights[local_address + 8] * l3,
                self.local_weights[local_address + 9] * l4,
                self.local_weights[local_address + 10] * sum,
                self.local_weights[local_address + 11] * product,
                self.local_weights[local_address + 12] * min,
                self.local_weights[local_address + 13] * max,
                self.local_weights[local_address + 15] * minus01,
                self.local_weights[local_address + 16] * minus12,
                self.local_weights[local_address + 17] * minus23,
                self.local_weights[local_address + 18] * minus34,
            ];

            let mut found_value: f32 = f32::MIN;
            let mut found_index: usize = 0;
            for (value_index, value) in values.iter().enumerate() {
                if *value > found_value {
                    found_value = *value;
                    found_index = value_index;
                }
            }
            // println!("pair_id: {} x: {} y: {}  value: {}", pair_id, x, y, found_value);
            let color: u8 = u8::try_from(found_index).unwrap_or(255);
            return Ok(color);
        }
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
