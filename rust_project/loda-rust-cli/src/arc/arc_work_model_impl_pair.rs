use super::arc_work_model;
use super::arc_work_model::{Object, ObjectType};
use super::{ActionLabel, PropertyOutput};
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

        // Save the objects on the input.
        self.input.input_objects.insert(ObjectType::RemovalOfMostPopularColorInThisImageAfterwardSegmentByNeighborAll, object_vec.clone());

        if object_vec.len() == 1 {
            // println!("OutputImage is only object in the input image");
            self.action_label_set.insert(ActionLabel::OutputImageIsTheOnlyObjectInInputImage);
        }

        // Reset the sorting
        object_vec.sort_unstable_by_key(|obj| obj.index);

        // Smallest objects first, biggest last
        object_vec.sort_unstable_by_key(|obj| obj.area());
        for _ in 0..1 {
            let object0: &Object = match object_vec.get(0) {
                Some(obj) => obj,
                None => break
            };
            let object1: &Object = match object_vec.get(1) {
                Some(obj) => obj,
                None => break
            };
            if object0.area() < object1.area() {
                // TODO: set a boolean on the object, that this is the smallest area.
                // TODO: loop over the objects. If the object has the smallest area, then insert it into the action_label_set.

                // println!("OutputImage is object with the smallest area");
                self.action_label_set.insert(ActionLabel::OutputImageIsTheObjectWithTheSmallestArea);
            }            
        }

        // Biggest objects first, smallest last
        object_vec.reverse();
        for _ in 0..1 {
            let object0: &Object = match object_vec.get(0) {
                Some(obj) => obj,
                None => break
            };
            let object1: &Object = match object_vec.get(1) {
                Some(obj) => obj,
                None => break
            };
            if object0.area() > object1.area() {
                // println!("OutputImage is object with the biggest area");
                self.action_label_set.insert(ActionLabel::OutputImageIsTheObjectWithTheBiggestArea);
            }            
        }

        // Reset the sorting
        object_vec.sort_unstable_by_key(|obj| obj.index);

        // Asymmetric objects first, symmetric last
        object_vec.sort_unstable_by_key(|obj| obj.is_symmetric_x());
        for _ in 0..1 {
            let object0: &Object = match object_vec.get(0) {
                Some(obj) => obj,
                None => break
            };
            let object1: &Object = match object_vec.get(1) {
                Some(obj) => obj,
                None => break
            };
            if object0.is_symmetric_x() != object1.is_symmetric_x() {
                // println!("OutputImage is only object that is asymmetric x, {:?}", self.id);
                self.action_label_set.insert(ActionLabel::OutputImageIsTheObjectThatIsAsymmetricX);
            }            
        }

        // Symmetric objects first, asymmetric last
        object_vec.reverse();
        for _ in 0..1 {
            let object0: &Object = match object_vec.get(0) {
                Some(obj) => obj,
                None => break
            };
            let object1: &Object = match object_vec.get(1) {
                Some(obj) => obj,
                None => break
            };
            if object0.is_symmetric_x() != object1.is_symmetric_x() {
                // println!("OutputImage is only object that is symmetric x, {:?}", self.id);
                self.action_label_set.insert(ActionLabel::OutputImageIsTheObjectThatIsSymmetricX);
            }            
        }

        // Reset the sorting
        object_vec.sort_unstable_by_key(|obj| obj.index);

        // Asymmetric objects first, symmetric last
        object_vec.sort_unstable_by_key(|obj| obj.is_symmetric_y());
        for _ in 0..1 {
            let object0: &Object = match object_vec.get(0) {
                Some(obj) => obj,
                None => break
            };
            let object1: &Object = match object_vec.get(1) {
                Some(obj) => obj,
                None => break
            };
            if object0.is_symmetric_y() != object1.is_symmetric_y() {
                // println!("OutputImage is only object that is asymmetric y, {:?}", self.id);
                self.action_label_set.insert(ActionLabel::OutputImageIsTheObjectThatIsAsymmetricY);
            }            
        }

        // Symmetric objects first, asymmetric last
        object_vec.reverse();
        for _ in 0..1 {
            let object0: &Object = match object_vec.get(0) {
                Some(obj) => obj,
                None => break
            };
            let object1: &Object = match object_vec.get(1) {
                Some(obj) => obj,
                None => break
            };
            if object0.is_symmetric_y() != object1.is_symmetric_y() {
                // println!("OutputImage is only object that is symmetric y, {:?}", self.id);
                self.action_label_set.insert(ActionLabel::OutputImageIsTheObjectThatIsSymmetricY);
            }            
        }

        Ok(())
    }
}
