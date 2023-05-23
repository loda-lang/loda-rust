use super::arc_work_model::{Task, PairType};
use super::{Image, ImageOverlay};
use crate::config::Config;
use std::path::{PathBuf, Path};
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use csv::WriterBuilder;
use std::error::Error;

#[derive(Clone, Debug)]
struct PixelColor {
    value: u8,
}

impl From<u8> for PixelColor {
    fn from(value: u8) -> Self {
        Self {
            value,
        }
    }
}

impl Serialize for PixelColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut color0: u8 = 0;
        let mut color1: u8 = 0;
        let mut color2: u8 = 0;
        let mut color3: u8 = 0;
        let mut color4: u8 = 0;
        let mut color5: u8 = 0;
        let mut color6: u8 = 0;
        let mut color7: u8 = 0;
        let mut color8: u8 = 0;
        let mut color9: u8 = 0;
        let mut color_other: u8 = 0;
        match self.value {
            0 => color0 = 1,
            1 => color1 = 1,
            2 => color2 = 1,
            3 => color3 = 1,
            4 => color4 = 1,
            5 => color5 = 1,
            6 => color6 = 1,
            7 => color7 = 1,
            8 => color8 = 1,
            9 => color9 = 1,
            _ => color_other = 1,
        };
        let mut s = serializer.serialize_struct("PixelColor", 11)?;
        s.serialize_field("color0", &color0)?;
        s.serialize_field("color1", &color1)?;
        s.serialize_field("color2", &color2)?;
        s.serialize_field("color3", &color3)?;
        s.serialize_field("color4", &color4)?;
        s.serialize_field("color5", &color5)?;
        s.serialize_field("color6", &color6)?;
        s.serialize_field("color7", &color7)?;
        s.serialize_field("color8", &color8)?;
        s.serialize_field("color9", &color9)?;
        s.serialize_field("color_other", &color_other)?;
        s.end()
    }
}

#[derive(Clone, Debug, Serialize)]
struct Record {
    classification: u8,
    is_test: u8,
    pair_id: u8,
    top_left: PixelColor,
    top: PixelColor,
    top_right: PixelColor,
    left: PixelColor,
    center: PixelColor,
    right: PixelColor,
    bottom_left: PixelColor,
    bottom: PixelColor,
    bottom_right: PixelColor,
    distance_top: u8,
    distance_bottom: u8,
    distance_left: u8,
    distance_right: u8,
}

pub struct ExperimentWithLogisticRegression {
    #[allow(dead_code)]
    tasks: Vec<Task>,

    config: Config,
}

impl ExperimentWithLogisticRegression {
    #[allow(dead_code)]
    pub fn new(tasks: Vec<Task>) -> Self {
        let config = Config::load();
        Self {
            tasks,
            config,
        }
    }

    #[allow(dead_code)]
    pub fn run(&mut self) -> anyhow::Result<()> {
        println!("loaded {} tasks", self.tasks.len());

        let task_ids = [
            "3618c87e",
            "3aa6fb7a",
            "6f8cd79b",
            "95990924",
            "a699fb00",
            "a79310a0",
            "b6afb2da",
            "bb43febb",
            "d364b489",
        ];
        for task_id in task_ids {
            self.export_task(task_id)?;
        }
        Ok(())
    }

    fn export_task(&self, task_id: &str) -> anyhow::Result<()> {
        // println!("exporting task: {}", task_id);
        let path: PathBuf = self.config.analytics_arc_dir().join(format!("{}.csv", task_id));

        let mut found_task: Option<&Task> = None;
        for task in &self.tasks {
            if task.id != task_id {
                continue;
            }
            found_task = Some(task);
        }
        let task: &Task = match found_task {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("didn't find a task_id: {}", task_id));
            }
        };
        println!("exporting task: {}", task.id);

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
                    let output_color: u8 = output.get(xx, yy).unwrap_or(255);

                    let top_left: u8 = input.get(xx -1, yy - 1).unwrap_or(255);
                    let top: u8 = input.get(xx, yy - 1).unwrap_or(255);
                    let top_right: u8 = input.get(xx + 1, yy - 1).unwrap_or(255);
                    let left: u8 = input.get(xx - 1, yy).unwrap_or(255);
                    let center: u8 = input.get(xx, yy).unwrap_or(255);
                    let right: u8 = input.get(xx + 1, yy).unwrap_or(255);
                    let bottom_left: u8 = input.get(xx - 1, yy + 1).unwrap_or(255);
                    let bottom: u8 = input.get(xx, yy + 1).unwrap_or(255);
                    let bottom_right: u8 = input.get(xx + 1, yy + 1).unwrap_or(255);

                    let distance_top: u8 = y.min(3);
                    let distance_bottom: u8 = ((height as i32) - 1 - yy).min(3) as u8;
                    let distance_left: u8 = x.min(3);
                    let distance_right: u8 = ((width as i32) - 1 - xx).min(3) as u8;

                    let record = Record {
                        classification: output_color,
                        is_test,
                        pair_id,
                        top_left: PixelColor::from(top_left),
                        top: PixelColor::from(top),
                        top_right: PixelColor::from(top_right),
                        left: PixelColor::from(left),
                        center: PixelColor::from(center),
                        right: PixelColor::from(right),
                        bottom_left: PixelColor::from(bottom_left),
                        bottom: PixelColor::from(bottom),
                        bottom_right: PixelColor::from(bottom_right),
                        distance_top,
                        distance_bottom,
                        distance_left,
                        distance_right,
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
