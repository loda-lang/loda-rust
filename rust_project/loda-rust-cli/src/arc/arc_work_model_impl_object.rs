use super::arc_work_model;
use super::ImageSymmetry;

impl arc_work_model::Object {
    pub fn area(&self) -> u16 {
        let image = &self.cropped_object_image;
        (image.width() as u16) * (image.height() as u16)
    }

    pub fn is_symmetric_x(&self) -> bool {
        let image = &self.cropped_object_image;
        match image.is_symmetric_x() {
            Ok(value) => {
                return value;
            },
            _ => {
                return false;
            }
        }
    }

    pub fn is_symmetric_y(&self) -> bool {
        let image = &self.cropped_object_image;
        match image.is_symmetric_y() {
            Ok(value) => {
                return value;
            },
            _ => {
                return false;
            }
        }
    }
}
