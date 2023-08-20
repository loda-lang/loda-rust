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

            let output_image: &Image;
            let output_padding_color: u8;
            match pair.pair_type {
                PairType::Train => {
                    output_image = &pair.output.image;
                    output_padding_color = color_padding;
                },
                PairType::Test => {
                    output_image = &pair.output.test_image;
                    output_padding_color = color_padding_highlight;
                },
            }
            let mut output = Image::color(30, 30, color_outside);
            output = output.overlay_with_position(output_image, 0, 0)?;
            output = output.padding_with_color(1, output_padding_color)?;
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
