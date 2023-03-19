use super::arc_work_model;
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
    }
}
