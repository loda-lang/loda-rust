use super::arc_work_model;
use super::arc_work_model::{Input, PairType};
use super::{Image, ImageMask, ImageMaskCount, ImageSegment, ImageSegmentAlgorithm, ImageTrim};
use super::{Label, LabelSet, PropertyInput, PropertyOutput};
use std::collections::{HashMap, HashSet};

#[allow(unused_imports)]
use crate::arc::{HtmlLog, ImageToHTML};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum RulePriority {
    Simple,
    Medium,
    Advanced,
}

impl arc_work_model::Task {
    fn update_input_properties_for_all_pairs(&mut self) {
        for pair in &mut self.pairs {
            pair.input.update_input_properties();
        }
    }

    fn update_label_set_intersection(&mut self) {
        let mut label_set = LabelSet::new();
        let mut is_first = true;
        for pair in &mut self.pairs {
            if pair.pair_type == PairType::Test {
                continue;
            }
            if is_first {
                label_set = pair.label_set.clone();
                is_first = false;
                continue;
            }
            label_set = label_set.intersection(&pair.label_set).map(|l| l.clone()).collect();
        }
        self.label_set_intersection = label_set;
    }

    fn update_input_properties_intersection(&mut self) {
        let mut input_properties_intersection: HashMap<PropertyInput, u8> = HashMap::new();
        let mut is_first = true;
        for pair in &mut self.pairs {
            if pair.pair_type == PairType::Test {
                continue;
            }
            if is_first {
                input_properties_intersection = pair.input.input_properties.clone();
                is_first = false;
                continue;
            }

            // Intersection between `input_properties_intersection` and `pair.input.input_properties`.
            let mut keys_for_removal: HashSet<PropertyInput> = HashSet::new();
            for key in input_properties_intersection.keys() {
                keys_for_removal.insert(*key);
            }
            for (key, value) in &pair.input.input_properties {
                if let Some(other_value) = input_properties_intersection.get(key) {
                    if *value == *other_value {
                        // Both hashmaps agree about the key and value. This is a keeper.
                        keys_for_removal.remove(key);
                    }
                }
            }
            for key in &keys_for_removal {
                input_properties_intersection.remove(key);
            }
        }
        self.input_properties_intersection = input_properties_intersection;
    }

    fn assign_labels_input_size_output_size(&mut self) {
        for pair in &mut self.pairs {
            if pair.pair_type == PairType::Test {
                continue;
            }

            {
                let width_output: u8 = pair.output.image.width();
                let label = Label::OutputPropertyIsConstant { 
                    output: PropertyOutput::OutputWidth, 
                    value: width_output
                };
                pair.label_set.insert(label);
            }

            {
                let height_output: u8 = pair.output.image.height();
                let label = Label::OutputPropertyIsConstant { 
                    output: PropertyOutput::OutputHeight, 
                    value: height_output
                };
                pair.label_set.insert(label);
            }

        }
    }

