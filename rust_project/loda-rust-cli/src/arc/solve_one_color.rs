//! Solve tasks that outputs a single color.
//! 
//! Example:
//! d631b094
use super::arc_json_model::GridFromImage;
use super::arc_work_model::{Task, PairType, Pair};
use super::{Image, ImageOverlay, arcathon_solution_coordinator, arc_json_model, ImageMix, MixMode, ObjectsAndMass, ImageCrop, Rectangle, ImageExtractRowColumn, ImageDenoise, TaskGraph, ShapeType, ImageSize, ShapeTransformation, SingleColorObject, ShapeIdentificationFromSingleColorObject, ImageDetectHole, ImagePadding, ImageRepairPattern, TaskNameToPredictionVec, CreateTaskWithSameSize, ImageReplaceColor, ImageCenterIndicator, ImageGravity, GravityDirection, DiagonalHistogram, RecordTrigram, ImageNgram, ImageExteriorCorners, LargestInteriorRectangle, ImageDrawRect, PropertyOutput, ImageProperty, ImageResize, ImageRepeat, rule, CellularAutomaton, ChangeItem};
use super::{ActionLabel, ImageLabel, ImageMaskDistance, LineSpan, LineSpanDirection, LineSpanMode, VerifyPrediction, VerifyPredictionWithTask};
use super::{HtmlLog, PixelConnectivity, ImageHistogram, Histogram, ImageEdge, ImageMask};
use super::{ImageNeighbour, ImageNeighbourDirection, ImageCornerAnalyze, ImageMaskGrow, Shape3x3};
use super::human_readable_utc_timestamp;
use anyhow::{Context, bail};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

const PROCESS_TASK_VARIANTS: [u8; 1] = [0];

#[derive(Clone, Debug)]
enum ProcessTaskMode {
    InputOutputSameSize,
    InputOutputDifferentSize,
}

#[derive(Clone, Debug)]
pub struct ProcessTaskContext {
    variant: u8,
    mode: ProcessTaskMode,
    input_size_vec: Vec<ImageSize>,
    output_size_vec: Vec<ImageSize>,
    scale_widthheight: Option<(u8, u8)>,
}

impl ProcessTaskContext {
    pub fn new(task: &Task, variant: u8) -> Self {
        let mode: ProcessTaskMode = if task.is_output_size_same_as_input_size() { 
            ProcessTaskMode::InputOutputSameSize 
        } else { 
            ProcessTaskMode::InputOutputDifferentSize 
        };
        let mut instance = Self {
            variant,
            mode,
            input_size_vec: Vec::<ImageSize>::new(),
            output_size_vec: Vec::<ImageSize>::new(),
            scale_widthheight: None,
        };
        instance.populate_input_size_vec(task);
        instance.populate_output_size_vec(task);
        instance.populate_scale_factor(task);
        instance
    }

    fn populate_input_size_vec(&mut self, task: &Task) {
        self.input_size_vec.clear();
        for pair in &task.pairs {
            let size: ImageSize = pair.input.image.size();
            self.input_size_vec.push(size);
        }
    }

    fn populate_output_size_vec(&mut self, task: &Task) {
        self.output_size_vec.clear();
        for pair in &task.pairs {
            match pair.pair_type {
                PairType::Train => {
                    let size: ImageSize = pair.output.image.size();
                    self.output_size_vec.push(size);
                },
                PairType::Test => {
                    let mut the_size: ImageSize = pair.output.test_image.size();
                    if let Some(size) = pair.predicted_output_size() {
                        the_size = size;
                    }
                    self.output_size_vec.push(the_size);
                }
            }
        }
    }

