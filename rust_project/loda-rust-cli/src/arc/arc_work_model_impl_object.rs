use super::arc_work_model;
use super::arc_work_model::Object;
use super::ObjectLabel;
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

    pub fn assign_labels_to_objects(object_vec: &mut Vec<Object>) {
        // Reset the sorting
        object_vec.sort_unstable_by_key(|obj| obj.index);

        // Smallest objects first, biggest last
        object_vec.sort_unstable_by_key(|obj| obj.area());
        for _ in 0..1 {
            let object1_area: u16 = match object_vec.get(1) {
                Some(obj) => obj.area(),
                None => break
            };
            let object0: &mut Object = match object_vec.get_mut(0) {
                Some(obj) => obj,
                None => break
            };
            let object0_area: u16 = object0.area();
            if object0_area < object1_area {
                // println!("OutputImage is object with the smallest area, area: {} id: {:?}", object0_area, self.id);
                object0.object_label_set.insert(ObjectLabel::TheOnlyOneWithSmallestArea);
            }            
        }

        // Biggest objects first, smallest last
        object_vec.reverse();
        for _ in 0..1 {
            let object1_area: u16 = match object_vec.get(1) {
                Some(obj) => obj.area(),
                None => break
            };
            let object0: &mut Object = match object_vec.get_mut(0) {
                Some(obj) => obj,
                None => break
            };
            let object0_area: u16 = object0.area();
            if object0_area > object1_area {
                // println!("OutputImage is object with the biggest area, area: {} id: {:?}", object0_area, self.id);
                object0.object_label_set.insert(ObjectLabel::TheOnlyOneWithBiggestArea);
            }            
        }

        // Reset the sorting
        object_vec.sort_unstable_by_key(|obj| obj.index);

        // Asymmetric objects first, symmetric last
        object_vec.sort_unstable_by_key(|obj| obj.is_symmetric_x());
        for _ in 0..1 {
            let object1_is_symmetric_x: bool = match object_vec.get(1) {
                Some(obj) => obj.is_symmetric_x(),
                None => break
            };
            let object0: &mut Object = match object_vec.get_mut(0) {
                Some(obj) => obj,
                None => break
            };
            if object0.is_symmetric_x() != object1_is_symmetric_x {
                // println!("OutputImage is only object that is asymmetric x, {:?}", self.id);
                object0.object_label_set.insert(ObjectLabel::TheOnlyOneWithAsymmetryX);
            }            
        }

        // Symmetric objects first, asymmetric last
        object_vec.reverse();
        for _ in 0..1 {
            let object1_is_symmetric_x: bool = match object_vec.get(1) {
                Some(obj) => obj.is_symmetric_x(),
                None => break
            };
            let object0: &mut Object = match object_vec.get_mut(0) {
                Some(obj) => obj,
                None => break
            };
            if object0.is_symmetric_x() != object1_is_symmetric_x {
                // println!("OutputImage is only object that is symmetric x, {:?}", self.id);
                object0.object_label_set.insert(ObjectLabel::TheOnlyOneWithSymmetryX);
            }            
        }

        // Reset the sorting
        object_vec.sort_unstable_by_key(|obj| obj.index);

        // Asymmetric objects first, symmetric last
        object_vec.sort_unstable_by_key(|obj| obj.is_symmetric_y());
        for _ in 0..1 {
            let object1_is_symmetric_y: bool = match object_vec.get(1) {
                Some(obj) => obj.is_symmetric_y(),
                None => break
            };
            let object0: &mut Object = match object_vec.get_mut(0) {
                Some(obj) => obj,
                None => break
            };
            if object0.is_symmetric_y() != object1_is_symmetric_y {
                // println!("OutputImage is only object that is asymmetric y, {:?}", self.id);
                object0.object_label_set.insert(ObjectLabel::TheOnlyOneWithAsymmetryY);
            }            
        }

        // Symmetric objects first, asymmetric last
        object_vec.reverse();
        for _ in 0..1 {
            let object1_is_symmetric_y: bool = match object_vec.get(1) {
                Some(obj) => obj.is_symmetric_y(),
                None => break
            };
            let object0: &mut Object = match object_vec.get_mut(0) {
                Some(obj) => obj,
                None => break
            };
            if object0.is_symmetric_y() != object1_is_symmetric_y {
                // println!("OutputImage is only object that is symmetric y, {:?}", self.id);
                object0.object_label_set.insert(ObjectLabel::TheOnlyOneWithSymmetryY);
            }            
        }
    }
}