    fn assign_labels_related_to_removal_histogram(&mut self) {
        let removal_pairs: Vec<(u32,u8)> = self.removal_histogram_intersection.pairs_descending();
        if removal_pairs.len() != 1 {
            return;
        }
        let background_color: u8 = match removal_pairs.first() {
            Some((_count, color)) => *color,
            None => {
                return;
            }
        };
                        
        for pair in &mut self.pairs {

            let image_mask: Image = pair.input.image.to_mask_where_color_is_different(background_color);
            // if self.id == "0934a4d8,task" {
            //     HtmlLog::image(&image_mask);
            // }

            // Determine if the removed color is a rectangle
            {
                match image_mask.trim_color(1) {
                    Ok(image) => {
                        // HtmlLog::image(&image);
                        let mass: u32 = image.mask_count_one();
                        if mass == 0 {
                            // println!("this is a rectangle");
                            pair.input.input_properties.insert(PropertyInput::InputWidthOfRemovedRectangleAfterSingleColorRemoval, image.width());
                            pair.input.input_properties.insert(PropertyInput::InputHeightOfRemovedRectangleAfterSingleColorRemoval, image.height());
                        } else {
                            // println!("this is not a rectangle");
                        }
                    },
                    Err(_) => {}
                }
            }

            let ignore_mask: Image = image_mask.to_mask_where_color_is(0);

            // let result = image_mask.find_objects(ImageSegmentAlgorithm::All);
            let result = image_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, ignore_mask);
            let object_images: Vec<Image> = match result {
                Ok(images) => images,
                Err(_) => {
                    continue;
                }
            };
            // println!("number of objects: {} task: {}", object_images.len(), self.displayName);
            // if self.id == "8a371977,task" {
            //     for image in &object_images {
            //         HtmlLog::image(image);
            //     }
            // }
            let mut mass_max: u32 = 0;
            let mut found_index_mass_max: Option<usize> = None;
            for (index, image) in object_images.iter().enumerate() {

                let mass: u32 = image.mask_count_one();
                if mass > mass_max {
                    mass_max = mass;
                    found_index_mass_max = Some(index);
                }
            }

            if mass_max > 0 && mass_max <= (u8::MAX as u32) {
                let mass_value: u8 = mass_max as u8;
                pair.input.input_properties.insert(PropertyInput::InputMassOfPrimaryObjectAfterSingleColorRemoval, mass_value);
            }

            if let Some(index) = found_index_mass_max {
                if let Some(image) = object_images.get(index) {

                    let trimmed_image: Image = match image.trim_color(0) {
                        Ok(value) => value,
                        Err(_) => {
                            continue;
                        }
                    };
                    
                    let width: u8 = trimmed_image.width();
                    let height: u8 = trimmed_image.height();
                    // println!("biggest object: {}x{}", width, height);

                    pair.input.input_properties.insert(PropertyInput::InputWidthOfPrimaryObjectAfterSingleColorRemoval, width);
                    pair.input.input_properties.insert(PropertyInput::InputHeightOfPrimaryObjectAfterSingleColorRemoval, height);
                }
            }
        }
    }

    fn assign_labels_related_to_input_histogram_intersection(&mut self) {
        let removal_pairs: Vec<(u32,u8)> = self.input_histogram_intersection.pairs_descending();
        if removal_pairs.len() != 1 {
            return;
        }
        let background_color: u8 = match removal_pairs.first() {
            Some((_count, color)) => *color,
            None => {
                return;
            }
        };
                        
        for pair in &mut self.pairs {

            let image_mask: Image = pair.input.image.to_mask_where_color_is_different(background_color);
            // if self.id == "28bf18c6,task" {
            //     HtmlLog::image(&image_mask);
            // }
            {
                let mass: u32 = image_mask.mask_count_zero();
                if mass > 0 && mass <= (u8::MAX as u32) {
                    let mass_value: u8 = mass as u8;
                    pair.input.input_properties.insert(PropertyInput::InputNumberOfPixelsCorrespondingToTheSingleIntersectionColor, mass_value);
                }
            }
            {
                let mass: u32 = image_mask.mask_count_one();
                if mass > 0 && mass <= (u8::MAX as u32) {
                    let mass_value: u8 = mass as u8;
                    pair.input.input_properties.insert(PropertyInput::InputNumberOfPixelsNotCorrespondingToTheSingleIntersectionColor, mass_value);
                }
            }

            let ignore_mask: Image = image_mask.to_mask_where_color_is(0);

            // let result = image_mask.find_objects(ImageSegmentAlgorithm::All);
            let result = image_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, ignore_mask);
            let object_images: Vec<Image> = match result {
                Ok(images) => images,
                Err(_) => {
                    continue;
                }
            };
            // println!("number of objects: {} task: {}", object_images.len(), self.displayName);
            // if self.id == "28bf18c6,task" {
            //     for image in &object_images {
            //         HtmlLog::image(image);
            //     }
            // }
            let mut mass_max: u32 = 0;
            let mut found_index_mass_max: Option<usize> = None;
            for (index, image) in object_images.iter().enumerate() {

                let mass: u32 = image.mask_count_one();
                if mass > mass_max {
                    mass_max = mass;
                    found_index_mass_max = Some(index);
                }
            }

            if mass_max > 0 && mass_max <= (u8::MAX as u32) {
                let mass_value: u8 = mass_max as u8;
                pair.input.input_properties.insert(PropertyInput::InputMassOfPrimaryObjectAfterSingleIntersectionColor, mass_value);
            }

            if let Some(index) = found_index_mass_max {
                if let Some(image) = object_images.get(index) {

                    let trimmed_image: Image = match image.trim_color(0) {
                        Ok(value) => value,
                        Err(_) => {
                            continue;
                        }
                    };
                    
                    let width: u8 = trimmed_image.width();
                    let height: u8 = trimmed_image.height();
                    // println!("biggest object: {}x{}", width, height);

                    pair.input.input_properties.insert(PropertyInput::InputWidthOfPrimaryObjectAfterSingleIntersectionColor, width);
                    pair.input.input_properties.insert(PropertyInput::InputHeightOfPrimaryObjectAfterSingleIntersectionColor, height);
                }
            }
        }
    }

    pub fn assign_labels(&mut self) -> anyhow::Result<()> {
        self.update_input_properties_for_all_pairs();
        self.update_input_properties_intersection();
        self.assign_labels_related_to_removal_histogram();
        self.assign_labels_related_to_input_histogram_intersection();

        self.assign_labels_input_size_output_size();


        let input_properties: [PropertyInput; 24] = [
            PropertyInput::InputWidth, 
            PropertyInput::InputWidthPlus1, 
            PropertyInput::InputWidthPlus2, 
            PropertyInput::InputWidthMinus1, 
            PropertyInput::InputWidthMinus2, 
            PropertyInput::InputHeight,
            PropertyInput::InputHeightPlus1,
            PropertyInput::InputHeightPlus2,
            PropertyInput::InputHeightMinus1,
            PropertyInput::InputHeightMinus2,
            PropertyInput::InputUniqueColorCount,
            PropertyInput::InputUniqueColorCountMinus1,
            PropertyInput::InputNumberOfPixelsWithMostPopularColor,
            PropertyInput::InputNumberOfPixelsWith2ndMostPopularColor,
            PropertyInput::InputWidthOfPrimaryObjectAfterSingleColorRemoval,
            PropertyInput::InputHeightOfPrimaryObjectAfterSingleColorRemoval,
            PropertyInput::InputMassOfPrimaryObjectAfterSingleColorRemoval,
            PropertyInput::InputWidthOfPrimaryObjectAfterSingleIntersectionColor,
            PropertyInput::InputHeightOfPrimaryObjectAfterSingleIntersectionColor,
            PropertyInput::InputMassOfPrimaryObjectAfterSingleIntersectionColor,
            PropertyInput::InputNumberOfPixelsCorrespondingToTheSingleIntersectionColor,
            PropertyInput::InputNumberOfPixelsNotCorrespondingToTheSingleIntersectionColor,
            PropertyInput::InputWidthOfRemovedRectangleAfterSingleColorRemoval,
            PropertyInput::InputHeightOfRemovedRectangleAfterSingleColorRemoval,
        ];
        let output_properties: [PropertyOutput; 2] = [
            PropertyOutput::OutputWidth, 
            PropertyOutput::OutputHeight
        ];
        for pair in &mut self.pairs {
            if pair.pair_type == PairType::Test {
                continue;
            }

            let width_output: u8 = pair.output.image.width();
            let height_output: u8 = pair.output.image.height();

            for input_property in &input_properties {
                let input_value_option: Option<&u8> = pair.input.input_properties.get(input_property);
                let input_value: u8 = match input_value_option {
                    Some(value) => *value,
                    None => {
                        continue;
                    }
                };

                // Future experiments to do.
                // Currently the input_properties are populated once.
                // There may be input properties that depend on other input properties being fully computed beforehand.
                // There may be input properties that depends on the intersection of whats in common between all the train pairs.
                // skip, if input_property is not yet computed
                // skip, if input_property is cannot be computed
                // save the computed input_property in HashSet

                for output_property in &output_properties {
                    let output_value: u8 = match output_property {
                        PropertyOutput::OutputWidth => width_output,
                        PropertyOutput::OutputHeight => height_output,
                    };
                    let input_image_size: u8 = match output_property {
                        PropertyOutput::OutputWidth => pair.input.image.width(),
                        PropertyOutput::OutputHeight => pair.input.image.height(),
                    };

                    // Future experiments to do.
                    // Currently the output_properties are populated once.
                    // skip, if output_property is not yet computed
                    // skip, if output_property is cannot be computed
                    // save the computed output_property in HashSet
    
                    let is_same = input_value == output_value;
                    if is_same {
                        let label = Label::OutputPropertyIsEqualToInputProperty { output: *output_property, input: *input_property };
                        pair.label_set.insert(label);
                    }

                    for scale in 2..8u8 {
                        let input_value_scaled: u32 = (input_value as u32) * (scale as u32);
                        if input_value_scaled == (output_value as u32) {
                            let label0 = Label::OutputPropertyIsInputPropertyMultipliedBy { output: *output_property, input: *input_property, scale };
                            pair.label_set.insert(label0);
                            let label1 = Label::OutputPropertyIsInputPropertyMultipliedBySomeScale { output: *output_property, input: *input_property };
                            pair.label_set.insert(label1);
                            break;
                        }
                    }

                    for scale in 2..8u8 {
                        let value: u32 = (input_value as u32) / (scale as u32);
                        let value_remain: u32 = (input_value as u32) % (scale as u32);
                        if value_remain == 0 && value == (output_value as u32) {
                            let label0 = Label::OutputPropertyIsInputPropertyDividedBy { output: *output_property, input: *input_property, scale };
                            pair.label_set.insert(label0);
                            let label1 = Label::OutputPropertyIsInputPropertyDividedBySomeScale { output: *output_property, input: *input_property };
                            pair.label_set.insert(label1);
                            break;
                        }
                    }

                    {
                        let input_value_scaled: u32 = (input_value as u32) * (input_image_size as u32);
                        if input_value_scaled == (output_value as u32) {
                            let label0 = Label::OutputPropertyIsInputPropertyMultipliedByInputSize { output: *output_property, input: *input_property };
                            pair.label_set.insert(label0);
                        }
                    }
                }
            }

        }

        self.update_label_set_intersection();

        Ok(())
    }

    /// Returns an array of tuples. Each tuple is a priority and a value.
    /// 
    /// These functions are nearly identical, and I think they can be merged.
    /// `output_size_rules_for()` 
    /// `predict_output_size_for_output_property_and_input()`
    /// so it's the same computation that is taking place.
    fn output_size_rules_for(&self, property_output: &PropertyOutput) -> Vec<(RulePriority, String)> {
        let mut rules: Vec<(RulePriority, String)> = vec!();

        for label in &self.label_set_intersection {
            match label {
                Label::OutputPropertyIsConstant { output, value } => {
                    if output != property_output {
                        continue;
                    }
                    let s = format!("{:?} is always {:?}", output, value);
                    rules.push((RulePriority::Simple, s));
                },
                Label::OutputPropertyIsEqualToInputProperty { output, input } => {
                    if output != property_output {
                        continue;
                    }
                    let s = format!("{:?} = {:?}", output, input);
                    let mut priority = RulePriority::Medium;
                    if *output == PropertyOutput::OutputWidth && *input == PropertyInput::InputWidth {
                        priority = RulePriority::Simple;
                    }
                    if *output == PropertyOutput::OutputHeight && *input == PropertyInput::InputHeight {
                        priority = RulePriority::Simple;
                    }
                    rules.push((priority, s));
                },
                Label::OutputPropertyIsInputPropertyMultipliedBy { output, input, scale } => {
                    if output != property_output {
                        continue;
                    }
                    let s = format!("{:?} = {:?} * {}", output, input, scale);
                    rules.push((RulePriority::Advanced, s));
                },
                Label::OutputPropertyIsInputPropertyMultipliedByInputSize { output, input } => {
                    if output != property_output {
                        continue;
                    }
                    let input_name: &str = match property_output {
                        PropertyOutput::OutputWidth => "InputWidth",
                        PropertyOutput::OutputHeight => "InputHeight"
                    };
                    let s = format!("{:?} = {:?} * {}", output, input, input_name);
                    rules.push((RulePriority::Advanced, s));
                },
                Label::OutputPropertyIsInputPropertyDividedBy { output, input, scale } => {
                    if output != property_output {
                        continue;
                    }
                    let s = format!("{:?} = {:?} / {}", output, input, scale);
                    rules.push((RulePriority::Advanced, s));
                },
                _ => {}
            }
        }

        // Simplest rules first, Advanced rules last
        rules.sort();

        rules
    }

    pub fn estimated_output_size(&self) -> String {
        let output_properties: [PropertyOutput; 2] = [
            PropertyOutput::OutputWidth, 
            PropertyOutput::OutputHeight
        ];
        let mut rules_vec: Vec<String> = vec!();
        for output_property in &output_properties {
            let rules: Vec<(RulePriority, String)> = self.output_size_rules_for(output_property);
            if rules.is_empty() {
                break;
            }
            let explanations: Vec<String> = rules.iter().map(|(_priority, explanation)| explanation.to_string()).collect();
            let name: &str = match output_property {
                PropertyOutput::OutputWidth => "width",
                PropertyOutput::OutputHeight => "height"
            };
            let combined_rule = format!("{}: {}", name, explanations.join(", "));
            rules_vec.push(combined_rule);
        }
        if rules_vec.len() == output_properties.len() {
            let rules_pretty: String = rules_vec.join("<br>");
            return rules_pretty;
        }

        "Undecided".to_string()
    }

    /// Returns an array of tuples. Each tuple is a priority and a value.
    /// 
    /// These functions are nearly identical, and I think they can be merged.
    /// `output_size_rules_for()` 
    /// `predict_output_size_for_output_property_and_input()`
    /// so it's the same computation that is taking place.
    fn predict_output_size_for_output_property_and_input(&self, property_output: &PropertyOutput, buffer_input: &Input) -> Vec<(RulePriority, u8)> {
        let mut rules: Vec<(RulePriority, u8)> = vec!();

        let dict: &HashMap<PropertyInput, u8> = &buffer_input.input_properties;
        for label in &self.label_set_intersection {
            match label {
                Label::OutputPropertyIsConstant { output, value } => {
                    if output != property_output {
                        continue;
                    }
                    rules.push((RulePriority::Medium, *value));
                },
                Label::OutputPropertyIsEqualToInputProperty { output, input } => {
                    if output != property_output {
                        continue;
                    }
                    let input_value_option: Option<&u8> = dict.get(input);
                    let input_value: u8 = match input_value_option {
                        Some(value) => *value,
                        None => {
                            continue;
                        }
                    };
                    let mut priority = RulePriority::Medium;
                    if *output == PropertyOutput::OutputWidth && *input == PropertyInput::InputWidth {
                        priority = RulePriority::Simple;
                    }
                    if *output == PropertyOutput::OutputHeight && *input == PropertyInput::InputHeight {
                        priority = RulePriority::Simple;
                    }
                    rules.push((priority, input_value));
                },
                Label::OutputPropertyIsInputPropertyMultipliedBy { output, input, scale } => {
                    if output != property_output {
                        continue;
                    }
                    let input_value_option: Option<&u8> = dict.get(input);
                    let input_value: u8 = match input_value_option {
                        Some(value) => *value,
                        None => {
                            continue;
                        }
                    };
                    let computed_value: u32 = (input_value as u32) * (*scale as u32);
                    if computed_value > (u8::MAX as u32) {
                        continue;
                    }
                    let value: u8 = computed_value as u8;
                    rules.push((RulePriority::Advanced, value));
                },
                Label::OutputPropertyIsInputPropertyMultipliedByInputSize { output, input } => {
                    if output != property_output {
                        continue;
                    }
                    let input_value_option: Option<&u8> = dict.get(input);
                    let input_value: u8 = match input_value_option {
                        Some(value) => *value,
                        None => {
                            continue;
                        }
                    };
                    let input_size: u8 = match property_output {
                        PropertyOutput::OutputWidth => buffer_input.image.width(),
                        PropertyOutput::OutputHeight => buffer_input.image.height()
                    };
                    let computed_value: u32 = (input_value as u32) * (input_size as u32);
                    if computed_value > (u8::MAX as u32) {
                        continue;
                    }
                    let value: u8 = computed_value as u8;
                    rules.push((RulePriority::Advanced, value));
                },
                Label::OutputPropertyIsInputPropertyDividedBy { output, input, scale } => {
                    if output != property_output {
                        continue;
                    }
                    let input_value_option: Option<&u8> = dict.get(input);
                    let input_value: u8 = match input_value_option {
                        Some(value) => *value,
                        None => {
                            continue;
                        }
                    };
                    let computed_value_remain: u8 = input_value % (*scale);
                    if computed_value_remain != 0 {
                        continue;
                    }
                    let computed_value: u8 = input_value / (*scale);
                    if computed_value < 1 {
                        continue;
                    }
                    rules.push((RulePriority::Advanced, computed_value));
                },
                _ => {}
            }
        }

        // Simplest rules first, Advanced rules last
        rules.sort();

        rules
    }

    pub fn predict_output_size_for_input(&self, input: &Input) -> String {
        let output_properties: [PropertyOutput; 2] = [
            PropertyOutput::OutputWidth, 
            PropertyOutput::OutputHeight
        ];
        let mut found_width: Option<u8> = None;
        let mut found_height: Option<u8> = None;
        for output_property in &output_properties {
            let rules: Vec<(RulePriority, u8)> = self.predict_output_size_for_output_property_and_input(output_property, input);

            // pick the simplest rule
            let value: u8 = match rules.first() {
                Some((_prio, value)) => *value,
                None => {
                    break;
                }
            };

            // Future experiments to do. 
            // Instead of picking the first item, then compute confidence score all the items.
            // if there are many advanced items that agree on a value, then it may be more likely
            // if there is one simple rule, and no advanced rules, then it may be the most likely
            // if all the rules agree on a single value, then it may be the most likely.
            // If there is an `IsSquare` label, then prefer the square above other choices
            // If there is an `Aspect ratio` label, then prefer that aspect ratio above other choices

            match output_property {
                PropertyOutput::OutputWidth => { found_width = Some(value) },
                PropertyOutput::OutputHeight => { found_height = Some(value) }
            }
        }

        match (found_width, found_height) {
            (Some(width), Some(height)) => {
                return format!("{}x{}", width, height);
            },
            _ => {
                return "Undecided".to_string()
            }
        }
    }

    fn labelset_to_html(label_set: &LabelSet) -> String {
        let mut label_vec: Vec<String> = label_set.iter().map(|label| format!("{:?}", label)).collect();
        if label_vec.is_empty() {
            return "empty".to_string();
        }
        label_vec.sort();
        label_vec = label_vec.iter().map(|label| format!("<li>{}</li>", label)).collect();
        format!("<ul>{}</ul>", label_vec.join(""))
    }

    fn input_properties_to_html(input_properties: &HashMap<PropertyInput, u8>) -> String {
        let mut items: Vec<String> = input_properties.iter().map(|(key,value)| format!("{:?} {}", key, value)).collect();
        if items.is_empty() {
            return "empty".to_string();
        }
        items.sort();
        let list_vec: Vec<String> = items.iter().map(|label| format!("<li>{}</li>", label)).collect();
        format!("<ul>{}</ul>", list_vec.join(""))
    }

    pub fn inspect(&self) -> anyhow::Result<()> {
        let mut row_title: String = "<tr><td></td>".to_string();
        let mut row_input_image: String = "<tr><td>Input image</td>".to_string();
        let mut row_input_properties: String = "<tr><td>Input properties</td>".to_string();
        let mut row_output_image: String = "<tr><td>Output image</td>".to_string();
        let mut row_action: String = "<tr><td>Action</td>".to_string();
        let mut row_labels: String = "<tr><td>Labels</td>".to_string();
        for pair in &self.pairs {
            {
                row_title += "<td>";
                let title: &str = match pair.pair_type {
                    PairType::Train => "Train",
                    PairType::Test => "Test",
                };
                row_title += title;
                row_title += "</td>";
            }
            {
                row_input_image += "<td>";
                row_input_image += &pair.input.image.to_html();
                row_input_image += "</td>";
            }
            {
                row_input_properties += "<td>";
                row_input_properties += &Self::input_properties_to_html(&pair.input.input_properties);
                row_input_properties += "</td>";
            }
            {
                row_output_image += "<td>";
                row_output_image += &pair.output.image.to_html();
                row_output_image += "</td>";
            }
            {
                row_action += "<td>Removal<br>";
                match pair.removal_histogram.color_image() {
                    Ok(image) => {
                        row_action += &image.to_html();
                    },
                    Err(_) => {
                        row_action += "N/A";
                    }
                }
                row_action += "<br>Insert<br>";
                match pair.insert_histogram.color_image() {
                    Ok(image) => {
                        row_action += &image.to_html();
                    },
                    Err(_) => {
                        row_action += "N/A";
                    }
                }
                row_action += "</td>";
            }
            {
                row_labels += "<td>";
                row_labels += &Self::labelset_to_html(&pair.label_set);
                row_labels += "</td>";
            }
        }

        row_title += "<td>Analysis</td>";

        row_input_image += "<td>Union<br>";
        match self.input_histogram_union.color_image() {
            Ok(image) => {
                row_input_image += &image.to_html();
            },
            Err(_) => {
                row_input_image += "N/A";
            }
        }
        row_input_image += "<br><br>Intersection<br>";
        match self.input_histogram_intersection.color_image() {
            Ok(image) => {
                row_input_image += &image.to_html();
            },
            Err(_) => {
                row_input_image += "N/A";
            }
        }
        row_input_image += "</td>";

        row_input_properties += "<td>";
        row_input_properties += &Self::input_properties_to_html(&self.input_properties_intersection);
        row_input_properties += "</td>";

        row_output_image += "<td>Union<br>";
        match self.output_histogram_union.color_image() {
            Ok(image) => {
                row_output_image += &image.to_html();
            },
            Err(_) => {
                row_output_image += "N/A";
            }
        }
        row_output_image += "<br><br>Intersection<br>";
        match self.output_histogram_intersection.color_image() {
            Ok(image) => {
                row_output_image += &image.to_html();
            },
            Err(_) => {
                row_output_image += "N/A";
            }
        }
        row_output_image += "</td>";

        row_action += "<td>Removal<br>";
        match self.removal_histogram_intersection.color_image() {
            Ok(image) => {
                row_action += &image.to_html();
            },
            Err(_) => {
                row_action += "N/A";
            }
        }
        row_action += "<br>Insert<br>";
        match self.insert_histogram_intersection.color_image() {
            Ok(image) => {
                row_action += &image.to_html();
            },
            Err(_) => {
                row_action += "N/A";
            }
        }
        row_action += "</td>";

        row_labels += "<td>";
        row_labels += &Self::labelset_to_html(&self.label_set_intersection);
        row_labels += "</td>";

        row_title += "</tr>";
        row_input_image += "</tr>";
        row_input_properties += "</tr>";
        row_output_image += "</tr>";
        row_action += "</tr>";
        row_labels += "</tr>";

        let html = format!(
            "<h2>{}</h2><p>Output size: {}</p><table>{}{}{}{}{}{}</table>",
            self.id, 
            self.estimated_output_size(),
            row_title,
            row_input_image, 
            row_input_properties, 
            row_output_image, 
            row_action,
            row_labels
        );
        HtmlLog::html(html);
        Ok(())
    }

    pub fn count_train(&self) -> usize {
        let mut count: usize = 0;
        for pair in &self.pairs {
            if pair.pair_type == PairType::Train {
                count += 1;
            }
        }
        count
    }

    pub fn count_test(&self) -> usize {
        let mut count: usize = 0;
        for pair in &self.pairs {
            if pair.pair_type == PairType::Test {
                count += 1;
            }
        }
        count
    }
}
