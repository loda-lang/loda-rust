use super::{arc_work_model, ImageProperty};
use super::arc_work_model::Object;
use super::{Image, Rectangle};
use super::{ConnectedComponent, PixelConnectivity, ImageMask, ImageCrop};
use std::collections::{HashMap, HashSet};

impl arc_work_model::Input {
    pub fn update_input_properties(&mut self) {
        self.input_properties = self.resolve_input_properties();
    }

    fn resolve_input_properties(&self) -> HashMap<ImageProperty, u8> {
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

        let input_unique_color_count_raw: u16 = self.image_meta.histogram.number_of_counters_greater_than_zero();
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
        let histogram_pairs: Vec<(u32,u8)> = self.image_meta.histogram.pairs_descending();
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

    pub fn update_image_meta(&mut self) -> anyhow::Result<()> {
        self.image_meta.analyze(&self.image)?;
        self.update_input_properties();

        Ok(())
    }

    pub fn find_object_masks_using_histogram_most_popular_color(&self) -> anyhow::Result<Vec<Image>> {
        let background_color: u8 = match self.image_meta.histogram.most_popular_color_disallow_ambiguous() {
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
