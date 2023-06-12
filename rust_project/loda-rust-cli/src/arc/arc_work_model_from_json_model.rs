use super::arc_json_model;
use super::arc_work_model;
use super::arc_work_model::ImageMeta;
use super::{Image, Histogram, ImageHistogram, ActionLabelSet};
use std::collections::{HashMap, HashSet};

impl TryFrom<&arc_json_model::Task> for arc_work_model::Task {
    type Error = anyhow::Error;

    fn try_from(json_task: &arc_json_model::Task) -> Result<Self, Self::Error> {
        let task_id: String = json_task.id().identifier();
        let mut result_pairs: Vec<arc_work_model::Pair> = vec!();

        let mut input_histogram_union: Histogram = Histogram::new();
        let mut input_histogram_intersection: Histogram = Histogram::new();
        let mut output_histogram_union: Histogram = Histogram::new();
        let mut output_histogram_intersection: Histogram = Histogram::new();
        let mut removal_histogram_intersection: Histogram = Histogram::new();
        let mut insert_histogram_intersection: Histogram = Histogram::new();
        {
            let pairs: Vec<arc_json_model::ImagePair> = json_task.images_train()?;
            for (index, pair) in pairs.iter().enumerate() {
                let histogram_input: Histogram = pair.input.histogram_all();
                let histogram_output: Histogram = pair.output.histogram_all();

                let mut histogram_removal: Histogram = histogram_input.clone();
                histogram_removal.subtract_histogram(&histogram_output);

                let mut histogram_insert: Histogram = histogram_output.clone();
                histogram_insert.subtract_histogram(&histogram_input);

                input_histogram_union.add_histogram(&histogram_input);
                output_histogram_union.add_histogram(&histogram_output);
                if index == 0 {
                    input_histogram_intersection = histogram_input.clone();
                    output_histogram_intersection = histogram_output.clone();
                    removal_histogram_intersection = histogram_removal.clone();
                    insert_histogram_intersection = histogram_insert.clone();
                } else {
                    input_histogram_intersection.intersection_histogram(&histogram_input);
                    output_histogram_intersection.intersection_histogram(&histogram_output);
                    removal_histogram_intersection.intersection_histogram(&histogram_removal);
                    insert_histogram_intersection.intersection_histogram(&histogram_insert);
                }
                let buffer_input = arc_work_model::Input {
                    id: format!("{},input{},train", task_id, index),
                    image: pair.input.clone(),
                    image_meta: ImageMeta::new(),
                    input_objects: HashMap::new(),
                    repair_mask: None,
                    repaired_image: None,
                    grid_pattern: None,
                    enumerated_objects: None,
                    substitution_rule_applied: None,
                    predicted_single_color_image: None,
                    removal_color: None,
                    most_popular_intersection_color: None,
                    single_pixel_noise_color: None,
                };
                let buffer_output = arc_work_model::Output {
                    id: format!("{},output{},train", task_id, index),
                    image: pair.output.clone(),
                    test_image: Image::empty(),
                    image_meta: ImageMeta::new(),
                };
                let result_pair = arc_work_model::Pair {
                    id: format!("{},pair{},train", task_id, index),
                    pair_type: arc_work_model::PairType::Train,
                    input: buffer_input,
                    output: buffer_output,
                    removal_histogram: histogram_removal,
                    insert_histogram: histogram_insert,
                    action_label_set: ActionLabelSet::new(),
                    prediction_set: arc_work_model::PredictionSet::new(),
                    output_specification_vec: vec!(),
                    input_output_image_label_set_intersection: HashSet::new(),
                    input_output_image_properties: HashMap::new(),
                };
                result_pairs.push(result_pair);
            }
        }
        {
            let pairs: Vec<arc_json_model::ImagePair> = json_task.images_test()?;
            for (index, pair) in pairs.iter().enumerate() {
                let buffer_input = arc_work_model::Input {
                    id: format!("{},input{},test", task_id, index),
                    image: pair.input.clone(),
                    image_meta: ImageMeta::new(),
                    input_objects: HashMap::new(),
                    repair_mask: None,
                    repaired_image: None,
                    grid_pattern: None,
                    enumerated_objects: None,
                    substitution_rule_applied: None,
                    predicted_single_color_image: None,
                    removal_color: None,
                    most_popular_intersection_color: None,
                    single_pixel_noise_color: None,
                };
                let buffer_output = arc_work_model::Output {
                    id: format!("{},output{},test", task_id, index),
                    image: Image::empty(),
                    test_image: pair.output.clone(),
                    image_meta: ImageMeta::new(),
                };
                let result_pair = arc_work_model::Pair {
                    id: format!("{},pair{},test", task_id, index),
                    pair_type: arc_work_model::PairType::Test,
                    input: buffer_input,
                    output: buffer_output,
                    removal_histogram: Histogram::new(),
                    insert_histogram: Histogram::new(),
                    action_label_set: ActionLabelSet::new(),
                    prediction_set: arc_work_model::PredictionSet::new(),
                    output_specification_vec: vec!(),
                    input_output_image_label_set_intersection: HashSet::new(),
                    input_output_image_properties: HashMap::new(),
                };
                result_pairs.push(result_pair);
            }
        }
    
        // Copies over the counters from the histogram_union to the histogram_intersection
        input_histogram_intersection.clamp01();
        input_histogram_intersection.multiply_histogram(&input_histogram_union);
        output_histogram_intersection.clamp01();
        output_histogram_intersection.multiply_histogram(&output_histogram_union);

        let mut task = arc_work_model::Task {
            id: task_id,
            pairs: result_pairs,
            input_histogram_union,
            input_histogram_intersection,
            output_histogram_union,
            output_histogram_intersection,
            removal_histogram_intersection,
            insert_histogram_intersection,
            input_properties_intersection: HashMap::new(),
            input_output_properties_intersection: HashMap::new(),
            action_label_set_intersection: ActionLabelSet::new(),
            input_image_label_set_intersection: HashSet::new(),
            output_image_label_set_intersection: HashSet::new(),
            input_output_image_label_set_intersection: HashSet::new(),
            occur_in_solutions_csv: false,
        };
        task.populate()?;
        Ok(task)
    }
}
