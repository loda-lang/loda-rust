use super::{arc_work_model, Image, ImageLabelSet, ImageLabel, Histogram, ImageHistogram, ImageFill, PixelConnectivity, ImageMask, ImageProperty, ImageExtractRowColumn, ImageStats, ImagePeriodicity};
use super::{SingleColorObject, SingleColorObjectToLabel};
use super::{Grid, GridToLabel};
use super::{Symmetry, SymmetryToLabel};
use std::collections::{HashSet, HashMap};

impl arc_work_model::ImageMeta {
    pub fn new() -> Self {
        Self {
            histogram_all: Histogram::new(),
            histogram_border: Histogram::new(),
            image_properties: HashMap::new(),
            image_label_set: HashSet::new(),
            grid: None,
            symmetry: None,
            image_stats: None,
            single_color_object: None,
        }
    }

    pub fn analyze(&mut self, image: &Image) -> anyhow::Result<()> {
        self.histogram_all = image.histogram_all();
        self.histogram_border = image.histogram_border();
        self.update_image_properties(image);
        self.assign_periodicity(image)?;
        self.assign_grid(image)?;
        self.assign_symmetry(image)?;
        self.assign_image_stats(image)?;
        self.assign_single_color_object(image)?;
        self.assign_single_border_color()?;
        self.assign_border_flood_fill(image)?;
        self.assign_most_popular_border_color_is_present_on_all_edges(image)?;
        self.assign_image_properties_about_noise_pixels()?;
        Ok(())
    }

    fn update_image_properties(&mut self, image: &Image) {
        self.image_properties = Self::resolve_image_properties(image, &self.histogram_all);
    }

    fn resolve_image_properties(image: &Image, histogram: &Histogram) -> HashMap<ImageProperty, u8> {
        let width_input: u8 = image.width();
        let height_input: u8 = image.height();

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

        let input_unique_color_count_raw: u16 = histogram.number_of_counters_greater_than_zero();
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
        let histogram_pairs: Vec<(u32,u8)> = histogram.pairs_descending();
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

        let mut dict = HashMap::<ImageProperty, u8>::new();
        dict.insert(ImageProperty::Width, width_input);
        dict.insert(ImageProperty::Height, height_input);
        if let Some(value) = width_input_plus1 {
            dict.insert(ImageProperty::WidthPlus1, value);
        }
        if let Some(value) = width_input_plus2 {
            dict.insert(ImageProperty::WidthPlus2, value);
        }
        if let Some(value) = width_input_minus1 {
            dict.insert(ImageProperty::WidthMinus1, value);
        }
        if let Some(value) = width_input_minus2 {
            dict.insert(ImageProperty::WidthMinus2, value);
        }
        if let Some(value) = height_input_plus1 {
            dict.insert(ImageProperty::HeightPlus1, value);
        }
        if let Some(value) = height_input_plus2 {
            dict.insert(ImageProperty::HeightPlus2, value);
        }
        if let Some(value) = height_input_minus1 {
            dict.insert(ImageProperty::HeightMinus1, value);
        }
        if let Some(value) = height_input_minus2 {
            dict.insert(ImageProperty::HeightMinus2, value);
        }
        if let Some(value) = biggest_value_that_divides_width_and_height {
            dict.insert(ImageProperty::BiggestValueThatDividesWidthAndHeight, value);
        }
        if let Some(value) = input_unique_color_count {
            dict.insert(ImageProperty::UniqueColorCount, value);
        }
        if let Some(value) = input_unique_color_count_minus1 {
            dict.insert(ImageProperty::UniqueColorCountMinus1, value);
        }
        if let Some(value) = input_number_of_pixels_with_most_popular_color {
            dict.insert(ImageProperty::NumberOfPixelsWithMostPopularColor, value);
        }
        if let Some(value) = input_number_of_pixels_with_2nd_most_popular_color {
            dict.insert(ImageProperty::NumberOfPixelsWith2ndMostPopularColor, value);
        }
        dict
    }

    fn assign_periodicity(&mut self, image: &Image) -> anyhow::Result<()> {
        let periodicity_x: Option<u8>;
        let periodicity_y: Option<u8>;
        {
            let ignore_mask: Image = Image::zero(image.width(), image.height());
            periodicity_x = match image.horizontal_periodicity(&ignore_mask) {
                Ok(value) => value,
                Err(_) => None,
            };
            periodicity_y = match image.vertical_periodicity(&ignore_mask) {
                Ok(value) => value,
                Err(_) => None,
            };
        }
        if let Some(value) = periodicity_x {
            let label = ImageLabel::PeriodicityX { period: value };
            self.image_label_set.insert(label);
        }
        if let Some(value) = periodicity_y {
            let label = ImageLabel::PeriodicityY { period: value };
            self.image_label_set.insert(label);
        }
        Ok(())
    }

