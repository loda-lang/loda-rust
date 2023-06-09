use super::{arc_work_model, Image, SingleColorObject, ImageLabelSet, SingleColorObjectToLabel};
use std::collections::HashSet;

impl arc_work_model::ImageMeta {
    pub fn new() -> Self {
        Self {
            image_label_set: HashSet::new(),
            single_color_object: None,
        }
    }

    pub fn assign_single_color_object(&mut self, image: &Image) -> anyhow::Result<()> {
        let single_color_object: SingleColorObject = match SingleColorObject::find_objects(image) {
            Ok(value) => value,
            Err(_) => {
                return Ok(());
            }
        };
        let image_label_set: ImageLabelSet = match single_color_object.to_image_label_set() {
            Ok(value) => value,
            Err(error) => {
                error!("Unable to convert single_color_object to image_label_set. {:?}", error);
                return Ok(());
            }
        };
        self.image_label_set.extend(image_label_set);
        self.single_color_object = Some(single_color_object);
        Ok(())
    }
}