    fn populate_scale_factor(&mut self, task: &Task) {
        let mut scale_width_factor: Option<u8> = None;
        let mut scale_height_factor: Option<u8> = None;
        for action_label in &task.action_label_set_intersection {
            match action_label {
                ActionLabel::OutputPropertyIsInputPropertyMultipliedBy { output, input, scale } => {
                    match (output, input) {
                        (PropertyOutput::OutputWidth, ImageProperty::Width) => {
                            scale_width_factor = Some(*scale);
                        },
                        (PropertyOutput::OutputHeight, ImageProperty::Height) => {
                            scale_height_factor = Some(*scale);
                        },
                        _ => {}
                    }
                },
                ActionLabel::OutputPropertyIsEqualToInputProperty { output, input } => {
                    match (output, input) {
                        (PropertyOutput::OutputWidth, ImageProperty::Width) => {
                            scale_width_factor = Some(1);
                        },
                        (PropertyOutput::OutputHeight, ImageProperty::Height) => {
                            scale_height_factor = Some(1);
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        if let Some(scale_x) = scale_width_factor {
            if let Some(scale_y) = scale_height_factor {
                if scale_x > 0 && scale_y > 0 && scale_x <= 6 && scale_y <= 6 {
                    if scale_x != 1 && scale_y != 1 {
                        self.scale_widthheight = Some((scale_x, scale_y));
                    }
                }
            }
        }
    }
}

pub struct SolveOneColor {
    tasks: Vec<Task>,
}

impl SolveOneColor {
    pub fn new(tasks: Vec<Task>) -> Self {
        // println!("loaded {} tasks", tasks.len());
        Self {
            tasks,
        }
    }

    /// Checks that the predicted output is the same as the expected output.
    /// 
    /// This can be run with the public ARC dataset contains expected output for the test pairs.
    /// 
    /// This cannot be run with the hidden ARC dataset, which doesn't contain expected output for the test pairs.
    pub fn run_and_verify(&self) -> anyhow::Result<()> {
        let run_and_verify_htmllog = true;
        let number_of_tasks: u64 = self.tasks.len() as u64;
        println!("{} - run start - will process {} tasks with SolveOneColor", human_readable_utc_timestamp(), number_of_tasks);
        let count_solved_full = AtomicUsize::new(0);
        let count_solved_partial = AtomicUsize::new(0);
        let pb = ProgressBar::new(number_of_tasks as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-")
        );
        
        self.tasks.par_iter().for_each(|task| {
            pb.inc(1);

            // Only process tasks where all pairs agree that the output images have just one color.
            if !Self::all_pairs_have_one_output_color(task) {
                return;
            }
            HtmlLog::text(format!("task {}", task.id));

            // Only process tasks where the task has a predicted size.
            // if the task doesn't have a predicted size, then skip it.

            let task_count_test: usize = task.count_test();

            // Make predictions
            let processed_task: ProcessedTask = match Self::process_task(task) {
                Ok(value) => value,
                Err(error) => {
                    pb.println(format!("task {} - error: {:?}", task.id, error));
                    return;
                }
            };

            // Verify predictions
            let mut correct_vec = Vec::<bool>::new();
            let mut test_index_to_correct_count = HashMap::<usize, usize>::new();
            for prediction in &processed_task.prediction_vec {
                let mut is_correct = false;
                match prediction.verify_prediction(task) {
                    Ok(verify_prediction) => {
                        match verify_prediction {
                            VerifyPrediction::Correct => {
                                is_correct = true;

                                // Count the number of times each test pair is solved correctly.
                                test_index_to_correct_count.entry(prediction.output_id as usize).and_modify(|e| *e += 1).or_insert(1);
                            },
                            _ => {}
                        }
                    },
                    Err(error) => {
                        pb.println(format!("task: {} - output_id: {} - verify_prediction - error: {:?}", task.id, prediction.output_id, error));
                    }
                }
                correct_vec.push(is_correct);
            }

            let mut fully_solved_test_pairs = true;
            let mut number_of_solved_test_pairs: usize = 0;
            for i in 0..task_count_test {
                let count: usize = match test_index_to_correct_count.get(&i) {
                    Some(value) => *value,
                    None => {
                        fully_solved_test_pairs = false;
                        continue;
                    }
                };
                if count == 0 {
                    fully_solved_test_pairs = false;
                }
                if count >= 1 {
                    number_of_solved_test_pairs += 1;
                }
            }

            if fully_solved_test_pairs {
                count_solved_full.fetch_add(1, Ordering::Relaxed);
                pb.println(format!("task {} - solved full, {} test pairs", task.id, number_of_solved_test_pairs));
                HtmlLog::text(format!("task {} - solved full, {} test pairs", task.id, number_of_solved_test_pairs));
            } else {
                if number_of_solved_test_pairs >= 1 {
                    count_solved_partial.fetch_add(1, Ordering::Relaxed);
                    pb.println(format!("task {} - solved partial, {} correct of {} test pairs", task.id, number_of_solved_test_pairs, task_count_test));
                    HtmlLog::text(format!("task {} - solved partial, {} correct of {} test pairs", task.id, number_of_solved_test_pairs, task_count_test));
                }
            }
            let count_full: usize = count_solved_full.load(Ordering::Relaxed);
            let count_partial: usize = count_solved_partial.load(Ordering::Relaxed);
            pb.set_message(format!("Solved full: {}, partial: {}", count_full, count_partial));

            // Display the internal computed image to the html log
            if run_and_verify_htmllog {
                for (index, ptwotp) in processed_task.ptwotp_vec.iter().enumerate() {
                    let is_correct: bool = correct_vec[index];
                    if is_correct {
                        HtmlLog::text(format!("{} - test_index: {} - correct", task.id, ptwotp.test_index));
                        HtmlLog::image(&ptwotp.cropped_image);
                    } else {
                        HtmlLog::text(format!("{} - test_index: {} - incorrect", task.id, ptwotp.test_index));
                        let pair: &Pair = match task.pair_for_test_index(ptwotp.test_index) {
                            Ok(pair) => pair,
                            Err(error) => {
                                pb.println(format!("{} - error: {:?}", task.id, error));
                                continue;
                            }
                        };
                        let images: Vec<Image> = vec![
                            pair.input.image.clone(),
                            pair.output.test_image.clone(),
                            ptwotp.cropped_image.clone(),
                        ];
                        HtmlLog::compare_images(images);
                    }
                }
            }
        });
        pb.finish_and_clear();
        let count_full: usize = count_solved_full.load(Ordering::Relaxed);
        let count_partial: usize = count_solved_partial.load(Ordering::Relaxed);
        println!("{} - run - end", human_readable_utc_timestamp());
        println!("{} - out of {} tasks, fully solved {} and partially solved {}", human_readable_utc_timestamp(), number_of_tasks, count_full, count_partial);
        Ok(())
    }

    fn process_task(task: &Task) -> anyhow::Result<ProcessedTask> {
        let mut accumulated_processed_task = ProcessedTask {
            ptwotp_vec: vec!(),
            prediction_vec: vec!(),
        };

        for variant in &PROCESS_TASK_VARIANTS {
            let processed_task: ProcessedTask = Self::process_task_item(task, *variant)
                .with_context(|| format!("task: {} Unable to process_task_item() with variant: {}", task.id, variant))?;

            accumulated_processed_task.ptwotp_vec.extend(processed_task.ptwotp_vec);
            accumulated_processed_task.prediction_vec.extend(processed_task.prediction_vec);
        }
        if accumulated_processed_task.prediction_vec.is_empty() || accumulated_processed_task.ptwotp_vec.is_empty() {
            return Err(anyhow::anyhow!("task: {} prediction_vec.is_empty() or ptwotp_vec.is_empty(). It's supposed to be non-empty.", task.id));
        }
        Ok(accumulated_processed_task)
    }

    fn process_task_item(task: &Task, variant: u8) -> anyhow::Result<ProcessedTask> {
        let count_test: u8 = task.count_test().min(255) as u8;
        if count_test < 1 {
            return Err(anyhow::anyhow!("skipping task: {} because it has no test pairs", task.id));
        }    

        let context = ProcessTaskContext::new(task, variant);

        let task_for_processing: Task = task.clone();
        let prediction_type: arcathon_solution_coordinator::PredictionType;
        if task.is_output_size_same_as_input_size() {
            prediction_type = arcathon_solution_coordinator::PredictionType::SolveLogisticRegressionSameSize;
        } else {
            prediction_type = arcathon_solution_coordinator::PredictionType::SolveLogisticRegressionDifferentSize;
        }

        let mut ptwotp_vec = Vec::<ProcessedTaskWithOneTestPair>::new();
        for test_index in 0..count_test {
            let ptwotp: ProcessedTaskWithOneTestPair = match Self::process_task_with_one_test_pair(&context, &task_for_processing, test_index) {
                Ok(value) => value,
                Err(error) => {
                    return Err(error);
                }
            };
            ptwotp_vec.push(ptwotp);
        }

        let mut prediction_vec = Vec::<arcathon_solution_coordinator::Prediction>::new();
        for (test_index, ptwotp) in ptwotp_vec.iter().enumerate() {
            let grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&ptwotp.cropped_image);
            let prediction = arcathon_solution_coordinator::Prediction {
                output_id: test_index.min(255) as u8,
                output: grid,
                prediction_type,
            };
            prediction_vec.push(prediction);
        }
    
        if prediction_vec.len() != (count_test as usize) {
            return Err(anyhow::anyhow!("task: {} predictions.len() != task.count_test()", task.id));
        }
        let instance = ProcessedTask {
            ptwotp_vec,
            prediction_vec,
        };
        Ok(instance)
    }

    fn process_task_with_one_test_pair(context: &ProcessTaskContext, task: &Task, test_index: u8) -> anyhow::Result<ProcessedTaskWithOneTestPair> {
        if context.input_size_vec.len() != task.pairs.len() {
            return Err(anyhow::anyhow!("context.output_size_vec.len() != task.pairs.len()"));
        }
        if context.output_size_vec.len() != task.pairs.len() {
            return Err(anyhow::anyhow!("context.output_size_vec.len() != task.pairs.len()"));
        }

        // Obtain `pair_index` from `test_index`.
        let mut found_pair_index: Option<u8> = None;
        for pair in &task.pairs {
            if pair.test_index == Some(test_index) {
                found_pair_index = Some(pair.pair_index);
                break;
            }
        }
        let pair_index: u8 = match found_pair_index {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Unable to find pair with test_index: {}", test_index));
            }
        };

        let mut available_colors = Histogram::new();
        for color in 0..=9u8 {
            available_colors.increment(color);
        }

        if Self::output_image_colors_comes_from_input_image(task) {
            for pair in &task.pairs {
                if pair.test_index == Some(test_index) {
                    available_colors = pair.input.image_meta.histogram_all.clone();
                }
            }
        }

        // If all pairs agree on the same removal colors, then make sure none of these are present in the available colors.
        available_colors.subtract_histogram(&task.removal_histogram_intersection);

        if task.insert_histogram_intersection.number_of_counters_greater_than_zero() > 0 {
            available_colors = task.insert_histogram_intersection.clone();
        }

        // The most popular color specific for each pair, is used for the output color.
        if let Some(color) = Self::pair_input_most_popular_color(task, pair_index) {
            available_colors = Histogram::new();
            available_colors.increment(color);
        }

        // The least popular color specific for each pair, is used for the output color.
        if let Some(color) = Self::pair_input_least_popular_color(task, pair_index) {
            available_colors = Histogram::new();
            available_colors.increment(color);
        }

        // All pairs agree on the exact same color.
        if task.output_histogram_intersection == task.output_histogram_union {
            available_colors = task.output_histogram_intersection.clone();
        }

        // HtmlLog::text(format!("task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, available_colors.pairs_descending()));
        HtmlLog::text(format!("task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, available_colors.pairs_ordered_by_color()));
        // HtmlLog::text(format!("task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, available_colors));
        
        let mut predicted_color: u8 = 42;

        if available_colors.number_of_counters_greater_than_zero() == 1 {
            if let Some(color) = available_colors.most_popular_color_disallow_ambiguous() {
                predicted_color = color;
            }
        }

        // If there are 3 or fewer colors, then make a prediction for each color.
        // If there are 4 or more colors, then do extra work to make max 3 predictions.
        // if available_colors.number_of_counters_greater_than_zero() <= 3 {
        // };

        let crop_output_size: ImageSize = context.output_size_vec[pair_index as usize];

        let cropped_image: Image = Image::color(crop_output_size.width, crop_output_size.height, predicted_color);

        let instance = ProcessedTaskWithOneTestPair {
            test_index,
            cropped_image,
        };
        Ok(instance)
    }

    /// This solver is only able to solve tasks where all pairs agrees that the output images have one color.
    fn all_pairs_have_one_output_color(task: &Task) -> bool {
        for action_label in &task.action_label_set_intersection {
            match action_label {
                ActionLabel::OutputImageUniqueColorCount { count } => {
                    if *count == 1 {
                        return true;
                    }
                },
                _ => {}
            }
        }
        false
    }

    /// Does the output colors come from the input image.
    fn output_image_colors_comes_from_input_image(task: &Task) -> bool {
        for action_label in &task.action_label_set_intersection {
            match action_label {
                ActionLabel::OutputImageColorsComesFromInputImage => {
                    return true;
                },
                _ => {}
            }
        }
        false
    }

    /// If the output color the same as the most popular color in the input image for pair.
    /// then returns the most popular color for that pair.
    fn pair_input_most_popular_color(task: &Task, pair_index: u8) -> Option<u8> {
        let mut found = false;
        for action_label in &task.action_label_set_intersection {
            match action_label {
                ActionLabel::PairInputMostPopularColorIsOutputMostPopularColor => {
                    found = true;
                    break;
                },
                _ => {}
            }
        }
        if !found {
            return None;
        }

        // Obtain the most popular color for the specified pair.
        for pair in &task.pairs {
            if pair.pair_index == pair_index {
                return pair.input.image_meta.histogram_all.most_popular_color_disallow_ambiguous();
            }
        }

        None
    }

    /// If the output color the same as the least popular color in the input image for pair.
    /// then returns the least popular color for that pair.
    fn pair_input_least_popular_color(task: &Task, pair_index: u8) -> Option<u8> {
        let mut found = false;
        for action_label in &task.action_label_set_intersection {
            match action_label {
                ActionLabel::PairInputLeastPopularColorIsOutputMostPopularColor => {
                    found = true;
                    break;
                },
                _ => {}
            }
        }
        if !found {
            return None;
        }

        // Obtain the most popular color for the specified pair.
        for pair in &task.pairs {
            if pair.pair_index == pair_index {
                return pair.input.image_meta.histogram_all.least_popular_color_disallow_ambiguous();
            }
        }

        None
    }
}

/// Returned from `process_task_with_one_test_pair`
struct ProcessedTaskWithOneTestPair {
    test_index: u8,
    cropped_image: Image,
}

struct ProcessedTask {
    ptwotp_vec: Vec<ProcessedTaskWithOneTestPair>,
    prediction_vec: Vec<arcathon_solution_coordinator::Prediction>,
}
