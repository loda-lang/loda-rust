use super::arc_work_model::Task;
use super::{Image, ImagePadding};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::distributions::{Distribution, Uniform};

#[derive(Clone, Debug)]
struct Sample {
    convolution3x3: Image,
    position_x: u8,
    position_y: u8,
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
        for task in &self.tasks {
            for pair in &task.pairs {
                let padded_image: Image = pair.input.image.padding_with_color(1, 255)?;
                let samples: Vec<Sample> = Self::samples_from_input(&padded_image)?;
                println!("pair: {} samples: {}", pair.id, samples.len());
            }
        }

        // train with the samples

        // query the model

        // undo if the mutation was too poor
        
        // repeat training

        Ok(())
    }

    fn samples_from_input(input: &Image) -> anyhow::Result<Vec<Sample>> {
        let width: u8 = input.width();
        let height: u8 = input.height();
        if width < 3 || height < 3 {
            return Err(anyhow::anyhow!("too small image, must be 3x3 or bigger"));
        }
        let mut samples = Vec::<Sample>::new();
        let mut conv_bitmap = Image::zero(3, 3);
        for self_y in 0..height-2 {
            for self_x in 0..width-2 {
                for conv_y in 0..3u8 {
                    for conv_x in 0..3u8 {
                        let get_x: i32 = (self_x as i32) + (conv_x as i32);
                        let get_y: i32 = (self_y as i32) + (conv_y as i32);
                        let pixel_value: u8 = input.get(get_x, get_y)
                            .ok_or_else(|| anyhow::anyhow!("image.get({},{}) returned None", get_x, get_y))?;
                        conv_bitmap.set(conv_x as i32, conv_y as i32, pixel_value)
                            .ok_or_else(|| anyhow::anyhow!("conv_bitmap.set({},{}) returned None", conv_x, conv_y))?;
                    }
                }
                let sample = Sample {
                    convolution3x3: conv_bitmap.clone(),
                    position_x: self_x,
                    position_y: self_y,
                };
                samples.push(sample);
            }
        }
        Ok(samples)
    }
}
