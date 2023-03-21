use crate::arc::{HtmlLog, ImageToHTML};

use super::{arc_work_model, ImageSegment, Image, ImageMask, ImageCrop};
use super::{ActionLabel, PropertyOutput};
use super::{ImageFind, ImageSymmetry};

impl arc_work_model::Pair {
    pub fn update_action_label_set(&mut self) {
        let width_input: u8 = self.input.image.width();
        let height_input: u8 = self.input.image.height();
        let width_output: u8 = self.output.image.width();
        let height_output: u8 = self.output.image.height();

        {
            let label = ActionLabel::OutputPropertyIsConstant { 
                output: PropertyOutput::OutputWidth, 
                value: width_output
            };
            self.action_label_set.insert(label);
        }

        {
            let label = ActionLabel::OutputPropertyIsConstant { 
                output: PropertyOutput::OutputHeight, 
                value: height_output
            };
            self.action_label_set.insert(label);
        }

        if width_output >= 2 || height_output >= 2 {
            if let Ok(is_symmetric) = self.output.image.is_symmetric_x() {
                if is_symmetric {
                    self.action_label_set.insert(ActionLabel::OutputImageIsSymmetricX);
                }
            }
        }

        if width_output >= 2 || height_output >= 2 {
            if let Ok(is_symmetric) = self.output.image.is_symmetric_y() {
                if is_symmetric {
                    self.action_label_set.insert(ActionLabel::OutputImageIsSymmetricY);
                }
            }
        }

        if width_input >= width_output && height_input >= height_output {
            if let Ok(count) = self.input.image.count_occurrences(&self.output.image) {
                if count >= 1 {
                    self.action_label_set.insert(ActionLabel::OutputImageOccurInsideInputImage { count });
                    self.action_label_set.insert(ActionLabel::OutputImageOccurInsideInputImageOneOrMoreTimes);
                }
                if count == 1 {
                    self.action_label_set.insert(ActionLabel::OutputImageIsPresentExactlyOnceInsideInputImage);
                }
            }
        }

        if width_output >= width_input && height_output >= height_input {
            if let Ok(count) = self.output.image.count_occurrences(&self.input.image) {
                if count >= 1 {
                    self.action_label_set.insert(ActionLabel::InputImageOccurInsideOutputImage { count });
                    self.action_label_set.insert(ActionLabel::InputImageOccurInsideOutputImageOneOrMoreTimes);
                }
                if count == 1 {
                    self.action_label_set.insert(ActionLabel::InputImageIsPresentExactlyOnceInsideOutputImage);
                }
            }
        }

        if self.input.histogram == self.output.histogram {
            self.action_label_set.insert(ActionLabel::OutputImageHistogramEqualToInputImageHistogram);
        }

        // _ = self.analyze_object_why_is_the_output_present_once_in_input();
    }

    fn analyze_object_why_is_the_output_present_once_in_input(&self) -> anyhow::Result<()> {
        if !self.action_label_set.contains(&ActionLabel::OutputImageIsPresentExactlyOnceInsideInputImage) {
            return Ok(());
        }

        let objects: Vec<Image> = self.input.image.find_objects(super::ImageSegmentAlgorithm::Neighbors)?;
        let mut found_count: usize = 0;
        let mut object_and_meta_vec: Vec<ObjectAndMeta> = vec!();
        for (index, object) in objects.iter().enumerate() {
            let (x, y, width, height) = match object.bounding_box() {
                Some(value) => value,
                None => continue
            };
            let cropped_object_image: Image = self.input.image.crop(x, y, width, height)?;

            let object_and_meta = ObjectAndMeta {
                index: index,
                mask: object.clone(),
                cropped_object_image: cropped_object_image.clone(),
            };
            object_and_meta_vec.push(object_and_meta);
            if cropped_object_image == self.output.image {
                found_count += 1;
            }
        }
        if found_count != 1 {
            return Ok(());
        }
        // println!("OutputImage is one of the objects in the input image");
        HtmlLog::html(self.input.image.to_html());
        HtmlLog::html(self.output.image.to_html());
        HtmlLog::text("separator");

        if object_and_meta_vec.len() == 1 {
            println!("OutputImage is only object in the input image");
        }

        // Reset the sorting
        object_and_meta_vec.sort_unstable_by_key(|obj| obj.index);

        // Smallest objects first, biggest last
        object_and_meta_vec.sort_unstable_by_key(|obj| obj.area());
        for _ in 0..1 {
            let object0: &ObjectAndMeta = match object_and_meta_vec.get(0) {
                Some(obj) => obj,
                None => break
            };
            let object1: &ObjectAndMeta = match object_and_meta_vec.get(1) {
                Some(obj) => obj,
                None => break
            };
            if object0.area() < object1.area() {
                println!("OutputImage is object with the smallest area");
            }            
        }

        // Biggest objects first, smallest last
        object_and_meta_vec.reverse();
        for _ in 0..1 {
            let object0: &ObjectAndMeta = match object_and_meta_vec.get(0) {
                Some(obj) => obj,
                None => break
            };
            let object1: &ObjectAndMeta = match object_and_meta_vec.get(1) {
                Some(obj) => obj,
                None => break
            };
            if object0.area() > object1.area() {
                println!("OutputImage is object with the biggest area");
            }            
        }

        Ok(())
    }
}

struct ObjectAndMeta {
    index: usize,
    mask: Image,
    cropped_object_image: Image,
}

impl ObjectAndMeta {
    fn area(&self) -> u16 {
        let image = &self.cropped_object_image;
        (image.width() as u16) * (image.height() as u16)
    }
}
