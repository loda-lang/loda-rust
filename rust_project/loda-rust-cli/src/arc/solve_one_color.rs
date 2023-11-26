//! Solve tasks that outputs a single color.
//! 
//! Solves zero of the hidden ARC tasks.
//! 
//! The ARC 1 dataset contains 800 tasks, where 17 of the tasks outputs a single color.
//! 1190e5a7, 1a2e2828, 239be575, 23b5c85d, 27a28665, 3194b014, 445eab21, 44f52bb0, 5582e5ca, 642d658d,
//! 7039b2d7, 8597cfd7, b9b7f026, d631b094, d9fac9be, de1cd16c, e872b94a, 
//! 
//! This solver is able to solve 14 of the 17 tasks.
//! Where 9 is solved with some confidence.
//! Where 5 of them is solved as a happy accident.
//! Where 3 tasks is not solved 1a2e2828, 27a28665, b9b7f026, because it has more than 4 colors to choose from, and chooses the wrong color.
//! 
//! Weakness:
//! When there are 4 or more colors to choose from, then it doesn't do any prediction, and takes the 3 first colors.
//! If the 3 first colors are not the correct colors, then it will fail.
//! The real solution will be to do the actual work needed to bring down the number of colors to max 3 colors,
//! by doing shape recognition, eliminating noise colors.
//! These tasks are exceeding 3 colors: 1a2e2828, 27a28665, 3194b014, 642d658d, b9b7f026, de1cd16c.
//! and thus are not solved by this solver. They are a happy accident if they are "guessed" correctly.
//! 
//! Future experiments:
//! Compare shapes across the input, is there a correlation between the shape and the output color.
//! Check symmetry in the input, is there a correlation between the symmetry and the output color.
//! Identify the densest color clusters, and pick the most popular color from the densest cluster.
//! Count number of holes, and return the object with the most holes or least holes.
use super::arc_json_model::GridFromImage;
use super::arc_work_model::{Task, PairType, Pair};
use super::{Image, arcathon_solution_coordinator, arc_json_model, ImageSize, TaskNameToPredictionVec, PropertyOutput, ImageProperty};
use super::{ActionLabel, VerifyPrediction, VerifyPredictionWithTask};
use super::{HtmlLog, Histogram};
use super::human_readable_utc_timestamp;
use anyhow::bail;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Debug)]
pub struct ProcessTaskContext {
    input_size_vec: Vec<ImageSize>,
    output_size_vec: Vec<ImageSize>,
    scale_widthheight: Option<(u8, u8)>,
}

