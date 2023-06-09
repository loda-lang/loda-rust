use super::{arc_work_model, Image, ImageLabelSet, ImageLabel, Histogram, ImageHistogram};
use super::{SingleColorObject, SingleColorObjectToLabel};
use super::{Grid, GridLabel, GridToLabel};
use super::{Symmetry, SymmetryLabel, SymmetryToLabel};
use std::collections::HashSet;

impl arc_work_model::ImageMeta {
    pub fn new() -> Self {
        Self {
            histogram: Histogram::new(),
            image_label_set: HashSet::new(),
            grid: None,
            symmetry: None,
            single_color_object: None,
        }
    }

    pub fn analyze(&mut self, image: &Image) -> anyhow::Result<()> {
        self.histogram = image.histogram_all();
        self.resolve_symmetry(image);
        self.resolve_grid(image);
        self.assign_symmetry_labels();
        self.assign_grid_labels();
        self.assign_single_color_object(image)?;
        Ok(())
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

    pub fn resolve_symmetry(&mut self, image: &Image) {
        if self.symmetry.is_some() {
            return;
        }

        let width: u8 = image.width();
        let height: u8 = image.height();
        if width == 0 || height == 0 {
            return;
        }
        if width == 1 && height == 1 {
            return;
        }

        let symmetry: Symmetry = match Symmetry::analyze(image) {
            Ok(value) => value,
            Err(error) => {
                println!("Unable to find symmetry. error: {:?}", error);
                return;
            }
        };
        self.symmetry = Some(symmetry);
    }

    pub fn assign_symmetry_labels(&mut self) {
        let symmetry_labels: HashSet<SymmetryLabel>;
        match &self.symmetry {
            Some(symmetry) => {
                symmetry_labels = symmetry.to_symmetry_labels();
            },
            None => {
                return;
            }
        };
        for symmetry_label in symmetry_labels {
            let label = ImageLabel::Symmetry { label: symmetry_label.clone() };
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
