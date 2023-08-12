use super::arc_work_model::{Task, Input, PairType};
use super::{ImageLabel, SplitLabel, ImageSplit, ImageSplitDirection, ImageOverlay, ImageHistogram};
use super::{Image, ImageMaskBoolean, Histogram};
use super::HtmlLog;
use itertools::Itertools;

#[derive(Debug, Clone, Default)]
pub struct OperationState {
    /// One or more `Operation::Overlay` caused an overlap.
    pub operation_overlay_detected_overlap: bool,
}

#[derive(Debug, Clone)]
pub enum Operation {
    MaskAnd,
    MaskOr,
    MaskXor,
    Overlay { mask_color: u8 },

    // Future experiments
    // KeepColorIfSame { background_color: u8, color_diff0: u8, color_diff1: u8 },
}

impl Operation {
    pub fn execute(&self, image0: &Image, image1: &Image, state: &mut OperationState) -> anyhow::Result<Image> {
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
            Self::Overlay { mask_color } => {
                let image_out: Image = image0.overlay_with_mask_color(image1, *mask_color)?;

                // Detect overlap by comparing histograms. 
                // The content is preserved if this histograms are the same. No overlap.
                // Some of the content was overdrawn if the histograms are different. Overlap detected.
                let mut histogram_input: Histogram = image0.histogram_all();
                histogram_input.add_histogram(&image1.histogram_all());
                histogram_input.set_counter_to_zero(*mask_color);

                let mut histogram_output: Histogram = image_out.histogram_all();
                histogram_output.set_counter_to_zero(*mask_color);

                if histogram_input != histogram_output {
                    state.operation_overlay_detected_overlap = true;
                }
                return Ok(image_out);
            },
        }
    }

    pub fn execute_with_images(&self, images: &Vec<Image>) -> anyhow::Result<Image> {
        let mut state = OperationState::default();
        self.execute_with_images_and_state(images, &mut state)
    }

    pub fn execute_with_images_and_state(&self, images: &Vec<Image>, state: &mut OperationState) -> anyhow::Result<Image> {
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
            work_image = self.execute(&work_image, image, state)?;
        }
        Ok(work_image)
    }

    pub fn execute_with_images_and_permutations(&self, images: &Vec<Image>, permutations: &Vec<&usize>) -> anyhow::Result<Image> {
        let mut state = OperationState::default();
        self.execute_with_images_and_permutations_and_state(images, permutations, &mut state)
    }

    pub fn execute_with_images_and_permutations_and_state(&self, images: &Vec<Image>, permutations: &Vec<&usize>, state: &mut OperationState) -> anyhow::Result<Image> {
        if images.len() != permutations.len() {
            return Err(anyhow::anyhow!("Length of images and permutations must be equal"));
        }
        let first_index: usize = match permutations.first() {
            Some(value) => **value,
            None => {
                return Err(anyhow::anyhow!("permutations is empty"));
            }
        };
        let mut work_image = match images.get(first_index) {
            Some(value) => value.clone(),
            None => {
                return Err(anyhow::anyhow!("first_index is out of bounds"));
            }
        };
        for (loop_index, permutation_index) in permutations.iter().enumerate() {
            if loop_index == 0 {
                continue;
            }
            let image: &Image = match images.get(**permutation_index) {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("permutation_index is out of bounds"));
                }
            };
    
            work_image = self.execute(&work_image, image, state)?;
        }
        Ok(work_image)
    }
}

#[derive(Debug, Clone)]
struct SplitRecord {
    part_count: u8,
    separator_size: u8,
}

