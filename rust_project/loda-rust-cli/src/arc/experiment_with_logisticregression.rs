use super::arc_work_model::{Task, PairType};
use super::{Image, ImageOverlay};
use crate::config::Config;
use std::path::{PathBuf, Path};
use serde::Serialize;
use csv::WriterBuilder;
use std::error::Error;

#[derive(Clone, Debug, Serialize)]
struct PixelColor {
    color0: u8,
    color1: u8,
    color2: u8,
    color3: u8,
    color4: u8,
    color5: u8,
    color6: u8,
    color7: u8,
    color8: u8,
    color9: u8,
    color_padding: u8,
}

impl PixelColor {
    fn new() -> Self {
        Self {
            color0: 0,
            color1: 0,
            color2: 0,
            color3: 0,
            color4: 0,
            color5: 0,
            color6: 0,
            color7: 0,
            color8: 0,
            color9: 0,
            color_padding: 0,
        }
    }

    fn set(&mut self, value: u8) {
        match value {
            0 => self.color0 = 1,
            1 => self.color1 = 1,
            2 => self.color2 = 1,
            3 => self.color3 = 1,
            4 => self.color4 = 1,
            5 => self.color5 = 1,
            6 => self.color6 = 1,
            7 => self.color7 = 1,
            8 => self.color8 = 1,
            9 => self.color9 = 1,
            _ => self.color_padding = 1,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct Record {
    classification: u8,
    is_test: u8,
    pair_id: u8,
    center: PixelColor,
    top: PixelColor,
}

pub struct ExperimentWithLogisticRegression {
    #[allow(dead_code)]
    tasks: Vec<Task>,
}

impl ExperimentWithLogisticRegression {
    #[allow(dead_code)]
    pub fn new(tasks: Vec<Task>) -> Self {
        Self {
            tasks,
        }
    }

    #[allow(dead_code)]
    pub fn run(&mut self) -> anyhow::Result<()> {
        let config = Config::load();
        let path: PathBuf = config.analytics_arc_dir().join("arc-task.csv");

        println!("will process {} tasks", self.tasks.len());

        let mut found_task: Option<&Task> = None;
        for task in &self.tasks {
            if task.id != "a79310a0" {
                continue;
            }
            found_task = Some(task);
        }
        let task: &Task = match found_task {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("didn't find a task with the id"));
            }
        };
        println!("task: {}", task.id);

        let mut records = Vec::<Record>::new();
        let mut pair_id: u8 = 0;
        for pair in &task.pairs {
            let is_test: u8;
            let original_output: Image;
            match pair.pair_type {
                PairType::Train => {
                    is_test = 0;
                    original_output = pair.output.image.clone();
                },
                PairType::Test => {
                    is_test = 1;
                    original_output = pair.output.test_image.clone();
                },
            }
            let original_input: Image = pair.input.image.clone();

            let width: u8 = original_input.width().max(original_output.width()).min(253) + 2;
            let height: u8 = original_input.height().max(original_output.height()).min(253) + 2;

            let background: Image = Image::color(width, height, 10);
            let input: Image = background.overlay_with_position(&original_input, 1, 1)?;
            let output: Image = background.overlay_with_position(&original_output, 1, 1)?;

            for y in 0..height {
                for x in 0..width {
                    let xx: i32 = x as i32;
                    let yy: i32 = y as i32;
                    let input_color: u8 = input.get(xx, yy).unwrap_or(255);
                    let input_color_top: u8 = input.get(xx, yy - 1).unwrap_or(255);
                    let output_color: u8 = output.get(xx, yy).unwrap_or(255);

                    let mut center = PixelColor::new();
                    center.set(input_color);

                    let mut top = PixelColor::new();
                    top.set(input_color_top);

                    let record = Record {
                        classification: output_color,
                        is_test,
                        pair_id,
                        center,
                        top,
                    };

                    records.push(record);
                }
            }

            pair_id += 1;
        }

        println!("saving file: {:?}", path);
        match create_csv_file_without_header(&records, &path) {
            Ok(()) => {},
            Err(error) => {
                return Err(anyhow::anyhow!("could not save: {:?}", error));
            }
        }

        Ok(())
    }
}

fn create_csv_file_without_header<S: Serialize>(records: &Vec<S>, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_path(output_path)?;
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}
