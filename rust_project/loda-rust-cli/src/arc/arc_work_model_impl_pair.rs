use super::arc_work_model;
use super::{Label, PropertyOutput};
use super::ImageSymmetry;

impl arc_work_model::Pair {
    pub fn assign_labels_for_output(&mut self) {
        let width_output: u8 = self.output.image.width();
        let height_output: u8 = self.output.image.height();

        {
            let label = Label::OutputPropertyIsConstant { 
                output: PropertyOutput::OutputWidth, 
                value: width_output
            };
            self.label_set.insert(label);
        }

        {
            let label = Label::OutputPropertyIsConstant { 
                output: PropertyOutput::OutputHeight, 
                value: height_output
            };
            self.label_set.insert(label);
        }

        if width_output >= 2 || height_output >= 2 {
            if let Ok(is_symmetric) = self.output.image.is_symmetric_x() {
                if is_symmetric {
                    self.label_set.insert(Label::OutputImageIsSymmetricX);
                }
            }
        }

        if width_output >= 2 || height_output >= 2 {
            if let Ok(is_symmetric) = self.output.image.is_symmetric_y() {
                if is_symmetric {
                    self.label_set.insert(Label::OutputImageIsSymmetricY);
                }
            }
        }
    }
}
