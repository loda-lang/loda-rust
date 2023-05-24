use super::arc_work_model::{Task, PairType};
use super::{Image, ImageOverlay};
use crate::common::create_csv_file;
use crate::config::Config;
use std::path::PathBuf;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
struct Record {
    pair_id: u8,
    is_test: u8,
    classification: u8,
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
            let mut is_test: u8 = 0;
            if pair.pair_type == PairType::Test {
                is_test = 1;
            }
            let original_input: Image = pair.input.image.clone();
            let original_output: Image = pair.output.image.clone();

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
                    let output_color: u8 = output.get(xx, yy).unwrap_or(255);

                    let mut record = Record {
                        is_test,
                        pair_id,
                        classification: output_color,
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
                    };

                    match input_color {
                        0 => record.color0 = 1,
                        1 => record.color1 = 1,
                        2 => record.color2 = 1,
                        3 => record.color3 = 1,
                        4 => record.color4 = 1,
                        5 => record.color5 = 1,
                        6 => record.color6 = 1,
                        7 => record.color7 = 1,
                        8 => record.color8 = 1,
                        9 => record.color9 = 1,
                        _ => record.color_padding = 1,
                    }

                    records.push(record);
                }
            }

            pair_id += 1;
        }

        println!("saving file: {:?}", path);
        match create_csv_file(&records, &path) {
            Ok(()) => {},
            Err(error) => {
                return Err(anyhow::anyhow!("could not save: {:?}", error));
            }
        }

        Ok(())
    }
}