    fn assign_grid(&mut self, image: &Image) -> anyhow::Result<()> {
        if self.grid.is_some() {
            return Ok(());
        }
        let grid: Grid = match Grid::analyze(image) {
            Ok(value) => value,
            Err(error) => {
                println!("Unable to find grid. error: {:?}", error);
                return Ok(());
            }
        };
        for grid_label in grid.to_grid_labels() {
            let label = ImageLabel::Grid { label: grid_label.clone() };
            self.image_label_set.insert(label);
        }
        self.grid = Some(grid);
        Ok(())
    }

    fn assign_symmetry(&mut self, image: &Image) -> anyhow::Result<()> {
        if self.symmetry.is_some() {
            return Ok(());
        }

        let width: u8 = image.width();
        let height: u8 = image.height();
        if width == 0 || height == 0 {
            return Ok(());
        }
        if width == 1 && height == 1 {
            return Ok(());
        }

        let symmetry: Symmetry = match Symmetry::analyze(image) {
            Ok(value) => value,
            Err(error) => {
                println!("Unable to find symmetry. error: {:?}", error);
                return Ok(());
            }
        };
        for symmetry_label in symmetry.to_symmetry_labels() {
            let label = ImageLabel::Symmetry { label: symmetry_label.clone() };
            self.image_label_set.insert(label);
        }
        self.symmetry = Some(symmetry);
        Ok(())
    }

    fn assign_image_stats(&mut self, image: &Image) -> anyhow::Result<()> {
        if self.image_stats.is_some() {
            return Ok(());
        }
        let image_stats: ImageStats = match ImageStats::new(image) {
            Ok(value) => value,
            Err(_error) => {
                // println!("Unable to compute image_stats. error: {:?}", error);
                return Ok(());
            }
        };
        self.image_stats = Some(image_stats);
        Ok(())
    }

    fn assign_single_color_object(&mut self, image: &Image) -> anyhow::Result<()> {
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

    fn assign_single_border_color(&mut self) -> anyhow::Result<()> {
        if self.histogram_border.number_of_counters_greater_than_zero() != 1 {
            return Ok(());
        }
        let color: u8 = match self.histogram_border.most_popular_color() {
            Some(value) => value,
            None => {
                return Ok(());
            }
        };
        let image_label = ImageLabel::SingleBorderColor { color };
        self.image_label_set.insert(image_label);
        Ok(())
    }

    fn assign_border_flood_fill(&mut self, image: &Image) -> anyhow::Result<()> {
        for (_count, color) in self.histogram_all.pairs_ordered_by_color() {
            let mut filled: Image = image.clone();
            let mask_before: Image = filled.to_mask_where_color_is(color);
            filled.border_flood_fill(color, 255, PixelConnectivity::Connectivity4);
            let mask_after: Image = filled.to_mask_where_color_is(255);
            if mask_before != mask_after {
                continue;
            }
            let image_label = ImageLabel::BorderFloodFillConnectivity4AllPixelsWithColor { color };
            self.image_label_set.insert(image_label);
        }
        Ok(())
    }

    fn assign_most_popular_border_color_is_present_on_all_edges(&mut self, image: &Image) -> anyhow::Result<()> {
        let border_color: u8 = match self.histogram_border.most_popular_color_disallow_ambiguous() {
            Some(value) => value,
            None => {
                return Ok(());
            }
        };
        let row_top: Image = image.top_rows(1)?;
        let row_bottom: Image = image.bottom_rows(1)?;
        let column_left: Image = image.left_columns(1)?;
        let column_right: Image = image.right_columns(1)?;
        let edge_images: Vec::<Image> = vec![row_top, row_bottom, column_left, column_right];
        for edge_image in &edge_images {
            let edge_histogram: Histogram = edge_image.histogram_all();
            if edge_histogram.get(border_color) == 0 {
                return Ok(());
            }
        }
        let image_label = ImageLabel::MostPopularBorderColorIsPresentOnAllEdges;
        self.image_label_set.insert(image_label);
        Ok(())
    }

    fn assign_image_properties_about_noise_pixels(&mut self) -> anyhow::Result<()> {
        if let Some(sco) = &self.single_color_object {
            let histogram: Histogram = sco.single_pixel_noise_histogram();
            let color_count: u8 = histogram.number_of_counters_greater_than_zero().min(255) as u8;
            if color_count > 0 {
                self.image_properties.insert(ImageProperty::UniqueNoiseColorCount, color_count);

                let mass: u32 = histogram.sum();
                if mass <= 255 {
                    self.image_properties.insert(ImageProperty::MassOfAllNoisePixels, mass as u8);
                }
            }
        }
        Ok(())
    }
}
