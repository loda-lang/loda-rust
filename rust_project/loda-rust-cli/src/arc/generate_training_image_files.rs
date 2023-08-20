use super::{Image, ImageExport, ImageOverlay, ImageStack, ImagePadding, Color};
use super::arc_work_model::{Task, PairType};
use std::path::PathBuf;

pub struct GenerateTrainingImageFiles;

impl GenerateTrainingImageFiles {
    pub fn export_task(task: &Task) -> anyhow::Result<()> {
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
                    let image: Image = pair.output.test_image.padding_with_color(1, color_padding_highlight)?;
                    output = output.overlay_with_position(&image, 0, 0)?;
                },
            }
            let pair_image: Image = input.vjoin(output)?;

            images.push(pair_image);
        }
        let task_image: Image = Image::hstack(images)?;

        let path = PathBuf::from("/Users/neoneye/Downloads/image_save/result.png");
        task_image.save_as_file(&path)?;
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
