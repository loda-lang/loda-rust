use super::arc_work_model::{PairType, Task, Input};
use super::{ActionLabel, ImageLabel, SplitLabel, ImageSplit, ImageSplitDirection};
use super::{Image, ImageMaskBoolean};
use super::HtmlLog;

#[derive(Debug, Clone)]
pub enum Operation {
    MaskAnd,
    MaskOr,
    MaskXor,
}

impl Operation {
    pub fn execute(&self, image0: &Image, image1: &Image) -> anyhow::Result<Image> {
        match self {
            Self::MaskAnd => {
                image0.mask_and(image1)
            },
            Self::MaskOr => {
                image0.mask_or(image1)
            },
            Self::MaskXor => {
                image0.mask_xor(image1)
            },
        }
    }

    pub fn execute_with_images(&self, images: &Vec<Image>) -> anyhow::Result<Image> {
        let mut work_image = match images.first() {
            Some(value) => value.clone(),
            None => {
                return Err(anyhow::anyhow!("Cannot apply operation to empty list of images"));
            }
        };
        for (image_index, image) in images.iter().enumerate() {
            if image_index == 0 {
                continue;
            }
            work_image = self.execute(&work_image, image)?;
        }
        Ok(work_image)
    }
}

pub struct SolveSplit;

impl SolveSplit {
    // Currently it only splits horizontally.
    // It should also split vertically.
    pub fn solve(task: &Task) -> anyhow::Result<()> {
        if !task.is_output_size_same_as_input_splitview_x() {
            return Ok(());
        }

        let mut part_count_vec = Vec::<u8>::new();
        let mut separator_size_vec = Vec::<u8>::new();
        for pair in &task.pairs {
            let input: &Input = &pair.input;
            let mut found_part_count: Option<u8> = None;
            let mut found_separator_size: Option<u8> = None;
            for image_label in &input.image_meta.image_label_set {
                let split_label: &SplitLabel = match image_label {
                    ImageLabel::Split { label } => label,
                    _ => continue
                };
                match split_label {
                    SplitLabel::SplitPartCountX { count } => {
                        found_part_count = Some(*count);
                    },
                    SplitLabel::SplitSeparatorSizeX { size } => {
                        found_separator_size = Some(*size);
                    },
                    _ => continue
                }
            }
            let part_count: u8 = match found_part_count {
                Some(value) => value,
                None => {
                    println!("Unable to determine how many parts to split into");
                    return Ok(());
                }
            };
            let separator_size: u8 = match found_separator_size {
                Some(value) => value,
                None => {
                    println!("Unable to determine the separator size");
                    return Ok(());
                }
            };
            part_count_vec.push(part_count);
            separator_size_vec.push(separator_size);
            // println!("part_count: {}, separator_size: {}", part_count, separator_size);
        }

        if part_count_vec.len() != task.pairs.len() {
            return Ok(());
        }
        if separator_size_vec.len() != task.pairs.len() {
            return Ok(());
        }
        println!("task: {} parts: {:?} separators: {:?}", task.id, part_count_vec, separator_size_vec);

        let s: String = format!("task: {} number of parts: {}", task.id, part_count_vec.len());
        HtmlLog::text(s);

        let mut pair_splitted_images = Vec::<Vec::<Image>>::new();
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            let part_count: u8 = part_count_vec[pair_index];
            let separator_size: u8 = separator_size_vec[pair_index];

            let input_image: &Image = &pair.input.image;
            let images: Vec<Image> = match input_image.split(part_count, separator_size, ImageSplitDirection::IntoColumns) {
                Ok(value) => value,
                Err(error) => {
                    println!("task: {} Unable to split image: {}", task.id, error);
                    return Ok(());
                }
            };
            println!("task: {} split into {} parts", task.id, images.len());
            pair_splitted_images.push(images);
        }

        let operations = [
            Operation::MaskAnd,
            Operation::MaskOr,
            Operation::MaskXor,
        ];
        for operation in &operations {
            HtmlLog::text(&format!("task: {} operation: {:?}", task.id, operation));
            for (pair_index, _pair) in task.pairs.iter().enumerate() {
                let images: &Vec<Image> = &pair_splitted_images[pair_index];

                let work_image: Image = match operation.execute_with_images(images) {
                    Ok(value) => value,
                    Err(error) => {
                        println!("task: {} Unable to execute operation: {}", task.id, error);
                        continue;
                    }
                };

                let mut image_comparison: Vec<Image> = images.clone();
                image_comparison.push(work_image);
                HtmlLog::compare_images(image_comparison);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_or() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 1, 1, 1,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(4, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0,
            1, 1, 1, 1,
            0, 0, 0, 0,
        ];
        let input1: Image = Image::try_create(4, 3, pixels1).expect("image");

        let pixels2: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            1, 1, 1, 1,
        ];
        let input2: Image = Image::try_create(4, 3, pixels2).expect("image");

        let images = vec![input0, input1, input2];

        // Act
        let actual: Image = Operation::MaskOr.execute_with_images(&images).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