impl SplitRecord {
    fn create_record_foreach_pair(task: &Task, is_horizontal_split: bool) -> anyhow::Result<Vec<SplitRecord>> {
        let mut record_vec = Vec::<SplitRecord>::new();
        for pair in &task.pairs {
            let input: &Input = &pair.input;
            let mut found_part_count: Option<u8> = None;
            let mut found_separator_size: Option<u8> = None;
            for image_label in &input.image_meta.image_label_set {
                let split_label: &SplitLabel = match image_label {
                    ImageLabel::Split { label } => label,
                    _ => continue
                };
                if is_horizontal_split {
                    match split_label {
                        SplitLabel::SplitPartCountX { count } => {
                            found_part_count = Some(*count);
                        },
                        SplitLabel::SplitSeparatorSizeX { size } => {
                            found_separator_size = Some(*size);
                        },
                        _ => continue
                    }
                } else {
                    match split_label {
                        SplitLabel::SplitPartCountY { count } => {
                            found_part_count = Some(*count);
                        },
                        SplitLabel::SplitSeparatorSizeY { size } => {
                            found_separator_size = Some(*size);
                        },
                        _ => continue
                    }
                }
            }
            let part_count: u8 = match found_part_count {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Unable to determine how many parts to split into"));
                }
            };
            let separator_size: u8 = match found_separator_size {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Unable to determine the separator size"));
                }
            };
            let record = SplitRecord {
                part_count,
                separator_size,
            };
            record_vec.push(record);
        }
        Ok(record_vec)
    }
}

pub struct SolveSplit;

impl SolveSplit {
    /// Can only split into columns or rows, not both.
    pub fn solve(task: &Task) -> anyhow::Result<()> {
        let is_split_x: bool = task.is_output_size_same_as_input_splitview_x();
        let is_split_y: bool = task.is_output_size_same_as_input_splitview_y();
        let is_horizontal_split: bool;
        let split_direction: ImageSplitDirection;
        match (is_split_x, is_split_y) {
            (true, true) => {
                return Err(anyhow::anyhow!("Cannot split both horizontally and vertically"));
            },
            (false, false) => {
                return Err(anyhow::anyhow!("Not a split in this task"));
            },
            (true, false) => {
                is_horizontal_split = true;
                split_direction = ImageSplitDirection::IntoColumns;
            },
            (false, true) => {
                is_horizontal_split = false;
                split_direction = ImageSplitDirection::IntoRows;
            }
        }

        let record_vec: Vec<SplitRecord> = SplitRecord::create_record_foreach_pair(task, is_horizontal_split)?;
        if record_vec.len() != task.pairs.len() {
            return Err(anyhow::anyhow!("task: {} mismatch in number of records and number of pairs", task.id));
        }

        let s: String = format!("task: {} parts: {:?}", task.id, record_vec);
        HtmlLog::text(s);

        let mut pair_splitted_images = Vec::<Vec::<Image>>::new();
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            let record: &SplitRecord = &record_vec[pair_index];
            let part_count: u8 = record.part_count;
            let separator_size: u8 = record.separator_size;

            let input_image: &Image = &pair.input.image;
            let images: Vec<Image> = match input_image.split(part_count, separator_size, split_direction) {
                Ok(value) => value,
                Err(error) => {
                    println!("task: {} Unable to split image: {}", task.id, error);
                    return Ok(());
                }
            };
            // println!("task: {} split into {} parts", task.id, images.len());
            pair_splitted_images.push(images);
        }

        // Is the output always the same as one of the inputs
        // Is the output sometimes the same as one of the inputs
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.pair_type != PairType::Train {
                continue;
            }
            let images: &Vec<Image> = &pair_splitted_images[pair_index];

