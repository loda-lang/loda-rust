use super::{arc_work_model, ImageFill};
use super::arc_work_model::Object;
use super::{PropertyInput, ImageLabel, GridLabel, SingleColorObjectRectangleLabel, SingleColorObjectSparseLabel};
use super::{Symmetry, Grid, GridToLabel, Image, Rectangle, SymmetryLabel, SymmetryToLabel};
use super::{ConnectedComponent, PixelConnectivity, ImageMask, ImageCrop, SingleColorObjects};
use std::collections::{HashMap, HashSet};

impl arc_work_model::Input {
    pub fn update_input_properties(&mut self) {
        self.input_properties = self.resolve_input_properties();
    }

    fn resolve_input_properties(&self) -> HashMap<PropertyInput, u8> {
        let width_input: u8 = self.image.width();
        let height_input: u8 = self.image.height();

        let mut width_input_plus1: Option<u8> = None;
        {
            let value: u16 = (width_input as u16) + 1;
            if value <= (u8::MAX as u16) {
                width_input_plus1 = Some(value as u8);
            }
        }

        let mut height_input_plus1: Option<u8> = None;
        {
            let value: u16 = (height_input as u16) + 1;
            if value <= (u8::MAX as u16) {
                height_input_plus1 = Some(value as u8);
            }
        }

        let mut width_input_plus2: Option<u8> = None;
        {
            let value: u16 = (width_input as u16) + 2;
            if value <= (u8::MAX as u16) {
                width_input_plus2 = Some(value as u8);
            }
        }

        let mut height_input_plus2: Option<u8> = None;
        {
            let value: u16 = (height_input as u16) + 2;
            if value <= (u8::MAX as u16) {
                height_input_plus2 = Some(value as u8);
            }
        }

        let mut width_input_minus1: Option<u8> = None;
        {
            if width_input >= 1 {
                width_input_minus1 = Some(width_input - 1);
            }
        }

        let mut height_input_minus1: Option<u8> = None;
        {
            if height_input >= 1 {
                height_input_minus1 = Some(height_input - 1);
            }
        }
        
        let mut width_input_minus2: Option<u8> = None;
        {
            if width_input >= 2 {
                width_input_minus2 = Some(width_input - 2);
            }
        }

        let mut height_input_minus2: Option<u8> = None;
        {
            if height_input >= 2 {
                height_input_minus2 = Some(height_input - 2);
            }
        }

        let mut biggest_value_that_divides_width_and_height: Option<u8> = None;
        if width_input == height_input {
            biggest_value_that_divides_width_and_height = Some(width_input);
        } else {
            let smallest: u8 = width_input.min(height_input);
            let biggest: u8 = width_input.max(height_input);
            if smallest >= 2 {
                let rem: u8 = biggest % smallest;
                if rem == 0 {
                    biggest_value_that_divides_width_and_height = Some(smallest);
                }
            }
        }

        let input_unique_color_count_raw: u16 = self.histogram.number_of_counters_greater_than_zero();
        let mut input_unique_color_count: Option<u8> = None;
        if input_unique_color_count_raw <= (u8::MAX as u16) {
            input_unique_color_count = Some(input_unique_color_count_raw as u8);
        }

        let mut input_unique_color_count_minus1: Option<u8> = None;
        if let Some(value) = input_unique_color_count {
            if value >= 1 {
                input_unique_color_count_minus1 = Some(value - 1);
            }
        }

        let mut input_number_of_pixels_with_most_popular_color: Option<u8> = None;
        let mut input_number_of_pixels_with_2nd_most_popular_color: Option<u8> = None;
        let histogram_pairs: Vec<(u32,u8)> = self.histogram.pairs_descending();
        for (histogram_index, histogram_pair) in histogram_pairs.iter().enumerate() {
            if histogram_index >= 2 {
                break;
            }
            let pixel_count: u32 = histogram_pair.0;
            if pixel_count <= (u8::MAX as u32) {
                if histogram_index == 0 {
                    input_number_of_pixels_with_most_popular_color = Some(pixel_count as u8);
                }
                if histogram_index == 1 {
                    input_number_of_pixels_with_2nd_most_popular_color = Some(pixel_count as u8);
                }
            }
        }

        let mut dict = HashMap::<PropertyInput, u8>::new();
        dict.insert(PropertyInput::InputWidth, width_input);
        dict.insert(PropertyInput::InputHeight, height_input);
        if let Some(value) = width_input_plus1 {
            dict.insert(PropertyInput::InputWidthPlus1, value);
        }
        if let Some(value) = width_input_plus2 {
            dict.insert(PropertyInput::InputWidthPlus2, value);
        }
        if let Some(value) = width_input_minus1 {
            dict.insert(PropertyInput::InputWidthMinus1, value);
        }
        if let Some(value) = width_input_minus2 {
            dict.insert(PropertyInput::InputWidthMinus2, value);
        }
        if let Some(value) = height_input_plus1 {
            dict.insert(PropertyInput::InputHeightPlus1, value);
        }
        if let Some(value) = height_input_plus2 {
            dict.insert(PropertyInput::InputHeightPlus2, value);
        }
        if let Some(value) = height_input_minus1 {
            dict.insert(PropertyInput::InputHeightMinus1, value);
        }
        if let Some(value) = height_input_minus2 {
            dict.insert(PropertyInput::InputHeightMinus2, value);
        }
        if let Some(value) = biggest_value_that_divides_width_and_height {
            dict.insert(PropertyInput::InputBiggestValueThatDividesWidthAndHeight, value);
        }
        if let Some(value) = input_unique_color_count {
            dict.insert(PropertyInput::InputUniqueColorCount, value);
        }
        if let Some(value) = input_unique_color_count_minus1 {
            dict.insert(PropertyInput::InputUniqueColorCountMinus1, value);
        }
        if let Some(value) = input_number_of_pixels_with_most_popular_color {
            dict.insert(PropertyInput::InputNumberOfPixelsWithMostPopularColor, value);
        }
        if let Some(value) = input_number_of_pixels_with_2nd_most_popular_color {
            dict.insert(PropertyInput::InputNumberOfPixelsWith2ndMostPopularColor, value);
        }
        dict
    }

