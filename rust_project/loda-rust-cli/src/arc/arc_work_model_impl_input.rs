use super::{arc_work_model, ImageNoiseColor};
use super::arc_work_model::Object;
use super::{PropertyInput, InputLabel};
use super::{DetectSymmetry, Image, ImageSymmetry, Rectangle};
use super::{ImageSegment, ImageSegmentAlgorithm, ImageSegmentItem, ImageMask, ImageCrop, ImageObjectEnumerate};
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

        let input_unique_color_count_raw: u32 = self.histogram.number_of_counters_greater_than_zero();
        let mut input_unique_color_count: Option<u8> = None;
        if input_unique_color_count_raw <= (u8::MAX as u32) {
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

    pub fn update_input_label_set(&mut self) {
        let width: u8 = self.image.width();
        let height: u8 = self.image.height();
        if width == 0 || height == 0 {
            return;
        }
        if width == 1 && height == 1 {
            return;
        }

        let detect: DetectSymmetry = match DetectSymmetry::analyze(&self.image) {
            Ok(value) => value,
            Err(_) => {
                println!("DetectSymmetry Unable to check symmetry. {}", self.id);
                return;
            }
        };

        if detect.found_horizontal_symmetry {
            if detect.horizontal_mismatches == 0 {
                if detect.left == 0 && detect.right == 0 {
                    self.input_label_set.insert(InputLabel::InputImageIsSymmetricX);
                    self.input_label_set.insert(InputLabel::InputImageIsSymmetricXWithInset);
                } else {
                    self.input_label_set.insert(InputLabel::InputImageIsSymmetricXWithInset);
                }
            } else {
                if detect.left == 0 && detect.right == 0 {
                    self.input_label_set.insert(InputLabel::InputImageIsSymmetricXWithMismatches);
                } else {
                    self.input_label_set.insert(InputLabel::InputImageIsSymmetricXWithInsetAndMismatches);
                }
            }
        }

        if detect.found_vertical_symmetry {
            if detect.vertical_mismatches == 0 {
                if detect.top == 0 && detect.bottom == 0 {
                    self.input_label_set.insert(InputLabel::InputImageIsSymmetricY);
                    self.input_label_set.insert(InputLabel::InputImageIsSymmetricYWithInset);
                } else {
                    self.input_label_set.insert(InputLabel::InputImageIsSymmetricYWithInset);
                }
            } else {
                if detect.top == 0 && detect.bottom == 0 {
                    self.input_label_set.insert(InputLabel::InputImageIsSymmetricYWithMismatches);
                } else {
                    self.input_label_set.insert(InputLabel::InputImageIsSymmetricYWithInsetAndMismatches);
                }
            }
        }

        // if width >= 2 || height >= 2 {
        //     if let Ok(is_symmetric) = self.image.is_symmetric_x() {
        //         if is_symmetric {
        //             self.input_label_set.insert(InputLabel::InputImageIsSymmetricX);
        //         }
        //     }
        // }

        // if width >= 2 || height >= 2 {
        //     if let Ok(is_symmetric) = self.image.is_symmetric_y() {
        //         if is_symmetric {
        //             self.input_label_set.insert(InputLabel::InputImageIsSymmetricY);
        //         }
        //     }
        // }
    }

    pub fn find_objects_using_histogram_most_popular_color(&self) -> anyhow::Result<Vec<Object>> {
        let background_color: u8 = match self.histogram.most_popular_color() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("unclear what the background color is"));
            }
        };
        let background_ignore_mask: Image = self.image.to_mask_where_color_is(background_color);
        
        let object_mask_vec: Vec<Image> = self.image.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &background_ignore_mask)?;
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

    pub fn assign_attention_mask_with_color(&mut self, color: u8) -> anyhow::Result<()> {
        let mask: Image = self.image.to_mask_where_color_is(color);
        self.attention_mask = Some(mask);
        Ok(())
    }

    pub fn assign_biggest_object_mask_with_ignore_color(&mut self, ignore_color: u8) -> anyhow::Result<()> {
        let ignore_mask: Image = self.image.to_mask_where_color_is(ignore_color);
        // let mut objects: Vec<ImageSegmentItem> = self.image.find_objects_with_ignore_mask_inner(ImageSegmentAlgorithm::Neighbors, &ignore_mask)?;
        let mut objects: Vec<ImageSegmentItem> = self.image.find_objects_with_ignore_mask_inner(ImageSegmentAlgorithm::All, &ignore_mask)?;
        objects.sort_unstable_by_key(|item| (item.mass(), item.x(), item.y()));
        objects.reverse();
        let images: Vec<Image> = objects.into_iter().map(|item| item.mask().clone() ).collect();
        let objects_enum: Image = Image::object_enumerate(&images)?;
        self.biggest_object_mask = Some(objects_enum);
        // let biggest_object: ImageSegmentItem = match objects.first() {
        //     Some(value) => value.clone(),
        //     None => {
        //         return Err(anyhow::anyhow!("biggest object"));
        //     }
        // };
        // self.biggest_object_mask = Some(biggest_object.mask().clone());
        Ok(())
    }

    pub fn xassign_biggest_object_mask(&mut self) -> anyhow::Result<()> {
        let color_count: Image = self.image.count_duplicate_pixels_in_3x3()?;
        let ignore_mask: Image = color_count.to_mask_where_color_is_equal_or_less_than(3);
        let mut objects: Vec<ImageSegmentItem> = self.image.find_objects_with_ignore_mask_inner(ImageSegmentAlgorithm::Neighbors, &ignore_mask)?;
        objects.sort_unstable_by_key(|item| (item.mass(), item.x(), item.y()));
        objects.reverse();
        let biggest_object: ImageSegmentItem = match objects.first() {
            Some(value) => value.clone(),
            None => {
                return Err(anyhow::anyhow!("biggest object"));
            }
        };
        self.biggest_object_mask = Some(biggest_object.mask().clone());
        Ok(())
    }
}
