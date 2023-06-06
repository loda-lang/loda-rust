use super::{arc_work_model, GridLabel, GridPattern, HtmlFromTask, InputLabel, SymmetryLabel, AutoRepairSymmetry, ImageObjectEnumerate, SingleColorObjectRectangleLabel, SingleColorObjects, SingleColorObjectRectangle};
use super::arc_work_model::{Input, PairType, Object, Prediction};
use super::{Image, ImageMask, ImageMaskCount, ConnectedComponent, PixelConnectivity, ImageSize, ImageTrim, Histogram, ImageHistogram, ObjectsSortByProperty};
use super::{SubstitutionRule, SingleColorObjectSatisfiesLabel};
use super::{InputLabelSet, ActionLabel, ActionLabelSet, ObjectLabel, PropertyInput, PropertyOutput, ActionLabelUtil};
use super::{OutputSpecification};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum RulePriority {
    Simple,
    Medium,
    Advanced,
}

impl arc_work_model::Task {
    #[allow(dead_code)]
    pub fn has_substitution_rule_applied(&self) -> bool {
        for pair in &self.pairs {
            if pair.input.substitution_rule_applied.is_none() {
                return false;
            }
        }
        true
    }

    pub fn compute_substitution_rule(&mut self) -> anyhow::Result<()> {
        let verbose = false;
        if !self.is_output_size_same_as_input_size() {
            return Ok(());
        }
        let mut image_pairs = Vec::<(Image, Image)>::new();
        for pair in &self.pairs {
            if pair.pair_type != PairType::Train {
                continue;
            }
            image_pairs.push((pair.input.image.clone(), pair.output.image.clone()));
        }

        if verbose {
            println!("task: {} searching for substitution rule", self.id);
        }
        let rule: SubstitutionRule = match SubstitutionRule::find_rule(image_pairs) {
            Ok(value) => value,
            Err(_) => {
                if verbose {
                    println!("task: {} no substitution rule found", self.id);
                }
                return Ok(());
            }
        };

        if verbose {
            println!("task: {} substitution rule: {:?}", self.id, rule);
        }

        let mut reset = false;
        for pair in &mut self.pairs {
            let applied: Image = match rule.apply(&pair.input.image) {
                Ok(value) => value,
                Err(_) => {
                    reset = true;
                    break;
                }
            };
            pair.input.substitution_rule_applied = Some(applied);
        }
        if reset {
            for pair in &mut self.pairs {
                pair.input.substitution_rule_applied = None;
            }
        }
        Ok(())
    }

    fn update_input_properties_for_all_pairs(&mut self) -> anyhow::Result<()> {
        for pair in &mut self.pairs {
            pair.input.update_input_properties();
            pair.input.update_input_label_set()?;
        }
        Ok(())
    }

    fn update_action_label_set_intersection(&mut self) {
        let mut label_set = ActionLabelSet::new();
        let mut is_first = true;
        for pair in &mut self.pairs {
            if pair.pair_type == PairType::Test {
                continue;
            }
            if is_first {
                label_set = pair.action_label_set.clone();
                is_first = false;
                continue;
            }
            label_set = label_set.intersection(&pair.action_label_set).map(|l| l.clone()).collect();
        }
        self.action_label_set_intersection = label_set;
    }

