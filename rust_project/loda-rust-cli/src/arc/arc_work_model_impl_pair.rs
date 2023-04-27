use super::{arc_work_model, ImageCompare, Image, ImageHistogram, ImageNoiseColor};
use super::arc_work_model::{Object, ObjectType};
use super::{ActionLabel, ObjectLabel, PropertyOutput};
use super::{ImageFind, ImageSize, ImageSymmetry, Histogram};

#[allow(unused_imports)]
use crate::arc::{HtmlLog, ImageToHTML};

impl arc_work_model::Pair {
    pub fn update_action_label_set(&mut self) {
        let width_input: u8 = self.input.image.width();
        let height_input: u8 = self.input.image.height();
        let width_output: u8 = self.output.image.width();
        let height_output: u8 = self.output.image.height();

        {
            let label = ActionLabel::OutputPropertyIsConstant { 
                output: PropertyOutput::OutputWidth, 
                value: width_output
            };
            self.action_label_set.insert(label);
        }

        {
            let label = ActionLabel::OutputPropertyIsConstant { 
                output: PropertyOutput::OutputHeight, 
                value: height_output
            };
            self.action_label_set.insert(label);
        }

        if width_output >= 2 || height_output >= 2 {
            if let Ok(is_symmetric) = self.output.image.is_symmetric_x() {
                if is_symmetric {
                    self.action_label_set.insert(ActionLabel::OutputImageIsSymmetricX);
                }
            }
        }

        if width_output >= 2 || height_output >= 2 {
            if let Ok(is_symmetric) = self.output.image.is_symmetric_y() {
                if is_symmetric {
                    self.action_label_set.insert(ActionLabel::OutputImageIsSymmetricY);
                }
            }
        }

        if width_input >= width_output && height_input >= height_output {
            if let Ok(count) = self.input.image.count_occurrences(&self.output.image) {
                if count >= 1 {
                    self.action_label_set.insert(ActionLabel::OutputImageOccurInsideInputImage { count });
                    self.action_label_set.insert(ActionLabel::OutputImageOccurInsideInputImageOneOrMoreTimes);
                }
                if count == 1 {
                    self.action_label_set.insert(ActionLabel::OutputImageIsPresentExactlyOnceInsideInputImage);
                }
            }
        }

        if width_output >= width_input && height_output >= height_input {
            if let Ok(count) = self.output.image.count_occurrences(&self.input.image) {
                if count >= 1 {
                    self.action_label_set.insert(ActionLabel::InputImageOccurInsideOutputImage { count });
                    self.action_label_set.insert(ActionLabel::InputImageOccurInsideOutputImageOneOrMoreTimes);
                }
                if count == 1 {
                    self.action_label_set.insert(ActionLabel::InputImageIsPresentExactlyOnceInsideOutputImage);
                }
            }
        }

        if self.input.histogram == self.output.histogram {
            self.action_label_set.insert(ActionLabel::OutputImageHistogramEqualToInputImageHistogram);
        }

        for _ in 0..1 {
            let count: u16 = self.removal_histogram.number_of_counters_greater_than_zero();
            if count != 1 {
                break;
            }
            let removal_color: u8 = match self.removal_histogram.most_popular_color() {
                Some(value) => value,
                None => break
            };
            let most_popular_color: u8 = match self.input.histogram.most_popular_color() {
                Some(value) => value,
                None => break
            };
            if removal_color == most_popular_color {
                self.action_label_set.insert(ActionLabel::RemovalColorIsThePrimaryColorOfInputImage);
            }
        }

        if width_input == width_output && height_input == height_output {
            _ = self.analyze_3x3_structure();
        }

        _ = self.analyze_object_why_is_the_output_present_once_in_input();
        _ = self.analyze_output_image_is_input_image_with_changes_to_pixels_with_color();
        _ = self.analyze_output_colors();
    }

    fn analyze_3x3_structure(&mut self) -> anyhow::Result<()> {
        let same_neighbours: bool;
        {
            let input_colors: Image = self.input.image.count_duplicate_pixels_in_neighbours()?;
            let output_colors: Image = self.output.image.count_duplicate_pixels_in_neighbours()?;
            same_neighbours = input_colors == output_colors;
        }

        let same_all_of_3x3: bool;
        if !same_neighbours {
            let input_colors: Image = self.input.image.count_duplicate_pixels_in_3x3()?;
            let output_colors: Image = self.output.image.count_duplicate_pixels_in_3x3()?;
            same_all_of_3x3 = input_colors == output_colors;
        } else {
            same_all_of_3x3 = false;
        }
        
        if same_neighbours || same_all_of_3x3 {
            self.action_label_set.insert(ActionLabel::OutputImageHasSameStructureAsInputImage);
        }
        Ok(())
    }