impl ProcessTaskContext {
    pub fn new(task: &Task) -> Self {
        let mut instance = Self {
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
        let count0: usize = tasks.len();
        let tasks_for_processing: Vec<Task> = tasks.iter().filter(|task| Self::can_process_task(task)).cloned().collect();
        println!("SolveOneColor::new() out of {} tasks, {} tasks will be processed", count0, tasks_for_processing.len());
        Self {
            tasks: tasks_for_processing,
        }
    }

    /// Checks that the predicted output is the same as the expected output.
    /// 
    /// This can be run with the public ARC dataset contains expected output for the test pairs.
    /// 
    /// This cannot be run with the hidden ARC dataset, which doesn't contain expected output for the test pairs.
    #[allow(dead_code)]
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
                } else {
                    pb.println(format!("task {} - unable to solve! zero of {} test pairs", task.id, task_count_test));
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

    /// Run without verifying that the predictions are correct.
    /// 
    /// This code is intended to run with the hidden ARC dataset, which doesn't contain expected output for the test pairs.
    pub fn run_predictions(&self) -> anyhow::Result<TaskNameToPredictionVec> {
        let number_of_tasks: u64 = self.tasks.len() as u64;
        println!("{} - run start - will process {} tasks with SolveOneColor", human_readable_utc_timestamp(), number_of_tasks);
        let count_solved = AtomicUsize::new(0);
        let pb = ProgressBar::new(number_of_tasks as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-")
        );
        let accumulated = Arc::new(Mutex::new(TaskNameToPredictionVec::new()));
        self.tasks.par_iter().for_each(|task| {
            pb.inc(1);

            // Make predictions
            let processed_task: ProcessedTask = match Self::process_task(task) {
                Ok(value) => value,
                Err(error) => {
                    pb.println(format!("task {} - error: {:?}", task.id, error));
                    return;
                }
            };

            // Show progress
            count_solved.fetch_add(1, Ordering::Relaxed);
            let count: usize = count_solved.load(Ordering::Relaxed);
            pb.set_message(format!("Solved: {}", count));
            pb.println(format!("task {} - solved", task.id));

            // Accumulate the predictions
            match accumulated.lock() {
                Ok(mut map) => {
                    map.entry(task.id.clone())
                        .or_insert(Vec::new())
                        .extend(processed_task.prediction_vec);
                },
                Err(error) => {
                    pb.println(format!("run_predictions. Unable to lock accumulated. error: {:?}", error));
                }
            };
        });
        pb.finish_and_clear();
        let count_solved: usize = count_solved.load(Ordering::Relaxed);
        println!("{} - run - end", human_readable_utc_timestamp());
        println!("{} - solved {} of {} tasks", human_readable_utc_timestamp(), count_solved, number_of_tasks);
        let taskname_to_prediction_vec: TaskNameToPredictionVec = match accumulated.lock() {
            Ok(map) => map.clone(),
            Err(error) => {
                return Err(anyhow::anyhow!("run_predictions. taskname_to_prediction_vec. Unable to lock accumulated. error: {:?}", error));
            }
        };
        Ok(taskname_to_prediction_vec)
    }

    fn can_process_task(task: &Task) -> bool {
        // Only process tasks where all pairs agree that the output images have just one color.
        if !Self::all_pairs_have_one_output_color(task) {
            return false;
        }
        true
    }

    fn process_task(task: &Task) -> anyhow::Result<ProcessedTask> {
        let count_test: u8 = task.count_test().min(255) as u8;
        if count_test < 1 {
            return Err(anyhow::anyhow!("skipping task: {} because it has no test pairs", task.id));
        }    

        // Only process tasks where all pairs agree that the output images have just one color.
        if !Self::all_pairs_have_one_output_color(task) {
            return Err(anyhow::anyhow!("skipping task: {} all_pairs_have_one_output_color is not satisfied", task.id));
        }

        // Future experiments:
        // Only process tasks where the task has a predicted size with high confidence.
        // if the task doesn't have a predicted size, then skip it, so that another solver can try solve it.

        let context = ProcessTaskContext::new(task);

        let mut accumulated_ptwotp_vec = Vec::<ProcessedTaskWithOneTestPair>::new();
        for test_index in 0..count_test {
            let ptwotp_vec: Vec<ProcessedTaskWithOneTestPair> = match Self::process_task_with_one_test_pair(&context, task, test_index) {
                Ok(value) => value,
                Err(error) => {
                    return Err(error);
                }
            };
            accumulated_ptwotp_vec.extend(ptwotp_vec);
        }

        let mut prediction_vec = Vec::<arcathon_solution_coordinator::Prediction>::new();
        for ptwotp in &accumulated_ptwotp_vec {
            let grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&ptwotp.cropped_image);
            let prediction = arcathon_solution_coordinator::Prediction {
                output_id: ptwotp.test_index.min(255) as u8,
                output: grid,
                prediction_type: arcathon_solution_coordinator::PredictionType::SolveOneColor,
            };
            prediction_vec.push(prediction);
        }
    
        let instance = ProcessedTask {
            ptwotp_vec: accumulated_ptwotp_vec,
            prediction_vec,
        };
        Ok(instance)
    }

    fn process_task_with_one_test_pair(context: &ProcessTaskContext, task: &Task, test_index: u8) -> anyhow::Result<Vec<ProcessedTaskWithOneTestPair>> {
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

        let predicted_colors: Vec<u8> = Self::predict_colors_for_task_with_test_index(context, task, test_index)?;

        let output_size: ImageSize = context.output_size_vec[pair_index as usize];

        let ptwotp_vec: Vec<ProcessedTaskWithOneTestPair> = predicted_colors.iter().map(|predicted_color| {
            let cropped_image: Image = Image::color(output_size.width, output_size.height, *predicted_color);
            let ptwotp = ProcessedTaskWithOneTestPair {
                test_index,
                cropped_image,
            };
            ptwotp
        }).collect();
        Ok(ptwotp_vec)
    }

