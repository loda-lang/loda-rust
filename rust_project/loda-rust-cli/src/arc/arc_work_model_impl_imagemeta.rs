use super::{arc_work_model, Image, ImageLabelSet, ImageLabel};
use super::{SingleColorObject, SingleColorObjectToLabel};
use super::{Grid, GridLabel, GridToLabel};
use std::collections::HashSet;

impl arc_work_model::ImageMeta {
    pub fn new() -> Self {
        Self {
            image_label_set: HashSet::new(),
            grid: None,
            single_color_object: None,
        }
    }

    pub fn resolve_grid(&mut self, image: &Image) {
        if self.grid.is_some() {
            return;
        }
        let grid: Grid = match Grid::analyze(image) {
            Ok(value) => value,
            Err(error) => {
                println!("Unable to find grid. error: {:?}", error);
                return;
            }
        };
        self.grid = Some(grid);
    }

    pub fn assign_grid_labels(&mut self) {
        let grid_labels: HashSet<GridLabel>;
        match &self.grid {
            Some(grid) => {
                grid_labels = grid.to_grid_labels();
            },
            None => {
                return;
            }
        };
        for grid_label in grid_labels {
            let label = ImageLabel::Grid { label: grid_label.clone() };
            self.image_label_set.insert(label);
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
