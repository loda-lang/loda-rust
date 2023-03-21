use super::arc_work_model;
use super::arc_work_model::{Object, ObjectType};
use super::{ActionLabel, ObjectLabel, PropertyOutput};
use super::{ImageFind, ImageSymmetry};

#[allow(unused_imports)]
use crate::arc::{HtmlLog, ImageToHTML};

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

        _ = self.analyze_object_why_is_the_output_present_once_in_input();
    }

    fn analyze_object_why_is_the_output_present_once_in_input(&mut self) -> anyhow::Result<()> {
        // if self.id != "72ca375d,pair1,train" {
        //     return Ok(());
        // }
        // if self.id != "d56f2372,pair0,train" {
        //     return Ok(());
        // }
        // if self.id != "d56f2372,pair1,train" {
        //     return Ok(());
        // }
        if !self.action_label_set.contains(&ActionLabel::OutputImageIsPresentExactlyOnceInsideInputImage) {
            // println!("a");
            return Ok(());
        }

        // TODO: don't recompute if it's already cached
        // if self.input.input_objects.contains(&ObjectType::RemovalOfMostPopularColorInThisImageAfterwardSegmentByNeighborAll) {
        //    return Ok(());
        // }

        let mut object_vec: Vec<Object> = self.input.find_objects_using_histogram_most_popular_color()?;
        let mut found_count: usize = 0;
        for object in &object_vec {
            if object.cropped_object_image == self.output.image {
                found_count += 1;
            }
        }

        if found_count != 1 {
            // println!("b");
            return Ok(());
        }
        // println!("OutputImage is one of the objects in the input image");
        // HtmlLog::html(self.input.image.to_html());
        // HtmlLog::html(self.output.image.to_html());
        // HtmlLog::text("separator");

        Object::assign_labels_to_objects(&mut object_vec);

        // Reset the sorting
        object_vec.sort_unstable_by_key(|obj| obj.index);

        // OutputImage is only object in the input image
        if object_vec.len() == 1 {
            if let Some(object) = object_vec.first() {
                if object.cropped_object_image == self.output.image {
                    self.action_label_set.insert(ActionLabel::OutputImageIsTheOnlyObjectInInputImage);
                }
            }
        }

        // Populate the action_label_set with data from the objects. If the object has the smallest area, then insert it into the action_label_set.
        let object_labels: [ObjectLabel; 6] = [
            ObjectLabel::TheOnlyOneWithSmallestArea,
            ObjectLabel::TheOnlyOneWithBiggestArea,
            ObjectLabel::TheOnlyOneWithAsymmetryX,
            ObjectLabel::TheOnlyOneWithSymmetryX,
            ObjectLabel::TheOnlyOneWithAsymmetryY,
            ObjectLabel::TheOnlyOneWithSymmetryY,
        ];
        for object in &object_vec {
            if object.cropped_object_image != self.output.image {
                continue;
            }

            for object_label in &object_labels {
                if object.object_label_set.contains(object_label) {
                    let label = ActionLabel::OutputImageIsTheObjectWithObjectLabel {
                        object_label: object_label.clone()
                    };
                    self.action_label_set.insert(label);
                }
            }
        }

        // caching - Save the objects on the input.
        self.input.input_objects.insert(ObjectType::RemovalOfMostPopularColorInThisImageAfterwardSegmentByNeighborAll, object_vec);

        Ok(())
    }
}