    pub fn resolve_symmetry(&mut self) {
        if self.symmetry.is_some() {
            return;
        }

        let width: u8 = self.image.width();
        let height: u8 = self.image.height();
        if width == 0 || height == 0 {
            return;
        }
        if width == 1 && height == 1 {
            return;
        }

        let symmetry: Symmetry = match Symmetry::analyze(&self.image) {
            Ok(value) => value,
            Err(error) => {
                println!("Unable to find symmetry. {} error: {:?}", self.id, error);
                return;
            }
        };
        self.symmetry = Some(symmetry);
    }

    pub fn resolve_grid(&mut self) {
        if self.grid.is_some() {
            return;
        }
        let grid: Grid = match Grid::analyze(&self.image) {
            Ok(value) => value,
            Err(error) => {
                println!("Unable to find grid. {} error: {:?}", self.id, error);
                return;
            }
        };
        self.grid = Some(grid);
    }

    pub fn update_input_label_set(&mut self) -> anyhow::Result<()> {
        self.resolve_symmetry();
        self.resolve_grid();
        self.assign_symmetry_labels();
        self.assign_grid_labels();
        self.assign_single_color_objects()?;
        self.assign_border_flood_fill()?;
        Ok(())
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

    pub fn assign_single_color_objects(&mut self) -> anyhow::Result<()> {
        let single_color_objects: SingleColorObjects = match SingleColorObjects::find_objects(&self.image) {
            Ok(value) => value,
            Err(_) => {
                return Ok(());
            }
        };
        for object in &single_color_objects.rectangle_vec {
            {
                let label = SingleColorObjectRectangleLabel::RectangleWithColor { color: object.color };
                let image_label = ImageLabel::SingleColorObjectRectangle { label };
                self.image_label_set.insert(image_label);
            }
            {
                let label = SingleColorObjectRectangleLabel::RectangleWithSomeColor;
                let image_label = ImageLabel::SingleColorObjectRectangle { label };
                self.image_label_set.insert(image_label);
            }
            if object.is_square {
                {
                    let label = SingleColorObjectRectangleLabel::SquareWithColor { color: object.color };
                    let image_label = ImageLabel::SingleColorObjectRectangle { label };
                    self.image_label_set.insert(image_label);
                }
                {
                    let label = SingleColorObjectRectangleLabel::SquareWithSomeColor;
                    let image_label = ImageLabel::SingleColorObjectRectangle { label };
                    self.image_label_set.insert(image_label);
                }
            } else {
                {
                    let label = SingleColorObjectRectangleLabel::NonSquareWithColor { color: object.color };
                    let image_label = ImageLabel::SingleColorObjectRectangle { label };
                    self.image_label_set.insert(image_label);
                }
                {
                    let label = SingleColorObjectRectangleLabel::NonSquareWithSomeColor;
                    let image_label = ImageLabel::SingleColorObjectRectangle { label };
                    self.image_label_set.insert(image_label);
                }
            }
        }
        for object in &single_color_objects.sparse_vec {
            {
                let label = SingleColorObjectSparseLabel::SparseWithColor { color: object.color };
                let image_label = ImageLabel::SingleColorObjectSparse { label };
                self.image_label_set.insert(image_label);
            }
            {
                let label = SingleColorObjectSparseLabel::SparseWithSomeColor;
                let image_label = ImageLabel::SingleColorObjectSparse { label };
                self.image_label_set.insert(image_label);
            }
        }
        {
            for object in &single_color_objects.rectangle_vec {
                let image_label = ImageLabel::UnambiguousConnectivityWithColor { color: object.color };
                self.image_label_set.insert(image_label);
            }
            let mut all_are_connectivity48_identical = true;
            for object in &single_color_objects.sparse_vec {
                if !object.connectivity48_identical {
                    all_are_connectivity48_identical = false;
                    continue;
                }
                let image_label = ImageLabel::UnambiguousConnectivityWithColor { color: object.color };
                self.image_label_set.insert(image_label);
            }
            if all_are_connectivity48_identical {
                let image_label = ImageLabel::UnambiguousConnectivityWithAllColors;
                self.image_label_set.insert(image_label);
            }
        }
        if let Some(color) = single_color_objects.single_pixel_noise_color() {
            {
                let image_label = ImageLabel::NoiseWithColor { color };
                self.image_label_set.insert(image_label);
            }
            {
                let image_label = ImageLabel::NoiseWithSomeColor;
                self.image_label_set.insert(image_label);
            }
        }
        self.single_color_objects = Some(single_color_objects);
        Ok(())
    }

    pub fn assign_border_flood_fill(&mut self) -> anyhow::Result<()> {
        for (_count, color) in self.histogram.pairs_ordered_by_color() {
            let mut image: Image = self.image.clone();
            let mask_before: Image = image.to_mask_where_color_is(color);
            image.border_flood_fill(color, 255, PixelConnectivity::Connectivity4);
            let mask_after: Image = image.to_mask_where_color_is(255);
            if mask_before != mask_after {
                continue;
            }
            let image_label = ImageLabel::BorderFloodFillConnectivity4AllPixelsWithColor { color };
            self.image_label_set.insert(image_label);
        }
        Ok(())
    }

    pub fn find_object_masks_using_histogram_most_popular_color(&self) -> anyhow::Result<Vec<Image>> {
        let background_color: u8 = match self.histogram.most_popular_color_disallow_ambiguous() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("unclear what the background color is"));
            }
        };
        let background_ignore_mask: Image = self.image.to_mask_where_color_is(background_color);
        let object_mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &self.image, &background_ignore_mask)?;
        Ok(object_mask_vec)
    }

    pub fn find_objects_using_histogram_most_popular_color(&self) -> anyhow::Result<Vec<Object>> {
        let object_mask_vec: Vec<Image> = self.find_object_masks_using_histogram_most_popular_color()?;
        let mut object_vec: Vec<Object> = vec!();
        for (index, object_mask) in object_mask_vec.iter().enumerate() {
            let rect: Rectangle = match object_mask.bounding_box() {
                Some(value) => value,
                None => continue
            };
            let cropped_object_image: Image = self.image.crop(rect)?;

            let object = Object {
                index: index,
                cropped_object_image: cropped_object_image.clone(),
                object_label_set: HashSet::new(),
            };
            object_vec.push(object);
        }

        Ok(object_vec)
    }

    pub fn assign_repair_mask_with_color(&mut self, color: u8) -> anyhow::Result<()> {
        let mask: Image = self.image.to_mask_where_color_is(color);
        self.repair_mask = Some(mask);
        Ok(())
    }
}