    fn analyze_object_why_is_the_output_present_once_in_input(&mut self) -> anyhow::Result<()> {
        if !self.action_label_set.contains(&ActionLabel::OutputImageIsPresentExactlyOnceInsideInputImage) {
            // println!("a");
            return Ok(());
        }

        // Future optimization idea. don't recompute if it's already cached
        // if self.input.input_objects.contains(&ObjectType::RemovalOfMostPopularColorInThisImageAfterwardSegmentByNeighborAll) {
        //    return Ok(());
        // }

        // Future experiment. Also find_objects using the most popular color from the histogram_intersection
        let mut object_vec: Vec<Object> = self.input.find_objects_using_histogram_most_popular_color()?;
        let mut found_count: usize = 0;
        for object in &object_vec {
            if object.cropped_object_image == self.output.image {
                found_count += 1;
            }
        }

        if found_count != 1 {
            // println!("b");
            return Ok(());
        }

        Object::assign_labels_to_objects(&mut object_vec);

        // Reset the sorting
        object_vec.sort_unstable_by_key(|obj| obj.index);

        // Populate the action_label_set with data from the objects. If the object has the smallest area, then insert it into the action_label_set.
        let object_labels: [ObjectLabel; 6] = [
            ObjectLabel::TheOnlyOneWithSmallestArea,
            ObjectLabel::TheOnlyOneWithBiggestArea,
            ObjectLabel::TheOnlyOneWithAsymmetryX,
            ObjectLabel::TheOnlyOneWithSymmetryX,
            ObjectLabel::TheOnlyOneWithAsymmetryY,
            ObjectLabel::TheOnlyOneWithSymmetryY,
        ];
        for object in &object_vec {
            if object.cropped_object_image != self.output.image {
                continue;
            }

            for object_label in &object_labels {
                if object.object_label_set.contains(object_label) {
                    let label = ActionLabel::OutputImageIsTheObjectWithObjectLabel {
                        object_label: object_label.clone()
                    };
                    self.action_label_set.insert(label);
                }
            }
        }

        // caching - Save the objects on the input.
        self.input.input_objects.insert(ObjectType::RemovalOfMostPopularColorInThisImageAfterwardSegmentByNeighborAll, object_vec);

        Ok(())
    }

    fn analyze_output_image_is_input_image_with_changes_to_pixels_with_color(&mut self) -> anyhow::Result<()> {
        if self.input.image.size() != self.output.image.size() {
            return Ok(());
        }

        let mask_where_there_is_differences: Image = self.input.image.diff(&self.output.image)?;
        let histogram: Histogram = self.input.image.histogram_with_mask(&mask_where_there_is_differences)?;

        if histogram.number_of_counters_greater_than_zero() > 1 {
            return Ok(());
        }

        let color: u8 = match histogram.most_popular_color() {
            Some(value) => value,
            None => {
                return Ok(());
            }
        };

        {
            let label = ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { color };
            self.action_label_set.insert(label);
        }
        if self.input.histogram.most_popular_color() == Some(color) {
            let label = ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithMostPopularColorOfTheInputImage;
            self.action_label_set.insert(label);
        }
        if self.input.histogram.least_popular_color() == Some(color) {
            let label = ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithLeastPopularColorOfTheInputImage;
            self.action_label_set.insert(label);
        }

        Ok(())
    }

    fn analyze_output_colors(&mut self) -> anyhow::Result<()> {
        let mut histogram: Histogram = self.output.histogram.clone();
        let output_histogram_unique_count: u16 = histogram.number_of_counters_greater_than_zero();
        histogram.intersection_histogram(&self.input.histogram);
        let intersection_count: u16 = histogram.number_of_counters_greater_than_zero();

        if output_histogram_unique_count <= (u8::MAX as u16) {
            let count: u8 = output_histogram_unique_count as u8;
            let label = ActionLabel::OutputImageUniqueColorCount { count };
            self.action_label_set.insert(label);
        }

        if output_histogram_unique_count == intersection_count {
            let label = ActionLabel::OutputImageColorsComesFromInputImage;
            self.action_label_set.insert(label);
        }

        Ok(())
    }

    pub fn predicted_output_size(&self) -> Option<ImageSize> {
        for prediction in &self.prediction_set {
            match prediction {
                arc_work_model::Prediction::OutputSize { size } => {
                    return Some(*size);
                },
                _ => {}
            }
        }
        None
    }

    pub fn predicted_output_palette(&self) -> Option<Histogram> {
        for prediction in &self.prediction_set {
            match prediction {
                arc_work_model::Prediction::OutputPalette { histogram } => {
                    return Some(histogram.clone());
                },
                _ => {}
            }
        }
        None
    }

    pub fn predicted_output_image_is_input_image_with_changes_limited_to_pixels_with_color(&self) -> Option<u8> {
        for prediction in &self.prediction_set {
            match prediction {
                arc_work_model::Prediction::OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { color } => {
                    return Some(*color);
                },
                _ => {}
            }
        }
        None
    }
}