            let mut number_of_matches: usize = 0;
            for (image_index, image) in images.iter().enumerate() {
                if *image == pair.output.image {
                    number_of_matches += 1;
                    HtmlLog::text(format!("task: {} output is the same as image: {}", task.id, image_index));
                    HtmlLog::image(&image);
                }
            }
        }

        let operations = [
            Operation::MaskAnd,
            Operation::MaskOr,
            Operation::MaskXor,
        ];
        for operation in &operations {
            HtmlLog::text(&format!("task: {} operation: {:?}", task.id, operation));
            let mut image_comparison = Vec::<Image>::new();
            for (pair_index, _pair) in task.pairs.iter().enumerate() {
                let images: &Vec<Image> = &pair_splitted_images[pair_index];

                let work_image: Image = match operation.execute_with_images(images) {
                    Ok(value) => value,
                    Err(error) => {
                        println!("task: {} Unable to execute operation: {}", task.id, error);
                        continue;
                    }
                };

                image_comparison.push(work_image);
            }
            HtmlLog::compare_images(image_comparison);
        }

        let mut shared_part_count: u8 = 0;
        let mut same_part_count_for_all_pairs = true;
        for (record_index, record) in record_vec.iter().enumerate() {
            if record_index == 0 {
                shared_part_count = record.part_count;
                continue;
            }
            if record.part_count != shared_part_count {
                same_part_count_for_all_pairs = false;
                break;
            }
        }

        // Is the input images overlayed on top of each other in a z-order
        if same_part_count_for_all_pairs && shared_part_count > 0 && shared_part_count <= 5 {
            for (pair_index, pair) in task.pairs.iter().enumerate() {
                if pair.pair_type != PairType::Train {
                    continue;
                }
                let images: &Vec<Image> = &pair_splitted_images[pair_index];
                if images.len() != shared_part_count as usize {
                    return Err(anyhow::anyhow!("task: {} mismatch in number of images and number of parts", task.id));
                }

                println!("task: {} trying permutations: {}", task.id, shared_part_count);
                // Eliminate hard coded background color
                let operation = Operation::Overlay { mask_color: 0 };
                let indices: Vec<usize> = (0..shared_part_count as usize).collect();
                let mut count: usize = 0;
                for perm in indices.iter().permutations(shared_part_count as usize) {
                    println!("{:?}", perm);
                    let mut state = OperationState::default();
                    let image: Image = operation.execute_with_images_and_permutations_and_state(images, &perm, &mut state)?;
                    // detect overlap when overlaying images
                    if state.operation_overlay_detected_overlap {
                        HtmlLog::text(format!("task: {} permutation: {:?} detected overlap", task.id, perm));
                        HtmlLog::image(&image);
                    }
                    if image == pair.output.image {
                        HtmlLog::text(format!("task: {} permutation: {:?} same as training output", task.id, perm));
                        HtmlLog::image(&image);
                    }
                    // determine best fit
                    count += 1;
                    if count > 200 {
                        break;
                    }
                }

                break;
            }
        }

        // Future experiments:
        // * overlay images, by permuting the indexes of the images, if count <=5 then it's not too many permutations.
        // * preserve color
        // * consider background color being transparent

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_mask_or() {
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

    #[test]
    fn test_20000_overlay_without_overlap() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 1, 1, 1,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(4, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0,
            2, 2, 2, 2,
            0, 0, 0, 0,
        ];
        let input1: Image = Image::try_create(4, 3, pixels1).expect("image");

        let pixels2: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            3, 3, 3, 3,
        ];
        let input2: Image = Image::try_create(4, 3, pixels2).expect("image");

        let images = vec![input0, input1, input2];

        let operation = Operation::Overlay { mask_color: 0 };

        // Act
        let mut state = OperationState::default();
        let actual: Image = operation.execute_with_images_and_state(&images, &mut state).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            2, 2, 2, 2,
            3, 3, 3, 3,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
        assert_eq!(state.operation_overlay_detected_overlap, false);
    }

    #[test]
    fn test_20001_overlay_with_overlap() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 1, 1, 1,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(4, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0,
            2, 2, 2, 2,
            0, 0, 0, 0,
        ];
        let input1: Image = Image::try_create(4, 3, pixels1).expect("image");

        let pixels2: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 9, 0,
            3, 3, 3, 3,
        ];
        let input2: Image = Image::try_create(4, 3, pixels2).expect("image");

        let images = vec![input0, input1, input2];

        let operation = Operation::Overlay { mask_color: 0 };

        // Act
        let mut state = OperationState::default();
        let actual: Image = operation.execute_with_images_and_state(&images, &mut state).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            2, 2, 9, 2,
            3, 3, 3, 3,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
        assert_eq!(state.operation_overlay_detected_overlap, true);
    }
}