    fn predict_colors_for_task_with_test_index(context: &ProcessTaskContext, task: &Task, test_index: u8) -> anyhow::Result<Vec<u8>> {
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

        let output_image_colors_comes_from_input_image: bool = Self::output_image_colors_comes_from_input_image(task);

        // All pairs agree on the exact same color, and the color isn't impacted by the input image.
        if output_image_colors_comes_from_input_image == false && task.output_histogram_intersection == task.output_histogram_union {
            let available_colors: Histogram = task.output_histogram_intersection.clone();
            // println!("step0: task: {} - test_index: {} - available_colors: {:?} only one color is used across all outputs.", task.id, test_index, available_colors.pairs_ordered_by_color());
            let colors: Vec<u8> = available_colors.color_vec();
            return Ok(colors);
        }

        let mut available_colors = Histogram::new();
        for color in 0..=9u8 {
            available_colors.increment(color);
        }

        // println!("step0: task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, available_colors.pairs_ordered_by_color());

        if output_image_colors_comes_from_input_image {
            // The output colors are dictated by the input image
            for pair in &task.pairs {
                if pair.test_index == Some(test_index) {
                    available_colors = pair.input.image_meta.histogram_all.clone();
                    // println!("step1A: task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, available_colors.pairs_ordered_by_color());
                }
            }
        } else {
            // The output colors are not dictated by the input image
            available_colors = task.output_histogram_union.clone();
            // println!("step1B: task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, available_colors.pairs_ordered_by_color());
        }

        available_colors.add_histogram(&task.insert_histogram_intersection);
        // println!("step2: task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, available_colors.pairs_ordered_by_color());
        
        // If all pairs agree on the same removal colors, then make sure none of these are present in the available colors.
        // Future improvement: 
        // ARCathon allows for 3 predictions, we want to make all 3 predictions with different colors.
        // By removing too many colors, and we can make fewer predictions.
        // Aim for 3 colors in the available colors, after removing the removal colors.
        // Move the colors to a 3rd bin for the weakest predictions.
        available_colors.subtract_histogram(&task.removal_histogram_intersection);
        // println!("step3: task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, available_colors.pairs_ordered_by_color());

        let mut primary_color_predictions = Histogram::new();

        // The most popular color specific for each pair, is used for the output color.
        if let Some(color) = Self::pair_input_most_popular_color(task, pair_index) {
            primary_color_predictions.increment(color);
            available_colors.set_counter_to_zero(color);
            // println!("step5: task: {} - test_index: {} - primary_predictions: {:?} available_colors: {:?}", task.id, test_index, primary_color_predictions.pairs_ordered_by_color(), available_colors.pairs_ordered_by_color());
        }

        // The least popular color specific for each pair, is used for the output color.
        if let Some(color) = Self::pair_input_least_popular_color(task, pair_index) {
            primary_color_predictions.increment(color);
            available_colors.set_counter_to_zero(color);
            // println!("step6: task: {} - test_index: {} - primary_predictions: {:?} available_colors: {:?}", task.id, test_index, primary_color_predictions.pairs_ordered_by_color(), available_colors.pairs_ordered_by_color());
        }

        // println!("step-last: task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, available_colors.pairs_ordered_by_color());
        // HtmlLog::text(format!("task: {} - test_index: {} - primary_predictions: {:?} available_colors: {:?}", task.id, test_index, primary_color_predictions.pairs_ordered_by_color(), available_colors.pairs_ordered_by_color()));
        
        let primary_count: u16 = primary_color_predictions.number_of_counters_greater_than_zero();
        let secondary_count: u16 = available_colors.number_of_counters_greater_than_zero();
        let total_count: u16 = primary_count + secondary_count;
        if total_count == 0 {
            bail!("Unable to make prediction for task: {} - test_index: {} there are no available colors", task.id, test_index);
        }

        let mut the_colors: Vec<u8> = primary_color_predictions.color_vec();
        the_colors.extend(available_colors.color_vec());

        let high_confidence: bool = primary_count > 0 && primary_count <= 3;
        if high_confidence {
            debug!("high confidence: task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, the_colors);
        } else {
            let medium_confidence: bool = secondary_count > 0 && secondary_count <= 3;
            if medium_confidence {
                debug!("medium confidence: task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, the_colors);
            } else {
                // There are 4 or more colors to choose from. Extra work is needed to bring down the number of colors to make max 3 predictions.
                // Future experiments:
                // Assign a low confidence score to these predictions, so they are ranked lower than other high confidence predictions.
                // Rule out the noise color, grid color, most dense colors.
                debug!("low confidence: task: {} - test_index: {} - available_colors: {:?}", task.id, test_index, the_colors);
                // Taking the initial 3 colors doesn't solve any of the hidden ARC tasks.
                // Here is a crappy 2nd guess. By reversing the_colors.
                the_colors.reverse();
                the_colors.truncate(3);
            }
        }
        Ok(the_colors)
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

        // Obtain the least popular color for the specified pair.
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