    #[allow(dead_code)]
    pub fn has_repair_mask(&self) -> bool {
        for pair in &self.pairs {
            if pair.input.repair_mask.is_none() {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    pub fn has_repaired_image(&self) -> bool {
        for pair in &self.pairs {
            if pair.input.repaired_image.is_none() {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    pub fn has_predicted_single_color_image(&self) -> bool {
        for pair in &self.pairs {
            if pair.input.predicted_single_color_image.is_none() {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    pub fn has_grid_pattern(&self) -> bool {
        for pair in &self.pairs {
            if pair.input.grid_pattern.is_none() {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    pub fn has_enumerated_objects(&self) -> bool {
        for pair in &self.pairs {
            if pair.input.enumerated_objects.is_none() {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    pub fn has_removal_color(&self) -> bool {
        for pair in &self.pairs {
            if pair.input.removal_color.is_none() {
                return false;
            }
        }
        true
    }

    fn assign_repair_mask(&mut self) {
        self.assign_repair_mask_based_on_single_color_object_rectangle();
        self.assign_repair_mask_based_on_single_color_removal_and_changes_limited_to_color();
        self.assign_repair_mask_with_symmetry_repair_color();
        self.assign_repair_mask_based_on_most_popular_color();
    }

    /// Generate `repair_mask`
    /// 
    /// If there is a symmetric pattern with a possible repair color,
    /// then use the repair color for the repair mask.
    fn assign_repair_mask_with_symmetry_repair_color(&mut self) {
        if self.has_repair_mask() {
            return;
        }

        // proceed only if all the pairs have a repair color
        for pair in &mut self.pairs {
            if let Some(symmetry) = &pair.input.symmetry {
                if symmetry.repair_color.is_none() {
                    // One or more of the pairs is missing a repair color
                    return;
                }
            }
        }

        // create attention mask with the repair color.
        for pair in &mut self.pairs {
            let color: u8;
            if let Some(symmetry) = &pair.input.symmetry {
                if let Some(repair_color) = symmetry.repair_color {
                    color = repair_color;
                } else {
                    // One or more of the pairs is missing a repair color
                    continue;
                }
            } else {
                // Symmetry is not initialized
                continue;
            }
            _ = pair.input.assign_repair_mask_with_color(color);
        }

    }

    /// Generate `repair_mask`
    /// 
    /// Precondition
    /// `OutputImageIsInputImageWithChangesLimitedToPixelsWithColor` is the pixel colors that are going to be changed.
    /// 
    /// Precondition
    /// there is just 1 color for removal.
    /// 
    /// The removal color must be the same as the `OutputImageIsInputImageWithChangesLimitedToPixelsWithColor`.
    fn assign_repair_mask_based_on_single_color_removal_and_changes_limited_to_color(&mut self) {
        if self.has_repair_mask() {
            return;
        }

        if self.removal_histogram_intersection.number_of_counters_greater_than_zero() >= 2 {
            // too many colors to agree on a single color
            return;
        }
        let single_color_removal: u8 = match self.removal_histogram_intersection.most_popular_color_disallow_ambiguous() {
            Some(value) => value,
            None => {
                return;
            }
        };

        let mut found_color: Option<u8> = None;
        for label in &self.action_label_set_intersection {
            match label {
                ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { color } => {
                    found_color = Some(*color);
                    break;
                },
                _ => {}
            }
        }
        let color: u8 = match found_color {
            Some(value) => value,
            None => {
                return;
            }
        };

        if color != single_color_removal {
            // disagreement about what color is to be repaired
            return;
        }

        for pair in &mut self.pairs {
            _ = pair.input.assign_repair_mask_with_color(color);
        }
    }

    /// Generate `repair_mask`
    /// 
    /// Precondition
    /// `OutputSizeIsTheSameAsSingleColorObject` is a rectangle with a specific color.
    /// 
    /// Precondition
    /// There `OutputSizeIsTheSameAsSingleColorObject` has its color among the `removal_histogram_intersection`.
    fn assign_repair_mask_based_on_single_color_object_rectangle(&mut self) {
        if self.has_repair_mask() {
            return;
        }

        let mut found_colors = Histogram::new();
        for action_label in &self.action_label_set_intersection {
            match action_label {
                ActionLabel::OutputSizeIsTheSameAsSingleColorObject { label } => {
                    match label {
                        SingleColorObjectRectangleLabel::RectangleWithColor { color } => {
                            found_colors.increment(*color);
                        },
                        SingleColorObjectRectangleLabel::SquareWithColor { color } => {
                            found_colors.increment(*color);
                        },
                        SingleColorObjectRectangleLabel::NonSquareWithColor { color } => {
                            found_colors.increment(*color);
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        if found_colors.number_of_counters_greater_than_zero() > 1 {
            // Multiple colors found, no consensus on what color is to be repaired
            return;
        }
        let color: u8 = match found_colors.most_popular_color_disallow_ambiguous() {
            Some(value) => value,
            None => {
                // Did not find a rectangle with a specific color
                return;
            }
        };

        if self.removal_histogram_intersection.get(color) == 0 {
            // The color is not among the colors being removed
            return;
        }

        for pair in &mut self.pairs {
            _ = pair.input.assign_repair_mask_with_color(color);
        }
    }

    /// Generate `repair_mask`
    /// 
    fn assign_repair_mask_based_on_most_popular_color(&mut self) {
        if self.has_repair_mask() {
            return;
        }

        let mut histogram1: Histogram = self.input_histogram_union.clone();
        histogram1.subtract_histogram(&self.input_histogram_intersection);
        let mut histogram2: Histogram = self.input_histogram_union.clone();
        histogram2.subtract_histogram(&histogram1);
        let color: u8 = match histogram2.most_popular_color_disallow_ambiguous() {
            Some(color) => color,
            None => {
                return;
            }
        };
        for pair in &mut self.pairs {
            _ = pair.input.assign_repair_mask_with_color(color);
        }
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

    fn update_input_label_set_intersection(&mut self) {
        let mut input_label_set = InputLabelSet::new();
        let mut is_first = true;
        for pair in &mut self.pairs {
            if pair.pair_type != PairType::Train {
                continue;
            }
            if is_first {
                input_label_set = pair.input.input_label_set.clone();
                is_first = false;
                continue;
            }
            input_label_set = input_label_set.intersection(&pair.input.input_label_set).map(|l| l.clone()).collect();
        }
        self.input_label_set_intersection = input_label_set;
    }

    fn assign_action_labels_for_output_for_train(&mut self) {
        for pair in &mut self.pairs {
            if pair.pair_type != PairType::Train {
                continue;
            }
            pair.update_action_label_set();
        }
    }

    fn assign_input_properties_related_to_removal_histogram(&mut self) {
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

            // Determine if the removed color is a rectangle
            {
                match image_mask.trim_color(1) {
                    Ok(image) => {
                        let mass: u16 = image.mask_count_one();
                        if mass == 0 {
                            pair.input.input_properties.insert(PropertyInput::InputWidthOfRemovedRectangleAfterSingleColorRemoval, image.width());
                            pair.input.input_properties.insert(PropertyInput::InputHeightOfRemovedRectangleAfterSingleColorRemoval, image.height());
                        }
                    },
                    Err(_) => {}
                }
            }

            let ignore_mask: Image = image_mask.to_mask_where_color_is(0);

            let result = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &image_mask, &ignore_mask);
            let object_images: Vec<Image> = match result {
                Ok(images) => images,
                Err(_) => {
                    continue;
                }
            };

            let mut mass_max: u16 = 0;
            let mut found_index_mass_max: Option<usize> = None;
            for (index, image) in object_images.iter().enumerate() {

                let mass: u16 = image.mask_count_one();
                if mass > mass_max {
                    mass_max = mass;
                    found_index_mass_max = Some(index);
                }
            }

            if mass_max > 0 && mass_max <= (u8::MAX as u16) {
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
                    pair.input.input_properties.insert(PropertyInput::InputWidthOfPrimaryObjectAfterSingleColorRemoval, width);
                    pair.input.input_properties.insert(PropertyInput::InputHeightOfPrimaryObjectAfterSingleColorRemoval, height);
                }
            }
        }
    }

    fn assign_input_properties_related_to_input_histogram_intersection(&mut self) {
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
                let mass: u16 = image_mask.mask_count_zero();
                if mass > 0 && mass <= (u8::MAX as u16) {
                    let mass_value: u8 = mass as u8;
                    pair.input.input_properties.insert(PropertyInput::InputNumberOfPixelsCorrespondingToTheSingleIntersectionColor, mass_value);
                }
            }
            {
                let mass: u16 = image_mask.mask_count_one();
                if mass > 0 && mass <= (u8::MAX as u16) {
                    let mass_value: u8 = mass as u8;
                    pair.input.input_properties.insert(PropertyInput::InputNumberOfPixelsNotCorrespondingToTheSingleIntersectionColor, mass_value);
                }
            }

            let ignore_mask: Image = image_mask.to_mask_where_color_is(0);

            let result = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &image_mask, &ignore_mask);
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
            let mut mass_max: u16 = 0;
            let mut found_index_mass_max: Option<usize> = None;
            for (index, image) in object_images.iter().enumerate() {

                let mass: u16 = image.mask_count_one();
                if mass > mass_max {
                    mass_max = mass;
                    found_index_mass_max = Some(index);
                }
            }

            if mass_max > 0 && mass_max <= (u8::MAX as u16) {
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

    /// Extract `Vec<SingleColorObjectLabel>` from `input_label_set_intersection`.
    fn single_color_object_labels_from_input(&self) -> Vec<SingleColorObjectRectangleLabel> {
        let mut single_color_object_labels = Vec::<SingleColorObjectRectangleLabel>::new();
        for input_label in &self.input_label_set_intersection {
            let single_color_object_label: SingleColorObjectRectangleLabel = match input_label {
                InputLabel::InputSingleColorObjectRectangle { label } => label.clone(),
                _ => continue
            };
            single_color_object_labels.push(single_color_object_label);
        }
        single_color_object_labels
    }

    pub fn assign_action_labels_related_to_single_color_objects_and_output_size(&mut self) -> anyhow::Result<()> {
        let single_color_object_labels: Vec<SingleColorObjectRectangleLabel> = self.single_color_object_labels_from_input();
        if single_color_object_labels.is_empty() {
            return Ok(());
        }

        for query_id in 0..6u8 {
            let mut ambiguity_count: usize = 0;
            let mut found_label: Option<&SingleColorObjectRectangleLabel> = None;
            for single_color_object_label in &single_color_object_labels {
                match single_color_object_label {
                    SingleColorObjectRectangleLabel::SquareWithColor { color: _ } => {
                        if query_id == 0 {
                            ambiguity_count += 1;
                            found_label = Some(single_color_object_label);
                        }
                    },
                    SingleColorObjectRectangleLabel::NonSquareWithColor { color: _ } => {
                        if query_id == 1 {
                            ambiguity_count += 1;
                            found_label = Some(single_color_object_label);
                        }
                    },
                    SingleColorObjectRectangleLabel::RectangleWithColor { color: _ } => {
                        if query_id == 2 {
                            ambiguity_count += 1;
                            found_label = Some(single_color_object_label);
                        }
                    },
                    SingleColorObjectRectangleLabel::SquareWithSomeColor => {
                        if query_id == 3 {
                            ambiguity_count += 1;
                            found_label = Some(single_color_object_label);
                        }
                    },
                    SingleColorObjectRectangleLabel::NonSquareWithSomeColor => {
                        if query_id == 4 {
                            ambiguity_count += 1;
                            found_label = Some(single_color_object_label);
                        }
                    },
                    SingleColorObjectRectangleLabel::RectangleWithSomeColor => {
                        if query_id == 5 {
                            ambiguity_count += 1;
                            found_label = Some(single_color_object_label);
                        }
                    }
                }
            }
            if ambiguity_count > 1 {
                // Reject ambiguous scenarios with 2 or more labels.
                continue;
            }
            let single_color_object_label: &SingleColorObjectRectangleLabel = match found_label {
                Some(value) => value,
                None => continue
            };
            // The `single_color_object_label` is unambiguous.
            // println!("task: {} query_id: {} single_color_object_label: {:?}", self.id, query_id, single_color_object_label);

            match self.assign_output_size_for_single_color_objects_with_label(single_color_object_label, false) {
                Ok(()) => {
                    // yes, there is an unambiguous object/label combo that can be used
                },
                Err(_) => {
                    // unable to find a unique object that satisfy the label
                    continue;
                }
            }

            // go ahead and assign predictions
            match self.assign_output_size_for_single_color_objects_with_label(single_color_object_label, true) {
                Ok(()) => {
                    // successfully assigned a label
                },
                Err(_) => {
                    // unable to find a unique object that satisfy the label
                    continue;
                }
            }
            
            // println!("OutputSizeIsTheSameAsSingleColorObject task: {} query_id: {} single_color_object_label: {:?}", self.id, query_id, single_color_object_label);

            // Found a match, no point in continuing searching.
            // the first query_id's have the strongest confidence.
            // the last query_id's are weaker and less desired.
            break;
        }

        Ok(())
    }

    fn assign_output_size_for_single_color_objects_with_label(&mut self, single_color_object_label: &SingleColorObjectRectangleLabel, execute: bool) -> anyhow::Result<()> {
        let mut predicted_sizes = HashMap::<usize, ImageSize>::new();
        for (pair_index, pair) in self.pairs.iter().enumerate() {
            let single_color_objects: &SingleColorObjects = match &pair.input.single_color_objects {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("All input pairs must have some single_color_objects"));
                }
            };
            let mut ambiguity_count: usize = 0;
            let mut found_object: Option<&SingleColorObjectRectangle> = None;
            for object in &single_color_objects.rectangle_vec {
                if !object.satisfies_label(single_color_object_label) {
                    continue;
                }
                found_object = Some(object);
                ambiguity_count += 1;
            }
            if ambiguity_count > 1 {
                // Reject ambiguous scenarios with 2 or more objects that satisfy the label.
                return Err(anyhow::anyhow!("Multiple objects satisfy the label. Ambiguous which one to pick."));
            }
            let object: &SingleColorObjectRectangle = match found_object {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Didn't find any object that satisfy the label."));
                }
            };

            if pair.pair_type == PairType::Train {
                if object.bounding_box.size() != pair.output.image.size() {
                    return Err(anyhow::anyhow!("This object cannot not explain the output size."));
                }
            }

            predicted_sizes.insert(pair_index, object.bounding_box.size());
        }


        if predicted_sizes.len() != self.pairs.len() {
            return Err(anyhow::anyhow!("Unable to predict sizes for all pairs"));
        }

        // Don't attempt predicting the output size when data is too poor
        let mut bigger_than_2px = false;
        for (_pair_index, predicted_size) in &predicted_sizes {
            if predicted_size.width >= 2 || predicted_size.height >= 2 {
                bigger_than_2px = true;
                break;
            }
        }
        if !bigger_than_2px {
            // If the object sizes are all 1x1 then it's useless for predicting the output size.
            return Err(anyhow::anyhow!("All the objects are too small for doing predictions about the output size"));
        }

        // Go ahead and assign predictions. In first stage, then do nothing. In second state, then assign labels.
        if !execute {
            return Ok(());
        }

        // Future experiment: 
        // if the object sizes varies a lot and it corresponds with the output size then it's a strong connection.
        // Assign a confidence score to the predicted size.

        for (pair_index, pair) in self.pairs.iter_mut().enumerate() {
            let predicted_size: ImageSize = match predicted_sizes.get(&pair_index) {
                Some(value) => *value,
                None => {
                    return Err(anyhow::anyhow!("Missing predicted size for all pairs"));
                }
            };
            let label = Prediction::OutputSize {
                size: predicted_size,
            };
            pair.prediction_set.insert(label);
        }

        for pair in &mut self.pairs {
            let action_label = ActionLabel::OutputSizeIsTheSameAsSingleColorObject { label: single_color_object_label.clone() };
            pair.action_label_set.insert(action_label);
        }

        Ok(())
    }

    pub fn populate(&mut self) -> anyhow::Result<()> {
        self.assign_labels()?;
        self.assign_predicted_output_size();
        self.assign_predicted_output_palette();
        self.assign_predicted_output_image_is_input_image_with_changes_limited_to_pixels_with_color();
        _ = self.assign_predicted_single_color_image();
        _ = self.assign_removal_color();
        _ = self.assign_most_popular_intersection_color();
        _ = self.assign_single_pixel_noise_color();
        _ = self.assign_output_specification_vec();
        Ok(())
    }

    fn assign_labels(&mut self) -> anyhow::Result<()> {
        self.update_input_properties_for_all_pairs()?;
        self.update_input_properties_intersection();
        self.update_input_label_set_intersection();
        self.assign_input_properties_related_to_removal_histogram();
        self.assign_input_properties_related_to_input_histogram_intersection();
        self.assign_action_labels_for_output_for_train();
        _ = self.assign_action_labels_related_to_single_color_objects_and_output_size();

        let input_properties: [PropertyInput; 25] = [
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
            PropertyInput::InputBiggestValueThatDividesWidthAndHeight,
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
                        let label = ActionLabel::OutputPropertyIsEqualToInputProperty { output: *output_property, input: *input_property };
                        pair.action_label_set.insert(label);
                    }

                    for scale in 2..8u8 {
                        let input_value_scaled: u32 = (input_value as u32) * (scale as u32);
                        if input_value_scaled == (output_value as u32) {
                            let label0 = ActionLabel::OutputPropertyIsInputPropertyMultipliedBy { output: *output_property, input: *input_property, scale };
                            pair.action_label_set.insert(label0);
                            let label1 = ActionLabel::OutputPropertyIsInputPropertyMultipliedBySomeScale { output: *output_property, input: *input_property };
                            pair.action_label_set.insert(label1);
                            break;
                        }
                    }

                    for scale in 2..8u8 {
                        let value: u32 = (input_value as u32) / (scale as u32);
                        let value_remain: u32 = (input_value as u32) % (scale as u32);
                        if value_remain == 0 && value == (output_value as u32) {
                            let label0 = ActionLabel::OutputPropertyIsInputPropertyDividedBy { output: *output_property, input: *input_property, scale };
                            pair.action_label_set.insert(label0);
                            let label1 = ActionLabel::OutputPropertyIsInputPropertyDividedBySomeScale { output: *output_property, input: *input_property };
                            pair.action_label_set.insert(label1);
                            break;
                        }
                    }

                    {
                        let input_value_scaled: u32 = (input_value as u32) * (input_value as u32);
                        if input_value_scaled == (output_value as u32) {
                            let label = ActionLabel::OutputPropertyIsInputPropertySquared { output: *output_property, input: *input_property };
                            pair.action_label_set.insert(label);
                        }
                    }

                    {
                        let input_value_scaled: u32 = (input_value as u32) * (input_image_size as u32);
                        if input_value_scaled == (output_value as u32) {
                            let label = ActionLabel::OutputPropertyIsInputPropertyMultipliedByInputSize { output: *output_property, input: *input_property };
                            pair.action_label_set.insert(label);
                        }
                    }
                }
            }

        }

        self.update_action_label_set_intersection();

        self.compute_substitution_rule()?;

        self.assign_repair_mask();

        self.compute_input_repaired_image()?;

        self.compute_input_grid_pattern()?;

        self.compute_input_enumerated_objects()?;

        Ok(())
    }

    fn input_properties_intersection_get_unique_color_count(&self) -> Option<u8> {
        for (input_property, property_value) in &self.input_properties_intersection {
            match *input_property {
                PropertyInput::InputUniqueColorCount => {
                    return Some(*property_value);
                },
                _ => {}
            }
        }
        None
    }

    /// Returns an array of tuples. Each tuple is a priority and a value.
    /// 
    /// These functions are nearly identical, and I think they can be merged.
    /// `output_size_rules_for()` 
    /// `predict_output_size_for_output_property_and_input()`
    /// so it's the same computation that is taking place.
    fn output_size_rules_for(&self, property_output: &PropertyOutput) -> Vec<(RulePriority, String)> {
        let mut rules: Vec<(RulePriority, String)> = vec!();

        for label in &self.action_label_set_intersection {
            match label {
                ActionLabel::OutputPropertyIsConstant { output, value } => {
                    if output != property_output {
                        continue;
                    }
                    let s = format!("{:?} is always {:?}", output, value);
                    rules.push((RulePriority::Simple, s));
                },
                ActionLabel::OutputPropertyIsEqualToInputProperty { output, input } => {
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
                    if *input == PropertyInput::InputNumberOfPixelsNotCorrespondingToTheSingleIntersectionColor {
                        if self.input_properties_intersection_get_unique_color_count() == Some(2) {
                            priority = RulePriority::Simple;
                        }
                    }
                    if *input == PropertyInput::InputNumberOfPixelsCorrespondingToTheSingleIntersectionColor {
                        if self.input_properties_intersection_get_unique_color_count() == Some(2) {
                            priority = RulePriority::Simple;
                        }
                    }
                    rules.push((priority, s));
                },
                ActionLabel::OutputPropertyIsInputPropertyMultipliedBy { output, input, scale } => {
                    if output != property_output {
                        continue;
                    }
                    let s = format!("{:?} = {:?} * {}", output, input, scale);
                    rules.push((RulePriority::Advanced, s));
                },
                ActionLabel::OutputPropertyIsInputPropertyMultipliedByInputSize { output, input } => {
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
                ActionLabel::OutputPropertyIsInputPropertyDividedBy { output, input, scale } => {
                    if output != property_output {
                        continue;
                    }
                    let s = format!("{:?} = {:?} / {}", output, input, scale);
                    rules.push((RulePriority::Advanced, s));
                },
                ActionLabel::OutputPropertyIsInputPropertySquared { output, input } => {
                    if output != property_output {
                        continue;
                    }
                    let s = format!("{:?} = {:?}^2", output, input);
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
        for label in &self.action_label_set_intersection {
            // Future experiments: deal with multiple labels being satisfied, apply a score to the results, and pick the winner.
            match label {
                ActionLabel::OutputImageIsTheObjectWithObjectLabel { object_label } => {
                    return format!("{:?}", object_label);
                },
                _ => {}
            }
        }
        for action_label in &self.action_label_set_intersection {
            match action_label {
                ActionLabel::OutputSizeIsTheSameAsSingleColorObject { label } => {
                    return format!("{:?}", label);
                },
                _ => {}
            }
        }
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
        for label in &self.action_label_set_intersection {
            match label {
                ActionLabel::OutputPropertyIsConstant { output, value } => {
                    if output != property_output {
                        continue;
                    }
                    rules.push((RulePriority::Medium, *value));
                },
                ActionLabel::OutputPropertyIsEqualToInputProperty { output, input } => {
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
                    if *input == PropertyInput::InputNumberOfPixelsNotCorrespondingToTheSingleIntersectionColor {
                        if self.input_properties_intersection_get_unique_color_count() == Some(2) {
                            priority = RulePriority::Simple;
                        }
                    }
                    if *input == PropertyInput::InputNumberOfPixelsCorrespondingToTheSingleIntersectionColor {
                        if self.input_properties_intersection_get_unique_color_count() == Some(2) {
                            priority = RulePriority::Simple;
                        }
                    }

                    rules.push((priority, input_value));
                },
                ActionLabel::OutputPropertyIsInputPropertyMultipliedBy { output, input, scale } => {
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
                ActionLabel::OutputPropertyIsInputPropertyMultipliedByInputSize { output, input } => {
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
                ActionLabel::OutputPropertyIsInputPropertyDividedBy { output, input, scale } => {
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
                ActionLabel::OutputPropertyIsInputPropertySquared { output, input } => {
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
                    let computed_value: u32 = (input_value as u32) * (input_value as u32);
                    if computed_value > (u8::MAX as u32) {
                        continue;
                    }
                    let value: u8 = computed_value as u8;
                    rules.push((RulePriority::Advanced, value));
                },
                _ => {}
            }
        }

        // Simplest rules first, Advanced rules last
        rules.sort();

        rules
    }

    fn size_of_object(&self, input: &Input, object_label: &ObjectLabel) -> anyhow::Result<ImageSize> {
        let mut object_vec: Vec<arc_work_model::Object> = input.find_objects_using_histogram_most_popular_color()?;
        arc_work_model::Object::assign_labels_to_objects(&mut object_vec);
        for object in &object_vec {
            if object.object_label_set.contains(object_label) {
                let width: u8 = object.cropped_object_image.width();
                let height: u8 = object.cropped_object_image.height();
                let instance = ImageSize {
                    width, 
                    height
                };
                return Ok(instance);
            }
        }
        Err(anyhow::anyhow!("found no object with object_label: {:?}", object_label))
    }

    pub fn predict_output_size_for_input(&self, input: &Input) -> anyhow::Result<ImageSize> {
        for label in &self.action_label_set_intersection {
            // Future experiments: deal with multiple labels being satisfied, apply a score to the results, and pick the winner.
            match label {
                ActionLabel::OutputImageIsTheObjectWithObjectLabel { object_label } => {
                    match self.size_of_object(input, object_label) {
                        Ok(value) => {
                            return Ok(value);
                        },
                        Err(_) => {
                            // Didn't find an object with this property
                        }
                    }
                },
                _ => {}
            }
        }
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
                let instance = ImageSize {
                    width,
                    height
                };
                return Ok(instance);
            },
            _ => {
                return Err(anyhow::anyhow!("Undecided"));
            }
        }
    }

    fn assign_predicted_output_size(&mut self) {
        let estimate: String = self.estimated_output_size();
        if estimate == "Undecided" {
            // Idea: Flag the task as being undecided.
            return;
        }

        let mut predicted_size_dict = HashMap::<usize, ImageSize>::new();
        for (index, pair) in self.pairs.iter().enumerate() {
            let predicted_size: ImageSize = match self.predict_output_size_for_input(&pair.input) {
                Ok(value) => value,
                Err(_error) => {
                    // Idea: Flag the pair as being undecided.
                    continue;
                }
            };
            predicted_size_dict.insert(index, predicted_size);
        }

        // Idea: if one or more pairs are undecided, then don't assign predictions to any of the pairs.
        // Garbage data may confuse more than help.

        for (index, pair) in self.pairs.iter_mut().enumerate() {
            if let Some(predicted_size) = predicted_size_dict.get(&index) {
                pair.prediction_set.insert(arc_work_model::Prediction::OutputSize { size: *predicted_size });
            }
        }
    }

    fn predict_output_palette_for_input(&self, input: &Input) -> anyhow::Result<Histogram> {
        let mut histogram: Histogram = input.image.histogram_all();
        histogram.add_histogram(&self.insert_histogram_intersection);
        histogram.subtract_histogram(&self.removal_histogram_intersection);

        if self.removal_histogram_intersection.number_of_counters_greater_than_zero() == 0 {
            if self.action_label_set_intersection.contains(&ActionLabel::RemovalColorIsTheMostPopularColorOfInputImage) {
                if let Some(color) = histogram.most_popular_color() {
                    histogram.set_counter_to_zero(color);
                }
            }
        }

        // Future experiments:
        // What are the scenarios where this histogram is a bad prediction.
        // Are there scenarios where the histogram is "Undecided"
        //
        // There are tasks where the output colors depends on the shape of the object.
        // So one kind of shape gets assigned green.
        // Another kind of shape gets assigned blue.
        Ok(histogram)
    }

    fn assign_predicted_single_color_image(&mut self) -> anyhow::Result<()> {
        // OutputImageUniqueColorCount { 1 } must be preset
        let mut agreement_that_output_has_a_single_color = false;
        for action_label in &self.action_label_set_intersection {
            match *action_label {
                ActionLabel::OutputImageUniqueColorCount { count } => {
                    if count == 1 {
                        agreement_that_output_has_a_single_color = true;
                    }
                },
                _ => {}
            }
        }
        if !agreement_that_output_has_a_single_color {
            return Err(anyhow::anyhow!("No agreement that the output contains a single color."));
        }

        // Loop over the output palette predictions and output size predictions
        let mut predicted_output_images = HashMap::<usize, Image>::new();
        for (pair_index, pair) in self.pairs.iter().enumerate() {
            let mut found_color: Option<u8> = None;
            let mut found_size: Option<ImageSize> = None;
            for prediction in &pair.prediction_set {
                match prediction {
                    arc_work_model::Prediction::OutputPalette { histogram } => {
                        let unique_colors: u16 = histogram.number_of_counters_greater_than_zero();
                        if unique_colors > 1 {
                            return Err(anyhow::anyhow!("Multiple predicted colors. Ambiguous what color to pick."));
                        }
                        if unique_colors == 1 {
                            found_color = histogram.most_popular_color();
                        }
                    },
                    arc_work_model::Prediction::OutputSize { size } => {
                        found_size = Some(*size);
                    }
                    _ => {}
                }
            }
            let (size, color) = match (found_size, found_color) {
                (Some(size), Some(color)) => { (size, color) },
                _ => {
                    return Err(anyhow::anyhow!("Didn't find both size and color."));
                }
            };

            let image = Image::color(size.width, size.height, color);
            predicted_output_images.insert(pair_index, image);
        }

        for (pair_index, pair) in self.pairs.iter_mut().enumerate() {
            if let Some(image) = predicted_output_images.get(&pair_index) {
                pair.input.predicted_single_color_image = Some(image.clone());
            }
        }

        Ok(())
    }

    fn assign_removal_color(&mut self) -> anyhow::Result<()> {
        // All the training pairs agree on the same color
        if let Some(color) = self.removal_histogram_intersection.most_popular_color_disallow_ambiguous() {
            for pair in self.pairs.iter_mut() {
                pair.input.removal_color = Some(color);
            }
            return Ok(());
        }

        // In each pair, the color is the same as most popular color of the input
        if self.action_label_set_intersection.contains(&ActionLabel::RemovalColorIsTheMostPopularColorOfInputImage) {
            for pair in self.pairs.iter_mut() {
                let histogram: &Histogram = &pair.input.histogram;
                if let Some(color) = histogram.most_popular_color_disallow_ambiguous() {
                    pair.input.removal_color = Some(color);
                }
            }
            return Ok(());
        }
        Ok(())
    }

    fn assign_most_popular_intersection_color(&mut self) -> anyhow::Result<()> {
        // All the training pairs agree on the same color
        if let Some(color) = self.input_histogram_intersection.most_popular_color_disallow_ambiguous() {
            for pair in self.pairs.iter_mut() {
                pair.input.most_popular_intersection_color = Some(color);
            }
            return Ok(());
        }
        Ok(())
    }

    fn assign_single_pixel_noise_color(&mut self) -> anyhow::Result<()> {
        let mut found = false;
        for input_label in &self.input_label_set_intersection {
            match input_label {
                InputLabel::InputNoiseWithColor { color: _ } => {
                    found = true;
                    break;
                },
                InputLabel::InputNoiseWithSomeColor => {
                    found = true;
                    break;
                },
                _ => {}
            }
        }
        if !found {
            return Ok(());
        }
        for pair in self.pairs.iter_mut() {
            let single_color_objects: &SingleColorObjects = match &pair.input.single_color_objects {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let noise_color: u8 = match single_color_objects.single_pixel_noise_color() {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            pair.input.single_pixel_noise_color = Some(noise_color);
        }
        Ok(())
    }

    fn assign_output_specification_vec(&mut self) -> anyhow::Result<()> {
        let is_output_size_same_as_input_size: bool = self.is_output_size_same_as_input_size();
        for pair in self.pairs.iter_mut() {
            if is_output_size_same_as_input_size {
                let size: ImageSize = pair.input.image.size();
                pair.output_specification_vec.push(OutputSpecification::ImageSize { size });
            }
        }
        Ok(())
    }

    fn assign_predicted_output_palette(&mut self) {
        let mut predicted_histogram_dict = HashMap::<usize, Histogram>::new();
        for (index, pair) in self.pairs.iter().enumerate() {
            let predicted_histogram: Histogram = match self.predict_output_palette_for_input(&pair.input) {
                Ok(value) => value,
                Err(_error) => {
                    // Idea: Flag the pair as being undecided.
                    continue;
                }
            };
            predicted_histogram_dict.insert(index, predicted_histogram);
        }

        // Idea: if one or more pairs are undecided, then don't assign predictions to any of the pairs.
        // Garbage data may confuse more than help.

        for (index, pair) in self.pairs.iter_mut().enumerate() {
            if let Some(predicted_histogram) = predicted_histogram_dict.get(&index) {
                let histogram: Histogram = predicted_histogram.clone();
                pair.prediction_set.insert(arc_work_model::Prediction::OutputPalette { histogram });
            }
        }
    }

    fn assign_predicted_output_image_is_input_image_with_changes_limited_to_pixels_with_color(&mut self) {
        let mut use_specific_color: Option<u8> = None;
        let mut use_most_popular_color = false;
        let mut use_least_popular_color = false;
        for label in &self.action_label_set_intersection {
            match label {
                ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { color } => {
                    use_specific_color = Some(*color);
                },
                ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithMostPopularColorOfTheInputImage => {
                    use_most_popular_color = true;
                },
                ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithLeastPopularColorOfTheInputImage => {
                    use_least_popular_color = true;
                },
                _ => {}
            };
        }

        let mut predicted_color_dict = HashMap::<usize, u8>::new();
        for (index, pair) in self.pairs.iter().enumerate() {
            // If one or more pairs are undecided, then return out immediately, 
            // so that none of the pairs gets assigned a prediction.
            let predicted_color: u8;
            match (use_specific_color, use_most_popular_color, use_least_popular_color) {
                (Some(color), _, _) => {
                    predicted_color = color;
                },
                (None, true, false) => {
                    if let Some(color) = pair.input.histogram.most_popular_color() {
                        predicted_color = color;
                        // println!("predicted_color most popular: {}", color);
                    } else {
                        return;
                    }
                },
                (None, false, true) => {
                    if let Some(color) = pair.input.histogram.least_popular_color() {
                        predicted_color = color;
                        // println!("predicted_color least popular: {}", color);
                    } else {
                        return;
                    }
                }
                _ => return
            }
            predicted_color_dict.insert(index, predicted_color);
        }

        for (index, pair) in self.pairs.iter_mut().enumerate() {
            if let Some(predicted_color) = predicted_color_dict.get(&index) {
                // println!("predicted_color: {}", *predicted_color);
                pair.prediction_set.insert(arc_work_model::Prediction::OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { color: *predicted_color });
            }
        }
    }

    pub fn compute_input_repaired_image(&mut self) -> anyhow::Result<()> {
        if !self.has_repair_mask() {
            return Ok(());
        }

        let mut repair_horizontal: bool = false;
        let mut repair_vertical: bool = false;
        let mut repair_diagonal_a: bool = false;
        let mut repair_diagonal_b: bool = false;

        for input_label in &self.input_label_set_intersection {
            match input_label {
                InputLabel::InputSymmetry { label } => {
                    match label {
                        SymmetryLabel::HorizontalWithMismatches => {
                            repair_horizontal = true;
                        },
                        SymmetryLabel::HorizontalWithInsetAndMismatches => {
                            repair_horizontal = true;
                        },
                        SymmetryLabel::VerticalWithMismatches => {
                            repair_vertical = true;
                        },
                        SymmetryLabel::VerticalWithInsetAndMismatches => {
                            repair_vertical = true;
                        },
                        SymmetryLabel::DiagonalAWithMismatches => {
                            repair_diagonal_a = true;
                        },
                        SymmetryLabel::DiagonalBWithMismatches => {
                            repair_diagonal_b = true;
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        let both_horz_and_vert: bool = repair_horizontal && repair_vertical;
        let attempt_repair: bool = both_horz_and_vert || repair_diagonal_a || repair_diagonal_b;
        if !attempt_repair {
            return Ok(());
        }
        match self.compute_input_repaired_image_execute() {
            Ok(()) => {},
            Err(_) => {
                // could not repair, perhaps the image isn't symmetric.
                self.reset_input_repaired_image();
            }
        }
        Ok(())
    }

    fn reset_input_repaired_image(&mut self) {
        for (_index, pair) in self.pairs.iter_mut().enumerate() {
            pair.input.repaired_image = None;
        }
    }

    fn compute_input_repaired_image_execute(&mut self) -> anyhow::Result<()> {
        for (_index, pair) in self.pairs.iter_mut().enumerate() {
            let (symmetry, repair_mask) = match (&pair.input.symmetry, &pair.input.repair_mask) {
                (Some(a), Some(b)) => (a, b),
                _ => {
                    return Err(anyhow::anyhow!("symmetry and repair_mask"));
                }
            };
            let image_to_repair: &Image = &pair.input.image;
            let repaired_image: Image = AutoRepairSymmetry::execute(&symmetry, &repair_mask, image_to_repair)?;
            pair.input.repaired_image = Some(repaired_image);
        }
        Ok(())
    }

    /// Set `grid_pattern=None` for all pairs.
    fn reset_input_grid_pattern(&mut self) {
        for (_index, pair) in self.pairs.iter_mut().enumerate() {
            pair.input.grid_pattern = None;
        }
    }

    fn compute_input_grid_pattern(&mut self) -> anyhow::Result<()> {
        let mut prio1_grid_with_specific_color = false;
        let mut prio1_grid_color: u8 = u8::MAX;
        let mut prio2_grid_with_some_color = false;
        let mut prio3_grid_with_mismatches_and_specific_color = false;
        let mut prio3_grid_count: usize = 0;
        let mut prio3_grid_color: u8 = u8::MAX;

        for input_label in &self.input_label_set_intersection {
            match input_label {
                InputLabel::InputGrid { label } => {
                    match label {
                        GridLabel::GridColor { color } => {
                            prio1_grid_with_specific_color = true;
                            prio1_grid_color = *color;
                        },
                        GridLabel::GridWithSomeColor => {
                            prio2_grid_with_some_color = true;
                        },
                        GridLabel::GridWithMismatchesAndColor { color } => {
                            if prio3_grid_count == 0 {
                                prio3_grid_with_mismatches_and_specific_color = true;
                                prio3_grid_color = *color;
                                prio3_grid_count += 1;
                            } else {
                                // There are multiple grid colors to choose from.
                                // It's ambiguous what color to choose for this grid. 
                                // It happens for 3 task out of 800 tasks: 97239e3d, d37a1ef5, e681b708.
                                // Ignore this case entirely grid.
                                prio3_grid_with_mismatches_and_specific_color = false;
                                // println!("ambiguous what color to choose. task: {}", self.id);
                                prio3_grid_count += 1;
                            }
                        },
                        _ => {},
                    }
                },
                _ => {}
            }
        }

        if prio1_grid_with_specific_color {
            let grid_color: u8 = prio1_grid_color;
            let mut success = true;
            for pair in self.pairs.iter_mut() {
                let grid = match &pair.input.grid {
                    Some(value) => value.clone(),
                    None => {
                        // One or more of the grids are not initialized, aborting.
                        success = false;
                        break;
                    }
                };
                let pattern: GridPattern = match grid.find_full_pattern_with_color(grid_color) {
                    Some(value) => value.clone(),
                    None => {
                        // Could not find a pattern with that particular color, aborting.
                        success = false;
                        break;
                    }
                };
                pair.input.grid_pattern = Some(pattern);
            }
            if success {
                // This case is hit for 51 task out of the 800 tasks.
                // 09629e4f, 0b148d64, 11e1fe23, 1bfc4729, 1c0d0a4b, 29623171, 29c11459, 3906de3d, 3aa6fb7a, 3bdb4ada, 42918530, 
                // 48d8fb45, 4e45f183, 4f537728, 60b61512, 6773b310, 68b67ca3, 692cd3b6, 694f12f3, 6d0160f0, 6e19193c, 759f3fd3, 
                // 77fdfe62, 7c008303, 7d419a02, 88a62173, 8a371977, a096bf4d, a2fd1cf0, a68b268e, af24b4cc, b7249182, b7f8a4d8, 
                // bc1d5164, be03b35f, cbded52d, ce9e57f2, d22278a0, d4a91cb9, d6ad076f, d90796e8, d94c3b52, dc2aa30b, dc433765, 
                // e760a62e, e9614598, e99362f0, ed74f2f2, ef26cbf6, f8b3ba0a, fea12743.
                return Ok(());
            }
            self.reset_input_grid_pattern();
        }

        if prio2_grid_with_some_color {
            let mut success = true;
            for pair in self.pairs.iter_mut() {
                let grid = match &pair.input.grid {
                    Some(value) => value.clone(),
                    None => {
                        // One or more of the grids are not initialized, aborting.
                        success = false;
                        break;
                    }
                };
                let grid_color: u8 = grid.grid_color();

                let pattern: GridPattern = match grid.find_full_pattern_with_color(grid_color) {
                    Some(value) => value.clone(),
                    None => {
                        // Could not find a pattern with that particular color, aborting.
                        success = false;
                        break;
                    }
                };
                pair.input.grid_pattern = Some(pattern);
            }
            if success {
                // This case is hit for 14 task out of the 800 tasks.
                // 06df4c85, 0bb8deee, 1e32b0e9, 2546ccf6, 2dc579da, 39e1d7f9, 47c1f68c, 
                // 5a5a2103, 81c0276b, 92e50de0, 9f236235, 9f27f097, c3202e5a, e48d4e1a.
                return Ok(());
            }
            self.reset_input_grid_pattern();
        }

        if prio3_grid_with_mismatches_and_specific_color {
            let grid_color: u8 = prio3_grid_color;
            let mut success = true;
            for pair in self.pairs.iter_mut() {
                let grid = match &pair.input.grid {
                    Some(value) => value.clone(),
                    None => {
                        // One or more of the grids are not initialized, aborting.
                        success = false;
                        break;
                    }
                };
                let pattern: GridPattern = match grid.find_partial_pattern_with_color(grid_color) {
                    Some(value) => value.clone(),
                    None => {
                        // Could not find a pattern with that particular color, aborting.
                        success = false;
                        break;
                    }
                };
                pair.input.grid_pattern = Some(pattern);
            }
            if success {
                // This case is hit for 3 task out of the 800 tasks. 
                // 15113be4, 95a58926, 97239e3d.
                return Ok(());
            }
            self.reset_input_grid_pattern();
        }

        // This case is hit for 728 task out of the 800 tasks.
        Ok(())
    }

    fn compute_input_enumerated_objects(&mut self) -> anyhow::Result<()> {
        if self.has_enumerated_objects() {
            return Ok(());
        }

        // Don't care wether it succeeds or fails
        _ = self.compute_input_enumerated_objects_based_on_same_structure();

        // Reset all enumerated_objects if one or more is missing.
        if !self.has_enumerated_objects() {
            self.reset_input_enumerated_objects();
        }

        // Don't care wether it succeeds or fails
        _ = self.compute_input_enumerated_objects_using_grid();

        // Reset all enumerated_objects if one or more is missing.
        if !self.has_enumerated_objects() {
            self.reset_input_enumerated_objects();
        }

        // Don't care wether it succeeds or fails
        _ = self.compute_input_enumerated_objects_based_on_object_label();

        // Reset all enumerated_objects if one or more is missing.
        if !self.has_enumerated_objects() {
            self.reset_input_enumerated_objects();
        }

        // Don't care wether it succeeds or fails
        _ = self.compute_input_enumerated_objects_based_on_size_of_primary_object_after_single_intersection_color();
        
        // Reset all enumerated_objects if one or more is missing.
        if !self.has_enumerated_objects() {
            self.reset_input_enumerated_objects();
        }
        Ok(())
    }

    /// Set `enumerated_objects=None` for all pairs.
    fn reset_input_enumerated_objects(&mut self) {
        for pair in self.pairs.iter_mut() {
            pair.input.enumerated_objects = None;
        }
    }

    fn compute_input_enumerated_objects_based_on_same_structure(&mut self) -> anyhow::Result<()> {
        if self.has_enumerated_objects() {
            return Ok(());
        }

        if !self.action_label_set_intersection.contains(&ActionLabel::OutputImageHasSameStructureAsInputImage) {
            return Ok(());
        }

        let mut ambiguity_count0: usize = 0;
        let mut ambiguity_count1: usize = 0;
        let mut found_color0 = u8::MAX;
        let mut found_color1 = u8::MAX;
        for action_label in &self.action_label_set_intersection {
            match action_label {
                ActionLabel::OutputImageIsInputImageWithNoChangesToPixelsWithColor { color } => {
                    ambiguity_count0 += 1;
                    found_color0 = *color;
                },
                ActionLabel::InputImageIsOutputImageWithNoChangesToPixelsWithColor { color } => {
                    ambiguity_count1 += 1;
                    found_color1 = *color;
                },
                _ => {}
            }
        }
        if ambiguity_count0 != 1 || ambiguity_count1 != 1 {
            return Ok(());
        }
        if found_color0 != found_color1 {
            return Ok(());
        }
        // Agreement on a single color
        let background_color: u8 = found_color0;

        for pair in &mut self.pairs {
            let image_mask: Image = pair.input.image.to_mask_where_color_is_different(background_color);
            let ignore_mask: Image = image_mask.to_mask_where_color_is(0);

            let result = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &image_mask, &ignore_mask);
            let object_images: Vec<Image> = match result {
                Ok(images) => images,
                Err(_) => {
                    continue;
                }
            };
            let object_images_sorted: Vec<Image> = ObjectsSortByProperty::sort_by_mass_descending(&object_images)?;
            let enumerated_objects: Image = Image::object_enumerate(&object_images_sorted)?;
            pair.input.enumerated_objects = Some(enumerated_objects);
        }

        Ok(())
    }

    fn compute_input_enumerated_objects_based_on_size_of_primary_object_after_single_intersection_color(&mut self) -> anyhow::Result<()> {
        if self.has_enumerated_objects() {
            return Ok(());
        }

        let mut simple_explanation_for_width: bool = false;
        let mut simple_explanation_for_height: bool = false;
        let mut depends_on_object_width: bool = false;
        let mut depends_on_object_height: bool = false;

        for action in &self.action_label_set_intersection {
            match *action {
                ActionLabel::OutputPropertyIsEqualToInputProperty { output: _, input } => {
                    match input {
                        PropertyInput::InputWidth => {
                            simple_explanation_for_width = true;
                        },
                        PropertyInput::InputHeight => {
                            simple_explanation_for_height = true;
                        },
                        PropertyInput::InputWidthOfPrimaryObjectAfterSingleIntersectionColor => {
                            depends_on_object_width = true;
                        },
                        PropertyInput::InputHeightOfPrimaryObjectAfterSingleIntersectionColor => {
                            depends_on_object_height = true;
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if simple_explanation_for_width && simple_explanation_for_height {
            // println!("there exist a simple solution. No need to do advanced object stuff.");
            return Ok(());
        }

        let depends_on_object_size: bool = depends_on_object_width || depends_on_object_height;
        if !depends_on_object_size {
            // println!("no dependency on the object size. No need to do advanced object stuff.");
            return Ok(());
        }

        // println!("There exist a dependency on the biggest object.");

        let removal_pairs: Vec<(u32,u8)> = self.input_histogram_intersection.pairs_descending();
        if removal_pairs.len() != 1 {
            return Ok(());
        }
        let background_color: u8 = match removal_pairs.first() {
            Some((_count, color)) => *color,
            None => {
                return Ok(());
            }
        };

        // println!("preconditions are satisfied. will sort objects by mass");

        for pair in &mut self.pairs {
            let image_mask: Image = pair.input.image.to_mask_where_color_is_different(background_color);
            let ignore_mask: Image = image_mask.to_mask_where_color_is(0);

            let result = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &image_mask, &ignore_mask);
            let object_images: Vec<Image> = match result {
                Ok(images) => images,
                Err(_) => {
                    continue;
                }
            };
            let object_images_sorted: Vec<Image> = ObjectsSortByProperty::sort_by_mass_descending(&object_images)?;
            let enumerated_objects: Image = Image::object_enumerate(&object_images_sorted)?;
            pair.input.enumerated_objects = Some(enumerated_objects);
        }

        Ok(())
    }

    fn compute_input_enumerated_objects_based_on_object_label(&mut self) -> anyhow::Result<()> {
        if self.has_enumerated_objects() {
            return Ok(());
        }

        let mut find_smallest_area: bool = false;
        let mut find_biggest_area: bool = false;
        let mut find_symmetry_x: bool = false;
        let mut find_symmetry_y: bool = false;
        let mut find_asymmetry_x: bool = false;
        let mut find_asymmetry_y: bool = false;

        for action in &self.action_label_set_intersection {
            match action {
                ActionLabel::OutputImageIsTheObjectWithObjectLabel { object_label } => {
                    match object_label {
                        ObjectLabel::TheOnlyOneWithSmallestArea => {
                            find_smallest_area = true;
                        },
                        ObjectLabel::TheOnlyOneWithBiggestArea => {
                            find_biggest_area = true;
                        },
                        ObjectLabel::TheOnlyOneWithSymmetryX => {
                            find_symmetry_x = true;
                        },
                        ObjectLabel::TheOnlyOneWithSymmetryY => {
                            find_symmetry_y = true;
                        },
                        ObjectLabel::TheOnlyOneWithAsymmetryX => {
                            find_asymmetry_x = true;
                        },
                        ObjectLabel::TheOnlyOneWithAsymmetryY => {
                            find_asymmetry_y = true;
                        },
                    }
                },
                _ => {}
            }
        }

        // Check that only 1 find_xyz is true
        let values = [
            find_smallest_area,
            find_biggest_area,
            find_symmetry_x,
            find_symmetry_y,
            find_asymmetry_x,
            find_asymmetry_y,
        ];
        let mut find_count: usize = 0;
        for value in values {
            if value {
                find_count += 1;
            }
        }
        if find_count == 0 {
            // output image does not depend on object in input image
            return Ok(());
        }
        if find_count > 1 {
            return Err(anyhow::anyhow!("output image depends on object. But multiple of the find_xyz are true. Ambiguous which one to pick"));
        }

        for pair in &mut self.pairs {
            let object_mask_vec: Vec<Image> = pair.input.find_object_masks_using_histogram_most_popular_color()?;
            let mut object_vec: Vec<Object> = pair.input.find_objects_using_histogram_most_popular_color()?;
            Object::assign_labels_to_objects(&mut object_vec);
            if object_mask_vec.len() != object_vec.len() {
                return Err(anyhow::anyhow!("object_mask_vec.len() and object_vec.len() are supposed to have same length"));
            }

            let mut found_index: Option<usize> = None;
            for object in &object_vec {
                for object_label in &object.object_label_set {
                    match object_label {
                        ObjectLabel::TheOnlyOneWithSmallestArea => {
                            if find_smallest_area {
                                found_index = Some(object.index);
                            }
                        },
                        ObjectLabel::TheOnlyOneWithBiggestArea => {
                            if find_biggest_area {
                                found_index = Some(object.index);
                            }
                        },
                        ObjectLabel::TheOnlyOneWithSymmetryX => {
                            if find_symmetry_x {
                                found_index = Some(object.index);
                            }
                        },
                        ObjectLabel::TheOnlyOneWithSymmetryY => {
                            if find_symmetry_y {
                                found_index = Some(object.index);
                            }
                        },
                        ObjectLabel::TheOnlyOneWithAsymmetryX => {
                            if find_asymmetry_x {
                                found_index = Some(object.index);
                            }
                        },
                        ObjectLabel::TheOnlyOneWithAsymmetryY => {
                            if find_asymmetry_y {
                                found_index = Some(object.index);
                            }
                        },
                    }
                }
            }
            // println!("found object: {:?}", found_index);

            let index: usize = match found_index {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Did not find any object with the objectlabel"));
                }
            };
            if index >= object_mask_vec.len() {
                return Err(anyhow::anyhow!("index is out of bounds"));
            }
            let mask: &Image = &object_mask_vec[index];
            pair.input.enumerated_objects = Some(mask.clone());
        }

        Ok(())
    }

    fn compute_input_enumerated_objects_using_grid(&mut self) -> anyhow::Result<()> {
        if self.has_enumerated_objects() {
            return Ok(());
        }
        if !self.has_grid_pattern() {
            return Ok(());
        }

        for pair in self.pairs.iter_mut() {
            let grid = match &pair.input.grid_pattern {
                Some(value) => value.clone(),
                None => {
                    // One or more of the grid_patterns are not initialized, aborting.
                    return Err(anyhow::anyhow!("One or more grid_patters are not initialized"));
                }
            };

            let mask: &Image = &grid.line_mask;
            let blank: Image = Image::zero(mask.width(), mask.height());
            let cells: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity4, &blank, mask)?;
            if cells.is_empty() {
                return Err(anyhow::anyhow!("Expected grid to have 1 or more cells"));
            }
            let enumerated_objects: Image = Image::object_enumerate(&cells).expect("image");

            pair.input.enumerated_objects = Some(enumerated_objects);
        }

        Ok(())
    }

    pub fn inspect(&self) -> anyhow::Result<()> {
        HtmlFromTask::inspect(self)
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

    #[allow(dead_code)]
    pub fn is_output_size_same_as_input_size(&self) -> bool {
        ActionLabelUtil::is_output_size_same_as_input_size(&self.action_label_set_intersection)
    }

    #[allow(dead_code)]
    pub fn is_output_size_same_as_removed_rectangle_after_single_color_removal(&self) -> bool {
        ActionLabelUtil::is_output_size_same_as_removed_rectangle_after_single_color_removal(&self.action_label_set_intersection)
    }
}
