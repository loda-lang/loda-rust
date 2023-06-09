use std::collections::HashSet;

use super::{arc_work_model};

impl arc_work_model::ImageMeta {
    pub fn new() -> Self {
        Self {
            image_label_set: HashSet::new(),
            single_color_object: None,
        }
    }
}
