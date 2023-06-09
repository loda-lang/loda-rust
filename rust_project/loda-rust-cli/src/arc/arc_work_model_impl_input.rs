use super::{arc_work_model};
use super::arc_work_model::Object;
use super::{Image, Rectangle};
use super::{ConnectedComponent, PixelConnectivity, ImageMask, ImageCrop};
use std::collections::{HashSet};

impl arc_work_model::Input {
    pub fn update_image_meta(&mut self) -> anyhow::Result<()> {
        self.image_meta.analyze(&self.image)?;
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
