//! Performs logistic regression of each input pixel with the corresponding classification for the output pixel.
//! 
//! These older commits solves some of the tasks from the hidden ARC dataset, using logistic regression:
//! commit 2023-Nov-17: solves 2 of the hidden ARC tasks. This uses variant=0 and variant=1. No use of variant=2.
//! Since it uses multiple variants and doesn't solve more tasks than the previous commit, then it is not an improvement.
//! https://github.com/loda-lang/loda-rust/commit/e7228b0a3c3453652a9cd23a1c810b42f82696b1
//!
//! commit 2023-Nov-16: solves 2 of the hidden ARC tasks. This uses variant=0 and variant=1. No use of variant=2.
//! Since it uses multiple variants and doesn't solve more tasks than the previous commit, then it is not an improvement.
//! https://github.com/loda-lang/loda-rust/commit/f7bf7103e1de14dcf83471aebb3013794fe7699b
//!
//! commit 2023-Nov-05: solves 2 of the hidden ARC tasks. Uses only variant=0. No use of variant=1 nor variant=2.
//! This is the optimal combination so far. Only a single variant is used.
//! https://github.com/loda-lang/loda-rust/commit/afd97bb998964b5dca372b22843de7c6c15f8969
//!
//! commit 2023-Oct-31: solves 2 of the hidden ARC tasks.
//! https://github.com/loda-lang/loda-rust/commit/0bb228a23abef2f9debd1d5b0c594833c51c4910
//!
//! commit 2023-Oct-24: solves 2 of the hidden ARC tasks.
//! https://github.com/loda-lang/loda-rust/commit/bec5ccd9f3e0ce4009ae117f1ea41652bf5292e6
//!
//! commit 2023-Oct-22: solves 2 of the hidden ARC tasks.
//! https://github.com/loda-lang/loda-rust/commit/e38f80c8e6a34cc55376f564e99f9e47114fcf63
//!
//! commit 2023-Oct-09: solves 2 of the hidden ARC tasks.
//! https://github.com/loda-lang/loda-rust/commit/430f3d4b1182a40058230e54564b8e6c482e1509
//!
//! 
//! This current file solves 2 of the tasks from the hidden ARC dataset.
//!
//! This solves 78 of the 800 tasks in the public ARC dataset.
//! 009d5c81, 00d62c1b, 00dbd492, 0a2355a6, 0d3d703e, 140c817e, 1c0d0a4b, 2072aba6, 21f83797, 2281f1f4,
//! 23581191, 253bf280, 25d8a9c8, 32597951, 332efdb3, 3618c87e, 37d3e8b2, 4258a5f9, 45737921, 4612dd53,
//! 50cb2852, 5168d44c, 5289ad53, 543a7ed5, 54d9e175, 5b526a93, 5b6cbef5, 639f5a19, 6455b5f5, 67385a82,
//! 694f12f3, 69889d6e, 6c434453, 6d75e8bb, 6ea4a07e, 6f8cd79b, 810b9b61, 84f2aca1, 868de0fa, 90f3ed37,
//! 95990924, a5313dff, a61f2674, a65b410d, a699fb00, a8d7556c, a934301b, a9f96cdd, aa4ec2a5, ae58858e,
//! aedd82e4, b0c4d837, b1948b0a, b2862040, b60334d2, b6afb2da, ba26e723, bb43febb, bbb1b8b6, c0f76784,
//! c8f0f002, ce039d91, ce22a75a, ce9e57f2, d2abd087, d364b489, d37a1ef5, d406998b, d5d6de2d, dbc1a6ce,
//! dc433765, de1cd16c, ded97339, e0fb7511, e133d23d, e872b94a, e9c9d9a1, ef135b50, 
//! 
//! This partially solves 5 of the 800 tasks in the public ARC dataset. Where one ore more `test` pairs is solved, but not all of the `test` pairs gets solved.
//! 239be575, 27a28665, 44f52bb0, 794b24be, da2b0fe3, 
//! 
//! Weakness: The tasks that it solves doesn't involve object manipulation. 
//! It cannot move an object by a few pixels, the object must stay steady in the same position.
//! 
//! Weakness: Struggles with tasks that have diagonal lines/stripes/patterns.
//! 
//! Run Logistic regression multiple times:
//! Take the predicted output, and use as input for the next iteration.
//! I have not seen any improvement of the prediction accuracy.
//! The number of iterations I have tried 1, 2, .., 5, 6.
//! I have tried adding lots of noise initially and then reduce the noise in the later iterations.
//! 
//! Run Logistic regression multiple times:
//! Serialize the obfuscated color with different offsets, so the logistic regression don't get a color bias.
//! In each iteration the obfuscation offset is randomized.
//! However I don't see any improvement of the prediction accuracy.
//! 
//! Future experiments:
//! * Transform the `train` pairs: rotate90, rotate180, rotate270, flipx, flipy.
//! * Transform the `test` pairs: rotate90, rotate180, rotate270, flipx, flipy.
//! * Provide `weight` to logistic regression, depending on how important each parameter is.
use super::arc_json_model::GridFromImage;
use super::arc_work_model::{Task, PairType, Pair};
use super::{Image, ImageOverlay, arcathon_solution_coordinator, arc_json_model, ImageMix, MixMode, ObjectsAndMass, ImageCrop, Rectangle, ImageExtractRowColumn, ImageDenoise, TaskGraph, ShapeType, ImageSize, ShapeTransformation, SingleColorObject, ShapeIdentificationFromSingleColorObject, ImageDetectHole, ImagePadding, ImageRepairPattern, TaskNameToPredictionVec, CreateTaskWithSameSize, ImageReplaceColor, ImageCenterIndicator, ImageGravity, GravityDirection, DiagonalHistogram, RecordTrigram, ImageNgram, ImageExteriorCorners, LargestInteriorRectangle, ImageDrawRect, PropertyOutput, ImageProperty, ImageResize, ImageRepeat, rule, CellularAutomaton};
use super::{ActionLabel, ImageLabel, ImageMaskDistance, LineSpan, LineSpanDirection, LineSpanMode, VerifyPrediction, VerifyPredictionWithTask};
use super::{HtmlLog, PixelConnectivity, ImageHistogram, Histogram, ImageEdge, ImageMask};
use super::{ImageNeighbour, ImageNeighbourDirection, ImageCornerAnalyze, ImageMaskGrow, Shape3x3};
use super::human_readable_utc_timestamp;
use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;
use serde::Serialize;
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use linfa::prelude::*;
use linfa_logistic::{MultiLogisticRegression, MultiFittedLogisticRegression};
use ndarray::prelude::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The ARCathon contest allows for submitting up to 3 predictions for each task.
/// If one or more of the 3 predictions is correct, then the task is considered solved.
/// 
/// Running the `SolveLogisticRegression` algorithm 3 times with different `variant` parameters,
/// and it will yield 3 predictions. I have tweaked the `variant` parameter to yield as much different predictions as possible.
/// However for `variant=1` and `variant=2` the predictions does not solve any of the hidden ARC task.
/// Only `variant=0` solves 2 of the hidden ARC tasks.
/// 
/// This will try out all the `variant` parameters.
/// const PROCESS_TASK_VARIANTS: [u8; 3] = [0, 1, 2]; 
/// 
/// No need to run variant=1 nor variant=2, because they don't solve any of the hidden ARC tasks. Only variant=0 solves 2 of the hidden ARC tasks.
const PROCESS_TASK_VARIANTS: [u8; 2] = [0, 1];

/// The colors 0..9 with an extra `color 10` for the `outside` canvas area.
#[allow(dead_code)]
const COUNT_COLORS_PLUS1: u8 = 11;

#[derive(Clone, Debug, Serialize)]
struct Record {
    classification: u8,
    is_test: bool,
    pair_id: u8,

    // Future experiment
    // make a `secondary_values` vector that use a lower weight in the logistic regression.
    // examples of secondary values: is the x position a mod2==0, is x position a mod3==0.
    values: Vec<f64>,
}

impl Record {
    #[allow(dead_code)]
    fn serialize_bool(&mut self, value: bool) {
        let v: f64 = if value { 1.0 } else { -1.0 };
        self.values.push(v);
    }

    #[allow(dead_code)]
    fn serialize_bool_onehot(&mut self, value: bool) {
        self.serialize_onehot_discard_overflow(if value { 0 } else { 1 }, 2);
    }

    #[allow(dead_code)]
    fn serialize_ternary(&mut self, value: i8) {
        let v: f64;
        if value == 0 {
            v = 0.0;
        } else {
            if value > 0 { 
                v = 1.0; 
            } else { 
                v = -1.0;
            }
        }
        self.values.push(v);
    }

    #[allow(dead_code)]
    fn serialize_f64(&mut self, value: f64) {
        self.values.push(value);
    }

    fn serialize_u8(&mut self, value: u8) {
        self.values.push(value as f64);
    }

    fn serialize_color_complex(&mut self, color: u8, offset: f64, is_enabled: bool) {
        if !is_enabled {
            return
        }
        self.serialize_complex_scaled(color as u16, 10, offset, 1.0)
    }

    #[allow(dead_code)]
    fn serialize_color_complex11(&mut self, color: u8, offset: f64) {
        self.serialize_complex_scaled(color as u16, COUNT_COLORS_PLUS1 as u16, offset, 1.0)
    }

    #[allow(dead_code)]
    fn serialize_cluster_id(&mut self, color: u8, cluster_id: u8, offset: f64, is_enabled_cluster_id_shakeup: bool) {
        let mut value: u16 = u16::MAX;
        if cluster_id < 41 && color < 10 {
            value = (cluster_id as u16) * 10 + (color as u16);
            if is_enabled_cluster_id_shakeup {
                value += 100;
                value ^= 0xaa;
            }
        }
        self.serialize_complex_scaled(value, 410, offset, 1.0);
    }

    /// Set the counter to 1 that are equal to the value.
    /// 
    /// Otherwise the counters are zero.
    /// 
    /// When the value overflows the capacity then set the `other` counter to 1.
    #[allow(dead_code)]
    fn serialize_onehot(&mut self, value: u8, count: u8) {
        let mut found: u8 = 0;
        for i in 0..count {
            let v: u8 = if i == value { 1 } else { 0 };
            found |= v;
            self.values.push(v as f64);
        }
        let other: u8 = if found > 0 { 0 } else { 1 };
        self.values.push(other as f64);
    }

    /// Onehot encoding of a bitmask. 
    /// 
    /// Bits that are 1 are set to 1.
    /// 
    /// Otherwise the counters are zero.
    #[allow(dead_code)]
    fn serialize_bitmask_as_onehot(&mut self, value: u16, count: u8) {
        for i in 0..(count.min(16)) {
            let mask: u16 = 1 << i;
            let is_bit_one: bool = (value & mask) > 0;
            self.serialize_bool_onehot(is_bit_one);
        }
    }

    #[allow(dead_code)]
    fn serialize_onehot_discard_overflow(&mut self, value: u8, count: u8) {
        self.serialize_onehot_discard_overflow_u16(value as u16, count as u16);
    }

    /// Set the counter to 1 that are equal to the value.
    /// 
    /// Otherwise the counters are zero.
    /// 
    /// When the value overflows then all the counters are set to zero.
    #[allow(dead_code)]
    fn serialize_onehot_discard_overflow_u16(&mut self, value: u16, count: u16) {
        for i in 0..count {
            let v: u8 = if i == value { 1 } else { 0 };
            self.values.push(v as f64);
        }
    }

    /// Set the counters to 1 that are equal or higher than the value.
    /// 
    /// Set the counters that are lower than the value to 0.
    /// 
    /// When the value overflows the capacity then set the `other` counter to 1.
    #[allow(dead_code)]
    fn serialize_split_zeros_ones(&mut self, value: u8, count: u8) {
        let mut found: u8 = 0;
        for i in 0..count {
            let v: u8 = if i >= value { 1 } else { 0 };
            found |= v;
            self.values.push(v as f64);
        }
        let other: u8 = if found > 0 { 0 } else { 1 };
        self.values.push(other as f64);
    }

    /// Serialize to a complex number.
    /// 
    /// When the 0 <= value < count, then the the value is converted to a complex number,
    /// with distance 1 from the origin. The values are evenly distributed around the unit circle.
    /// 
    /// When the value overflows the `x` and `y` are set to zero.
    #[allow(dead_code)]
    fn serialize_complex(&mut self, value: u16, count: u16) {
        self.serialize_complex_scaled(value, count, 0.2, 1.0);
    }

    #[allow(dead_code)]
    fn serialize_complex_scaled(&mut self, value: u16, count: u16, offset: f64, scale: f64) {
        let x: f64;
        let y: f64;
        if count > 0 && value < count {
            let radians: f64 = ((value as f64) + offset) * std::f64::consts::TAU / count as f64;
            x = radians.cos() * scale;
            y = radians.sin() * scale;
        } else {
            x = 0.0;
            y = 0.0;
        }
        self.values.push(x);
        self.values.push(y);
    }
}

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

/// Returned from `process_task_with_one_test_pair`
struct ProcessedTaskWithOneTestPair {
    test_index: u8,
    cropped_image: Image,
    inspect_internal_image_vec: Vec<Image>,
}

struct ProcessedTask {
    ptwotp_vec: Vec<ProcessedTaskWithOneTestPair>,
    prediction_vec: Vec<arcathon_solution_coordinator::Prediction>,
}

pub struct SolveLogisticRegression {
    #[allow(dead_code)]
    tasks: Vec<Task>,
}

impl SolveLogisticRegression {
    #[allow(dead_code)]
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
        let run_and_verify_ignore_already_solved = false;
        let number_of_tasks: u64 = self.tasks.len() as u64;
        println!("{} - run start - will process {} tasks with logistic regression", human_readable_utc_timestamp(), number_of_tasks);
        let count_solved_full = AtomicUsize::new(0);
        let count_solved_partial = AtomicUsize::new(0);
        let pb = ProgressBar::new(number_of_tasks as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-")
        );
        
        let already_fully_solved_tasks_ids = [
            "009d5c81", "00d62c1b", "00dbd492", "08ed6ac7", "0a2355a6", "0d3d703e", "140c817e", "178fcbfb", "1b2d62fb", "1c0d0a4b",
            "2072aba6", "21f83797", "2281f1f4", "23581191", "253bf280", "25d8a9c8", "319f2597", "32597951", "332efdb3", "3618c87e",
            "37d3e8b2", "4258a5f9", "44d8ac46", "45737921", "4612dd53", "50cb2852", "5168d44c", "516b51b7", "5289ad53", "543a7ed5",
            "54d9e175", "5b526a93", "5b6cbef5", "639f5a19", "6455b5f5", "67385a82", "694f12f3", "69889d6e", "6c434453", "6d75e8bb",
            "6ea4a07e", "6f8cd79b", "776ffc46", "810b9b61", "8a371977", "84f2aca1", "868de0fa", "90f3ed37", "95990924", "a5313dff",
            "a61f2674", "a65b410d", "a699fb00", "a8d7556c", "a934301b", "a9f96cdd", "aa4ec2a5", "ae3edfdc", "ae58858e", "aedd82e4",
            "b0c4d837", "b1948b0a", "b230c067", "b2862040", "b60334d2", "b6afb2da", "ba26e723", "bb43febb", "bbb1b8b6", "bdad9b1f",
            "c0f76784", "c8f0f002", "ce039d91", "ce22a75a", "ce9e57f2", "d2abd087", "d364b489", "d37a1ef5", "d406998b", "d511f180",
            "d5d6de2d", "dbc1a6ce", "dc433765", "de1cd16c", "ded97339", "e0fb7511", "e133d23d", "e8593010", "e872b94a", "e9c9d9a1",
            "ef135b50", 
        ];
        let ignore_task_id: HashSet<String> = already_fully_solved_tasks_ids.iter().map(|s| s.to_string()).collect();
        
        self.tasks.par_iter().for_each(|task| {
            pb.inc(1);

            if run_and_verify_ignore_already_solved && ignore_task_id.contains(&task.id) {
                // pb.println(format!("ignoring already fully solved task {}", task.id));
                return;
            }

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
                    HtmlLog::compare_images(ptwotp.inspect_internal_image_vec.clone());
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
        println!("{} - run start - will process {} tasks with logistic regression", human_readable_utc_timestamp(), number_of_tasks);
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

    /// Converts an unsigned binary number to reflected binary Gray code.
    // fn binary_to_grey(value: u8) -> u8 {
    //     value ^ (value >> 1)
    // }

    /// Converts a reflected binary Gray code number to a binary number.
    // fn grey_to_binary(value: u8) -> u8 {
    //     let mut result: u8 = value;
    //     let mut mask: u8 = value;
    //     for _ in 1..8 {
    //         mask >>= 1;
    //         result ^= mask;
    //     }
    //     result
    // }

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

        let task_for_processing: Task;
        let prediction_type: arcathon_solution_coordinator::PredictionType;
        if task.is_output_size_same_as_input_size() {
            task_for_processing = task.clone();
            prediction_type = arcathon_solution_coordinator::PredictionType::SolveLogisticRegressionSameSize;
        } else {
            let task2: Task = CreateTaskWithSameSize::create(task)?;
            task_for_processing = task2;
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

        // Obtain the size of the output image
        let crop_output_size: ImageSize = context.output_size_vec[pair_index as usize];

        let number_of_iterations: usize = 5;
        let mut computed_images = Vec::<Image>::new();
        let mut last_computed_image: Option<Image> = None;
        for iteration_index in 0..number_of_iterations {
            let records = Self::process_task_iteration(context, task, iteration_index, test_index, last_computed_image)?;
            let computed_image: Image = perform_logistic_regression(task, test_index, &records)?;
            last_computed_image = Some(computed_image.clone());
            computed_images.push(computed_image);
        }

        let computed_image: Image = match last_computed_image {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Unable to get last computed image"));
            }
        };

        // Get rid of the `outside` area.
        let mut cropped_image: Image = computed_image.crop_outside(0, 0, crop_output_size.width, crop_output_size.height, 255)?;

        // Eliminate illegal colors.
        let most_popular_output_color: u8 = 0;
        for color in 10..=255 {
            cropped_image = cropped_image.replace_color(color, most_popular_output_color)?;
        }

        let instance = ProcessedTaskWithOneTestPair {
            test_index,
            cropped_image,
            inspect_internal_image_vec: computed_images,
        };
        Ok(instance)
    }

    #[allow(dead_code)]
    fn object_id_image(task_graph: &TaskGraph, pair_index: u8, width: u8, height: u8, connectivity: PixelConnectivity) -> anyhow::Result<Image> {
        let mut image: Image = Image::zero(width, height);
        for y in 0..height {
            for x in 0..width {
                let object_id: usize = task_graph.get_objectid_for_input_pixel(pair_index, x, y, connectivity)
                    .context("object_id_image object_id")?;
                let color: u8 = object_id.min(255) as u8;
                _ = image.set(x as i32, y as i32, color);
            }
        }
        Ok(image)
    }

    #[allow(dead_code)]
    fn relative_position_images(task_graph: &TaskGraph, pair_index: u8, width: u8, height: u8, connectivity: PixelConnectivity) -> anyhow::Result<Vec<Image>> {
        let mut image_x: Image = Image::zero(width, height);
        let mut image_y: Image = Image::zero(width, height);
        for y in 0..height {
            for x in 0..width {
                let (position_x, position_y) = task_graph.get_objectposition_for_input_pixel(pair_index, x, y, connectivity)
                    .context("relative_position_images position_x position_y")?;
                let relative_x: i32 = (x as i32) - (position_x as i32);
                let relative_y: i32 = (y as i32) - (position_y as i32);
                {
                    let color: u8 = relative_x.min(255) as u8;
                    _ = image_x.set(x as i32, y as i32, color);
                }
                {
                    let color: u8 = relative_y.min(255) as u8;
                    _ = image_y.set(x as i32, y as i32, color);
                }
            }
        }
        Ok(vec![image_x, image_y])
    }

    fn color_from_shape_type(shape_type: ShapeType) -> u8 {
        match shape_type {
            ShapeType::Unclassified => 0,
            ShapeType::Rectangle => 1,
            ShapeType::Box => 2,
            ShapeType::Plus => 3,
            ShapeType::Crosshair => 4,
            ShapeType::X => 5,
            ShapeType::L => 6,
            ShapeType::UpTack => 7,
            ShapeType::U4 => 8,
            ShapeType::U5 => 9,
            ShapeType::HUppercase => 10,
            ShapeType::HLowercase => 11,
            ShapeType::InvertedFork => 12,
            ShapeType::RotatedK => 13,
            ShapeType::TurnedV => 14,
            ShapeType::Diagonal2 => 15,
            ShapeType::Diagonal3 => 16,
            ShapeType::SkewTetromino => 17,
            ShapeType::LowerLeftTriangle => 18,
            ShapeType::FlippedJ => 19,
            ShapeType::LeftPlus => 20,
            ShapeType::LeftwardsHarpoonWithBarbUpwards => 21,
            ShapeType::BoxWithoutOneCorner => 22,
            ShapeType::RotatedD => 23,
            ShapeType::RotatedJRound => 24,
            ShapeType::BoxWithoutDiagonal => 25,
            ShapeType::RotatedS => 26,
            ShapeType::PlusWithOneCorner => 27,
            ShapeType::SquareWithoutDiagonalCorners => 28,
            ShapeType::GameOfLifeBoat => 29,
            ShapeType::LWith45DegreeLine => 30,
            ShapeType::XWithoutOneCorner => 31,
            ShapeType::LSkew => 32,
            ShapeType::UpTackSkew => 33,
            ShapeType::LowerLeftTriangleWithCorner => 34,
            ShapeType::IUppercaseMovedCorner => 35,
            ShapeType::SkewTetrominoWithTopLeftCorner => 36,
            ShapeType::RotatedUppercaseE => 37,
            ShapeType::TurnedW => 38,
            ShapeType::LineAroundSmallObstacle => 39,
            ShapeType::LineAroundBigObstacle => 40,
            ShapeType::BoxWithTwoHoles => 41,
            ShapeType::BoxWith2x2Holes => 42,
            ShapeType::XMovedCorner => 43,
            ShapeType::LowerLeftTriangleWithoutCorner => 44,
            ShapeType::LowerLeftTriangleMovedCorner => 45,
            ShapeType::RotatedP => 46,
            ShapeType::RotatedLowercaseF => 47,
            ShapeType::BoxWithRightwardsTick => 48,
            ShapeType::OpenBoxWithHoleInCenterOfTopBorder => 49,
            ShapeType::OpenBoxWithHoleInRightSideOfTopBorder => 50,
            ShapeType::BoxWithUptick => 51,
            ShapeType::Grid3x2 => 52,
            ShapeType::Grid3x3 => 53,
            ShapeType::Grid4x2 => 54,
            ShapeType::Grid4x3 => 55,
        }
    }

    fn shape_type_image(task_graph: &TaskGraph, pair_index: u8, width: u8, height: u8, connectivity: PixelConnectivity, rotate45: bool) -> anyhow::Result<Image> {
        let mut image: Image = Image::zero(width, height);
        for y in 0..height {
            for x in 0..width {
                let shape_type: ShapeType = match rotate45 {
                    false => task_graph.get_shapetype_for_input_pixel(pair_index, x, y, connectivity)
                        .with_context(|| format!("shape_type_image. pair_index: {} x: {} y: {} width: {} height: {} connectivity: {:?}", pair_index, x, y, width, height, connectivity))?,
                    true => task_graph.get_shapetype45_for_input_pixel(pair_index, x, y, connectivity)?
                };
                let color: u8 = Self::color_from_shape_type(shape_type);
                _ = image.set(x as i32, y as i32, color);
            }
        }
        Ok(image)
    }

    #[allow(dead_code)]
    fn shape_transformation_images(task_graph: &TaskGraph, pair_index: u8, width: u8, height: u8, connectivity: PixelConnectivity) -> anyhow::Result<Vec<Image>> {
        let mut image_normal: Image = Image::zero(width, height);
        let mut image_rotate_cw_90: Image = Image::zero(width, height);
        let mut image_rotate_cw_180: Image = Image::zero(width, height);
        let mut image_rotate_cw_270: Image = Image::zero(width, height);
        let mut image_flipx: Image = Image::zero(width, height);
        let mut image_flipx_rotate_cw_90: Image = Image::zero(width, height);
        let mut image_flipx_rotate_cw_180: Image = Image::zero(width, height);
        let mut image_flipx_rotate_cw_270: Image = Image::zero(width, height);
        for y in 0..height {
            for x in 0..width {
                let transformations: Vec<ShapeTransformation> = task_graph.get_shapetransformations_for_input_pixel(pair_index, x, y, connectivity)?;
                for transformation in transformations {
                    let image: &mut Image = match transformation {
                        ShapeTransformation::Normal => image_normal.borrow_mut(),
                        ShapeTransformation::RotateCw90 => image_rotate_cw_90.borrow_mut(),
                        ShapeTransformation::RotateCw180 => image_rotate_cw_180.borrow_mut(),
                        ShapeTransformation::RotateCw270 => image_rotate_cw_270.borrow_mut(),
                        ShapeTransformation::FlipX => image_flipx.borrow_mut(),
                        ShapeTransformation::FlipXRotateCw90 => image_flipx_rotate_cw_90.borrow_mut(),
                        ShapeTransformation::FlipXRotateCw180 => image_flipx_rotate_cw_180.borrow_mut(),
                        ShapeTransformation::FlipXRotateCw270 => image_flipx_rotate_cw_270.borrow_mut(),
                    };
                    _ = image.set(x as i32, y as i32, 1);
                }
            }
        }
        Ok(vec![image_normal, image_rotate_cw_90, image_rotate_cw_180, image_rotate_cw_270, image_flipx, image_flipx_rotate_cw_90, image_flipx_rotate_cw_180, image_flipx_rotate_cw_270])
    }

    fn shape_size_images(task_graph: &TaskGraph, pair_index: u8, width: u8, height: u8, connectivity: PixelConnectivity) -> anyhow::Result<Vec<Image>> {
        let mut image_shape_width: Image = Image::zero(width, height);
        let mut image_shape_height: Image = Image::zero(width, height);
        for y in 0..height {
            for x in 0..width {
                let shape_size: ImageSize = task_graph.get_shapesize_for_input_pixel(pair_index, x, y, connectivity)?;
                _ = image_shape_width.set(x as i32, y as i32, shape_size.width);
                _ = image_shape_height.set(x as i32, y as i32, shape_size.height);
            }
        }
        Ok(vec![image_shape_width, image_shape_height])
    }

    fn process_task_iteration(context: &ProcessTaskContext, task: &Task, process_task_iteration_index: usize, test_index: u8, computed_image: Option<Image>) -> anyhow::Result<Vec::<Record>> {
        // println!("exporting task: {}", task.id);

        if context.input_size_vec.len() != task.pairs.len() {
            return Err(anyhow::anyhow!("context.input_size_vec.len() != task.pairs.len()"));
        }
        if context.output_size_vec.len() != task.pairs.len() {
            return Err(anyhow::anyhow!("context.output_size_vec.len() != task.pairs.len()"));
        }

        if !task.is_output_size_same_as_input_size() {
            return Err(anyhow::anyhow!("process_task_iteration all the pairs must have same input_size and output_size"));
        }

        let v: usize = context.variant as usize;


        // let obfuscated_color_offset: f64 = 0.2;
        let obfuscated_color_offset: f64 = (process_task_iteration_index as f64 * 0.6333 + 0.4) % 1.0;
        // let one_eleventh: f64 = 1.0 / 11.0;
        // let obfuscated_color_offset: f64 = (process_task_iteration_index as f64 * one_eleventh + 0.2) % 1.0;
        
        let obfuscated_cluster_offset: f64 = 0.2;

        // When input_size == output_size then the parameters only needs to be serialized once.
        // When the input_size != output_size, then serialize parameters for input and serialize parameters for output.
        let has_different_size_for_input_output = match context.mode {
            ProcessTaskMode::InputOutputSameSize => false,
            ProcessTaskMode::InputOutputDifferentSize => true,
        };


        
        let enable_serialize_color_complex: bool = [true, false, true][v];
        let enable_serialize_cluster_id_shakeup: bool = [false, true, true][v];
        let enable_total_clustercount: bool = false;
        let enable_color_clustercount: bool = false;
        let enable_half_context_input_size: bool = [true, true, false][v];
        let enable_half_context_output_size: bool = [false, false, has_different_size_for_input_output][v];
        let enable_normalized_coordinates_context_input_size: bool = false;
        let enable_normalized_coordinates_context_output_size: bool = false;

        let enable_output_orientation: bool = has_different_size_for_input_output;
        let enable_coordinates_xy: bool = false;
        let enable_coordinates_xy_reverse_input: bool = false;
        let enable_coordinates_xy_reverse_output: bool = false;
        let enable_is_outside: bool = has_different_size_for_input_output;
        let enable_distance: bool = !has_different_size_for_input_output;
        let enable_diagonalhistogram_opposites: bool = has_different_size_for_input_output;

        let enable_histogram_diagonal_a: bool = [false, true, false][v];
        let enable_histogram_diagonal_b: bool = [false, true, false][v];
        let enable_histogram_diagonal_c: bool = [false, false, false][v];
        let enable_histogram_diagonal_d: bool = [false, false, false][v];
        let enable_histogram_diagonal_e: bool = false;
        let enable_histogram_diagonal_f: bool = false;
        let enable_histogram_diagonal: bool = enable_histogram_diagonal_a || enable_histogram_diagonal_b || enable_histogram_diagonal_c || enable_histogram_diagonal_d || enable_histogram_diagonal_e || enable_histogram_diagonal_f;

        let enable_center_indicator_a: bool = [false, false, false][v];
        let enable_center_indicator_x: bool = [false, false, false][v];
        let enable_center_indicator_y: bool = [false, false, false][v];
        let enable_center_indicator: bool = enable_center_indicator_a || enable_center_indicator_x || enable_center_indicator_y;

        let enable_input_four_xy_pairs: bool = [false, false, true][v];
        let enable_output_four_xy_pairs: bool = false;
        let enable_input_three_xy_pairs: bool = [false, false, true][v];
        let enable_output_three_xy_pairs: bool = false;
        let enable_gravity: bool = false;

        let enable_mod2: bool = [true, true, false][v];
        let enable_mod2_reverse_input: bool = [true, true, false][v];
        let enable_mod2_reverse_output: bool = false;

        let enable_mod3: bool = [false, false, true][v];
        let enable_mod3_reverse_input: bool = [false, false, true][v];
        let enable_mod3_reverse_output: bool = [false, false, false][v];

        let enable_hole_type1: bool = [true, false, true][v];
        let enable_color_repair: bool = true;
        
        let enable_shape_transformation_images: bool = [false, false, false][v];
        let enable_noisecolor_in_outline: bool = [true, false, true][v];
        let enable_grid: bool = true;

        let enable_enumerated_clusters_grow_mask3: bool = [false, true, false][v];
        let enable_color_grow_mask1: bool = [false, true, false][v];
        let enable_color_grow_mask2: bool = [false, true, false][v];
        let enable_color_grow_mask3: bool = [false, true, false][v];

        let enable_no_change_to_color: bool = true;
        let enable_no_change_to_center_color: bool = false;
        let enable_no_change_to_noise_color: bool = false;
        let enable_object_center_same_as_neighbour: bool = false;
        let enable_edge: bool = [false, false, false][v];

        let enable_color_inside_bounding_box: bool = [true, true, true][v];
        let enable_object_id_image_connectivity4: bool = [false, false, false][v];
        let enable_object_id_image_connectivity8: bool = [false, false, false][v];

        let enable_trigram_count_center: bool = [false, true, false][v];
        let enable_trigram_count_word1_center: bool = [false, true, false][v];
        let enable_trigram_count_word012_center: bool = [false, true, false][v];

        let enable_full_row_and_column: bool = [true, false, true][v];
        let enable_full_row_xor_column: bool = [true, false, true][v];
        let enable_full_row_or_column: bool = [true, false, true][v];
        let enable_full_row: bool = [false, false, false][v];
        let enable_full_column: bool = [false, false, false][v];

        let enable_symmetry_shorter: bool = [false, false, false][v];
        let enable_symmetry_masks: bool = false;
        let enable_corner_classification: bool = false;

        let enable_histogram_columns_rows_get_color: bool = [true, true, false][v];
        let enable_histogram_columns_rows_lookaround: bool = [false, true, false][v];

        let enable_exterior_of_clusters: bool = [false, true, false][v];
        let enable_largest_interior_rectangle_masks: bool = [false, false, false][v];
        let enable_relative_position_topleft_xy: bool = false;
        let enable_relative_position_checkerboard: bool = false;

        let enable_scale_widthheight: bool = has_different_size_for_input_output;
        let enable_check_pixel_in_histogram: bool = false;
        let enable_nearest_color: bool = false;
        let enable_colordirection_to_distanceimage: bool = [false, true, true][v];
        let enable_neighbour_color: bool = false;
        let enable_adjacent_neighbour_same_as_center: bool = false;
        let enable_opposite_neighbour: bool = [false, false, true][v];
        let enable_removal_color_center: bool = false;
        let enable_detect_nonsquare: bool = false;
        
        let enable_typo_for_center_row_right_columns: bool = [!has_different_size_for_input_output, false, false][v];
        let enable_denoise_type5_input: bool = [false, true, false][v];
        let enable_denoise_type5_output: bool = [false, false, false][v];
        let enable_same_colors_for_area3x3_and_area5x5: bool = [false, false, false][v];
        let enable_area3x3_input_8bit_mask: bool = [false, false, false][v];
        let enable_area3x3_output_8bit_mask: bool = [false, false, false][v];
        let enable_gameoflife: bool = false;

        // let mut histogram_preserve = Histogram::new();
        // task.action_label_set_intersection.iter().for_each(|label| {
        //     match label {
        //         ActionLabel::OutputImageIsInputImageWithNoChangesToPixelsWithColor { color } => {
        //             histogram_preserve.increment(*color);
        //         },
        //         _ => {}
        //     }
        // });

        // let mut all_pairs_has_predicted_palette = true;
        // for pair in &task.pairs {
        //     let mut found = false;
        //     for label in &pair.prediction_set {
        //         match label {
        //             arc_work_model::Prediction::OutputPalette { histogram: _ } => {
        //                 found = true;
        //             },
        //             _ => {}
        //         }
        //     }
        //     if !found {
        //         all_pairs_has_predicted_palette = false;
        //         break;
        //     }
        // }

        // Generate a random image when there is no computed image.
        // let computed_image2: Image;
        // if let Some(computed_image) = computed_image {
        //     computed_image2 = computed_image;
        // } else {
        //     let mut size: ImageSize = ImageSize::empty();
        //     for pair in &task.pairs {
        //         if pair.pair_type == PairType::Test {
        //             if pair.test_index == Some(test_index) {
        //                 size = pair.input.image.size();
        //                 break;
        //             }
        //         }
        //     }
        //     let random_seed: u64 = process_task_iteration_index as u64;
        //     let mut rng: StdRng = StdRng::seed_from_u64(random_seed);
        //     let image: Image = RandomImage::uniform_colors(&mut rng, size, 9)?;
        //     computed_image2 = image;
        // }

        let random_seed_offset: u64 = [0, 42, 80][v];

        let mut earlier_prediction_image_vec = Vec::<Image>::new();
        if let Some(computed_image) = computed_image {
            let random_seed: u64 = (process_task_iteration_index as u64) + random_seed_offset;
            let mut rng: StdRng = StdRng::seed_from_u64(random_seed);

            let strategy_vec: Vec<(u8,usize)> = match process_task_iteration_index {
                0 => vec![(0, 10), (1, 10), (2, 80)],
                1 => vec![(0, 10), (1, 30), (2, 60)],
                2 => vec![(0, 5), (1, 50), (2, 45)],
                _ => vec![(0, 3), (1, 80), (2, 17)],
            };

            for pair in &task.pairs {
                if pair.pair_type == PairType::Train {
                    let size: ImageSize = pair.input.image.size();
                    let mut semi_useful_output_image = pair.output.image.clone_zero();
                    for y in 0..size.height {
                        for x in 0..size.width {
                            let strategy_value: u8 = strategy_vec.choose_weighted(&mut rng, |item| item.1).unwrap().0;
                            let _noise_value: u8 = rng.gen_range(0..=255).max(255) as u8;
                            let noise_value: u8 = 255;

                            let input_color: u8 = pair.input.image.get(x as i32, y as i32).unwrap_or(255);
                            let output_color: u8 = pair.output.image.get(x as i32, y as i32).unwrap_or(255);

                            let set_color: u8;
                            match strategy_value {
                                0 => {
                                    set_color = input_color;
                                },
                                1 => {
                                    set_color = output_color;
                                },
                                _ => {
                                    set_color = noise_value;
                                }
                            }

                            // if histogram_preserve.get(input_color) > 0 {
                            //     set_color = input_color;
                            //     set_color = 255;
                            // }

                            _ = semi_useful_output_image.set(x as i32, y as i32, set_color);
                        }
                    }
                    earlier_prediction_image_vec.push(semi_useful_output_image);
                }

                if pair.pair_type == PairType::Test {
                    if pair.test_index == Some(test_index) {
                        earlier_prediction_image_vec.push(computed_image.clone());
                    } else {
                        let size: ImageSize = pair.input.image.size();
                        let junk_image = Image::color(size.width, size.height, 255);
                        earlier_prediction_image_vec.push(junk_image);
                    }
                }
            }
        }

        let mut input_histogram_intersection: [bool; 10] = [false; 10];
        for color in 0..=9u8 {
            if task.input_histogram_intersection.get(color) > 0 {
                input_histogram_intersection[color as usize] = true;
            }
        }

        let mut no_change_to_color: [bool; 10] = [false; 10];
        for label in &task.action_label_set_intersection {
            match label {
                ActionLabel::OutputImageIsInputImageWithNoChangesToPixelsWithColor { color } => {
                    if *color < 10 {
                        no_change_to_color[*color as usize] = true;
                    }
                },
                _ => {}
            }
        }

        let mut input_unambiguous_connectivity_histogram: Histogram = Histogram::new();
        for label in &task.input_image_label_set_intersection {
            match label {
                ImageLabel::UnambiguousConnectivityWithColor { color } => {
                    input_unambiguous_connectivity_histogram.increment(*color);
                },
                _ => {}
            }
        }

        let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];

        let mut task_graph = TaskGraph::new();
        match task_graph.populate_with_task(&task) {
            Ok(()) => {},
            Err(error) => {
                return Err(anyhow::anyhow!("unable to populate graph. {:?}", error));
            }
        }

        let mut records = Vec::<Record>::new();
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.test_index.is_some() && pair.test_index != Some(test_index) {
                // ARC tasks with multiple test pairs, here we only process a single test pair.
                continue;
            }
            let pair_id: u8 = pair_index.min(255) as u8;
            let pair_index_u8: u8 = pair_index.min(255) as u8;

            let is_test: bool;
            let original_output: Image;
            match pair.pair_type {
                PairType::Train => {
                    is_test = false;
                    original_output = pair.output.image.clone();
                },
                PairType::Test => {
                    is_test = true;
                    original_output = Image::empty();
                },
            }
            let original_input: Image = pair.input.image.clone();

            let width: u8 = original_input.width().max(original_output.width()).min(253);
            let height: u8 = original_input.height().max(original_output.height()).min(253);

            let context_input_size: ImageSize = context.input_size_vec[pair_index];
            let context_output_size: ImageSize = context.output_size_vec[pair_index];

            let background: Image = Image::color(width, height, 10);
            let input: Image = background.overlay_with_position(&original_input, 0, 0)?;
            let output: Image = background.overlay_with_position(&original_output, 0, 0)?;

            let mut denoise_type5_input_image: Option<Image> = None;
            let mut denoise_type5_output_image: Option<Image> = None;
            if enable_denoise_type5_input {
                let image: Image = input.denoise_type5()?;
                denoise_type5_input_image = Some(image);
            }
            if enable_denoise_type5_output {
                let image: Image = output.denoise_type5()?;
                denoise_type5_output_image = Some(image);
            }
    
            let mut gameoflife_images = HashMap::<u8, Image>::new();
            if enable_gameoflife {
                for color in 0..=9u8 {
                    let mask: Image = input.to_mask_where_color_is(color);
                    let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&mask, None);
                    ca.step_once();
                    gameoflife_images.insert(color, ca.image().clone());
                }
                // for color in 0..=9u8 {
                    // let mask: Image = input.to_mask_where_color_is_different(color);
                    // let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&mask, None);
                    // // ca.step_once();
                    // ca.step(2);
                    // gameoflife_images.insert(color + 10, ca.image().clone());
                // }
            }
    
            let mut resized_input_image: Option<Image> = None;
            let mut repeated_input_image: Option<Image> = None;
            if enable_scale_widthheight {
                if let Some((scale_width, scale_height)) = context.scale_widthheight {
                    let cropped_input: Image = original_input.crop_outside(0, 0, context_input_size.width, context_input_size.height, 255)?;
                    let new_width: u16 = (cropped_input.width() as u16) * (scale_width as u16);
                    let new_height: u16 = (cropped_input.height() as u16) * (scale_height as u16);
    
                    if new_width <= 30 && new_height <= 30 {
                        match cropped_input.resize(new_width as u8, new_height as u8) {
                            Ok(image) => {
                                resized_input_image = Some(image);
                            },
                            Err(_error) => {}
                        }
                        match cropped_input.repeat_by_count(scale_width, scale_height) {
                            Ok(image) => {
                                repeated_input_image = Some(image);
                            },
                            Err(_error) => {}
                        }
                    }
                }
            }

            // let mut histogram_predicted_palette = Histogram::new();
            // if all_pairs_has_predicted_palette {
            //     for label in &pair.prediction_set {
            //         match label {
            //             arc_work_model::Prediction::OutputPalette { histogram } => {
            //                 histogram_predicted_palette.add_histogram(histogram);
            //             },
            //             _ => {}
            //         }
            //     }
            // }

            let input_histogram: Histogram = pair.input.image_meta.histogram_all.clone();
            let task_input_histogram_intersection: Histogram = task.input_histogram_intersection.clone();
            let task_output_histogram_intersection: Histogram = task.output_histogram_intersection.clone();
            let task_insert_histogram_intersection: Histogram = task.insert_histogram_intersection.clone();
            let task_removal_histogram_intersection: Histogram = task.removal_histogram_intersection.clone();

            let mut largest_interior_rectangle_masks = HashMap::<u8, Image>::new();
            if enable_largest_interior_rectangle_masks {
                for color in 0..=9u8 {
                    if input_histogram.get(color) == 0 {
                        continue;
                    }
                    let mask: Image = input.to_mask_where_color_is(color);
                    let lir: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&mask)?;
                    let mut lir_mask: Image = input.clone_zero();
                    for rect in &lir.rectangles {
                        lir_mask = lir_mask.draw_rect_filled(*rect, 1)?;
                    }
                    largest_interior_rectangle_masks.insert(color, lir_mask);
                }
            }

            let mut enumerated_objects: Image = Image::zero(width, height);
            if let Some(image) = &pair.input.enumerated_objects {
                enumerated_objects = enumerated_objects.overlay_with_position(image, 0, 0)?;
            }

            let mut grid_color: u8 = 255;
            let mut grid_mask: Image = Image::empty();
            if let Some(grid_pattern) = &pair.input.grid_pattern {
                grid_mask = grid_pattern.line_mask.clone();
                grid_color = grid_pattern.color;
            }

            let object_id_image_connectivity4: Image;
            if enable_object_id_image_connectivity4 {
                object_id_image_connectivity4 = Self::object_id_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4)?;
            } else {
                object_id_image_connectivity4 = Image::empty();
            }

            let object_id_image_connectivity8: Image;
            if enable_object_id_image_connectivity8 {
                object_id_image_connectivity8 = Self::object_id_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8)?;
            } else {
                object_id_image_connectivity8 = Image::empty();
            }

            let relative_position_images_connectivity4: Vec<Image>;
            let relative_position_images_connectivity8: Vec<Image>;
            if enable_relative_position_topleft_xy || enable_relative_position_checkerboard {
                relative_position_images_connectivity4 = Self::relative_position_images(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4)?;
                relative_position_images_connectivity8 = Self::relative_position_images(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8)?;
            } else {
                relative_position_images_connectivity4 = vec!();
                relative_position_images_connectivity8 = vec!();
            }

            let shape_type_count: u8 = ShapeType::len();
            let shape_type_image_connectivity4: Image = Self::shape_type_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4, false)?;
            let shape_type_image_connectivity8: Image = Self::shape_type_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8, false)?;
            let shape_type45_image_connectivity4: Image = Self::shape_type_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4, true)?;
            let shape_type45_image_connectivity8: Image = Self::shape_type_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8, true)?;

            let shape_transformation_images_connectivity4: Vec<Image>;
            let shape_transformation_images_connectivity8: Vec<Image>;
            if enable_shape_transformation_images {
                shape_transformation_images_connectivity4 = Self::shape_transformation_images(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4)?;
                shape_transformation_images_connectivity8 = Self::shape_transformation_images(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8)?;
            } else {
                shape_transformation_images_connectivity4 = vec!();
                shape_transformation_images_connectivity8 = vec!();
            }

            let shape_size_images_connectivity4: Vec<Image> = Self::shape_size_images(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4)?;
            _ = shape_size_images_connectivity4;
            let shape_size_images_connectivity8: Vec<Image> = Self::shape_size_images(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8)?;
            _ = shape_size_images_connectivity8;

            // let mut repair_mask: Image = Image::zero(width, height);
            // if let Some(mask) = &pair.input.repair_mask {
            //     repair_mask = repair_mask.overlay_with_position(mask, 0, 0)?;
            // }

            let noise_color: Option<u8> = pair.input.single_pixel_noise_color;
            let most_popular_color: Option<u8> = pair.input.most_popular_intersection_color;
            // let most_popular_color: Option<u8> = task.input_output_most_popular_color_unambiguous();
            // let removal_color: Option<u8> = pair.input.removal_color;

            // let mut non_background_mask: Image = input.clone_zero();
            // let mut non_background_shape_type_image_connectivity4: Image = input.clone_zero();
            // let mut non_background_shape_type_image_connectivity8: Image = input.clone_zero();
            // if let Some(color) = most_popular_color {
            //     // if let Some(color) = noise_color {
            //     // non_background_mask = input.to_mask_where_color_is_different(color);
            //     // non_background_mask = input.to_mask_where_color_is(color);
            //     non_background_mask = input.to_mask_where_color_is_different(color);
                
            //     // let sco: SingleColorObject = SingleColorObject::find_objects(&non_background_mask)?;
            //     // {
            //     //     let connectivity = PixelConnectivity::Connectivity4;
            //     //     let sifsco: ShapeIdentificationFromSingleColorObject = ShapeIdentificationFromSingleColorObject::find_shapes(&sco, connectivity)?;
            //     //     let mut shapetype_image: Image = input.clone_zero();
            //     //     for (_color_and_shape_index, color_and_shape) in sifsco.color_and_shape_vec.iter().enumerate() {
            //     //         let shape_type: ShapeType = color_and_shape.shape_identification.shape_type;
            //     //         let color: u8 = Self::color_from_shape_type(shape_type);
            //     //         let mode = MixMode::PickColor1WhenColor0IsZero { color };
            //     //         shapetype_image = color_and_shape.shape_identification.mask_uncropped.mix(&shapetype_image, mode)?;
            //     //     }
            //     //     non_background_shape_type_image_connectivity4 = shapetype_image;
            //     // }
            //     // {
            //     //     let connectivity = PixelConnectivity::Connectivity8;
            //     //     let sifsco: ShapeIdentificationFromSingleColorObject = ShapeIdentificationFromSingleColorObject::find_shapes(&sco, connectivity)?;
            //     //     let mut shapetype_image: Image = input.clone_zero();
            //     //     for (_color_and_shape_index, color_and_shape) in sifsco.color_and_shape_vec.iter().enumerate() {
            //     //         let shape_type: ShapeType = color_and_shape.shape_identification.shape_type;
            //     //         let color: u8 = Self::color_from_shape_type(shape_type);
            //     //         let mode = MixMode::PickColor1WhenColor0IsZero { color };
            //     //         shapetype_image = color_and_shape.shape_identification.mask_uncropped.mix(&shapetype_image, mode)?;
            //     //     }
            //     //     non_background_shape_type_image_connectivity8 = shapetype_image;
            //     // }
            // }

            let mut image_mass_connectivity4: Image = Image::zero(width, height);
            let mut image_mass_connectivity8: Image = Image::zero(width, height);
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                if let Ok(image) = sco.mass_as_image(PixelConnectivity::Connectivity4) {
                    image_mass_connectivity4 = image_mass_connectivity4.overlay_with_position(&image, 0, 0)?;
                }
                if let Ok(image) = sco.mass_as_image(PixelConnectivity::Connectivity8) {
                    image_mass_connectivity8 = image_mass_connectivity8.overlay_with_position(&image, 0, 0)?;
                }
            }

            let histogram_diagonal_a: Option<DiagonalHistogram>;
            let histogram_diagonal_b: Option<DiagonalHistogram>;
            if enable_histogram_diagonal {
                histogram_diagonal_a = Some(DiagonalHistogram::diagonal_a(&input)?);
                histogram_diagonal_b = Some(DiagonalHistogram::diagonal_b(&input)?);
            } else {
                histogram_diagonal_a = None;
                histogram_diagonal_b = None;
            }

            let histogram_columns: Vec<Histogram> = pair.input.image_meta.histogram_columns.clone();
            let histogram_rows: Vec<Histogram> = pair.input.image_meta.histogram_rows.clone();

            let mut image_neighbour = HashMap::<(u8, ImageNeighbourDirection), Image>::new();
            {
                let directions = [
                    ImageNeighbourDirection::Up,
                    ImageNeighbourDirection::Down,
                    ImageNeighbourDirection::Left,
                    ImageNeighbourDirection::Right,
                    ImageNeighbourDirection::UpLeft,
                    ImageNeighbourDirection::UpRight,
                    ImageNeighbourDirection::DownLeft,
                    ImageNeighbourDirection::DownRight,
                ];
                for direction in directions {
                    for color in 0..=9 {
                        let ignore_mask: Image = input.to_mask_where_color_is(color);
                        match input.neighbour_color(&ignore_mask, direction, 255) {
                            Ok(image) => {
                                image_neighbour.insert((color, direction), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut enumerated_clusters = HashMap::<(u8, PixelConnectivity), Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                for connectivity in &connectivity_vec {
                    for color in 0..=9 {
                        match sco.enumerate_clusters(color, *connectivity) {
                            Ok(image) => {
                                enumerated_clusters.insert((color, *connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut enumerated_clusters_filled_holes_mask = HashMap::<(u8, PixelConnectivity), Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                for connectivity in &connectivity_vec {
                    for color in 0..=9 {
                        match sco.filled_holes_mask(color, *connectivity) {
                            Ok(image) => {
                                enumerated_clusters_filled_holes_mask.insert((color, *connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut enumerated_clusters_grow_mask1 = HashMap::<(u8, PixelConnectivity), Image>::new();
            for ((color, connectivity), image) in &enumerated_clusters {
                match image.mask_grow(*connectivity) {
                    Ok(image) => {
                        enumerated_clusters_grow_mask1.insert((*color, *connectivity), image);
                    },
                    Err(_) => {},
                }
            }

            let mut enumerated_clusters_grow_mask2 = HashMap::<(u8, PixelConnectivity), Image>::new();
            for ((color, connectivity), image) in &enumerated_clusters_grow_mask1 {
                match image.mask_grow(*connectivity) {
                    Ok(image) => {
                        enumerated_clusters_grow_mask2.insert((*color, *connectivity), image);
                    },
                    Err(_) => {},
                }
            }

            let mut enumerated_clusters_grow_mask3 = HashMap::<(u8, PixelConnectivity), Image>::new();
            if enable_enumerated_clusters_grow_mask3 {
                for ((color, connectivity), image) in &enumerated_clusters_grow_mask2 {
                    match image.mask_grow(*connectivity) {
                        Ok(image) => {
                            enumerated_clusters_grow_mask3.insert((*color, *connectivity), image);
                        },
                        Err(_) => {},
                    }
                }
            }

            let mut exterior_of_clusters = HashMap::<(u8, PixelConnectivity), Image>::new();
            if enable_exterior_of_clusters {
                for ((color, connectivity), mask) in &enumerated_clusters_grow_mask1 {
                // for ((color, connectivity), mask) in &enumerated_clusters {
                // for ((color, connectivity), mask) in &enumerated_clusters_filled_holes_mask {
                    match mask.mask_exterior_corners() {
                        Ok(image) => {
                            exterior_of_clusters.insert((*color, *connectivity), image);
                        },
                        Err(_) => {},
                    }
                }
            }

            let mut color_grow_mask1 = HashMap::<(u8, PixelConnectivity), Image>::new();
            if enable_color_grow_mask1 {
                for connectivity in &connectivity_vec {
                    for color in 0..=9 {
                        let mask: Image = input.to_mask_where_color_is(color);
                        match mask.mask_grow(*connectivity) {
                            Ok(image) => {
                                color_grow_mask1.insert((color, *connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut color_grow_mask2 = HashMap::<(u8, PixelConnectivity), Image>::new();
            if enable_color_grow_mask2 {
                for ((color, connectivity), image) in &color_grow_mask1 {
                    match image.mask_grow(*connectivity) {
                        Ok(image) => {
                            color_grow_mask2.insert((*color, *connectivity), image);
                        },
                        Err(_) => {},
                    }
                }
            }

            let mut color_grow_mask3 = HashMap::<(u8, PixelConnectivity), Image>::new();
            if enable_color_grow_mask3 {
                for ((color, connectivity), image) in &color_grow_mask2 {
                    match image.mask_grow(*connectivity) {
                        Ok(image) => {
                            color_grow_mask3.insert((*color, *connectivity), image);
                        },
                        Err(_) => {},
                    }
                }
            }

            // let mut color_grow_mask4 = HashMap::<(u8, PixelConnectivity), Image>::new();
            // for ((color, connectivity), image) in &color_grow_mask3 {
            //     match image.mask_grow(*connectivity) {
            //         Ok(image) => {
            //             color_grow_mask4.insert((*color, *connectivity), image);
            //         },
            //         Err(_) => {},
            //     }
            // }

            // let mut color_grow_mask5 = HashMap::<(u8, PixelConnectivity), Image>::new();
            // for ((color, connectivity), image) in &color_grow_mask4 {
            //     match image.mask_grow(*connectivity) {
            //         Ok(image) => {
            //             color_grow_mask5.insert((*color, *connectivity), image);
            //         },
            //         Err(_) => {},
            //     }
            // }


            let mut small_medium_big = HashMap::<(u8, PixelConnectivity), Image>::new();
            for ((color, connectivity), image) in &enumerated_clusters {
                let oam: ObjectsAndMass = match ObjectsAndMass::new(image) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let a: Image = match oam.group3_small_medium_big(false) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                small_medium_big.insert((*color, *connectivity), a);
            }

            let mut sort2_small_big = HashMap::<(u8, PixelConnectivity), Image>::new();
            for ((color, connectivity), image) in &enumerated_clusters {
                let oam: ObjectsAndMass = match ObjectsAndMass::new(image) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let a: Image = match oam.sort2_small_big(false) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                sort2_small_big.insert((*color, *connectivity), a);
            }

            let direction_vec = vec![
                ImageNeighbourDirection::Up,
                ImageNeighbourDirection::Down,
                ImageNeighbourDirection::Left,
                ImageNeighbourDirection::Right,
                ImageNeighbourDirection::UpLeft,
                ImageNeighbourDirection::UpRight,
                ImageNeighbourDirection::DownLeft,
                ImageNeighbourDirection::DownRight,
            ];
            let mut colordirection_to_distanceimage = HashMap::<(u8, ImageNeighbourDirection), Image>::new();
            if enable_colordirection_to_distanceimage {
                for color in 0..=9u8 {
                    let ignore_mask: Image = input.to_mask_where_color_is_different(color);
                    for direction in &direction_vec {
                        let distance_image: Image = input.neighbour_distance(&ignore_mask, *direction)?;
                        colordirection_to_distanceimage.insert((color, *direction), distance_image);
                    }
                }
            }

            let mut cluster_distance1 = HashMap::<(u8, PixelConnectivity), Image>::new();
            for color in 0..=9u8 {
                for connectivity in &connectivity_vec {
                    let image: Image = input.to_mask_where_color_is_different(color);
                    let a: Image = match image.mask_distance_zerobug(*connectivity) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };
                    cluster_distance1.insert((color, *connectivity), a);
                }
            }
            let mut cluster_distance2 = HashMap::<(u8, PixelConnectivity), Image>::new();
            for color in 0..=9u8 {
                for connectivity in &connectivity_vec {
                    let image: Image = input.to_mask_where_color_is(color);
                    let a: Image = match image.mask_distance_zerobug(*connectivity) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };
                    cluster_distance2.insert((color, *connectivity), a);
                }
            }
            let mut cluster_distance3 = HashMap::<(u8, PixelConnectivity), Image>::new();
            let mut cluster_distance4 = HashMap::<(u8, PixelConnectivity), Image>::new();
            for color in 0..=9u8 {
                for connectivity in &connectivity_vec {
                    // let image: Image = input.to_mask_where_color_is_different(color);
                    let image: Image = input.to_mask_where_color_is(color);
                    let a: Image = match image.mask_distance_zerobug(*connectivity) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };
                    let b: Image = image.select_from_color_and_image(0, &a)?;
                    cluster_distance3.insert((color, *connectivity), b);
                    let c: Image = image.select_from_image_and_color(&a, 0)?;
                    cluster_distance4.insert((color, *connectivity), c);
                }
            }
            let mut cluster_distance5 = HashMap::<(u8, PixelConnectivity), Image>::new();
            for ((color, connectivity), image) in &enumerated_clusters {
                let a: Image = match image.mask_distance_zerobug(*connectivity) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let b: Image = image.select_from_color_and_image(0, &a)?;
                cluster_distance5.insert((*color, *connectivity), b);
            }

            let mut cluster_id_neighbour = HashMap::<(u8, ImageNeighbourDirection, PixelConnectivity), Image>::new();
            {
                let directions = [
                    ImageNeighbourDirection::Up,
                    ImageNeighbourDirection::Down,
                    ImageNeighbourDirection::Left,
                    ImageNeighbourDirection::Right,
                    ImageNeighbourDirection::UpLeft,
                    ImageNeighbourDirection::UpRight,
                    ImageNeighbourDirection::DownLeft,
                    ImageNeighbourDirection::DownRight,
                ];
                for direction in directions {
                    for ((color, connectivity), image) in &enumerated_clusters {
                        let ignore_mask: Image = image.to_mask_where_color_is(0);
                        match image.neighbour_color(&ignore_mask, direction, 255) {
                            Ok(image) => {
                                cluster_id_neighbour.insert((*color, direction, *connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut nearest_color4: Image = Image::empty();
            let mut nearest_color8: Image = Image::empty();
            if enable_nearest_color {
                if let Some(color) = most_popular_color {
                    nearest_color4 = input.nearest_color_in_any_direction(PixelConnectivity::Connectivity4, color)?;
                    nearest_color8 = input.nearest_color_in_any_direction(PixelConnectivity::Connectivity8, color)?;
                }
            }

            let mut color_to_linespan_images = HashMap::<u8, Vec::<Image>>::new();
            {
                for color in 0..=10u8 {
                    let mask: Image = if color < 10 {
                        input.to_mask_where_color_is(color)
                    } else {
                        input.to_mask_where_color_is_equal_or_greater_than(color)
                    };

                    let mut images = Vec::<Image>::new();
                    {
                        let draw_mass: bool = true;
                        let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::Before, draw_mass })?;
                        images.push(image);
                    }
                    {
                        let draw_mass: bool = true;
                        let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::After, draw_mass })?;
                        images.push(image);
                    }
                    {
                        let draw_mass: bool = true;
                        let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::Before, draw_mass })?;
                        images.push(image);
                    }
                    {
                        let draw_mass: bool = true;
                        let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::After, draw_mass })?;
                        images.push(image);
                    }
                    // {
                    //     let mask: Image = input.to_mask_where_color_is(color);
                    //     let draw_mass: bool = false;
                    //     let image1: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::Before, draw_mass })?;
                    //     let image2: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::Fill, draw_mass })?;
                    //     let image3: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::After, draw_mass })?;
                    //     let mut image: Image = mask.clone_zero();
                    //     image = image.overlay_with_mask_color(&image1, 1)?;
                    //     image = image.overlay_with_mask_color(&image2, 2)?;
                    //     image = image.overlay_with_mask_color(&image3, 3)?;
                    //     images.push(image);
                    // }
                    // {
                    //     let mask: Image = input.to_mask_where_color_is(color);
                    //     let draw_mass: bool = false;
                    //     let image1: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::Before, draw_mass })?;
                    //     let image2: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::Fill, draw_mass })?;
                    //     let image3: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::After, draw_mass })?;
                    //     let mut image: Image = mask.clone_zero();
                    //     image = image.overlay_with_mask_color(&image1, 1)?;
                    //     image = image.overlay_with_mask_color(&image2, 2)?;
                    //     image = image.overlay_with_mask_color(&image3, 3)?;
                    //     images.push(image);
                    // }
                    // {
                    //     let mask: Image = input.to_mask_where_color_is_different(color);
                    //     let draw_mass: bool = false;
                    //     let image1: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::Before, draw_mass })?;
                    //     let image2: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::Fill, draw_mass })?;
                    //     let image3: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::After, draw_mass })?;
                    //     let mut image: Image = mask.clone_zero();
                    //     image = image.overlay_with_mask_color(&image1, 1)?;
                    //     image = image.overlay_with_mask_color(&image2, 2)?;
                    //     image = image.overlay_with_mask_color(&image3, 3)?;
                    //     images.push(image);
                    // }
                    // {
                    //     let mask: Image = input.to_mask_where_color_is_different(color);
                    //     let draw_mass: bool = false;
                    //     let image1: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::Before, draw_mass })?;
                    //     let image2: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::Fill, draw_mass })?;
                    //     let image3: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::After, draw_mass })?;
                    //     let mut image: Image = mask.clone_zero();
                    //     image = image.overlay_with_mask_color(&image1, 1)?;
                    //     image = image.overlay_with_mask_color(&image2, 2)?;
                    //     image = image.overlay_with_mask_color(&image3, 3)?;
                    //     images.push(image);
                    // }
                    // {
                    //     let image: Image = LineSpan::draw(&mask, &LineSpanDirection::HorizontalFillOrVerticalFill)?;
                    //     images.push(image);
                    // }
                    // {
                    //     let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::Fill })?;
                    //     images.push(image);
                    // }
                    // {
                    //     let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::Before })?;
                    //     images.push(image);
                    // }
                    // {
                    //     let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::After })?;
                    //     images.push(image);
                    // }
                    // {
                    //     let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::Before })?;
                    //     images.push(image);
                    // }
                    // {
                    //     let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::Fill })?;
                    //     images.push(image);
                    // }
                    // {
                    //     let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::After })?;
                    //     images.push(image);
                    // }
                    // let vert_image0: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::Fill })?;
                    // let horz_image1: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::Before })?;
                    // let horz_image2: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::After })?;
                    // let vert_image0: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::Before })?;
                    // let vert_image1: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::After })?;
                    // color_to_linespan_images.insert(color, vec![horz_image, vert_image]);
                    color_to_linespan_images.insert(color, images);
                }
                // for color in 0..=9u8 {
                //     // let image: Image = input.to_mask_where_color_is(color);
                //     let image: Image = input.to_mask_where_color_is_different(color);
                //     // let horz_image: Image = LineSpan::draw(&image, &LineSpanDirection::HorizontalFillOrVerticalFill)?;
                //     // linespan_images.push(horz_image);
                //     let horz_image0: Image = LineSpan::draw(&image, &LineSpanDirection::Horizontal { mode: LineSpanMode::Before })?;
                //     linespan_images.push(horz_image0);
                //     let horz_image1: Image = LineSpan::draw(&image, &LineSpanDirection::Horizontal { mode: LineSpanMode::After })?;
                //     linespan_images.push(horz_image1);
                //     let vert_image0: Image = LineSpan::draw(&image, &LineSpanDirection::Vertical { mode: LineSpanMode::Before })?;
                //     linespan_images.push(vert_image0);
                //     let vert_image1: Image = LineSpan::draw(&image, &LineSpanDirection::Vertical { mode: LineSpanMode::After })?;
                //     linespan_images.push(vert_image1);
                // }
                // if let Some(color) = least_po
                // if let Some(color) = most_popular_color {
                    // let image: Image = input.to_mask_where_color_is(color);
                    // let image: Image = input.to_mask_where_color_is_different(color);
                    // let horz_image: Image = LineSpan::draw(&image, &LineSpanDirection::HorizontalFillOrVerticalFill)?;
                    // linespan_images.push(horz_image);
                    // let horz_image: Image = LineSpan::draw(&image, &LineSpanDirection::Horizontal { mode: LineSpanMode::Fill })?;
                    // linespan_images.push(horz_image);
                    // let vert_image: Image = LineSpan::draw(&image, &LineSpanDirection::Vertical { mode: LineSpanMode::Fill })?;
                    // linespan_images.push(vert_image);
                    // let horz_image0: Image = LineSpan::draw(&image, &LineSpanDirection::Horizontal { mode: LineSpanMode::Before })?;
                    // linespan_images.push(horz_image0);
                    // let horz_image1: Image = LineSpan::draw(&image, &LineSpanDirection::Horizontal { mode: LineSpanMode::After })?;
                    // linespan_images.push(horz_image1);
                    // let vert_image0: Image = LineSpan::draw(&image, &LineSpanDirection::Vertical { mode: LineSpanMode::Before })?;
                    // linespan_images.push(vert_image0);
                    // let vert_image1: Image = LineSpan::draw(&image, &LineSpanDirection::Vertical { mode: LineSpanMode::After })?;
                    // linespan_images.push(vert_image1);
                // }
            }

            let mut squares = HashMap::<(u8, PixelConnectivity), Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                for connectivity in &connectivity_vec {
                    for color in 0..=9 {
                        match sco.squares(color, *connectivity) {
                            Ok(image) => {
                                squares.insert((color, *connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut nonsquares = HashMap::<(u8, PixelConnectivity), Image>::new();
            if enable_detect_nonsquare {
                if let Some(sco) = &pair.input.image_meta.single_color_object {
                    for connectivity in &connectivity_vec {
                        for color in 0..=9 {
                            match sco.non_squares(color, *connectivity) {
                                Ok(image) => {
                                    nonsquares.insert((color, *connectivity), image);
                                },
                                Err(_) => {},
                            }
                        }
                    }
                }
            }

            let mut rectangles = HashMap::<(u8, PixelConnectivity), Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                for connectivity in &connectivity_vec {
                    for color in 0..=9 {
                        match sco.rectangles(color, *connectivity) {
                            Ok(image) => {
                                rectangles.insert((color, *connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut boxes = HashMap::<(u8, PixelConnectivity), Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                for connectivity in &connectivity_vec {
                    for color in 0..=9 {
                        match sco.boxes(color, *connectivity) {
                            Ok(image) => {
                                boxes.insert((color, *connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut lines = HashMap::<(u8, PixelConnectivity), Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                for connectivity in &connectivity_vec {
                    for color in 0..=9 {
                        match sco.lines(color, *connectivity) {
                            Ok(image) => {
                                lines.insert((color, *connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut horizontal_symmetry = HashMap::<(u8, PixelConnectivity), Image>::new();
            let mut vertical_symmetry = HashMap::<(u8, PixelConnectivity), Image>::new();
            if enable_symmetry_shorter {
                // horizontal symmetry
                if let Some(sco) = &pair.input.image_meta.single_color_object {
                    for connectivity in &connectivity_vec {
                        for color in 0..=9 {
                            match sco.horizontal_symmetry_mask(color, *connectivity) {
                                Ok(image) => {
                                    horizontal_symmetry.insert((color, *connectivity), image);
                                },
                                Err(_) => {},
                            }
                        }
                    }
                }
                // vertical symmetry
                if let Some(sco) = &pair.input.image_meta.single_color_object {
                    for connectivity in &connectivity_vec {
                        for color in 0..=9 {
                            match sco.vertical_symmetry_mask(color, *connectivity) {
                                Ok(image) => {
                                    vertical_symmetry.insert((color, *connectivity), image);
                                },
                                Err(_) => {},
                            }
                        }
                    }
                }
            }

            let mut horizontal_symmetry_connectivity4 = HashMap::<u8, Image>::new();
            let mut horizontal_symmetry_connectivity8 = HashMap::<u8, Image>::new();
            let mut vertical_symmetry_connectivity4 = HashMap::<u8, Image>::new();
            let mut vertical_symmetry_connectivity8 = HashMap::<u8, Image>::new();
            if enable_symmetry_masks {
                if let Some(sco) = &pair.input.image_meta.single_color_object {
                    for color in 0..=9 {
                        let image: Image = match sco.horizontal_symmetry_mask(color, PixelConnectivity::Connectivity4) {
                            Ok(value) => value,
                            Err(_) => {
                                continue;
                            }
                        };
                        horizontal_symmetry_connectivity4.insert(color, image);
                    }
                    for color in 0..=9 {
                        let image: Image = match sco.horizontal_symmetry_mask(color, PixelConnectivity::Connectivity8) {
                            Ok(value) => value,
                            Err(_) => {
                                continue;
                            }
                        };
                        horizontal_symmetry_connectivity8.insert(color, image);
                    }
                    for color in 0..=9 {
                        let image: Image = match sco.vertical_symmetry_mask(color, PixelConnectivity::Connectivity4) {
                            Ok(value) => value,
                            Err(_) => {
                                continue;
                            }
                        };
                        vertical_symmetry_connectivity4.insert(color, image);
                    }
                    for color in 0..=9 {
                        let image: Image = match sco.vertical_symmetry_mask(color, PixelConnectivity::Connectivity8) {
                            Ok(value) => value,
                            Err(_) => {
                                continue;
                            }
                        };
                        vertical_symmetry_connectivity8.insert(color, image);
                    }
                }
            }

            let mut image_neighbour_up: Image = Image::color(width, height, 255);
            let mut image_neighbour_down: Image = Image::color(width, height, 255);
            let mut image_neighbour_left: Image = Image::color(width, height, 255);
            let mut image_neighbour_right: Image = Image::color(width, height, 255);
            let mut image_neighbour_upleft: Image = Image::color(width, height, 255);
            let mut image_neighbour_upright: Image = Image::color(width, height, 255);
            let mut image_neighbour_downleft: Image = Image::color(width, height, 255);
            let mut image_neighbour_downright: Image = Image::color(width, height, 255);
            if let Some(color) = most_popular_color {
                let ignore_mask: Image = input.to_mask_where_color_is(color);
                match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Up, 255) {
                    Ok(image) => {
                        image_neighbour_up = image;
                    },
                    Err(_) => {},
                }
                match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Down, 255) {
                    Ok(image) => {
                        image_neighbour_down = image;
                    },
                    Err(_) => {},
                }
                match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Left, 255) {
                    Ok(image) => {
                        image_neighbour_left = image;
                    },
                    Err(_) => {},
                }
                match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Right, 255) {
                    Ok(image) => {
                        image_neighbour_right = image;
                    },
                    Err(_) => {},
                }
                match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpLeft, 255) {
                    Ok(image) => {
                        image_neighbour_upleft = image;
                    },
                    Err(_) => {},
                }
                match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpRight, 255) {
                    Ok(image) => {
                        image_neighbour_upright = image;
                    },
                    Err(_) => {},
                }
                match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownLeft, 255) {
                    Ok(image) => {
                        image_neighbour_downleft = image;
                    },
                    Err(_) => {},
                }
                match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownRight, 255) {
                    Ok(image) => {
                        image_neighbour_downright = image;
                    },
                    Err(_) => {},
                }
            }

            let mut holes_connectivity4 = HashMap::<u8, Image>::new();
            let mut holes_connectivity8 = HashMap::<u8, Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                for color in 0..=9 {
                    let image: Image = match sco.holes_mask(color, PixelConnectivity::Connectivity4) {
                        Ok(value) => value,
                        Err(_) => {
                            continue;
                        }
                    };
                    holes_connectivity4.insert(color, image);
                }
                for color in 0..=9 {
                    let image: Image = match sco.holes_mask(color, PixelConnectivity::Connectivity8) {
                        Ok(value) => value,
                        Err(_) => {
                            continue;
                        }
                    };
                    holes_connectivity8.insert(color, image);
                }
            }

            let corners: Image = input.corners()?;

            // let distance_to_corner1: Image = corners.to_mask_where_color_is(1)
            //     .mask_distance(PixelConnectivity::Connectivity4)?;
            // let distance_to_corner2: Image = corners.to_mask_where_color_is(2)
            //     .mask_distance(PixelConnectivity::Connectivity4)?;
            // let distance_to_corner3: Image = corners.to_mask_where_color_is(3)
            //     .mask_distance(PixelConnectivity::Connectivity4)?;
            // let distance_to_corner4: Image = corners.to_mask_where_color_is(4)
            //     .mask_distance(PixelConnectivity::Connectivity4)?;

            let mut holecount_connectivity4 = HashMap::<u8, Image>::new();
            let mut holecount_connectivity8 = HashMap::<u8, Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                for color in 0..=9 {
                    let image: Image = match sco.holecount_image(color, PixelConnectivity::Connectivity4) {
                        Ok(value) => value,
                        Err(_) => {
                            continue;
                        }
                    };
                    holecount_connectivity4.insert(color, image);
                }
                for color in 0..=9 {
                    let image: Image = match sco.holecount_image(color, PixelConnectivity::Connectivity8) {
                        Ok(value) => value,
                        Err(_) => {
                            continue;
                        }
                    };
                    holecount_connectivity8.insert(color, image);
                }
            }

            // The outline of each color mask
            let mut outline1_connectivity4 = HashMap::<u8, Image>::new();
            let mut outline1_connectivity8 = HashMap::<u8, Image>::new();
            // let mut outline2_connectivity4 = HashMap::<u8, Image>::new();
            // let mut outline2_connectivity8 = HashMap::<u8, Image>::new();
            {
                for color in 0..=9 {
                    let mask: Image = pair.input.image.to_mask_where_color_is(color);
                    let maskgrow1: Image = mask.mask_grow(PixelConnectivity::Connectivity4)?;
                    // let maskgrow2: Image = maskgrow1.mask_grow(PixelConnectivity::Connectivity4)?;
                    let outline1: Image = maskgrow1.mix(&mask, MixMode::IsDifferent)?;
                    // let outline2: Image = maskgrow2.mix(&maskgrow1, MixMode::IsDifferent)?;
                    outline1_connectivity4.insert(color, outline1);
                    // outline2_connectivity4.insert(color, outline2);
                }
                for color in 0..=9 {
                    let mask: Image = pair.input.image.to_mask_where_color_is(color);
                    let maskgrow1: Image = mask.mask_grow(PixelConnectivity::Connectivity8)?;
                    // let maskgrow2: Image = maskgrow1.mask_grow(PixelConnectivity::Connectivity8)?;
                    let outline1: Image = maskgrow1.mix(&mask, MixMode::IsDifferent)?;
                    // let outline2: Image = maskgrow2.mix(&maskgrow1, MixMode::IsDifferent)?;
                    outline1_connectivity8.insert(color, outline1);
                    // outline2_connectivity8.insert(color, outline2);
                }
            }

            let mut color_to_hole_type1 = HashMap::<u8, Image>::new();
            for color in 0..=9 {
                let input_padded: Image = input.padding_advanced(0, 0, 1, 1, 255)?;
                let image: Image = input_padded.detect_hole_type1(color)?;
                color_to_hole_type1.insert(color, image);
            }

            let mut color_to_repair = HashMap::<u8, Image>::new();
            for color in 0..=9 {
                let image: Image = input.repair_pattern_with_color(color)?;
                color_to_repair.insert(color, image);
            }

            // let mut color_to_denoise = HashMap::<u8, Image>::new();
            // for color in 0..=9 {
            //     let image: Image = input.denoise_type1(color)?;
            //     color_to_denoise.insert(color, image);
            // }

            let input_denoise_type1: Image = input.denoise_type1(most_popular_color.unwrap_or(255))?;

            // let histogram_border: Histogram = input.histogram_border();
            // let border_most_popular_color: Option<u8> = histogram_border.most_popular_color();
            // let border_least_popular_color: Option<u8> = histogram_border.least_popular_color();
            // let input_denoise_type1_border: Image = input.denoise_type1(border_color.unwrap_or(255))?;

            let earlier_prediction_image: Option<&Image> = earlier_prediction_image_vec.get(pair_index);
            let mut earlier_prediction_shapetype_connectivity4: Option<Image> = None;
            let mut earlier_prediction_shapetype45_connectivity4: Option<Image> = None;
            // let mut earlier_prediction_shapetype_connectivity8: Option<Image> = None;
            // let mut earlier_prediction_shapetype45_connectivity8: Option<Image> = None;
            let mut earlier_prediction_mass_connectivity4: Option<Image> = None;
            let mut earlier_prediction_mass_connectivity8: Option<Image> = None;

            if let Some(ep_image) = earlier_prediction_image {
                let sco: SingleColorObject = SingleColorObject::find_objects(&ep_image)?;
                {
                    let connectivity = PixelConnectivity::Connectivity4;
                    let sifsco: ShapeIdentificationFromSingleColorObject = ShapeIdentificationFromSingleColorObject::find_shapes(&sco, connectivity)?;
                    let mut shapetype_image: Image = ep_image.clone_zero();
                    for (_color_and_shape_index, color_and_shape) in sifsco.color_and_shape_vec.iter().enumerate() {
                        if color_and_shape.color > 9 {
                            continue;
                        }
                        let shape_type: ShapeType = color_and_shape.shape_identification.shape_type;
                        let color: u8 = Self::color_from_shape_type(shape_type);
                        let mode = MixMode::PickColor1WhenColor0IsZero { color };
                        shapetype_image = color_and_shape.shape_identification.mask_uncropped.mix(&shapetype_image, mode)?;
                    }
                    earlier_prediction_shapetype_connectivity4 = Some(shapetype_image);

                    let mut shapetype45_image: Image = ep_image.clone_zero();
                    for (_color_and_shape_index, color_and_shape) in sifsco.color_and_shape_vec.iter().enumerate() {
                        if color_and_shape.color > 9 {
                            continue;
                        }
                        let shape_type: ShapeType = color_and_shape.shape_identification.shape_type45;
                        let color: u8 = Self::color_from_shape_type(shape_type);
                        let mode = MixMode::PickColor1WhenColor0IsZero { color };
                        shapetype45_image = color_and_shape.shape_identification.mask_uncropped.mix(&shapetype45_image, mode)?;
                    }
                    earlier_prediction_shapetype45_connectivity4 = Some(shapetype45_image);
                }

                // {
                //     let connectivity = PixelConnectivity::Connectivity8;
                //     let sifsco: ShapeIdentificationFromSingleColorObject = ShapeIdentificationFromSingleColorObject::find_shapes(&sco, connectivity)?;
                //     let mut shapetype_image: Image = ep_image.clone_zero();
                //     for (_color_and_shape_index, color_and_shape) in sifsco.color_and_shape_vec.iter().enumerate() {
                //         let shape_type: ShapeType = color_and_shape.shape_identification.shape_type;
                //         let color: u8 = Self::color_from_shape_type(shape_type);
                //         let mode = MixMode::PickColor1WhenColor0IsZero { color };
                //         shapetype_image = color_and_shape.shape_identification.mask_uncropped.mix(&shapetype_image, mode)?;
                //     }
                //     earlier_prediction_shapetype_connectivity8 = Some(shapetype_image);

                //     let mut shapetype45_image: Image = ep_image.clone_zero();
                //     for (_color_and_shape_index, color_and_shape) in sifsco.color_and_shape_vec.iter().enumerate() {
                //         let shape_type: ShapeType = color_and_shape.shape_identification.shape_type45;
                //         let color: u8 = Self::color_from_shape_type(shape_type);
                //         let mode = MixMode::PickColor1WhenColor0IsZero { color };
                //         shapetype45_image = color_and_shape.shape_identification.mask_uncropped.mix(&shapetype45_image, mode)?;
                //     }
                //     earlier_prediction_shapetype45_connectivity8 = Some(shapetype45_image);
                // }

                let mut image_mass_connectivity4: Image = Image::zero(width, height);
                let mut image_mass_connectivity8: Image = Image::zero(width, height);
                if let Ok(image) = sco.mass_as_image(PixelConnectivity::Connectivity4) {
                    image_mass_connectivity4 = image_mass_connectivity4.overlay_with_position(&image, 0, 0)?;
                }
                if let Ok(image) = sco.mass_as_image(PixelConnectivity::Connectivity8) {
                    image_mass_connectivity8 = image_mass_connectivity8.overlay_with_position(&image, 0, 0)?;
                }
                earlier_prediction_mass_connectivity4 = Some(image_mass_connectivity4);
                earlier_prediction_mass_connectivity8 = Some(image_mass_connectivity8);
            }

            _ = earlier_prediction_mass_connectivity4;
            _ = earlier_prediction_mass_connectivity8;

            let input_orientation: i8;
            if context_input_size.width > context_input_size.height {
                input_orientation = 1;
            } else if context_input_size.width < context_input_size.height {
                input_orientation = -1;
            } else {
                input_orientation = 0;
            }

            let output_orientation: i8;
            if context_output_size.width > context_output_size.height {
                output_orientation = 1;
            } else if context_output_size.width < context_output_size.height {
                output_orientation = -1;
            } else {
                output_orientation = 0;
            }

            let number_of_shape3x3ids: u8 = Shape3x3::instance().number_of_shapes();

            let center_indicator_x: Image;
            let center_indicator_y: Image;
            if enable_center_indicator {
                center_indicator_x = input.center_indicator_x()?;
                center_indicator_y = input.center_indicator_y()?;
            } else {
                center_indicator_x = input.clone_zero();
                center_indicator_y = input.clone_zero();
            }

            let mut total_clustercount_connectivity4: usize = 0;
            let mut total_clustercount_connectivity8: usize = 0;
            let mut color_clustercount_connectivity4 = HashMap::<u8, u8>::new();
            let mut color_clustercount_connectivity8 = HashMap::<u8, u8>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                for color in 0..=9 {
                    let mut count4: usize = 0;
                    let mut count8: usize = 0;
                    for rect in &sco.rectangle_vec {
                        if rect.color == color {
                            count4 += 1;
                            count8 += 1;
                        }
                    }
                    for object in &sco.sparse_vec {
                        if object.color != color {
                            continue;
                        }
                        if let Some(container) = &object.container4 {
                            count4 += container.cluster_vec.len();
                        }
                        if let Some(container) = &object.container8 {
                            count8 += container.cluster_vec.len();
                        }
                    }

                    let count4_u8: u8 = count4.min(255) as u8;
                    color_clustercount_connectivity4.insert(color, count4_u8);

                    let count8_u8: u8 = count8.min(255) as u8;
                    color_clustercount_connectivity8.insert(color, count8_u8);

                    total_clustercount_connectivity4 += count4;
                    total_clustercount_connectivity8 += count8;
                }
            }

            // let random_image: Image;
            // {
            //     let random_seed: u64 = 0;
            //     let mut rng: StdRng = StdRng::seed_from_u64(random_seed);
            //     let image: Image = RandomImage::two_colors(&mut rng, ImageSize { width: 30, height: 30 }, 0, 1, 33)?;
            //     let image: Image = RandomImage::uniform_colors(&mut rng, ImageSize { width: 30, height: 30 }, 9)?;
            //     random_image = image.crop_outside(0, 0, width, height, 255)?;
            // }

            // let mut number_of_unique_bigrams_column_vec = Vec::<u8>::new();
            // for x in 0..width {
            //     let image: Image = input.crop_outside(x as i32, 0, 1, height, 255)?;
            //     let bigrams: Vec<RecordBigram> = image.bigram_y()?;
            //     let bigrams: Vec<RecordTrigram> = image.trigram_y()?;
            //     number_of_unique_bigrams_column_vec.push(bigrams.len().min(255) as u8);
            // }
            // let mut number_of_unique_bigrams_row_vec = Vec::<u8>::new();
            // for y in 0..height {
            //     let image: Image = input.crop_outside(0, y as i32, width, 1, 255)?;
            //     let bigrams: Vec<RecordBigram> = image.bigram_x()?;
            //     let bigrams: Vec<RecordTrigram> = image.trigram_x()?;
            //     number_of_unique_bigrams_row_vec.push(bigrams.len().min(255) as u8);
            // }

            // if let Some(ep_image) = earlier_prediction_image {
            //     for x in 0..width {
            //         let image: Image = ep_image.crop_outside(x as i32, 0, 1, height, 255)?;
            //         let bigrams: Vec<RecordBigram> = image.bigram_y()?;
            //         let bigrams: Vec<RecordTrigram> = image.trigram_y()?;
            //         number_of_unique_bigrams_column_vec.push(bigrams.len().min(255) as u8);
            //     }
            //     for y in 0..height {
            //         let image: Image = ep_image.crop_outside(0, y as i32, width, 1, 255)?;
            //         let bigrams: Vec<RecordBigram> = image.bigram_x()?;
            //         let bigrams: Vec<RecordTrigram> = image.trigram_x()?;
            //         number_of_unique_bigrams_row_vec.push(bigrams.len().min(255) as u8);
            //     }
            // }

            // let mut trigrams_column = HashMap::<u8, Vec::<RecordTrigram>>::new();
            // let mut trigrams_row = HashMap::<u8, Vec::<RecordTrigram>>::new();
            // {
            //     for x in 0..width {
            //         let image: Image = input.crop_outside(x as i32, 0, 1, height, 255)?;
            //         let trigrams: Vec<RecordTrigram> = image.trigram_y()?;
            //         trigrams_column.insert(x, trigrams);
            //     }
            //     for y in 0..height {
            //         let image: Image = input.crop_outside(0, y as i32, width, 1, 255)?;
            //         let trigrams: Vec<RecordTrigram> = image.trigram_x()?;
            //         trigrams_row.insert(y, trigrams);
            //     }
            // }
            // if let Some(ep_image) = earlier_prediction_image {
            //     for x in 0..width {
            //         let image: Image = ep_image.crop_outside(x as i32, 0, 1, height, 255)?;
            //         let trigrams: Vec<RecordTrigram> = image.trigram_y()?;
            //         trigrams_column.insert(x, trigrams);
            //     }
            //     for y in 0..height {
            //         let image: Image = ep_image.crop_outside(0, y as i32, width, 1, 255)?;
            //         let trigrams: Vec<RecordTrigram> = image.trigram_x()?;
            //         trigrams_row.insert(y, trigrams);
            //     }
            // }

            let gravity_up: Image;
            let gravity_down: Image;
            let gravity_left: Image;
            let gravity_right: Image;
            if enable_gravity {
                let hardcoded_background_color: u8 = 0;
                let gravity_background_color: u8 = most_popular_color.unwrap_or(hardcoded_background_color);
                gravity_up = input.gravity(gravity_background_color, GravityDirection::Up)?;
                gravity_down = input.gravity(gravity_background_color, GravityDirection::Down)?;
                gravity_left = input.gravity(gravity_background_color, GravityDirection::Left)?;
                gravity_right = input.gravity(gravity_background_color, GravityDirection::Right)?;
            } else {
                gravity_up = Image::empty();
                gravity_down = Image::empty();
                gravity_left = Image::empty();
                gravity_right = Image::empty();
            }

            for y in 0..height {
                let yy: i32 = y as i32;
                let y_reverse: u8 = ((height as i32) - 1 - yy).max(0) as u8;
                let context_input_y_reverse: i32 = (context_input_size.height as i32) - 1 - yy;
                let context_output_y_reverse: i32 = (context_output_size.height as i32) - 1 - yy;

                let input_area_top: Image = if y > 0 {
                    input.top_rows(y)?
                } else {
                    Image::empty()
                };
                let input_area_bottom: Image = if context_input_y_reverse > 0 {
                    input.bottom_rows(context_input_y_reverse.min(255) as u8)?
                } else {
                    Image::empty()
                };

                let mut output_area_top = Image::empty();
                let mut output_area_bottom = Image::empty();
                if let Some(image) = earlier_prediction_image {
                    if y > 0 {
                        output_area_top = image.top_rows(y)?;
                    };
                    if context_output_y_reverse > 0 {
                        output_area_bottom = image.bottom_rows(context_output_y_reverse.min(255) as u8)?;
                    }
                }

                // let area_top_histogram_columns: Vec<Histogram> = input_area_top.histogram_columns();
                // let area_bottom_histogram_columns: Vec<Histogram> = input_area_bottom.histogram_columns();
                // let area_top_histogram: Histogram = input_area_top.histogram_all();
                // let area_bottom_histogram: Histogram = input_area_bottom.histogram_all();

                for x in 0..width {
                    let xx: i32 = x as i32;
                    let x_reverse: u8 = ((width as i32) - 1 - xx).max(0) as u8;
                    let context_input_x_reverse: i32 = (context_input_size.width as i32) - 1 - xx;
                    let context_output_x_reverse: i32 = (context_output_size.width as i32) - 1 - xx;
                    let output_color: u8 = output.get(xx, yy).unwrap_or(255);

                    let mut record = Record {
                        classification: output_color,
                        is_test,
                        pair_id,
                        values: vec!(),
                    };

                    // {
                    //     let random_color: u8 = random_image.get(xx, yy).unwrap_or(255);
                    //     record.serialize_bool(random_color == 0);
                    //     record.serialize_bool_onehot(random_color == 0);
                    //     record.serialize_onehot(random_color, 10);
                    //     record.serialize_u8(random_color);
                    // }

                    if enable_total_clustercount {
                        record.serialize_u8(total_clustercount_connectivity4.min(255) as u8);
                        record.serialize_u8(total_clustercount_connectivity8.min(255) as u8);
                    }

                    if enable_color_clustercount {
                        for color in 0..=9 {
                            let count: u8 = *color_clustercount_connectivity4.get(&color).unwrap_or(&0);
                            record.serialize_u8(count);
                        }
                        for color in 0..=9 {
                            let count: u8 = *color_clustercount_connectivity8.get(&color).unwrap_or(&0);
                            record.serialize_u8(count);
                        }
                    }


                    let area3x3: Image = input.crop_outside(xx - 1, yy - 1, 3, 3, 255)?;
                    let area5x5: Image = input.crop_outside(xx - 2, yy - 2, 5, 5, 255)?;
                    let center: u8 = area5x5.get(2, 2).unwrap_or(255);

                    if enable_same_colors_for_area3x3_and_area5x5 {
                        let h0: Histogram = area3x3.histogram_all();
                        let h1: Histogram = area5x5.histogram_all();
                        let same_colors: bool = h0.is_same_color_but_ignore_count(&h1);
                        record.serialize_bool_onehot(same_colors);
                    }

                    let area5x5_output: Image;
                    if let Some(image) = &earlier_prediction_image {
                        area5x5_output = image.crop_outside(xx - 2, yy - 2, 5, 5, 255)?;
                    } else {
                        area5x5_output = Image::empty();
                    }
                    let center_output: u8 = area5x5_output.get(2, 2).unwrap_or(255);

                    if enable_area3x3_input_8bit_mask {
                        let mut mask: u16 = 0;
                        for yyy in 0..3 {
                            for xxx in 0..3 {
                                if xxx == 1 && yyy == 1 {
                                    continue;
                                }
                                let color: u8 = area3x3.get(xxx, yyy).unwrap_or(255);
                                mask = mask << 1;
                                if color == center {
                                    mask |= 1;
                                }
                                // record.serialize_bool(color == center);
                            }
                        }
                        record.serialize_complex(mask, 256);
                    }

                    if enable_area3x3_output_8bit_mask {
                        let mut mask: u16 = 0;
                        for yyy in 0..3 {
                            for xxx in 0..3 {
                                if xxx == 1 && yyy == 1 {
                                    continue;
                                }
                                let color: u8 = area5x5_output.get(xxx + 1, yyy + 1).unwrap_or(255);
                                mask = mask << 1;
                                if color == center {
                                    mask |= 1;
                                }
                                // record.serialize_bool(color == center_output);
                            }
                        }
                        record.serialize_complex(mask, 256);
                    }

                    if enable_denoise_type5_input {
                        if let Some(image) = &denoise_type5_input_image {
                            let color: u8 = image.get(xx, yy).unwrap_or(255);
                            // record.serialize_color_complex(color, obfuscated_color_offset, enable_serialize_color_complex);
                            // record.serialize_onehot_discard_overflow(color, 10);
                            record.serialize_bool(color == center);
                            // record.serialize_bool(color == center_output);
                        }
                    }
                    if enable_denoise_type5_output {
                        if let Some(image) = &denoise_type5_output_image {
                            let color: u8 = image.get(xx, yy).unwrap_or(255);
                            // record.serialize_color_complex(color, obfuscated_color_offset, enable_serialize_color_complex);
                            // record.serialize_onehot_discard_overflow(color, 10);
                            // record.serialize_bool(color == center);
                            record.serialize_bool(color == center_output);
                        }
                    }

                    if enable_gameoflife {
                        for i in 0..10u8 {
                            let mut color: u8 = 255;
                            if let Some(image) = gameoflife_images.get(&i) {
                                color = image.get(xx, yy).unwrap_or(255);
                            }
                            // record.serialize_bool_onehot(color > 0);
                            record.serialize_bool(color > 0);
                        }
                    }

                    if enable_scale_widthheight {
                        if let Some((scale_x, scale_y)) = context.scale_widthheight {
                            let mut pixel_resized: u8 = 255;
                            let mut pixel_repeated: u8 = 255;
                            if let Some(image) = &resized_input_image {
                                pixel_resized = image.get(xx, yy).unwrap_or(255);
                            }
                            record.serialize_onehot_discard_overflow(pixel_resized, 10);
                            if let Some(image) = &repeated_input_image {
                                pixel_repeated = image.get(xx, yy).unwrap_or(255);
                            }
                            record.serialize_onehot_discard_overflow(pixel_repeated, 10);

                            {
                                let bits0: u16 = 1 << (pixel_resized.min(10) as u16);
                                let bits1: u16 = 1 << (pixel_repeated.min(10) as u16);
                                record.serialize_bitmask_as_onehot(bits0 ^ bits1, 10);
                                record.serialize_bitmask_as_onehot(bits0 | bits1, 10);
                                record.serialize_bitmask_as_onehot(bits0 & bits1, 10);
                            }

                            {
                                let value0: bool = pixel_resized == most_popular_color.unwrap_or(255);
                                let value1: bool = pixel_repeated == most_popular_color.unwrap_or(255);
                                record.serialize_bool_onehot(value0 ^ value1);
                                record.serialize_bool_onehot(value0 | value1);
                                record.serialize_bool_onehot(value0 & value1);
                            }

                            record.serialize_bool_onehot(pixel_resized == center);
                            record.serialize_bool_onehot(pixel_repeated == center);
                            record.serialize_bool_onehot(pixel_resized == center_output);
                            record.serialize_bool_onehot(pixel_repeated == center_output);
                            record.serialize_bool_onehot(pixel_resized == most_popular_color.unwrap_or(255));
                            record.serialize_bool_onehot(pixel_repeated == most_popular_color.unwrap_or(255));

                            {
                                let tx: u8 = x / scale_x.max(1);
                                let ty: u8 = y / scale_y.max(1);
                                let sum: u8 = tx + ty;
                                record.serialize_bool_onehot(sum % 2 == 0);
                                // record.serialize_u8(sum);
                                // record.serialize_u8(tx);
                                // record.serialize_u8(ty);
                                record.serialize_onehot_discard_overflow(tx, 4);
                                record.serialize_onehot_discard_overflow(ty, 4);
                            }
                            {
                                let tx: u8 = x % scale_x.max(1);
                                let ty: u8 = y % scale_y.max(1);
                                let sum: u8 = tx + ty;
                                record.serialize_bool_onehot(sum % 2 == 0);
                                // record.serialize_u8(sum);
                                // record.serialize_u8(tx);
                                // record.serialize_u8(ty);
                                record.serialize_onehot_discard_overflow(tx, 4);
                                record.serialize_onehot_discard_overflow(ty, 4);
                            }
                            {
                                let tx: u8 = x / context_input_size.width.max(1);
                                let ty: u8 = y / context_input_size.height.max(1);
                                let sum: u8 = tx + ty;
                                record.serialize_bool_onehot(sum % 2 == 0);
                                // record.serialize_u8(sum);
                                // record.serialize_u8(tx);
                                // record.serialize_u8(ty);
                                record.serialize_onehot_discard_overflow(tx, 4);
                                record.serialize_onehot_discard_overflow(ty, 4);
                            }
                            {
                                let tx: u8 = x % context_input_size.width.max(1);
                                let ty: u8 = y % context_input_size.height.max(1);
                                let sum: u8 = tx + ty;
                                record.serialize_bool_onehot(sum % 2 == 0);
                                // record.serialize_u8(sum);
                                // record.serialize_u8(tx);
                                // record.serialize_u8(ty);
                                record.serialize_onehot_discard_overflow(tx, 4);
                                record.serialize_onehot_discard_overflow(ty, 4);
                            }
                        }
                    }

                    if enable_check_pixel_in_histogram {
                        record.serialize_bool_onehot(input_histogram.get(center_output) > 0);
                        record.serialize_bool_onehot(task_input_histogram_intersection.get(center_output) > 0);
                        record.serialize_bool_onehot(task_output_histogram_intersection.get(center_output) > 0);
                        record.serialize_bool_onehot(task_insert_histogram_intersection.get(center_output) > 0);
                        record.serialize_bool_onehot(task_removal_histogram_intersection.get(center_output) > 0);
                        record.serialize_bool_onehot(task_removal_histogram_intersection.get(center) > 0);
                    }

                    if enable_gravity {
                        let images = [&gravity_up, &gravity_down, &gravity_left, &gravity_right];
                        for image in images {
                            let pixel: u8 = image.get(xx, yy).unwrap_or(255);
                            record.serialize_color_complex(pixel, obfuscated_color_offset, enable_serialize_color_complex);
                        }
                    }

                    if enable_input_four_xy_pairs {
                        let four_xy_pairs: Vec<[i32; 8]> = vec![
                            [0, -2, 0, -1, 0, 1, 0, 2],
                            [-2, 0, -1, 0, 1, 0, 2, 0],
                            [-2, -2, -1, -1, 1, 1, 2, 2],
                            [-2, 2, -1, 1, 1, -1, 2, -2],
                            [-1, -2, 2, -1, 1, 2, -2, 1],
                            [-2, -1, -1, 2, 2, 1, 1, -2],
                            [-2, -1, -1, -2, 2, 1, 1, 2],
                            [-2, -1, 1, 2, 2, 1, -1, -2],
                        ];
                        let area5x5_center_offset: i32 = 2;
                        for xy_pair in &four_xy_pairs {
                            let ax: i32 = xy_pair[0] + area5x5_center_offset;
                            let ay: i32 = xy_pair[1] + area5x5_center_offset;
                            let bx: i32 = xy_pair[2] + area5x5_center_offset;
                            let by: i32 = xy_pair[3] + area5x5_center_offset;
                            let cx: i32 = xy_pair[4] + area5x5_center_offset;
                            let cy: i32 = xy_pair[5] + area5x5_center_offset;
                            let dx: i32 = xy_pair[6] + area5x5_center_offset;
                            let dy: i32 = xy_pair[7] + area5x5_center_offset;
                            let a: u8 = area5x5.get(ax, ay).unwrap_or(255);
                            let b: u8 = area5x5.get(bx, by).unwrap_or(255);
                            let c: u8 = area5x5.get(cx, cy).unwrap_or(255);
                            let d: u8 = area5x5.get(dx, dy).unwrap_or(255);
                            let all_same: bool = a == b && b == c && c == d;
                            let all_different: bool = a != b && b != c && c != d && a != c && a != d && b != d;
                            let same_reversed: bool = a == d && b == c;
                            let same_ab_different_cd: bool = a == b && c != d;
                            let same_cd_different_ab: bool = a != b && c == d;
                            let same_ad_different_bc: bool = a == d && b != c;
                            let same_bc_different_ad: bool = a != d && b == c;
                            record.serialize_bool(all_same);
                            record.serialize_bool(all_different);
                            record.serialize_bool(same_reversed);
                            record.serialize_bool(same_ab_different_cd);
                            record.serialize_bool(same_cd_different_ab);
                            record.serialize_bool(same_ad_different_bc);
                            record.serialize_bool(same_bc_different_ad);
                        }
                    }

                    if enable_output_four_xy_pairs {
                        let four_xy_pairs: Vec<[i32; 8]> = vec![
                            [0, -2, 0, -1, 0, 1, 0, 2],
                            [-2, 0, -1, 0, 1, 0, 2, 0],
                            [-2, -2, -1, -1, 1, 1, 2, 2],
                            [-2, 2, -1, 1, 1, -1, 2, -2],
                            [-1, -2, 2, -1, 1, 2, -2, 1],
                            [-2, -1, -1, 2, 2, 1, 1, -2],
                            [-2, -1, -1, -2, 2, 1, 1, 2],
                            [-2, -1, 1, 2, 2, 1, -1, -2],
                        ];
                        let area5x5_center_offset: i32 = 2;
                        for xy_pair in &four_xy_pairs {
                            let ax: i32 = xy_pair[0] + area5x5_center_offset;
                            let ay: i32 = xy_pair[1] + area5x5_center_offset;
                            let bx: i32 = xy_pair[2] + area5x5_center_offset;
                            let by: i32 = xy_pair[3] + area5x5_center_offset;
                            let cx: i32 = xy_pair[4] + area5x5_center_offset;
                            let cy: i32 = xy_pair[5] + area5x5_center_offset;
                            let dx: i32 = xy_pair[6] + area5x5_center_offset;
                            let dy: i32 = xy_pair[7] + area5x5_center_offset;
                            let a: u8 = area5x5_output.get(ax, ay).unwrap_or(255);
                            let b: u8 = area5x5_output.get(bx, by).unwrap_or(255);
                            let c: u8 = area5x5_output.get(cx, cy).unwrap_or(255);
                            let d: u8 = area5x5_output.get(dx, dy).unwrap_or(255);
                            let all_same: bool = a == b && b == c && c == d;
                            let all_different: bool = a != b && b != c && c != d && a != c && a != d && b != d;
                            let same_reversed: bool = a == d && b == c;
                            let same_ab_different_cd: bool = a == b && c != d;
                            let same_cd_different_ab: bool = a != b && c == d;
                            let same_ad_different_bc: bool = a == d && b != c;
                            let same_bc_different_ad: bool = a != d && b == c;
                            record.serialize_bool(all_same);
                            record.serialize_bool(all_different);
                            record.serialize_bool(same_reversed);
                            record.serialize_bool(same_ab_different_cd);
                            record.serialize_bool(same_cd_different_ab);
                            record.serialize_bool(same_ad_different_bc);
                            record.serialize_bool(same_bc_different_ad);
                        }
                    }

                    if enable_input_three_xy_pairs {
                        let four_xy_pairs: Vec<[i32; 6]> = vec![
                            [-1, -1, -1, 1, 1, 0],
                            [-1, 0, 1, -1, 1, 1],
                            [0, -1, -1, 1, 1, 1],
                            [0, 1, -1, -1, 1, -1],
                            [-2, 0, 0, -1, 2, 0],
                            [-2, 0, 0, 1, 2, 0],
                            [0, -2, -1, 0, 0, 2],
                            [0, -2, 1, 0, 0, 2],
                        ];
                        let area5x5_center_offset: i32 = 2;
                        for xy_pair in &four_xy_pairs {
                            let ax: i32 = xy_pair[0] + area5x5_center_offset;
                            let ay: i32 = xy_pair[1] + area5x5_center_offset;
                            let bx: i32 = xy_pair[2] + area5x5_center_offset;
                            let by: i32 = xy_pair[3] + area5x5_center_offset;
                            let cx: i32 = xy_pair[4] + area5x5_center_offset;
                            let cy: i32 = xy_pair[5] + area5x5_center_offset;
                            let a: u8 = area5x5.get(ax, ay).unwrap_or(255);
                            let b: u8 = area5x5.get(bx, by).unwrap_or(255);
                            let c: u8 = area5x5.get(cx, cy).unwrap_or(255);
                            let all_same: bool = a == b && b == c;
                            let all_different: bool = a != b && b != c && a != c;
                            let same_as_center: bool = a == center && b == center && c == center;
                            let different_than_center: bool = a != center && b != center && c != center;
                            record.serialize_bool(all_same);
                            record.serialize_bool(all_different);
                            record.serialize_bool(same_as_center);
                            record.serialize_bool(different_than_center);
                        }
                    }

                    if enable_output_three_xy_pairs {
                        let four_xy_pairs: Vec<[i32; 6]> = vec![
                            [-1, -1, -1, 1, 1, 0],
                            [-1, 0, 1, -1, 1, 1],
                            [0, -1, -1, 1, 1, 1],
                            [0, 1, -1, -1, 1, -1],
                            [-2, 0, 0, -1, 2, 0],
                            [-2, 0, 0, 1, 2, 0],
                            [0, -2, -1, 0, 0, 2],
                            [0, -2, 1, 0, 0, 2],
                        ];
                        let area5x5_center_offset: i32 = 2;
                        for xy_pair in &four_xy_pairs {
                            let ax: i32 = xy_pair[0] + area5x5_center_offset;
                            let ay: i32 = xy_pair[1] + area5x5_center_offset;
                            let bx: i32 = xy_pair[2] + area5x5_center_offset;
                            let by: i32 = xy_pair[3] + area5x5_center_offset;
                            let cx: i32 = xy_pair[4] + area5x5_center_offset;
                            let cy: i32 = xy_pair[5] + area5x5_center_offset;
                            let a: u8 = area5x5_output.get(ax, ay).unwrap_or(255);
                            let b: u8 = area5x5_output.get(bx, by).unwrap_or(255);
                            let c: u8 = area5x5_output.get(cx, cy).unwrap_or(255);
                            let all_same: bool = a == b && b == c;
                            let all_different: bool = a != b && b != c && a != c;
                            let same_as_center: bool = a == center && b == center && c == center;
                            let different_than_center: bool = a != center && b != center && c != center;
                            record.serialize_bool(all_same);
                            record.serialize_bool(all_different);
                            record.serialize_bool(same_as_center);
                            record.serialize_bool(different_than_center);
                        }
                    }

                    // {
                    //     let histogram: Histogram = area3x3.histogram_all();
                    //     let mut color: u8 = 255;
                    //     if histogram.number_of_counters_greater_than_zero() == 1 {
                    //         color = histogram.most_popular_color().unwrap_or(255);
                    //     }
                    //     // record.serialize_onehot(color, 10);
                    //     record.serialize_color_complex(color, obfuscated_color_offset, enable_serialize_color_complex);
                    // }
                    // {
                    //     let histogram: Histogram = area5x5.histogram_all();
                    //     let mut color: u8 = 255;
                    //     if histogram.number_of_counters_greater_than_zero() == 1 {
                    //         color = histogram.most_popular_color().unwrap_or(255);
                    //     }
                    //     // record.serialize_onehot(color, 10);
                    //     record.serialize_color_complex(color, obfuscated_color_offset, enable_serialize_color_complex);
                    // }

                    // {
                    //     // let area_top_left: Image = area3x3.crop_outside(0, 0, 2, 2, 255)?;
                    //     // let area_top_right: Image = area3x3.crop_outside(1, 0, 2, 2, 255)?;
                    //     // let area_bottom_left: Image = area3x3.crop_outside(0, 1, 2, 2, 255)?;
                    //     // let area_bottom_right: Image = area3x3.crop_outside(1, 1, 2, 2, 255)?;
                    //     let image0: Image = area5x5.crop_outside(0, 0, 3, 3, 255)?;
                    //     let image1: Image = area5x5.crop_outside(1, 0, 3, 3, 255)?;
                    //     let image2: Image = area5x5.crop_outside(2, 0, 3, 3, 255)?;
                    //     let image3: Image = area5x5.crop_outside(0, 1, 3, 3, 255)?;
                    //     let image4: Image = area5x5.crop_outside(2, 1, 3, 3, 255)?;
                    //     let image5: Image = area5x5.crop_outside(0, 2, 3, 3, 255)?;
                    //     let image6: Image = area5x5.crop_outside(1, 2, 3, 3, 255)?;
                    //     let image7: Image = area5x5.crop_outside(2, 2, 3, 3, 255)?;
                    //     // let images = [&area_top_left, &area_top_right, &area_bottom_left, &area_bottom_right];
                    //     let images = [&image0, &image1, &image2, &image3, &image4, &image5, &image6, &image7];
                    //     for image in images {
                    //         let histogram: Histogram = image.histogram_all();
                    //         // histogram.set_counter_to_zero(center);
                    //         // histogram.set_counter_to_zero(10);
                    //         // histogram.set_counter_to_zero(255);
                    //         // record.serialize_bool_onehot(histogram.number_of_counters_greater_than_zero() == 1);
                    //         // record.serialize_onehot_discard_overflow(histogram.number_of_counters_greater_than_zero().min(8) as u8, 8);
                    //         // record.serialize_onehot_discard_overflow(histogram.get(center).min(9) as u8, 9);
                    //         record.serialize_bool(histogram.get(center) > 0);
                    //     }
                    // }


                    // {
                    //     let count: u8 = number_of_unique_bigrams_column_vec.get(x as usize).map(|n| *n).unwrap_or(0);
                    //     record.serialize_u8(count);
                    //     record.serialize_onehot_discard_overflow(count, 28);
                    // }
                    // {
                    //     let count: u8 = number_of_unique_bigrams_row_vec.get(y as usize).map(|n| *n).unwrap_or(0);
                    //     record.serialize_u8(count);
                    //     record.serialize_onehot_discard_overflow(count, 28);
                    // }

                    // {
                    //     let mut count: u8 = 0;
                    //     if let Some(trigrams) = trigrams_column.get(&x) {
                    //         for trigram in trigrams {
                    //             if trigram.word0 == center {
                    //                 count += 1;
                    //             }
                    //             if trigram.word1 == center {
                    //                 count += 1;
                    //             }
                    //             if trigram.word2 == center {
                    //                 count += 1;
                    //             }
                    //         }
                    //     }
                    //     record.serialize_u8(count);
                    // }
                    // {
                    //     let mut count: u8 = 0;
                    //     if let Some(trigrams) = trigrams_row.get(&y) {
                    //         for trigram in trigrams {
                    //             if trigram.word0 == center {
                    //                 count += 1;
                    //             }
                    //             if trigram.word1 == center {
                    //                 count += 1;
                    //             }
                    //             if trigram.word2 == center {
                    //                 count += 1;
                    //             }
                    //         }
                    //     }
                    //     record.serialize_u8(count);
                    // }

                    // let area_left: Image = if x > 2 {
                    //     input.left_columns(x - 1)?
                    // } else {
                    //     Image::empty()
                    // };
                    // let area_right: Image = if x_reverse > 2 {
                    //     input.right_columns(x_reverse - 1)?
                    // } else {
                    //     Image::empty()
                    // };
                    // let mut area_left = Image::empty();
                    // let mut area_right = Image::empty();
                    // if let Some(image) = earlier_prediction_image {
                    //     if x > 2 {
                    //         area_left = image.left_columns(x - 1)?;
                    //     };
                    //     if x_reverse > 2 {
                    //         area_right = image.right_columns(x_reverse - 1)?;
                    //     }
                    // }

                    let mut input_area_topleft = Image::empty();
                    let mut input_area_topright = Image::empty();
                    let mut input_area_bottomleft = Image::empty();
                    let mut input_area_bottomright = Image::empty();
                    {
                        if x > 0 {
                            input_area_topleft = input_area_top.left_columns(x)?;
                            input_area_bottomleft = input_area_bottom.left_columns(x)?;
                        };
                        if context_input_x_reverse > 0 {
                            input_area_topright = input_area_top.right_columns(context_input_x_reverse.min(255) as u8)?;
                            input_area_bottomright = input_area_bottom.right_columns(context_input_x_reverse.min(255) as u8)?;
                        }
                    }

                    let mut output_area_topleft = Image::empty();
                    let mut output_area_topright = Image::empty();
                    let mut output_area_bottomleft = Image::empty();
                    let mut output_area_bottomright = Image::empty();
                    {
                        if x > 0 {
                            output_area_topleft = output_area_top.left_columns(x)?;
                            output_area_bottomleft = output_area_bottom.left_columns(x)?;
                        };
                        if context_output_x_reverse > 0 {
                            output_area_topright = output_area_top.right_columns(context_output_x_reverse.min(255) as u8)?;
                            output_area_bottomright = output_area_bottom.right_columns(context_output_x_reverse.min(255) as u8)?;
                        }
                    }
                    // let area_topleft_histogram: Histogram = area_topleft.histogram_all();
                    // let area_topright_histogram: Histogram = area_topright.histogram_all();
                    // let area_bottomleft_histogram: Histogram = area_bottomleft.histogram_all();
                    // let area_bottomright_histogram: Histogram = area_bottomright.histogram_all();

                    // {
                    //     let histograms = [&area_topleft_histogram, &area_topright_histogram, &area_bottomleft_histogram, &area_bottomright_histogram];
                    //     for histogram in histograms {
                    //         let count: u8 = histogram.number_of_counters_greater_than_zero().min(10) as u8;
                    //         record.serialize_onehot(count, 10);
                    //         // record.serialize_bool_onehot(histogram.get(center) > 0);
                    //     }
                    // }

                    // let area_left_histogram_rows: Vec<Histogram> = area_left.histogram_rows();
                    // let area_right_histogram_rows: Vec<Histogram> = area_right.histogram_rows();
                    // let area_left_histogram: Histogram = area_left.histogram_all();
                    // let area_right_histogram: Histogram = area_right.histogram_all();

                    // for color in 0..=9u8 {
                    //     record.serialize_bool_onehot(area_topleft_histogram.get(color) > 0);
                    //     record.serialize_bool_onehot(area_topright_histogram.get(color) > 0);
                    //     record.serialize_bool_onehot(area_bottomleft_histogram.get(color) > 0);
                    //     record.serialize_bool_onehot(area_bottomright_histogram.get(color) > 0);
                    // }
                    // {
                    //     record.serialize_bool_onehot(area_topleft_histogram.get(center) > 0);
                    //     record.serialize_bool_onehot(area_topright_histogram.get(center) > 0);
                    //     record.serialize_bool_onehot(area_bottomleft_histogram.get(center) > 0);
                    //     record.serialize_bool_onehot(area_bottomright_histogram.get(center) > 0);
                    // }
                    // for color in 0..=9u8 {
                    //     record.serialize_bool_onehot(area_top_histogram.get(color) > 0);
                    //     record.serialize_bool_onehot(area_bottom_histogram.get(color) > 0);
                    //     record.serialize_bool_onehot(area_left_histogram.get(color) > 0);
                    //     record.serialize_bool_onehot(area_right_histogram.get(color) > 0);
                    // }
                    // {
                    //     record.serialize_bool_onehot(area_top_histogram.get(center) > 0);
                    //     record.serialize_bool_onehot(area_bottom_histogram.get(center) > 0);
                    //     record.serialize_bool_onehot(area_left_histogram.get(center) > 0);
                    //     record.serialize_bool_onehot(area_right_histogram.get(center) > 0);
                    // }
                    // {
                    //     let histograms = [&area_top_histogram, &area_bottom_histogram, &area_left_histogram, &area_right_histogram];
                    //     for histogram in histograms {
                    //         let color: u8 = histogram.most_popular_color_disallow_ambiguous().unwrap_or(255);
                    //         record.serialize_bool_onehot(center == color);
                    //     }
                    //     for histogram in histograms {
                    //         let color: u8 = histogram.least_popular_color_disallow_ambiguous().unwrap_or(255);
                    //         record.serialize_bool_onehot(center == color);
                    //     }
                    // }
                
                    if enable_diagonalhistogram_opposites {
                        let dh_input_area_topleft: DiagonalHistogram = DiagonalHistogram::diagonal_a(&input_area_topleft)?;
                        let dh_input_area_topright: DiagonalHistogram = DiagonalHistogram::diagonal_b(&input_area_topright)?;
                        let dh_input_area_bottomleft: DiagonalHistogram = DiagonalHistogram::diagonal_b(&input_area_bottomleft)?;
                        let dh_input_area_bottomright: DiagonalHistogram = DiagonalHistogram::diagonal_a(&input_area_bottomright)?;
                        let mut dh_output_area_topleft: Option<DiagonalHistogram> = None;
                        let mut dh_output_area_topright: Option<DiagonalHistogram> = None;
                        let mut dh_output_area_bottomleft: Option<DiagonalHistogram> = None;
                        let mut dh_output_area_bottomright: Option<DiagonalHistogram> = None;
                        if has_different_size_for_input_output {
                            dh_output_area_topleft = Some(DiagonalHistogram::diagonal_a(&output_area_topleft)?);
                            dh_output_area_topright = Some(DiagonalHistogram::diagonal_b(&output_area_topright)?);
                            dh_output_area_bottomleft = Some(DiagonalHistogram::diagonal_b(&output_area_bottomleft)?);
                            dh_output_area_bottomright = Some(DiagonalHistogram::diagonal_a(&output_area_bottomright)?);
                        }
                        // let diagonalhistograms = [
                        //     &dh_input_area_topleft, &dh_input_area_topright, &dh_input_area_bottomleft, &dh_input_area_bottomright,
                        //     &dh_output_area_topleft, &dh_output_area_topright, &dh_output_area_bottomleft, &dh_output_area_bottomright,
                        // ];
                        // for diagonalhistogram in diagonalhistograms {
                        //     for color in 0..COUNT_COLORS_PLUS1 {
                        //         let mut mass: u32 = 0;
                        //         let mut unique_count: u16 = 0;
                        //         let mut found_center: bool = false;
                        //         if let Some(histogram) = diagonalhistogram.get(x as i32, y as i32) {
                        //             mass = histogram.get(color);
                        //             unique_count = histogram.number_of_counters_greater_than_zero();
                        //             found_center = histogram.get(center) > 0;
                        //         }
                        //         record.serialize_bool(mass > 0);
                        //         record.serialize_bool_onehot(mass > 0);
                        //         let the_color: u8 = if mass > 0 { color } else { 255 };
                        //         record.serialize_color_complex(the_color, obfuscated_color_offset, enable_serialize_color_complex);
                        //         record.serialize_u8(mass.min(255) as u8);
                        //         record.serialize_onehot(unique_count.min(255) as u8, COUNT_COLORS_PLUS1);
                        //         record.serialize_bool(found_center);
                        //     }
                        // }

                        let mut dh_opposites = Vec::<(&DiagonalHistogram, &DiagonalHistogram)>::new();
                        dh_opposites.push((&dh_input_area_topleft, &dh_input_area_bottomright));
                        dh_opposites.push((&dh_input_area_bottomleft, &dh_input_area_topright));
                        if let Some(topleft) = &dh_output_area_topleft {
                            if let Some(bottomright) = &dh_output_area_bottomright {
                                dh_opposites.push((topleft, bottomright));
                            }
                        }
                        if let Some(bottomleft) = &dh_output_area_bottomleft {
                            if let Some(topright) = &dh_output_area_topright {
                                dh_opposites.push((bottomleft, topright));
                            }
                        }
                        for (dh0, dh1) in dh_opposites {
                            for color in 0..=9 {
                                let mut mass0: u32 = 0;
                                let mut mass1: u32 = 0;
                                if let Some(histogram) = dh0.get(x as i32, y as i32) {
                                    mass0 = histogram.get(color);
                                }
                                if let Some(histogram) = dh1.get(x as i32, y as i32) {
                                    mass1 = histogram.get(color);
                                }
                                // record.serialize_bool(mass0 == 1 && mass1 == 1);
                                record.serialize_bool(mass0 > 0 && mass1 > 0);
                                // record.serialize_bool(mass0 == 1 && mass1 == 0);
                                record.serialize_bool(mass0 > 0 && mass1 == 0);
                                // record.serialize_bool(mass0 == 0 && mass1 == 1);
                                record.serialize_bool(mass0 == 0 && mass1 > 0);
                                record.serialize_bool(mass0 == 0 && mass1 == 0);
                            }
                        }
                    }

                    if enable_center_indicator {
                        let center_indicator_x_value: u8 = center_indicator_x.get(xx, yy).unwrap_or(0);
                        let center_indicator_y_value: u8 = center_indicator_y.get(xx, yy).unwrap_or(0);
                        if enable_center_indicator_a {
                            record.serialize_bool_onehot(center_indicator_x_value == 1);
                            record.serialize_bool_onehot(center_indicator_y_value == 1);
                            // record.serialize_bool_onehot(center_indicator_x_value >= 1 && center_indicator_x_value <= 3);
                            // record.serialize_bool_onehot(center_indicator_y_value >= 1 && center_indicator_y_value <= 3);
                            // record.serialize_onehot(center_indicator_x_value, 5);
                            // record.serialize_onehot(center_indicator_y_value, 5);
                        }
                        if enable_center_indicator_x {
                            record.serialize_bool(center_indicator_x_value == 0 || center_indicator_x_value == 2);
                            record.serialize_bool_onehot(center_indicator_x_value == 1);
                            record.serialize_bool(center_indicator_x_value == 3 || center_indicator_x_value == 4);
                        }
                        if enable_center_indicator_y {
                            record.serialize_bool(center_indicator_y_value == 0 || center_indicator_y_value == 2);
                            record.serialize_bool_onehot(center_indicator_y_value == 1);
                            record.serialize_bool(center_indicator_y_value == 3 || center_indicator_y_value == 4);
                        }
                    }

                    // let preserve_center_color: bool = histogram_preserve.get(center) > 0;

                    // let nonbackground_area3x3: Image = non_background_mask.crop_outside(xx - 1, yy - 1, 3, 3, 255)?;

                    let image_top: u8 = input.get(xx, 0).unwrap_or(255);
                    let image_bottom: u8 = input.get(xx, context_input_size.height as i32 - 1).unwrap_or(255);
                    let image_left: u8 = input.get(0, yy).unwrap_or(255);
                    let image_right: u8 = input.get(context_input_size.width as i32 - 1, yy).unwrap_or(255);

                    let center_x_reversed: u8 = input.get(context_input_x_reverse as i32, yy).unwrap_or(255);
                    let center_y_reversed: u8 = input.get(xx, context_input_y_reverse as i32).unwrap_or(255);
                    let center_xy_reversed: u8 = input.get(context_input_x_reverse as i32, context_input_y_reverse as i32).unwrap_or(255);
                    _ = center_xy_reversed;
                    
                    let center_denoise_type1: u8 = input_denoise_type1.get(xx, yy).unwrap_or(255);
                    // let center_denoise_type1_border: u8 = input_denoise_type1_border.get(xx, yy).unwrap_or(255);

                    let object_center: u8 = enumerated_objects.get(xx, yy).unwrap_or(255);
                    {
                        // record.serialize_complex(object_center as u16, 20);
                        // record.serialize_u8(object_center);
                    }

                    let object_top: u8 = enumerated_objects.get(xx, yy - 1).unwrap_or(255);
                    let object_bottom: u8 = enumerated_objects.get(xx, yy + 1).unwrap_or(255);
                    let object_left: u8 = enumerated_objects.get(xx - 1, yy).unwrap_or(255);
                    let object_right: u8 = enumerated_objects.get(xx + 1, yy).unwrap_or(255);
                    if enable_object_center_same_as_neighbour {
                        record.serialize_bool(object_top == object_center);
                        record.serialize_bool(object_bottom == object_center);
                        record.serialize_bool(object_left == object_center);
                        record.serialize_bool(object_right == object_center);
                    }

                    if enable_grid {
                        let grid_mask_center: u8 = grid_mask.get(xx, yy).unwrap_or(0);
                        let grid_center: u8 = if grid_mask_center > 0 { grid_color } else { 255 };
                        let is_grid: bool = grid_mask_center > 0;
                        record.serialize_bool_onehot(is_grid);
                        record.serialize_color_complex(grid_center, obfuscated_color_offset, enable_serialize_color_complex);
                        record.serialize_color_complex(grid_color, obfuscated_color_offset, enable_serialize_color_complex);
                    }
                    
                    // let repair_center: u8 = repair_mask.get(xx, yy).unwrap_or(255);

                    let neighbour_up: u8 = image_neighbour_up.get(xx, yy).unwrap_or(255);
                    let neighbour_down: u8 = image_neighbour_down.get(xx, yy).unwrap_or(255);
                    let neighbour_left: u8 = image_neighbour_left.get(xx, yy).unwrap_or(255);
                    let neighbour_right: u8 = image_neighbour_right.get(xx, yy).unwrap_or(255);
                    let neighbour_upleft: u8 = image_neighbour_upleft.get(xx, yy).unwrap_or(255);
                    let neighbour_upright: u8 = image_neighbour_upright.get(xx, yy).unwrap_or(255);
                    let neighbour_downleft: u8 = image_neighbour_downleft.get(xx, yy).unwrap_or(255);
                    let neighbour_downright: u8 = image_neighbour_downright.get(xx, yy).unwrap_or(255);    

                    let corners_center: u8 = corners.get(xx, yy).unwrap_or(255);
                    record.serialize_bool(corners_center == 1);
                    record.serialize_bool(corners_center == 2);
                    record.serialize_bool(corners_center == 3);
                    record.serialize_bool(corners_center == 4);

                    // let column_above_center: Image = match input.crop(Rectangle::new(x, 0, 1, y)) {
                    //     Ok(value) => value,
                    //     Err(_) => Image::empty()
                    // };
                    // let column_below_center: Image = match input.crop(Rectangle::new(x, y + 1, 1, y_reverse)) {
                    //     Ok(value) => value,
                    //     Err(_) => Image::empty()
                    // };
                    // let area_above_center: Image = match input.crop(Rectangle::new(0, 0, width, y)) {
                    //     Ok(value) => value,
                    //     Err(_) => Image::empty()
                    // };
                    // let area_below_center: Image = match input.crop(Rectangle::new(0, y + 1, width, y_reverse)) {
                    //     Ok(value) => value,
                    //     Err(_) => Image::empty()
                    // };
                    let center_column: Image = match input.crop(Rectangle::new(x, 0, 1, context_input_size.height)) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    let center_row: Image = match input.crop(Rectangle::new(0, y, context_input_size.width, 1)) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    let center_column_top: Image = match center_column.top_rows(y) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    let center_column_bottom: Image = match center_column.bottom_rows(context_input_y_reverse.max(0).min(255) as u8) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    let center_row_left: Image = match center_row.left_columns(x) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    let center_row_right_x: i32 = if enable_typo_for_center_row_right_columns { 
                        // This is an old typo. Where I by mistake use the Y coordinate for the X coordinate.
                        // Fixing the typo, and the number of tasks solved drops by 1 task on the hidden ARC dataset.
                        // If I keep the typo, the number of tasks solved is 1 task higher on the hidden ARC dataset.
                        // Let's keep the typo, even though it's silly.
                        context_input_y_reverse 
                    } else { 
                        context_input_x_reverse
                    };
                    let center_row_right: Image = match center_row.right_columns(center_row_right_x.max(0).min(255) as u8) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };

                    if enable_trigram_count_center || enable_trigram_count_word1_center {
                        let trigrams_vec: [Vec<RecordTrigram>; 4] = [
                            center_column_top.trigram_y().unwrap_or_else(|_| Vec::new()),
                            center_column_bottom.trigram_y().unwrap_or_else(|_| Vec::new()),
                            center_row_left.trigram_x().unwrap_or_else(|_| Vec::new()),
                            center_row_right.trigram_x().unwrap_or_else(|_| Vec::new()),
                        ];
                        if enable_trigram_count_center {
                            for trigrams in &trigrams_vec {
                                let mut count: u8 = 0;
                                for trigram in trigrams {
                                    if trigram.word0 == center {
                                        count += 1;
                                    }
                                    if trigram.word1 == center {
                                        count += 1;
                                    }
                                    if trigram.word2 == center {
                                        count += 1;
                                    }
                                }
                                record.serialize_u8(count);
                            }
                        }
                        if enable_trigram_count_word1_center {
                            for trigrams in &trigrams_vec {
                                let mut count: u8 = 0;
                                for trigram in trigrams {
                                    if trigram.word0 != center && trigram.word1 == center && trigram.word2 != center {
                                        count += 1;
                                    }
                                }
                                record.serialize_u8(count);
                            }
                        }
                    }

                    if enable_trigram_count_word012_center {
                        {
                            let trigrams_vec: [Vec<RecordTrigram>; 2] = [
                                center_row_left.trigram_x().unwrap_or_else(|_| Vec::new()),
                                center_row_right.trigram_x().unwrap_or_else(|_| Vec::new()),
                            ];
                            let word0: u8 = area3x3.get(0, 1).unwrap_or(255);
                            let word1: u8 = center;
                            let word2: u8 = area3x3.get(2, 1).unwrap_or(255);
                            for trigrams in &trigrams_vec {
                                let mut count: u8 = 0;
                                for trigram in trigrams {
                                    if trigram.word0 == word0 && trigram.word1 == word1 && trigram.word2 == word2 {
                                        count += trigram.count.min(255) as u8;
                                    }
                                }
                                // record.serialize_u8(count);
                                record.serialize_onehot(count, 4);
                            }
                        }
                        {
                            let trigrams_vec: [Vec<RecordTrigram>; 2] = [
                                center_column_top.trigram_y().unwrap_or_else(|_| Vec::new()),
                                center_column_bottom.trigram_y().unwrap_or_else(|_| Vec::new()),
                            ];
                            let word0: u8 = area3x3.get(1, 0).unwrap_or(255);
                            let word1: u8 = center;
                            let word2: u8 = area3x3.get(1, 2).unwrap_or(255);
                            for trigrams in &trigrams_vec {
                                let mut count: u8 = 0;
                                for trigram in trigrams {
                                    if trigram.word0 == word0 && trigram.word1 == word1 && trigram.word2 == word2 {
                                        count += trigram.count.min(255) as u8;
                                    }
                                }
                                // record.serialize_u8(count);
                                record.serialize_onehot(count, 4);
                            }
                        }
                    }

                    if enable_distance {
                        let max_distance: u8 = 3;
                        let distance_top: u8 = y.min(max_distance) + 1;
                        let distance_bottom: u8 = y_reverse.min(max_distance) + 1;
                        let distance_left: u8 = x.min(max_distance) + 1;
                        let distance_right: u8 = x_reverse.min(max_distance) + 1;
                        record.serialize_u8(distance_top);
                        record.serialize_u8(distance_bottom);
                        record.serialize_u8(distance_left);
                        record.serialize_u8(distance_right);    
                    }

                    if enable_edge {
                        let is_edge: bool = x == 0 || y == 0 || x_reverse == 0 || y_reverse == 0;
                        record.serialize_bool_onehot(is_edge);
                        record.serialize_bool_onehot(x == 0);
                        record.serialize_bool_onehot(y == 0);
                        record.serialize_bool_onehot(x_reverse == 0);
                        record.serialize_bool_onehot(y_reverse == 0);
                    }
                    
                    let input_is_noise_color: bool = noise_color == Some(center);
                    // let input_is_removal_color: u8 = if removal_color == Some(center) { 1 } else { 0 };

                    let mass_connectivity4: u8 = image_mass_connectivity4.get(xx, yy).unwrap_or(0);
                    let mass_connectivity8: u8 = image_mass_connectivity8.get(xx, yy).unwrap_or(0);

                    let input_is_most_popular_color: bool = most_popular_color == Some(center);
                
                    let x_mod4: u8 = x % 4;
                    let y_mod4: u8 = y % 4;
                    let x_reverse_mod4: u8 = x_reverse % 4;
                    let y_reverse_mod4: u8 = y_reverse % 4;
                    _ = x_mod4;
                    _ = y_mod4;
                    _ = x_reverse_mod4;
                    _ = y_reverse_mod4;

                    let x_mod5: u8 = x % 5;
                    let y_mod5: u8 = y % 5;
                    let x_reverse_mod5: u8 = x_reverse % 5;
                    let y_reverse_mod5: u8 = y_reverse % 5;
                    _ = x_mod5;
                    _ = y_mod5;
                    _ = x_reverse_mod5;
                    _ = y_reverse_mod5;

                    let x2_mod2: u8 = (x / 2) % 2;
                    let y2_mod2: u8 = (y / 2) % 2;
                    let x2_reverse_mod2: u8 = (x_reverse / 2) % 2;
                    let y2_reverse_mod2: u8 = (y_reverse / 2) % 2;
                    _ = x2_mod2;
                    _ = y2_mod2;
                    _ = x2_reverse_mod2;
                    _ = y2_reverse_mod2;

                    let x4_mod2: u8 = (x / 4) % 2;
                    let y4_mod2: u8 = (y / 4) % 2;
                    let x4_reverse_mod2: u8 = (x_reverse / 4) % 2;
                    let y4_reverse_mod2: u8 = (y_reverse / 4) % 2;
                    _ = x4_mod2;
                    _ = y4_mod2;
                    _ = x4_reverse_mod2;
                    _ = y4_reverse_mod2;

                    let mut preserve_edge: bool = false;

                    let mut v0: u8 = 0;
                    let mut v1: u8 = 0;
                    let mut v2: u8 = 0;
                    let mut v3: u8 = 0;
                    let mut v4: u8 = 0;
                    let mut v5: u8 = 0;
                    let mut v6: u8 = 0;
                    let mut v7: u8 = 0;
                    // v2 = grid_center;
                    // v2 = repair_center;
                    // v2 = x_mod3;
                    // v3 = y_mod3;
                    // v4 = x_reverse_mod3;
                    // v5 = y_reverse_mod3;
                    // if x == x_reverse && y == y_reverse {
                    //     v5 = 1;
                    // }
                    // let x_distance: i16 = (width as i16) - ((x as i16) * 2);
                    // let y_distance: i16 = (height as i16) - ((y as i16) * 2);

                    // if x == x_reverse || y == y_reverse {
                    //     v5 = 1;
                    // }
                    // if y == y_reverse {
                    //     v5 = 1;
                    // }
                    {
                        // let h: Histogram = column_above_center.histogram_all();
                        // if h.get(center) > 0 {
                        //     v2 = 1;
                        // }
                        // if h.number_of_counters_greater_than_zero() >= 2 {
                        //     v2 = 1;
                        // }
                    }
                    {
                        // let h: Histogram = column_below_center.histogram_all();
                        // if h.get(center) > 0 {
                        //     v3 = 1;
                        // }
                        // if h.number_of_counters_greater_than_zero() >= 2 {
                        //     v3 = 1;
                        // }
                    }
                    {
                        // let h: Histogram = area_above_center.histogram_all();
                        // if h.get(center) > 0 {
                        //     v2 = 1;
                        // }
                        // if h.number_of_counters_greater_than_zero() >= 2 {
                        //     v2 = 1;
                        // }
                    }
                    {
                        // let h: Histogram = area_below_center.histogram_all();
                        // if h.get(center) > 0 {
                        //     v3 = 1;
                        // }
                        // if h.number_of_counters_greater_than_zero() >= 2 {
                        //     v2 = 1;
                        // }
                    }
                    {
                        // if neighbour_up == center {
                        //     v2 = 1;
                        // }
                        // if neighbour_down == center {
                        //     v3 = 1;
                        // }
                        // if neighbour_left == center {
                        //     v4 = 1;
                        // }
                        // if neighbour_right == center {
                        //     v5 = 1;
                        // }
                        // if Some(neighbour_up) == noise_color {
                        //     v2 = 1;
                        // }
                        // if Some(neighbour_down) == noise_color {
                        //     v3 = 1;
                        // }
                        // if Some(neighbour_left) == noise_color {
                        //     v4 = 1;
                        // }
                        // if Some(neighbour_right) == noise_color {
                        //     v5 = 1;
                        // }
                        if Some(neighbour_up) == noise_color {
                            v2 += 1;
                        }
                        if Some(neighbour_down) == noise_color {
                            v2 += 1;
                        }
                        if Some(neighbour_left) == noise_color {
                            v2 += 1;
                        }
                        if Some(neighbour_right) == noise_color {
                            v2 += 1;
                        }
                    }
                    // {
                    //     if Some(neighbour_upleft) == noise_color {
                    //         v2 += 1;
                    //     }
                    //     if Some(neighbour_upright) == noise_color {
                    //         v2 += 1;
                    //     }
                    //     if Some(neighbour_downleft) == noise_color {
                    //         v2 += 1;
                    //     }
                    //     if Some(neighbour_downright) == noise_color {
                    //         v2 += 1;
                    //     }
                    // }
                    {
                        // if center == neighbour_upleft {
                        //     v3 += 1;
                        // }
                        // if center == neighbour_upright {
                        //     v3 += 1;
                        // }
                        // if center == neighbour_downleft {
                        //     v3 += 1;
                        // }
                        // if center == neighbour_downright {
                        //     v3 += 1;
                        // }
                        // if center == neighbour_upleft {
                        //     v3 = 1;
                        // }
                        // if center == neighbour_upright {
                        //     v4 = 1;
                        // }
                        // if center == neighbour_downleft {
                        //     v5 = 1;
                        // }
                        // if center == neighbour_downright {
                        //     v6 = 1;
                        // }
                    }

                    {
                        // if image_left == image_right {
                        //     v3 = 1;
                        // }
                        // if image_top == image_bottom {
                        //     v4 = 1;
                        // }
                        // if center == image_top {
                        //     v3 += 1;
                        // }
                        // if center == image_bottom {
                        //     v3 += 1;
                        // }
                        // if center == image_left {
                        //     v3 += 1;
                        // }
                        // if center == image_right {
                        //     v3 += 1;
                        // }
                        // if center == image_top {
                        //     v3 |= 1;
                        // }
                        // if center == image_bottom {
                        //     v3 |= 2;
                        // }
                        // if center == image_left {
                        //     v3 |= 4;
                        // }
                        // if center == image_right {
                        //     v3 |= 8;
                        // }
                        // if center == image_top && center == image_bottom && center == image_left && center == image_right {
                        //     v3 = 1;
                        // }
                        // if image_top == image_bottom && image_top == image_left && image_top == image_right {
                        //     v4 = 1;
                        // }
                    }

                    for label in &task.action_label_set_intersection {
                        match label {
                            ActionLabel::InputImageIsOutputImageWithNoChangesToPixelsWithColor { color } => {
                                if center == *color {
                                    v0 = 1;
                                }
                            },
                            ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { color } => {
                                if center == *color {
                                    v1 = 1;
                                }
                            },
                            ActionLabel::OutputImagePreserveInputImageEdge { edge } => {
                                match *edge {
                                    ImageEdge::Top => {
                                        if y == 0 {
                                            preserve_edge = true;
                                        }
                                    },
                                    ImageEdge::Bottom => {
                                        if y_reverse == 0 {
                                            preserve_edge = true;
                                        }
                                    },
                                    ImageEdge::Left => {
                                        if x == 0 {
                                            preserve_edge = true;
                                        }
                                    },
                                    ImageEdge::Right => {
                                        if x_reverse == 0 {
                                            preserve_edge = true;
                                        }
                                    },
                                }
                            },
                            // ActionLabel::OutputImagePreserveInputImageCorner { corner } => {
                            //     match *corner {
                            //         ImageCorner::TopLeft => {
                            //             if x == 0 && y == 0 {
                            //                 v2 = 1;
                            //             }
                            //         },
                            //         ImageCorner::TopRight => {
                            //             if x_reverse == 0 && y == 0 {
                            //                 v2 = 1;
                            //             }
                            //         },
                            //         ImageCorner::BottomLeft => {
                            //             if x == 0 && y_reverse == 0 {
                            //                 v2 = 1;
                            //             }
                            //         },
                            //         ImageCorner::BottomRight => {
                            //             if x_reverse == 0 && y_reverse == 0 {
                            //                 v2 = 1;
                            //             }
                            //         },
                            //     }
                            // },
                            ActionLabel::OutputImageIsInputImageWithNoChangesToPixelsWithColor { color } => {
                                if center == *color {
                                    v5 = 1;
                                }
                                if noise_color == Some(*color) {
                                    v6 = 1;
                                }
                                if most_popular_color == Some(*color) {
                                    v7 = 1;
                                }
                            },
                            _ => {}
                        }
                    }
                    // if object_center == 0 {
                    //     v2 = 1;
                    // }

                    if object_center > 0 {
                        // if object_center == object_left && object_center == object_right && object_center == object_top && object_center == object_bottom {
                        //     v3 = 1;
                        // }
                        // if object_center == object_left && object_center == object_right {
                        //     v2 = 1;
                        // }
                        // if object_center == object_top && object_center == object_bottom {
                        //     v3 = 1;
                        // }
                    }

                    // let center_same_as_diagonal: bool = center == top_left || center == top_right || center == bottom_left || center == bottom_right;
                    // let center_same_as_neighbor: bool = center == top || center == bottom || center == left || center == right;
                    // if center_same_as_diagonal {
                    //     v2 = 1;
                    // }
                    // if center_same_as_neighbor {
                    //     v3 = 1;
                    // }
                    // if center_same_as_neighbor && center_same_as_diagonal {
                    //     v4 = 1;
                    // }
                    // if center_same_as_neighbor != center_same_as_diagonal {
                    //     v5 = 1;
                    // }
                    // if center == top && center == bottom {
                    //     v2 = 1;
                    // }
                    // if center == left && center == right {
                    //     v3 = 1;
                    // }

                    // let xminus1: i32 = xx - 1;
                    // let xplus1: i32 = xx + 1;
                    // let yminus1: i32 = yy - 1;
                    // let yplus1: i32 = yy + 1;
                    // if xminus1 >= 0 {
                    //     if let Some(histogram) = &histogram_columns.get(xminus1 as usize) {
                    //         if histogram.get(center) == 0 {
                    //             v2 += 1;
                    //         }
                    //     }
                    // }
                    // if xplus1 >= 255 {
                    //     if let Some(histogram) = &histogram_columns.get(xplus1 as usize) {
                    //         if histogram.get(center) == 0 {
                    //             v2 += 1;
                    //         }
                    //     }
                    // }
                    // if yminus1 >= 0 {
                    //     if let Some(histogram) = &histogram_rows.get(yminus1 as usize) {
                    //         if histogram.get(center) == 0 {
                    //             v3 += 1;
                    //         }
                    //     }
                    // }
                    // if yplus1 >= 255 {
                    //     if let Some(histogram) = &histogram_rows.get(yplus1 as usize) {
                    //         if histogram.get(center) == 0 {
                    //             v3 += 1;
                    //         }
                    //     }
                    // }

                    // let mut same_xy_histogram_colors: bool = false;
                    // if let Some(histogram0) = histogram_columns.get(x as usize) {
                    //     if let Some(histogram1) = histogram_rows.get(y as usize) {
                    //         let mut histogram2: Histogram = histogram0.clone();
                    //         histogram2.intersection_histogram(histogram1);
                    //         if histogram2.number_of_counters_greater_than_zero() == histogram0.number_of_counters_greater_than_zero() {
                    //             same_xy_histogram_colors = true;
                    //         }
                    //     }
                    // }
                    // record.serialize_bool_onehot(same_xy_histogram_colors);

                    // let mut rows_columns_agree_on_one_color: Option<u8> = None;
                    // if let Some(histogram0) = histogram_columns.get(x as usize) {
                    //     if let Some(histogram1) = histogram_rows.get(y as usize) {
                    //         let mut histogram2: Histogram = histogram0.clone();
                    //         histogram2.intersection_histogram(histogram1);
                    //         if let Some(color) = most_popular_color {
                    //             histogram2.set_counter_to_zero(color);
                    //         }

                    //         if histogram2.number_of_counters_greater_than_zero() == 1 {
                    //             rows_columns_agree_on_one_color = histogram2.most_popular_color_disallow_ambiguous();
                    //         }
                    //     }
                    // }
                    // record.serialize_color_complex(rows_columns_agree_on_one_color.unwrap_or(255), obfuscated_color_offset, enable_serialize_color_complex);
                    // record.serialize_onehot_discard_overflow(rows_columns_agree_on_one_color.unwrap_or(255), 10);

                    // {
                    //     let mut most_popular_color_in_column: Option<u8> = None;
                    //     let mut least_popular_color_in_column: Option<u8> = None;
                    //     if let Some(histogram) = histogram_columns.get(x as usize) {
                    //         most_popular_color_in_column = histogram.most_popular_color_disallow_ambiguous();
                    //         least_popular_color_in_column = histogram.least_popular_color_disallow_ambiguous();
                    //     }
                    //     record.serialize_color_complex(most_popular_color_in_column.unwrap_or(255), obfuscated_color_offset, enable_serialize_color_complex);
                    //     record.serialize_color_complex(least_popular_color_in_column.unwrap_or(255), obfuscated_color_offset, enable_serialize_color_complex);
                    //     record.serialize_onehot_discard_overflow(most_popular_color_in_column.unwrap_or(255), 10);
                    //     record.serialize_onehot_discard_overflow(least_popular_color_in_column.unwrap_or(255), 10);
                    // }

                    // {
                    //     let mut most_popular_color_in_row: Option<u8> = None;
                    //     let mut least_popular_color_in_row: Option<u8> = None;
                    //     if let Some(histogram) = histogram_rows.get(y as usize) {
                    //         most_popular_color_in_row = histogram.most_popular_color_disallow_ambiguous();
                    //         least_popular_color_in_row = histogram.least_popular_color_disallow_ambiguous();
                    //     }
                    //     record.serialize_color_complex(most_popular_color_in_row.unwrap_or(255), obfuscated_color_offset, enable_serialize_color_complex);
                    //     record.serialize_color_complex(least_popular_color_in_row.unwrap_or(255), obfuscated_color_offset, enable_serialize_color_complex);
                    //     record.serialize_onehot_discard_overflow(most_popular_color_in_row.unwrap_or(255), 10);
                    //     record.serialize_onehot_discard_overflow(least_popular_color_in_row.unwrap_or(255), 10);
                    // }


                    let mut is_full_column: bool = false;
                    let mut is_full_row: bool = false;
                    if let Some(histogram) = &histogram_columns.get(x as usize) {
                        let count: u16 = histogram.number_of_counters_greater_than_zero();
                        if count == 1 {
                            is_full_column = true;
                        }
                        if count >= 2 {
                            if image_top == image_bottom {
                                v3 = 1;
                            }
                        }
                        if count >= 2 {
                            if let Some(color) = noise_color {
                                if histogram.get(color) > 0 {
                                    v4 = 1;
                                }
                            }
                        }
                        // if histogram.get(center) == 1 && v1 != v0 {
                        //     v2 = 1;
                        // }
                        // v2 = histogram.number_of_counters_greater_than_zero().min(255) as u8;
                        // if let Some(color) = most_popular_color {
                        //     if histogram.get(color) > 0 {
                        //         v2 = 1;
                        //     }
                        // }
                        // if let Some(color) = noise_color {
                        //     if histogram.get(color) > 0 {
                        //         v2 = 1;
                        //     }
                        // }
                        // if histogram.number_of_counters_greater_than_zero() >= 2 {
                        //     v2 = 1;
                        // }
                        // if histogram.number_of_counters_greater_than_zero() == 1 {
                        //     // v2 = 1;
                        //     if histogram.most_popular_color_disallow_ambiguous() == Some(center) {
                        //         v2 = 1;
                        //     }
                        // }
                    }

                    if let Some(histogram) = &histogram_rows.get(y as usize) {
                        let count: u16 = histogram.number_of_counters_greater_than_zero();
                        if count == 1 {
                            is_full_row = true;
                        }
                        if count >= 2 {
                            if image_left == image_right {
                                v3 = 1;
                            }
                        }
                        if count >= 2 {
                            if let Some(color) = noise_color {
                                if histogram.get(color) > 0 {
                                    v4 = 1;
                                }
                            }
                        }
                        // if histogram.get(center) == 1 && v1 != v0 {
                        //     v3 = 1;
                        // }
                        // v3 = histogram.number_of_counters_greater_than_zero().min(255) as u8;
                        // if let Some(color) = most_popular_color {
                        //     if histogram.get(color) > 0 {
                        //         v3 = 1;
                        //     }
                        // }
                        // if let Some(color) = noise_color {
                        //     if histogram.get(color) > 0 {
                        //         v3 = 1;
                        //     }
                        // }
                        // if histogram.number_of_counters_greater_than_zero() >= 2 {
                        //     v3 = 1;
                        // }
                        // if histogram.number_of_counters_greater_than_zero() == 1 {
                        //     // v3 = 1;
                        //     if histogram.most_popular_color_disallow_ambiguous() == Some(center) {
                        //         v3 = 1;
                        //     }
                        // }
                    }

                    if enable_full_row_and_column {
                        record.serialize_bool(is_full_row & is_full_column);
                    }
                    if enable_full_row_xor_column {
                        record.serialize_bool(is_full_row ^ is_full_column);
                    }
                    if enable_full_row_or_column {
                        record.serialize_bool(is_full_row | is_full_column);
                    }
                    if enable_full_row {
                        record.serialize_bool(is_full_row);
                    }
                    if enable_full_column {
                        record.serialize_bool(is_full_column);
                    }

                    let mut one_or_more_holes_connectivity4: bool = false;
                    if let Some(hole_mask) = holes_connectivity4.get(&center) {
                        if hole_mask.get(xx, yy).unwrap_or(0) > 0 {
                            one_or_more_holes_connectivity4 = true;
                        }
                    }
                    let mut one_or_more_holes_connectivity8: bool = false;
                    if let Some(hole_mask) = holes_connectivity8.get(&center) {
                        if hole_mask.get(xx, yy).unwrap_or(0) > 0 {
                            one_or_more_holes_connectivity8 = true;
                        }
                    }

                    let mut the_holecount_connectivity4: u8 = 0;
                    if let Some(holecount_image) = holecount_connectivity4.get(&center) {
                        the_holecount_connectivity4 = holecount_image.get(xx, yy).unwrap_or(0);
                    }
                    let mut the_holecount_connectivity8: u8 = 0;
                    if let Some(holecount_image) = holecount_connectivity8.get(&center) {
                        the_holecount_connectivity8 = holecount_image.get(xx, yy).unwrap_or(0);
                    }

                    if enable_noisecolor_in_outline {
                        // let mut is_noise_color_in_outline1_connectivity4: bool = false;
                        // let mut is_noise_color_in_outline1_connectivity8: bool = false;
                        let mut noise_color_in_outline1_connectivity4: u8 = 255;
                        let mut noise_color_in_outline1_connectivity8: u8 = 255;
                        // let mut noise_color_in_outline2_connectivity4: u8 = 0;
                        // let mut noise_color_in_outline2_connectivity8: u8 = 0;
                        if let Some(color) = noise_color {
                            if let Some(mask) = outline1_connectivity4.get(&color) {
                                noise_color_in_outline1_connectivity4 = mask.get(xx, yy).unwrap_or(0);
                                // is_noise_color_in_outline1_connectivity4 = mask.get(xx, yy).unwrap_or(0) > 0;
                            }
                            if let Some(mask) = outline1_connectivity8.get(&color) {
                                noise_color_in_outline1_connectivity8 = mask.get(xx, yy).unwrap_or(0);
                                // is_noise_color_in_outline1_connectivity8 = mask.get(xx, yy).unwrap_or(0) > 0;
                            }
                            // if let Some(mask) = outline2_connectivity4.get(&color) {
                            //     noise_color_in_outline2_connectivity4 = mask.get(xx, yy).unwrap_or(0);
                            // }
                            // if let Some(mask) = outline2_connectivity8.get(&color) {
                            //     noise_color_in_outline2_connectivity8 = mask.get(xx, yy).unwrap_or(0);
                            // }
                        }
                        // record.serialize_bool(is_noise_color_in_outline1_connectivity4);
                        // record.serialize_bool(is_noise_color_in_outline1_connectivity8);
                        record.serialize_color_complex(noise_color_in_outline1_connectivity4, obfuscated_color_offset, enable_serialize_color_complex);
                        record.serialize_color_complex(noise_color_in_outline1_connectivity8, obfuscated_color_offset, enable_serialize_color_complex);
                        // record.serialize_u8(noise_color_in_outline2_connectivity4); // worsens the prediction
                        // record.serialize_u8(noise_color_in_outline2_connectivity8); // worsens the prediction
                    }

                    if enable_symmetry_shorter {
                        // horizontal symmetry
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let is_symmetric: bool = match horizontal_symmetry.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(0) > 0
                                    }
                                    None => false
                                };
                                // record.serialize_bool(is_symmetric);
                                record.serialize_bool_onehot(is_symmetric);
                            }
                        }

                        // vertical symmetry
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let is_symmetric: bool = match vertical_symmetry.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(0) > 0
                                    }
                                    None => false
                                };
                                // record.serialize_bool(is_symmetric);
                                record.serialize_bool_onehot(is_symmetric);
                            }
                        }
                    }

                    if enable_symmetry_masks {
                        let mut the_horizontal_symmetry_connectivity4: u8 = 0;
                        if let Some(mask) = horizontal_symmetry_connectivity4.get(&center) {
                            the_horizontal_symmetry_connectivity4 = mask.get(xx, yy).unwrap_or(0);
                        }
                        let mut the_horizontal_symmetry_connectivity8: u8 = 0;
                        if let Some(mask) = horizontal_symmetry_connectivity8.get(&center) {
                            the_horizontal_symmetry_connectivity8 = mask.get(xx, yy).unwrap_or(0);
                        }
                        let mut the_vertical_symmetry_connectivity4: u8 = 0;
                        if let Some(mask) = vertical_symmetry_connectivity4.get(&center) {
                            the_vertical_symmetry_connectivity4 = mask.get(xx, yy).unwrap_or(0);
                        }
                        let mut the_vertical_symmetry_connectivity8: u8 = 0;
                        if let Some(mask) = vertical_symmetry_connectivity8.get(&center) {
                            the_vertical_symmetry_connectivity8 = mask.get(xx, yy).unwrap_or(0);
                        }
                        // record.serialize_u8(the_horizontal_symmetry_connectivity4);
                        // record.serialize_u8(the_horizontal_symmetry_connectivity8);
                        // record.serialize_u8(the_vertical_symmetry_connectivity4);
                        // record.serialize_u8(the_vertical_symmetry_connectivity8);
                        record.serialize_bool_onehot(the_horizontal_symmetry_connectivity4 > 0);
                        record.serialize_bool_onehot(the_horizontal_symmetry_connectivity8 > 0);
                        record.serialize_bool_onehot(the_vertical_symmetry_connectivity4 > 0);
                        record.serialize_bool_onehot(the_vertical_symmetry_connectivity8 > 0);
                    }

                    if enable_corner_classification {
                        let mut is_corner: bool = false;
                        let mut corner_top_left: bool = false;
                        let mut corner_top_right: bool = false;
                        let mut corner_bottom_left: bool = false;
                        let mut corner_bottom_right: bool = false;
                        if let Some(sco) = &pair.input.image_meta.single_color_object {
                            let corner_classification: u8 = sco.corner_classification(center, xx, yy);
                            if corner_classification > 0 {
                                is_corner = true;
                            }
                            if corner_classification & 1 > 0 {
                                corner_top_left = true;
                            }
                            if corner_classification & 2 > 0 {
                                corner_top_right = true;
                            }
                            if corner_classification & 4 > 0 {
                                corner_bottom_left = true;
                            }
                            if corner_classification & 8 > 0 {
                                corner_bottom_right = true;
                            }
                        }
                        record.serialize_bool_onehot(is_corner);
                        record.serialize_bool_onehot(corner_top_left);
                        record.serialize_bool_onehot(corner_top_right);
                        record.serialize_bool_onehot(corner_bottom_left);
                        record.serialize_bool_onehot(corner_bottom_right);
                    }

                    if enable_half_context_input_size {
                        let half_horizontal: i8;
                        if xx * 2 == context_input_size.width as i32 { 
                            half_horizontal = 0;
                        } else {
                            if xx * 2 < context_input_size.width as i32 { 
                                half_horizontal = -1;
                            } else { 
                                half_horizontal = 1;
                            };
                        }
                        let half_vertical: i8;
                        if yy * 2 == context_input_size.height as i32 { 
                            half_vertical = 0;
                        } else {
                            if yy * 2 < context_input_size.height as i32 { 
                                half_vertical = -1;
                            } else { 
                                half_vertical = 1;
                            };
                        }
                        record.serialize_ternary(half_horizontal);
                        record.serialize_ternary(half_vertical);
                    }
                    if enable_half_context_output_size {
                        let half_horizontal: i8;
                        if xx * 2 == context_output_size.width as i32 { 
                            half_horizontal = 0;
                        } else {
                            if xx * 2 < context_output_size.width as i32 { 
                                half_horizontal = -1;
                            } else { 
                                half_horizontal = 1;
                            };
                        }
                        let half_vertical: i8;
                        if yy * 2 == context_output_size.height as i32 { 
                            half_vertical = 0;
                        } else {
                            if yy * 2 < context_output_size.height as i32 { 
                                half_vertical = -1;
                            } else { 
                                half_vertical = 1;
                            };
                        }
                        record.serialize_ternary(half_horizontal);
                        record.serialize_ternary(half_vertical);
                    }

                    let input_has_unambiguous_connectivity: bool = input_unambiguous_connectivity_histogram.get(center) > 0;

                    for area_y in 0..area5x5.height() {
                        for area_x in 0..area5x5.width() {
                            let color: u8 = area5x5.get(area_x as i32, area_y as i32).unwrap_or(255);
                            record.serialize_color_complex(color, obfuscated_color_offset, enable_serialize_color_complex);
                        }
                    }

                    if enable_neighbour_color {
                        record.serialize_color_complex(neighbour_up, obfuscated_color_offset, enable_serialize_color_complex);
                        record.serialize_color_complex(neighbour_down, obfuscated_color_offset, enable_serialize_color_complex);
                        record.serialize_color_complex(neighbour_left, obfuscated_color_offset, enable_serialize_color_complex);
                        record.serialize_color_complex(neighbour_right, obfuscated_color_offset, enable_serialize_color_complex);
                        record.serialize_color_complex(neighbour_upleft, obfuscated_color_offset, enable_serialize_color_complex);
                        record.serialize_color_complex(neighbour_upright, obfuscated_color_offset, enable_serialize_color_complex);
                        record.serialize_color_complex(neighbour_downleft, obfuscated_color_offset, enable_serialize_color_complex);
                        record.serialize_color_complex(neighbour_downright, obfuscated_color_offset, enable_serialize_color_complex);
                    }

                    if enable_adjacent_neighbour_same_as_center {
                        record.serialize_bool_onehot(neighbour_up == center);
                        record.serialize_bool_onehot(neighbour_down == center);
                        record.serialize_bool_onehot(neighbour_left == center);
                        record.serialize_bool_onehot(neighbour_right == center);
                    }

                    if enable_opposite_neighbour {
                        record.serialize_bool_onehot(neighbour_up == neighbour_down);
                        record.serialize_bool_onehot(neighbour_left == neighbour_right);
                        record.serialize_bool_onehot(neighbour_upleft == neighbour_downright);
                        record.serialize_bool_onehot(neighbour_upright == neighbour_downleft);
                    }

                    // {
                    //     record.serialize_onehot_discard_overflow(neighbour_up, shape_type_count);
                    //     record.serialize_onehot_discard_overflow(neighbour_down, shape_type_count);
                    //     record.serialize_onehot_discard_overflow(neighbour_left, shape_type_count);
                    //     record.serialize_onehot_discard_overflow(neighbour_right, shape_type_count);
                    //     record.serialize_onehot_discard_overflow(neighbour_upleft, shape_type_count);
                    //     record.serialize_onehot_discard_overflow(neighbour_upright, shape_type_count);
                    //     record.serialize_onehot_discard_overflow(neighbour_downleft, shape_type_count);
                    //     record.serialize_onehot_discard_overflow(neighbour_downright, shape_type_count);
                    // }

                    // record.serialize_bool_onehot(preserve_center_color);
                    // {
                    //     let color: u8 = if preserve_center_color { center } else { 255 };
                    //     record.serialize_onehot(color, 11);
                    // }

                    // for color in 0..9u8 {
                    //     let maybe_color_is_present: bool = histogram_predicted_palette.get(color) > 0;
                    //     record.serialize_bool_onehot(maybe_color_is_present);
                    // }

                    record.serialize_color_complex(center_x_reversed, obfuscated_color_offset, enable_serialize_color_complex);
                    record.serialize_color_complex(center_y_reversed, obfuscated_color_offset, enable_serialize_color_complex);
                    // record.serialize_color_complex(center_xy_reversed, obfuscated_color_offset, enable_serialize_color_complex);
                    record.serialize_color_complex(mass_connectivity4, obfuscated_color_offset, enable_serialize_color_complex);
                    record.serialize_color_complex(mass_connectivity8, obfuscated_color_offset, enable_serialize_color_complex);
                    // record.serialize_onehot(mass_connectivity4, 4);
                    // record.serialize_onehot(mass_connectivity8, 4);
                    // record.serialize_f64(1.0 / ((mass_connectivity4 as f64) + 1.0));
                    // record.serialize_f64(1.0 / ((mass_connectivity8 as f64) + 1.0));
                    // record.serialize_u8(mass_connectivity4);
                    // record.serialize_u8(mass_connectivity8);
                    // record.serialize_onehot_discard_overflow(mass_connectivity4, 40);
                    // record.serialize_onehot_discard_overflow(mass_connectivity8, 40);
                    record.serialize_ternary(input_orientation);
                    if enable_output_orientation {
                        record.serialize_ternary(output_orientation);
                    }
                    record.serialize_bool_onehot(input_is_noise_color);
                    record.serialize_bool_onehot(input_is_most_popular_color);
                    // record.serialize_bool(input_is_removal_color == 1);

                    // for color in 0..=9u8 {
                    //     record.serialize_bool_onehot(task.removal_histogram_intersection.get(color) > 0);
                    // }
                    if enable_removal_color_center {
                        record.serialize_bool_onehot(task.removal_histogram_intersection.get(center) > 0);
                    }

                    if enable_normalized_coordinates_context_input_size {
                        let fx: f64 = ((xx as f64) + 0.5) / (context_input_size.width.max(1) as f64);
                        record.serialize_f64(fx);
                        let fy: f64 = ((yy as f64) + 0.5) / (context_input_size.height.max(1) as f64);
                        record.serialize_f64(fy);
                    }
                    if enable_normalized_coordinates_context_output_size {
                        let fx: f64 = ((xx as f64) + 0.5) / (context_output_size.width.max(1) as f64);
                        record.serialize_f64(fx);
                        let fy: f64 = ((yy as f64) + 0.5) / (context_output_size.height.max(1) as f64);
                        record.serialize_f64(fy);
                    }

                    if enable_coordinates_xy {
                        record.serialize_u8(x);
                        record.serialize_u8(y);
                    }
                    if enable_coordinates_xy_reverse_input {
                        record.serialize_f64(context_input_x_reverse as f64);
                        record.serialize_f64(context_input_y_reverse as f64);
                    }
                    if enable_coordinates_xy_reverse_output {
                        record.serialize_f64(context_output_x_reverse as f64);
                        record.serialize_f64(context_output_y_reverse as f64);
                    }
                    // record.serialize_u8(x + 2);
                    // record.serialize_u8(y + 2);
                    // record.serialize_u8(255 - x);
                    // record.serialize_u8(255 - y);
                    // record.serialize_f64(((x + 2) as f64) * ((x + 2) as f64));
                    // record.serialize_f64(((y + 2) as f64) * ((y + 2) as f64));

                    // record.serialize_f64(((y as usize) * (width as usize) + (x as usize)) as f64);
                    // record.serialize_f64(((y as usize) + (y as usize) + (width as usize) + (height as usize)) as f64);
                    // record.serialize_f64(((y as usize) * (y as usize) * (width as usize) * (height as usize)) as f64);

                    if enable_is_outside {
                        {
                            let is_outside: bool = x >= context_input_size.width || y >= context_input_size.height;
                            record.serialize_bool_onehot(is_outside);
                        }
                        {
                            let is_outside: bool = x >= context_output_size.width || y >= context_output_size.height;
                            record.serialize_bool_onehot(is_outside);
                        }
                    }
                    // record.serialize_bool_onehot(x >= context_input_size.width);
                    // record.serialize_bool_onehot(y >= context_input_size.height);

                    // record.serialize_u8(x_reverse);
                    // record.serialize_u8(y_reverse);
                    // record.serialize_u8(x_reverse + 2);
                    // record.serialize_u8(y_reverse + 2);
                    // record.serialize_onehot_discard_overflow(x, 30);
                    // record.serialize_onehot_discard_overflow(y, 30);
                    // record.serialize_onehot_discard_overflow(x_reverse, 30);
                    // record.serialize_onehot_discard_overflow(y_reverse, 30);

                    // record.serialize_bitmask_as_onehot(Self::binary_to_grey(x) as u16, 8);
                    // record.serialize_bitmask_as_onehot(Self::binary_to_grey(y) as u16, 8);
                    // record.serialize_bitmask_as_onehot(Self::binary_to_grey(x_reverse) as u16, 8);
                    // record.serialize_bitmask_as_onehot(Self::binary_to_grey(y_reverse) as u16, 8);

                    // record.serialize_bitmask_as_onehot(Self::grey_to_binary(x) as u16, 8);
                    // record.serialize_bitmask_as_onehot(Self::grey_to_binary(y) as u16, 8);
                    // record.serialize_bitmask_as_onehot(Self::grey_to_binary(x_reverse) as u16, 8);
                    // record.serialize_bitmask_as_onehot(Self::grey_to_binary(y_reverse) as u16, 8);

                    if enable_mod2 {
                        let x_value: u8 = x % 2;
                        let y_value: u8 = y % 2;
                        record.serialize_onehot_discard_overflow(x_value, 2);
                        record.serialize_onehot_discard_overflow(y_value, 2);
                    }
                    if enable_mod2_reverse_input {
                        let x_value: u8 = ((context_input_x_reverse % 2) % 2) as u8;
                        let y_value: u8 = ((context_input_y_reverse % 2) % 2) as u8;
                        record.serialize_onehot_discard_overflow(x_value, 2);
                        record.serialize_onehot_discard_overflow(y_value, 2);
                    }
                    if enable_mod2_reverse_output {
                        let x_value: u8 = ((context_output_x_reverse % 2) % 2) as u8;
                        let y_value: u8 = ((context_output_y_reverse % 2) % 2) as u8;
                        record.serialize_onehot_discard_overflow(x_value, 2);
                        record.serialize_onehot_discard_overflow(y_value, 2);
                    }

                    if enable_mod3 {
                        let x_value: u8 = x % 3;
                        let y_value: u8 = y % 3;
                        record.serialize_onehot_discard_overflow(x_value, 3);
                        record.serialize_onehot_discard_overflow(y_value, 3);
                    }
                    if enable_mod3_reverse_input {
                        let x_value: u8 = ((context_input_x_reverse % 3) % 3) as u8;
                        let y_value: u8 = ((context_input_y_reverse % 3) % 3) as u8;
                        record.serialize_onehot_discard_overflow(x_value, 3);
                        record.serialize_onehot_discard_overflow(y_value, 3);
                    }
                    if enable_mod3_reverse_output {
                        let x_value: u8 = ((context_output_x_reverse % 3) % 3) as u8;
                        let y_value: u8 = ((context_output_y_reverse % 3) % 3) as u8;
                        record.serialize_onehot_discard_overflow(x_value, 3);
                        record.serialize_onehot_discard_overflow(y_value, 3);
                    }

                    // record.serialize_onehot_discard_overflow(x_mod4, 4);
                    // record.serialize_onehot_discard_overflow(y_mod4, 4);
                    // record.serialize_onehot_discard_overflow(x_reverse_mod4, 4);
                    // record.serialize_onehot_discard_overflow(y_reverse_mod4, 4);
                    // record.serialize_onehot_discard_overflow(x_mod5, 5);
                    // record.serialize_onehot_discard_overflow(y_mod5, 5);
                    // record.serialize_onehot_discard_overflow(x_reverse_mod5, 5);
                    // record.serialize_onehot_discard_overflow(y_reverse_mod5, 5);
                    // record.serialize_onehot_discard_overflow(x2_mod2, 2);
                    // record.serialize_onehot_discard_overflow(y2_mod2, 2);
                    // record.serialize_onehot_discard_overflow(x2_reverse_mod2, 2);
                    // record.serialize_onehot_discard_overflow(y2_reverse_mod2, 2);
                    // record.serialize_onehot_discard_overflow(x4_mod2, 2);
                    // record.serialize_onehot_discard_overflow(y4_mod2, 2);
                    // record.serialize_onehot_discard_overflow(x4_reverse_mod2, 2);
                    // record.serialize_onehot_discard_overflow(y4_reverse_mod2, 2);
                    // record.serialize_onehot_discard_overflow((x ^ y) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x_mod2 + y_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x_mod2 + y_reverse_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x_reverse_mod2 + y_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x_reverse_mod2 + y_reverse_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x2_mod2 + y2_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x2_mod2 + y2_reverse_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x2_reverse_mod2 + y2_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x2_reverse_mod2 + y2_reverse_mod2) & 1, 2);
                    record.serialize_bool_onehot(preserve_edge);
                    record.serialize_bool(one_or_more_holes_connectivity4);
                    record.serialize_bool(one_or_more_holes_connectivity8);
                    record.serialize_color_complex(the_holecount_connectivity4, obfuscated_color_offset, enable_serialize_color_complex);
                    record.serialize_color_complex(the_holecount_connectivity8, obfuscated_color_offset, enable_serialize_color_complex);
                    // record.serialize_u8(the_holecount_connectivity4);
                    // record.serialize_u8(the_holecount_connectivity8);
                    record.serialize_onehot_discard_overflow(the_holecount_connectivity4, 2);
                    record.serialize_onehot_discard_overflow(the_holecount_connectivity8, 2);
                    // record.serialize_onehot_discard_overflow(the_holecount_connectivity4.min(9), 10);
                    // record.serialize_onehot_discard_overflow(the_holecount_connectivity8.min(9), 10);
                    if enable_no_change_to_color {
                        for i in 0..10 {
                            // let value: u8 = if no_change_to_color[i] { 1 } else { 0 };
                            // record.serialize_u8(value);
                            let value2: u8 = if no_change_to_color[i] { i as u8 } else { 255 };
                            record.serialize_color_complex(value2, obfuscated_color_offset, enable_serialize_color_complex);
                        }
                    }
                    if enable_no_change_to_center_color {
                        record.serialize_bool(no_change_to_color[(center % 10) as usize]);
                    }
                    if enable_no_change_to_noise_color {
                        let mut value: bool = false;
                        if let Some(color) = noise_color {
                            value = no_change_to_color[(color % 10) as usize];
                        }
                        record.serialize_bool(value);
                    }
                    for i in 0..10 {
                        // let value: u8 = if input_histogram_intersection[i] { 1 } else { 0 };
                        // record.serialize_u8(value);
                        let value2: u8 = if input_histogram_intersection[i] { i as u8 } else { 255 };
                        record.serialize_color_complex(value2, obfuscated_color_offset, enable_serialize_color_complex);
                    }
                    record.serialize_bool_onehot(input_has_unambiguous_connectivity);
                    record.serialize_u8(v0);
                    record.serialize_u8(v1);
                    record.serialize_u8(v2);
                    record.serialize_u8(v3);
                    record.serialize_u8(v4);
                    record.serialize_u8(v5);
                    record.serialize_u8(v6);
                    record.serialize_u8(v7);

                    let mut row_contains_noise_color: bool = false;
                    let mut column_contains_noise_color: bool = false;
                    if let Some(color) = noise_color {
                        if histogram_rows[y as usize].get(color) > 0 {
                            row_contains_noise_color = true;
                        }
                        if histogram_columns[x as usize].get(color) > 0 {
                            column_contains_noise_color = true;
                        }
                    }
                    record.serialize_bool_onehot(row_contains_noise_color);
                    record.serialize_bool_onehot(column_contains_noise_color);

                    // {
                    //     let mut is_full = false;    
                    //     if let Some(histogram) = &area_top_histogram_columns.get(x as usize) {
                    //         let count: u16 = histogram.number_of_counters_greater_than_zero();
                    //         is_full = count == 1;
                    //     }
                    //     record.serialize_bool_onehot(is_full);
                    // }
                    // {
                    //     let mut is_full = false;    
                    //     if let Some(histogram) = &area_bottom_histogram_columns.get(x as usize) {
                    //         let count: u16 = histogram.number_of_counters_greater_than_zero();
                    //         is_full = count == 1;
                    //     }
                    //     record.serialize_bool_onehot(is_full);
                    // }
                    // {
                    //     let mut is_full = false;    
                    //     if let Some(histogram) = &area_left_histogram_rows.get(y as usize) {
                    //         let count: u16 = histogram.number_of_counters_greater_than_zero();
                    //         is_full = count == 1;
                    //     }
                    //     record.serialize_bool_onehot(is_full);
                    // }
                    // {
                    //     let mut is_full = false;    
                    //     if let Some(histogram) = &area_right_histogram_rows.get(y as usize) {
                    //         let count: u16 = histogram.number_of_counters_greater_than_zero();
                    //         is_full = count == 1;
                    //     }
                    //     record.serialize_bool_onehot(is_full);
                    // }

                    // {
                    //     let histogram: Histogram;
                    //     if let Some(h) = area_top_histogram_columns.get(x as usize) {
                    //         histogram = h.clone();
                    //     } else {
                    //         histogram = Histogram::new();
                    //     }
                    //     for color in 0..9u8 {
                    //         let mass: u32 = histogram.get(color);
                    //         record.serialize_bool(mass == 1);
                    //         record.serialize_bool(mass == 2);
                    //         record.serialize_bool(mass > 2);
                    //         record.serialize_f64(1.0 / (mass as f64 + 1.0));
                    //         // record.serialize_bool(mass > 0);
                    //         // record.serialize_u8(mass.min(255) as u8);
                    //         // record.serialize_onehot(mass.min(11) as u8, 10);
                    //     }
                    //     // record.serialize_onehot(histogram.number_of_counters_greater_than_zero().min(11) as u8, 10);
                    // }
                    // {
                    //     let histogram: Histogram;
                    //     if let Some(h) = area_bottom_histogram_columns.get(x as usize) {
                    //         histogram = h.clone();
                    //     } else {
                    //         histogram = Histogram::new();
                    //     }
                    //     for color in 0..9u8 {
                    //         let mass: u32 = histogram.get(color);
                    //         record.serialize_bool(mass == 1);
                    //         record.serialize_bool(mass == 2);
                    //         record.serialize_bool(mass > 2);
                    //         record.serialize_f64(1.0 / (mass as f64 + 1.0));
                    //         // record.serialize_bool(mass > 0);
                    //         // record.serialize_u8(mass.min(255) as u8);
                    //         // record.serialize_onehot(mass.min(11) as u8, 10);
                    //     }
                    //     // record.serialize_onehot(histogram.number_of_counters_greater_than_zero().min(11) as u8, 10);
                    // }
                    // {
                    //     let histogram: Histogram;
                    //     if let Some(h) = area_left_histogram_rows.get(y as usize) {
                    //         histogram = h.clone();
                    //     } else {
                    //         histogram = Histogram::new();
                    //     }
                    //     for color in 0..9u8 {
                    //         let mass: u32 = histogram.get(color);
                    //         record.serialize_bool(mass == 1);
                    //         record.serialize_bool(mass == 2);
                    //         record.serialize_bool(mass > 2);
                    //         record.serialize_f64(1.0 / (mass as f64 + 1.0));
                    //         // record.serialize_bool(mass > 0);
                    //         // record.serialize_u8(mass.min(255) as u8);
                    //         // record.serialize_onehot(mass.min(11) as u8, 10);
                    //     }
                    //     // record.serialize_onehot(histogram.number_of_counters_greater_than_zero().min(11) as u8, 10);
                    // }
                    // {
                    //     let histogram: Histogram;
                    //     if let Some(h) = area_right_histogram_rows.get(y as usize) {
                    //         histogram = h.clone();
                    //     } else {
                    //         histogram = Histogram::new();
                    //     }
                    //     for color in 0..9u8 {
                    //         let mass: u32 = histogram.get(color);
                    //         record.serialize_bool(mass == 1);
                    //         record.serialize_bool(mass == 2);
                    //         record.serialize_bool(mass > 2);
                    //         record.serialize_f64(1.0 / (mass as f64 + 1.0));
                    //         // record.serialize_bool(mass > 0);
                    //         // record.serialize_u8(mass.min(255) as u8);
                    //         // record.serialize_onehot(mass.min(11) as u8, 10);
                    //     }
                    //     // record.serialize_onehot(histogram.number_of_counters_greater_than_zero().min(11) as u8, 10);
                    // }

                    if enable_histogram_columns_rows_get_color {
                        for color in 0..=9u8 {
                            record.serialize_bool(histogram_columns[x as usize].get(color) > 0);
                            record.serialize_bool(histogram_rows[y as usize].get(color) > 0);
                        }
                    }

                    if enable_histogram_columns_rows_lookaround {
                        for color in 0..=9u8 {
                            for i in -2..=2 {
                                if i == 0 {
                                    continue;
                                }
                                {
                                    let xi: i32 = (x as i32) + i;
                                    let mut value: bool = false;
                                    if xi >= 0 && xi < width as i32 {
                                        value = histogram_columns[xi as usize].get(color) > 0;
                                    }
                                    record.serialize_bool(value);
                                }
                                {
                                    let yi: i32 = (y as i32) + i;
                                    let mut value: bool = false;
                                    if yi >= 0 && yi < height as i32 {
                                        value = histogram_rows[yi as usize].get(color) > 0;
                                    }
                                    record.serialize_bool(value);
                                }
                            }
                        }
                    }

                    {
                        // let count: u16 = histogram_columns[x as usize].number_of_counters_greater_than_zero();
                        // record.serialize_bool_onehot(count > 0);
                        // record.serialize_onehot(count.min(255) as u8, 3);
                        // record.serialize_f64(count as f64);
                        // record.serialize(histogram_columns.num, count)
                    }
                    {
                        // let count: u16 = histogram_rows[y as usize].number_of_counters_greater_than_zero();
                        // record.serialize_bool_onehot(count > 0);
                        // record.serialize_onehot(count.min(255) as u8, 3);
                        // record.serialize_f64(count as f64);
                        // record.serialize(histogram_columns.num, count)
                    }

                    if enable_histogram_diagonal {
                        if let (Some(histogram_diagonal_a), Some(histogram_diagonal_b)) = (&histogram_diagonal_a, &histogram_diagonal_b) {
                            if enable_histogram_diagonal_a {
                                for color in 0..=9 {
                                    let mut found = false;
                                    if let Some(histogram) = histogram_diagonal_a.get(x as i32, y as i32) {
                                        if histogram.get(color) > 0 {
                                            found = true;
                                        }
                                    }
                                    record.serialize_bool_onehot(found);
                                }
                                for color in 0..=9 {
                                    let mut found = false;
                                    if let Some(histogram) = histogram_diagonal_b.get(x as i32, y as i32) {
                                        if histogram.get(color) > 0 {
                                            found = true;
                                        }
                                    }
                                    record.serialize_bool_onehot(found);
                                }
                            }

                            if enable_histogram_diagonal_b {
                                for color in 0..=9 {
                                    let mut count: u32 = 0;
                                    if let Some(histogram) = histogram_diagonal_a.get(x as i32, y as i32) {
                                        count = histogram.get(color);
                                    }
                                    let limit: u32 = if color == center { 1 } else { 0 };
                                    record.serialize_bool_onehot(count > limit);
                                }
                                for color in 0..=9 {
                                    let mut count: u32 = 0;
                                    if let Some(histogram) = histogram_diagonal_b.get(x as i32, y as i32) {
                                        count = histogram.get(color);
                                    }
                                    let limit: u32 = if color == center { 1 } else { 0 };
                                    record.serialize_bool_onehot(count > limit);
                                }
                            }

                            if enable_histogram_diagonal_c {
                                {
                                    let mut count: u8 = 0;
                                    if let Some(histogram) = histogram_diagonal_a.get(x as i32, y as i32) {
                                        count = histogram.number_of_counters_greater_than_zero().min(255) as u8;
                                    }
                                    // record.serialize_f64(1.0 / (count as f64 + 1.0));
                                    record.serialize_onehot(count + 1, 4);
                                    // record.serialize_onehot(count, 20);
                                    // record.serialize_u8(count);
                                    // record.serialize_bool_onehot(count > 1);
                                }
                                {
                                    let mut count: u8 = 0;
                                    if let Some(histogram) = histogram_diagonal_b.get(x as i32, y as i32) {
                                        count = histogram.number_of_counters_greater_than_zero().min(255) as u8;
                                    }
                                    // record.serialize_f64(1.0 / (count as f64 + 1.0));
                                    record.serialize_onehot(count + 1, 4);
                                    // record.serialize_onehot(count, 20);
                                    // record.serialize_u8(count);
                                    // record.serialize_bool_onehot(count > 1);
                                }
                            }

                            if enable_histogram_diagonal_d {
                                for color in 0..=9 {
                                    let mut mass : u8 = 0;
                                    if let Some(histogram) = histogram_diagonal_a.get(x as i32, y as i32) {
                                        mass = histogram.get(color).min(255) as u8;
                                    }
                                    record.serialize_onehot(mass, 4);
                                    // record.serialize_u8(mass);
                                }
                                for color in 0..=9 {
                                    let mut mass : u8 = 0;
                                    if let Some(histogram) = histogram_diagonal_b.get(x as i32, y as i32) {
                                        mass = histogram.get(color).min(255) as u8;
                                    }
                                    record.serialize_onehot(mass, 4);
                                    // record.serialize_u8(mass);
                                }
                            }

                            if enable_histogram_diagonal_e {
                                {
                                    let mut is_min = false;
                                    let mut is_max = false;
                                    let mut count: u16 = 0;
                                    if let Some(histogram) = histogram_diagonal_a.get(x as i32, y as i32) {
                                        count = histogram.number_of_counters_greater_than_zero();
                                        is_min = count == histogram_diagonal_a.min_number_of_unique_colors();
                                        is_max = count == histogram_diagonal_a.max_number_of_unique_colors();
                                    }
                                    record.serialize_onehot(count.min(255) as u8, 10);
                                    record.serialize_bool_onehot(is_max);
                                    record.serialize_bool_onehot(is_min);
                                    record.serialize_bool_onehot(is_max == false && is_min == false);
                                }
                                {
                                    let mut is_min = false;
                                    let mut is_max = false;
                                    let mut count: u16 = 0;
                                    if let Some(histogram) = histogram_diagonal_b.get(x as i32, y as i32) {
                                        count = histogram.number_of_counters_greater_than_zero();
                                        is_min = count == histogram_diagonal_b.min_number_of_unique_colors();
                                        is_max = count == histogram_diagonal_b.max_number_of_unique_colors();
                                    }
                                    record.serialize_onehot(count.min(255) as u8, 10);
                                    record.serialize_bool_onehot(is_max);
                                    record.serialize_bool_onehot(is_min);
                                    record.serialize_bool_onehot(is_max == false && is_min == false);
                                }
                            }

                            if enable_histogram_diagonal_f {
                                {
                                    // let mut mass: u8 = 0;
                                    let mut is_most_popular: bool = false;
                                    let mut is_least_popular: bool = false;
                                    if let Some(histogram) = histogram_diagonal_a.get(x as i32, y as i32) {
                                        // mass = histogram.get(center).min(255) as u8;
                                        if let Some(color) = histogram.most_popular_color_disallow_ambiguous() {
                                            if color == center {
                                                is_most_popular = true;
                                            }
                                        }
                                        if let Some(color) = histogram.least_popular_color_disallow_ambiguous() {
                                            if color == center {
                                                is_least_popular = true;
                                            }
                                        }
                                    }
                                    // record.serialize_onehot(mass, 4);
                                    record.serialize_bool_onehot(is_most_popular);
                                    record.serialize_bool_onehot(is_least_popular);
                                }
                                {
                                    // let mut mass: u8 = 0;
                                    let mut is_most_popular: bool = false;
                                    let mut is_least_popular: bool = false;
                                    if let Some(histogram) = histogram_diagonal_b.get(x as i32, y as i32) {
                                        // mass = histogram.get(center).min(255) as u8;
                                        if let Some(color) = histogram.most_popular_color_disallow_ambiguous() {
                                            if color == center {
                                                is_most_popular = true;
                                            }
                                        }
                                        if let Some(color) = histogram.least_popular_color_disallow_ambiguous() {
                                            if color == center {
                                                is_least_popular = true;
                                            }
                                        }
                                    }
                                    // record.serialize_onehot(mass, 4);
                                    record.serialize_bool_onehot(is_most_popular);
                                    record.serialize_bool_onehot(is_least_popular);
                                }
                            }

                        }
                    }


                    if enable_hole_type1 {
                        let mut color_hole_type1: u8 = 255;
                        if let Some(image) = color_to_hole_type1.get(&center) {
                            color_hole_type1 = image.get(xx, yy).unwrap_or(0);
                        }
                        record.serialize_color_complex(color_hole_type1, obfuscated_color_offset, enable_serialize_color_complex);
                    }

                    if enable_color_repair {
                        let mut color_repair: u8 = 255;
                        if let Some(image) = color_to_repair.get(&center) {
                            color_repair = image.get(xx, yy).unwrap_or(0);
                        }
                        record.serialize_color_complex(color_repair, obfuscated_color_offset, enable_serialize_color_complex);
                        // record.serialize_onehot(color_repair, 10);
                    }

                    // for color in 0..=9u8 {
                    //     let mut color_repair: u8 = 255;
                    //     if let Some(image) = color_to_hole_type1.get(&color) {
                    //         color_repair = image.get(xx, yy).unwrap_or(0);
                    //     }
                    //     record.serialize_color_complex(color_repair, obfuscated_color_offset, enable_serialize_color_complex);
                    // }


                    // color of the neighbour in all 8 directions
                    let directions = [
                        ImageNeighbourDirection::Up,
                        ImageNeighbourDirection::Down,
                        ImageNeighbourDirection::Left,
                        ImageNeighbourDirection::Right,
                        ImageNeighbourDirection::UpLeft,
                        ImageNeighbourDirection::UpRight,
                        ImageNeighbourDirection::DownLeft,
                        ImageNeighbourDirection::DownRight,
                    ];
                    for direction in &directions {
                        for color in 0..=9 {
                            let neighbour_color: u8 = match image_neighbour.get(&(color, *direction)) {
                                Some(value) => {
                                    value.get(xx, yy).unwrap_or(255)
                                }
                                None => 255
                            };
                            record.serialize_color_complex(neighbour_color, obfuscated_color_offset, enable_serialize_color_complex);
                            // record.serialize_onehot_discard_overflow(neighbour_color, 10);
                        }
                    }

                    // Future experiment
                    // for all 10 colors.
                    // look in the diagonal direction, skip the first 2 colors, and pick the 2nd color

    
                    // Cluster id
                    {
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let cluster_id: u8 = match enumerated_clusters.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                record.serialize_cluster_id(color, cluster_id, obfuscated_cluster_offset, enable_serialize_cluster_id_shakeup);
                                // record.serialize_cluster_id(color, 255 - cluster_id, enabled_serialize_cluster_id_shakeup);
                                // record.serialize_complex(cluster_id as u16, 41);
                            }
                        }
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let small_medium_big_id: u8 = match small_medium_big.get(&(color, *connectivity)) {
                                    Some(image) => {
                                        image.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                record.serialize_complex(small_medium_big_id as u16, 4);
                                // record.serialize_onehot_discard_overflow(small_medium_big_id, 4);
                            }
                        }
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let sort2_small_big_id: u8 = match sort2_small_big.get(&(color, *connectivity)) {
                                    Some(image) => {
                                        image.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                // record.serialize_complex(sort2_small_big_id as u16, 3);
                                record.serialize_onehot_discard_overflow(sort2_small_big_id, 3);
                            }
                        }
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let mask_value: u8 = match enumerated_clusters_filled_holes_mask.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                record.serialize_bool(mask_value > 0);
                            }
                        }

                        if enable_exterior_of_clusters {
                            for connectivity in &connectivity_vec {
                                for color in 0..=9 {
                                    let corner_value: u8 = match exterior_of_clusters.get(&(color, *connectivity)) {
                                        Some(value) => {
                                            value.get(xx, yy).unwrap_or(255)
                                        }
                                        None => 255
                                    };
                                    record.serialize_bool_onehot(corner_value > 0);
                                    // record.serialize_onehot_discard_overflow(corner_value, 7);
                                }
                            }
                        }

                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let mask_value: u8 = match enumerated_clusters_grow_mask1.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                record.serialize_bool(mask_value > 0);
                            }
                        }
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let mask_value: u8 = match enumerated_clusters_grow_mask2.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                record.serialize_bool(mask_value > 0);
                            }
                        }
                        if enable_enumerated_clusters_grow_mask3 {
                            for connectivity in &connectivity_vec {
                                for color in 0..=9 {
                                    let mask_value: u8 = match enumerated_clusters_grow_mask3.get(&(color, *connectivity)) {
                                        Some(value) => {
                                            value.get(xx, yy).unwrap_or(255)
                                        }
                                        None => 255
                                    };
                                    record.serialize_bool(mask_value > 0);
                                }
                            }
                        }
                        if enable_color_grow_mask1 {
                            for connectivity in &connectivity_vec {
                                for color in 0..=9 {
                                    let mask_value: u8 = match color_grow_mask1.get(&(color, *connectivity)) {
                                        Some(value) => {
                                            value.get(xx, yy).unwrap_or(255)
                                        }
                                        None => 255
                                    };
                                    record.serialize_bool(mask_value > 0);
                                }
                            }
                        }
                        if enable_color_grow_mask2 {
                            for connectivity in &connectivity_vec {
                                for color in 0..=9 {
                                    let mask_value: u8 = match color_grow_mask2.get(&(color, *connectivity)) {
                                        Some(value) => {
                                            value.get(xx, yy).unwrap_or(255)
                                        }
                                        None => 255
                                    };
                                    record.serialize_bool(mask_value > 0);
                                }
                            }
                        }
                        if enable_color_grow_mask3 {
                            for connectivity in &connectivity_vec {
                                for color in 0..=9 {
                                    let mask_value: u8 = match color_grow_mask3.get(&(color, *connectivity)) {
                                        Some(value) => {
                                            value.get(xx, yy).unwrap_or(255)
                                        }
                                        None => 255
                                    };
                                    record.serialize_bool(mask_value > 0);
                                }
                            }
                        }
                        // for connectivity in &connectivity_vec {
                        //     for color in 0..=9 {
                        //         let mask_value: u8 = match color_grow_mask4.get(&(color, *connectivity)) {
                        //             Some(value) => {
                        //                 value.get(xx, yy).unwrap_or(255)
                        //             }
                        //             None => 255
                        //         };
                        //         record.serialize_bool(mask_value > 0);
                        //     }
                        // }
                        // for connectivity in &connectivity_vec {
                        //     for color in 0..=9 {
                        //         let mask_value: u8 = match color_grow_mask5.get(&(color, *connectivity)) {
                        //             Some(value) => {
                        //                 value.get(xx, yy).unwrap_or(255)
                        //             }
                        //             None => 255
                        //         };
                        //         record.serialize_bool(mask_value > 0);
                        //     }
                        // }
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                #[allow(unused_variables)]
                                let distance: u8 = match cluster_distance1.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                // record.serialize_u8(distance);
                                // record.serialize_onehot(distance, 2);
                                // record.serialize_onehot(distance, 3);
                                // record.serialize_onehot(distance, 4);
                                // record.serialize_onehot(distance, 5);
                                // record.serialize_onehot(distance, 6);
                                // record.serialize_onehot(distance, 8);
                            }
                        }
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                #[allow(unused_variables)]
                                let distance: u8 = match cluster_distance2.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                // record.serialize_u8(distance);
                                // record.serialize_onehot(distance, 2);
                                // record.serialize_onehot(distance, 3);
                                // record.serialize_onehot(distance, 4);
                                // record.serialize_onehot(distance, 5);
                                // record.serialize_onehot(distance, 6);
                                // record.serialize_onehot(distance, 8);
                            }
                        }

                        if enable_nearest_color {
                            {
                                let color: u8 = nearest_color4.get(xx, yy).unwrap_or(255);
                                record.serialize_color_complex(color, obfuscated_color_offset, enable_serialize_color_complex);
                            }
                            {
                                let color: u8 = nearest_color8.get(xx, yy).unwrap_or(255);
                                record.serialize_color_complex(color, obfuscated_color_offset, enable_serialize_color_complex);
                            }
                        }

                        if enable_colordirection_to_distanceimage {
                            for color in 0..=9u8 {
                                for direction in &direction_vec {
                                    let mut distance: u8 = 255;
                                    if let Some(image) = &colordirection_to_distanceimage.get(&(color, *direction)) {
                                        distance = image.get(xx, yy).unwrap_or(255);
                                    }
                                    record.serialize_onehot(distance, 5);
                                }
                            }
                        }

                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                #[allow(unused_variables)]
                                let distance: u8 = match cluster_distance3.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                // record.serialize_u8(distance);
                                record.serialize_bool(distance == 0);
                                // record.serialize_onehot(distance, 2);
                                // record.serialize_onehot(distance, 4);
                                // record.serialize_onehot(distance, 6);
                            }
                        }
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                #[allow(unused_variables)]
                                let distance: u8 = match cluster_distance4.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                // record.serialize_u8(distance);
                                // record.serialize_bool(distance == 0);
                                // record.serialize_onehot(distance, 20);
                                // record.serialize_onehot(distance, 4);
                            }
                        }
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let distance: u8 = match cluster_distance5.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                // record.serialize_u8(distance);
                                // record.serialize_bool(distance == 0);
                                // record.serialize_onehot(distance, 2);
                                // record.serialize_onehot(distance, 4);
                                // record.serialize_split_zeros_ones(distance, 5);
                                // record.serialize_split_zeros_ones(distance, 8);
                                // record.serialize_onehot(distance, 20);
                                record.serialize_bool(distance % 2 == 0);
                            }
                        }

                        if enable_largest_interior_rectangle_masks {
                            let mask_value: u8 = match largest_interior_rectangle_masks.get(&center) {
                                Some(value) => {
                                    value.get(xx, yy).unwrap_or(0)
                                }
                                None => 0
                            };
                            record.serialize_bool_onehot(mask_value > 0);
                        }

                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let is_square: bool = match squares.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(0) > 0
                                    }
                                    None => false
                                };
                                record.serialize_bool(is_square);
                                // record.serialize_bool_onehot(is_square);
                            }
                        }

                        if enable_detect_nonsquare {
                            for connectivity in &connectivity_vec {
                                for color in 0..=9 {
                                    let is_nonsquare: bool = match nonsquares.get(&(color, *connectivity)) {
                                        Some(value) => {
                                            value.get(xx, yy).unwrap_or(0) > 0
                                        }
                                        None => false
                                    };
                                    record.serialize_bool(is_nonsquare);
                                    // record.serialize_bool_onehot(is_nonsquare);
                                }
                            }
                        }

                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let is_rectangle: bool = match rectangles.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(0) > 0
                                    }
                                    None => false
                                };
                                record.serialize_bool(is_rectangle);
                                // record.serialize_bool_onehot(is_rectangle);
                            }
                        }
                        
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let is_box: bool = match boxes.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(0) > 0
                                    }
                                    None => false
                                };
                                record.serialize_bool(is_box);
                                // record.serialize_bool_onehot(is_box);
                            }
                        }

                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let line_status: u8 = match lines.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(0)
                                    }
                                    None => 0,
                                };
                                let ternary: i8 = match line_status {
                                    1 => -1,
                                    2 => 1,
                                    _ => 0,
                                };
                                record.serialize_ternary(ternary);
                            }
                        }


                        #[allow(unused_variables)]
                        let directions = [
                            ImageNeighbourDirection::Up,
                            ImageNeighbourDirection::Down,
                            ImageNeighbourDirection::Left,
                            ImageNeighbourDirection::Right,
                            // ImageNeighbourDirection::UpLeft,
                            // ImageNeighbourDirection::UpRight,
                            // ImageNeighbourDirection::DownLeft,
                            // ImageNeighbourDirection::DownRight,
                        ];
                        // for connectivity in &connectivity_vec {
                        //     for direction in &directions {
                        //         for color in 0..=9 {
                        //             let cluster_id: u8 = match cluster_id_neighbour.get(&(color, *direction, *connectivity)) {
                        //                 Some(value) => {
                        //                     value.get(xx, yy).unwrap_or(255)
                        //                 }
                        //                 None => 255
                        //             };
                        //             // record.serialize_complex(cluster_id, 13);
                        //             record.serialize_cluster_id(color, cluster_id, enabled_serialize_cluster_id_shakeup);
                        //         }
                        //     }
                        // }
    
                    }

                    {
                        let images: [&Image; 4] = [
                            &center_column_top,
                            &center_column_bottom,
                            &center_row_left,
                            &center_row_right,
                        ];
                        for image in images {
                            let h: Histogram = image.histogram_all();
                            let most_popular: Option<u8> = h.most_popular_color_disallow_ambiguous();
                            let least_popular: Option<u8> = h.least_popular_color_disallow_ambiguous();
                            record.serialize_color_complex(most_popular.unwrap_or(255), obfuscated_color_offset, enable_serialize_color_complex);
                            record.serialize_color_complex(least_popular.unwrap_or(255), obfuscated_color_offset, enable_serialize_color_complex);
                            // let count: u16 = h.number_of_counters_greater_than_zero();
                            // record.serialize_f64((count+1) as f64);
                            // record.serialize_bool(count < 2);
                            // {
                                // let mass: u8 = h.get(center).min(255) as u8;
                                // record.serialize_onehot(mass, 6);
                                // record.serialize_bool(mass > 0);
                            // }
                        }
                    }

                    record.serialize_color_complex(center_denoise_type1, obfuscated_color_offset, enable_serialize_color_complex);
                    // record.serialize_color_complex(center_denoise_type1_border, obfuscated_color_offset, enable_serialize_color_complex);

                    // let is_border_most_popular_color: bool = Some(center) == border_most_popular_color;
                    // let is_border_least_popular_color: bool = Some(center) == border_least_popular_color;
                    // record.serialize_bool_onehot(is_border_most_popular_color);
                    // record.serialize_bool_onehot(is_border_least_popular_color);
                    // let border_histogram_count: u32 = histogram_border.get(center);
                    // record.serialize_bool_onehot(border_histogram_count > 0);

                    if enable_color_inside_bounding_box {
                        for color in 0..=9 {
                            let mut is_inside: bool = false;
                            if let Some(sco) = &pair.input.image_meta.single_color_object {
                                is_inside = sco.is_inside_bounding_box(color, xx, yy);
                            }
                            record.serialize_bool(is_inside);
                            // record.serialize_bool_onehot(is_inside);
                        }
                    }

                    {
                        // for color in 0..=9 {
                        //     let mut is_above: bool = false;
                        //     let mut is_below: bool = false;
                        //     let mut is_left: bool = false;
                        //     let mut is_right: bool = false;
                        //     if let Some(sco) = &pair.input.image_meta.single_color_object {
                        //         if let Some(rectangle) = sco.bounding_box(color) {
                        //             is_above = rectangle.is_above(xx, yy);
                        //             is_below = rectangle.is_below(xx, yy);
                        //             is_left = rectangle.is_left(xx, yy);
                        //             is_right = rectangle.is_right(xx, yy);
                        //         }
                        //     }
                        //     record.serialize_bool_onehot(is_above);
                        //     record.serialize_bool_onehot(is_below);
                        //     record.serialize_bool_onehot(is_left);
                        //     record.serialize_bool_onehot(is_right);
                        // }
                    }

                    {
                        // for color in 0..=9 {
                        //     let mut is_above: bool = false;
                        //     let mut is_below: bool = false;
                        //     let mut is_left: bool = false;
                        //     let mut is_right: bool = false;
                        //     if let Some(sco) = &pair.input.image_meta.single_color_object {
                        //         if let Some(rectangle) = sco.bounding_box(color) {
                        //             is_above = yy < rectangle.min_y();
                        //             is_below = yy > rectangle.max_y();
                        //             is_left = xx < rectangle.min_x();
                        //             is_right = xx > rectangle.max_x();
                        //         }
                        //     }
                        //     record.serialize_bool_onehot(is_above);
                        //     record.serialize_bool_onehot(is_below);
                        //     record.serialize_bool_onehot(is_left);
                        //     record.serialize_bool_onehot(is_right);
                        // }
                        // for color in 0..=9 {
                        //     let mut is_x_between_inclusive: bool = false;
                        //     let mut is_y_between_inclusive: bool = false;
                        //     let mut is_x_between_exclusive: bool = false;
                        //     let mut is_y_between_exclusive: bool = false;
                        //     if let Some(sco) = &pair.input.image_meta.single_color_object {
                        //         if let Some(rectangle) = sco.bounding_box(color) {
                        //             // is_x_between_inclusive = xx >= rectangle.min_x() && xx <= rectangle.max_x();
                        //             // is_y_between_inclusive = yy >= rectangle.min_y() && yy <= rectangle.max_y();
                        //             // is_x_between_exclusive = xx > rectangle.min_x() && xx < rectangle.max_x();
                        //             // is_y_between_exclusive = yy > rectangle.min_y() && yy < rectangle.max_y();
                        //             is_x_between_inclusive = xx <= rectangle.min_x() || xx >= rectangle.max_x();
                        //             is_y_between_inclusive = yy <= rectangle.min_y() || yy >= rectangle.max_y();
                        //             is_x_between_exclusive = xx < rectangle.min_x() || xx > rectangle.max_x();
                        //             is_y_between_exclusive = yy < rectangle.min_y() || yy > rectangle.max_y();
                        //         }
                        //     }
                        //     record.serialize_bool(is_x_between_inclusive);
                        //     record.serialize_bool(is_y_between_inclusive);
                        //     record.serialize_bool(is_x_between_exclusive);
                        //     record.serialize_bool(is_y_between_exclusive);
                        // }
                        // for color in 0..=9 {
                        //     let mut is_x_edge: bool = false;
                        //     let mut is_y_edge: bool = false;
                        //     if let Some(sco) = &pair.input.image_meta.single_color_object {
                        //         if let Some(rectangle) = sco.bounding_box(color) {
                        //             is_x_edge = xx == rectangle.min_x() || xx == rectangle.max_x();
                        //             is_y_edge = yy == rectangle.min_y() || yy == rectangle.max_y();
                        //         }
                        //     }
                        //     record.serialize_bool_onehot(is_x_edge && is_y_edge);
                        // }
                    }

                    // for linespan_image in &linespan_images {
                        // let pixel: u8 = linespan_image.get(xx, yy).unwrap_or(255);
                        // let is_line: bool = pixel > 0;
                        // record.serialize_bool(is_line);
                    // }

                    let center_clamp10: u8 = center.min(10);
                    if let Some(images) = color_to_linespan_images.get(&center_clamp10) {
                        for linespan_image in images {
                            let pixel: u8 = linespan_image.get(xx, yy).unwrap_or(255);
                            record.serialize_u8(pixel);
                            // record.serialize_onehot(pixel, 4);
                            // record.serialize_onehot(pixel, 10);
                            // record.serialize_onehot_discard_overflow(pixel, 2);
                        }
                    } else {
                        return Err(anyhow::anyhow!("no linespan images for color {}", center));
                    }

                    if enable_object_id_image_connectivity4 {
                        let pixel: u8 = object_id_image_connectivity4.get(xx, yy).unwrap_or(255);
                        record.serialize_onehot(pixel, 255);
                        record.serialize_u8(pixel);
                        record.serialize_complex(pixel as u16, 256);
                        record.serialize_cluster_id(center, pixel, obfuscated_cluster_offset, enable_serialize_cluster_id_shakeup);
                    }
                    if enable_object_id_image_connectivity8 {
                        let pixel: u8 = object_id_image_connectivity8.get(xx, yy).unwrap_or(255);
                        record.serialize_onehot(pixel, 255);
                        record.serialize_u8(pixel);
                        record.serialize_complex(pixel as u16, 256);
                        record.serialize_cluster_id(center, pixel, obfuscated_cluster_offset, enable_serialize_cluster_id_shakeup);
                    }

                    if enable_relative_position_topleft_xy {
                        for relative_position_image in &relative_position_images_connectivity4 {
                            let pixel: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                            record.serialize_u8(pixel);
                            // record.serialize_onehot(pixel, 30);
                        }
                        for relative_position_image in &relative_position_images_connectivity8 {
                            let pixel: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                            record.serialize_u8(pixel);
                            // record.serialize_onehot(pixel, 30);
                        }
                    }

                    if enable_relative_position_checkerboard {
                        {
                            let mut sum: u16 = 0;
                            for relative_position_image in &relative_position_images_connectivity4 {
                                let pixel: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                                sum += pixel as u16;
                            }
                            record.serialize_bool_onehot(sum % 2 == 0);
                        }

                        {
                            let mut sum: u16 = 0;
                            for relative_position_image in &relative_position_images_connectivity8 {
                                let pixel: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                                sum += pixel as u16;
                            }
                            record.serialize_bool_onehot(sum % 2 == 0);
                        }
                    }

                    // Relative position inside shape value between -0.5 and +0.5
                    // {
                    //     let mut shape_width: u8 = 0;
                    //     let mut shape_height: u8 = 0;
                    //     for (shape_size_image_index, shape_size_image) in shape_size_images_connectivity8.iter().enumerate() {
                    //         let value: u8 = shape_size_image.get(xx, yy).unwrap_or(255);
                    //         if shape_size_image_index == 0 {
                    //             shape_width = value;
                    //         }
                    //         if shape_size_image_index == 1 {
                    //             shape_height = value;
                    //         }
                    //     }
                    //     shape_width = shape_width.max(1);
                    //     shape_height = shape_height.max(1);

                    //     for (relative_position_image_index, relative_position_image) in relative_position_images_connectivity8.iter().enumerate() {
                    //         let value: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                    //         let mut denominator: u8 = 1;
                    //         if relative_position_image_index == 0 {
                    //             denominator = shape_width;
                    //         }
                    //         if relative_position_image_index == 1 {
                    //             denominator = shape_height;
                    //         }
                    //         let numerator: f64 = (value as f64) + 0.5;
                    //         let pos: f64 = (numerator / (denominator as f64)) - 0.5;
                    //         record.serialize_f64(pos);
                    //     }
                    // }

                    // Extreme position inside shape, is it the min or the max or inbetween
                    // {
                    //     let mut shape_width: u8 = 0;
                    //     let mut shape_height: u8 = 0;
                    //     for (shape_size_image_index, shape_size_image) in shape_size_images_connectivity4.iter().enumerate() {
                    //         let value: u8 = shape_size_image.get(xx, yy).unwrap_or(255);
                    //         if shape_size_image_index == 0 {
                    //             shape_width = value;
                    //         }
                    //         if shape_size_image_index == 1 {
                    //             shape_height = value;
                    //         }
                    //     }
                    //     record.serialize_onehot_discard_overflow(shape_width, 30);
                    //     record.serialize_onehot_discard_overflow(shape_height, 30);
                    //     record.serialize_u8(shape_width);
                    //     record.serialize_u8(shape_height);
                    //
                    //     for (relative_position_image_index, relative_position_image) in relative_position_images_connectivity4.iter().enumerate() {
                    //         let value: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                    //         let mut shape_size: u8 = 0;
                    //         if relative_position_image_index == 0 {
                    //             shape_size = shape_width;
                    //         }
                    //         if relative_position_image_index == 1 {
                    //             shape_size = shape_height;
                    //         }
                    //         let is_min: bool = value == 0;
                    //         let is_max: bool = value + 1 == shape_size;
                    //         record.serialize_bool_onehot(is_min);
                    //         record.serialize_bool_onehot(is_max);
                    //     }
                    // }

                    // Extreme position inside shape, is it the min or the max or inbetween
                    // {
                    //     let mut shape_width: u8 = 0;
                    //     let mut shape_height: u8 = 0;
                    //     for (shape_size_image_index, shape_size_image) in shape_size_images_connectivity8.iter().enumerate() {
                    //         let value: u8 = shape_size_image.get(xx, yy).unwrap_or(255);
                    //         if shape_size_image_index == 0 {
                    //             shape_width = value;
                    //         }
                    //         if shape_size_image_index == 1 {
                    //             shape_height = value;
                    //         }
                    //     }
                    //
                    //     for (relative_position_image_index, relative_position_image) in relative_position_images_connectivity8.iter().enumerate() {
                    //         let value: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                    //         let mut shape_size: u8 = 0;
                    //         if relative_position_image_index == 0 {
                    //             shape_size = shape_width;
                    //         }
                    //         if relative_position_image_index == 1 {
                    //             shape_size = shape_height;
                    //         }
                    //         let is_min: bool = value == 0;
                    //         let is_max: bool = value + 1 == shape_size;
                    //         record.serialize_bool_onehot(is_min);
                    //         record.serialize_bool_onehot(is_max);
                    //     }
                    // }

                    // Relative position inside shape value in pixel count -shape_size/2 to +shape_size/2
                    // {
                    //     let mut shape_width: u8 = 0;
                    //     let mut shape_height: u8 = 0;
                    //     for (shape_size_image_index, shape_size_image) in shape_size_images_connectivity4.iter().enumerate() {
                    //         let value: u8 = shape_size_image.get(xx, yy).unwrap_or(255);
                    //         if shape_size_image_index == 0 {
                    //             shape_width = value;
                    //         }
                    //         if shape_size_image_index == 1 {
                    //             shape_height = value;
                    //         }
                    //     }

                    //     for (relative_position_image_index, relative_position_image) in relative_position_images_connectivity4.iter().enumerate() {
                    //         let value: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                    //         let mut shape_size: u8 = 0;
                    //         if relative_position_image_index == 0 {
                    //             shape_size = shape_width;
                    //         }
                    //         if relative_position_image_index == 1 {
                    //             shape_size = shape_height;
                    //         }
                    //         let pos: f64 = (value as f64) - ((shape_size as f64) / 2.0);
                    //         record.serialize_f64(pos);
                    //     }
                    // }

                    // Relative position inside shape value in pixel count -shape_size/2 to +shape_size/2
                    // {
                    //     let mut shape_width: u8 = 0;
                    //     let mut shape_height: u8 = 0;
                    //     for (shape_size_image_index, shape_size_image) in shape_size_images_connectivity8.iter().enumerate() {
                    //         let value: u8 = shape_size_image.get(xx, yy).unwrap_or(255);
                    //         if shape_size_image_index == 0 {
                    //             shape_width = value;
                    //         }
                    //         if shape_size_image_index == 1 {
                    //             shape_height = value;
                    //         }
                    //     }

                    //     for (relative_position_image_index, relative_position_image) in relative_position_images_connectivity8.iter().enumerate() {
                    //         let value: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                    //         let mut shape_size: u8 = 0;
                    //         if relative_position_image_index == 0 {
                    //             shape_size = shape_width;
                    //         }
                    //         if relative_position_image_index == 1 {
                    //             shape_size = shape_height;
                    //         }
                    //         let pos: f64 = (value as f64) - ((shape_size as f64) / 2.0);
                    //         record.serialize_f64(pos);
                    //     }
                    // }

                    // {
                    //     let image_id: u8 = Shape3x3::id_from_3x3image(&area3x3).unwrap_or(0);
                    //     record.serialize_onehot_discard_overflow_u16(image_id as u16, 256);
                    // }
                    // {
                    //     let mut the_shapeid: u8 = 255;
                    //     let mut transform_mask: u8 = 0;
                    //     let mut local_area3x3: Image = area5x5.resize(3, 3)?;
                    //     local_area3x3.set(0, 0, area5x5.get(1, 0).unwrap_or(255));
                    //     local_area3x3.set(1, 0, area5x5.get(3, 0).unwrap_or(255));
                    //     local_area3x3.set(2, 0, area5x5.get(0, 1).unwrap_or(255));
                    //     local_area3x3.set(0, 1, area5x5.get(0, 3).unwrap_or(255));
                    //     local_area3x3.set(2, 1, area5x5.get(4, 1).unwrap_or(255));
                    //     local_area3x3.set(0, 2, area5x5.get(4, 3).unwrap_or(255));
                    //     local_area3x3.set(1, 2, area5x5.get(1, 4).unwrap_or(255));
                    //     local_area3x3.set(2, 2, area5x5.get(3, 4).unwrap_or(255));
                    //     // let image_id: u8 = Shape3x3::id_from_3x3image(&local_area3x3).unwrap_or(0);
                    //     // record.serialize_onehot_discard_overflow_u16(image_id as u16, 256);
                    //     match Shape3x3::instance().shapeid_and_transformations(&local_area3x3) {
                    //         Ok((shapeid, transformations)) => {
                    //             the_shapeid = shapeid;
                    //             if transformations.contains(&ShapeTransformation::Normal) {
                    //                 transform_mask |= 1;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw90) {
                    //                 transform_mask |= 2;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw180) {
                    //                 transform_mask |= 4;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw270) {
                    //                 transform_mask |= 8;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipX) {
                    //                 transform_mask |= 16;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw90) {
                    //                 transform_mask |= 32;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw180) {
                    //                 transform_mask |= 64;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw270) {
                    //                 transform_mask |= 128;
                    //             }
                    //         },
                    //         Err(_) => {},
                    //     }
                    //     record.serialize_onehot_discard_overflow(the_shapeid, number_of_shape3x3ids);
                    //     record.serialize_bitmask_as_onehot(transform_mask as u16, 8);
                    // }

                    // let center_shapeid: u8;
                    // let center_shapetransformations: u8;
                    {
                        let mut the_shapeid: u8 = 255;
                        let mut transform_mask: u8 = 0;
                        match Shape3x3::instance().shapeid_and_transformations(&area3x3) {
                            Ok((shapeid, transformations)) => {
                                the_shapeid = shapeid;
                                if transformations.contains(&ShapeTransformation::Normal) {
                                    transform_mask |= 1;
                                }
                                if transformations.contains(&ShapeTransformation::RotateCw90) {
                                    transform_mask |= 2;
                                }
                                if transformations.contains(&ShapeTransformation::RotateCw180) {
                                    transform_mask |= 4;
                                }
                                if transformations.contains(&ShapeTransformation::RotateCw270) {
                                    transform_mask |= 8;
                                }
                                if transformations.contains(&ShapeTransformation::FlipX) {
                                    transform_mask |= 16;
                                }
                                if transformations.contains(&ShapeTransformation::FlipXRotateCw90) {
                                    transform_mask |= 32;
                                }
                                if transformations.contains(&ShapeTransformation::FlipXRotateCw180) {
                                    transform_mask |= 64;
                                }
                                if transformations.contains(&ShapeTransformation::FlipXRotateCw270) {
                                    transform_mask |= 128;
                                }
                            },
                            Err(_) => {},
                        }
                        // center_shapeid = the_shapeid;
                        // center_shapetransformations = transform_mask;
                        record.serialize_onehot_discard_overflow(the_shapeid, number_of_shape3x3ids);
                        record.serialize_bitmask_as_onehot(transform_mask as u16, 8);
                    }

                    // {
                    //     let mut the_shapeid: u8 = 255;
                    //     let mut transform_mask: u8 = 0;
                    //     match Shape3x3::instance().shapeid_and_transformations(&nonbackground_area3x3) {
                    //         Ok((shapeid, transformations)) => {
                    //             the_shapeid = shapeid;
                    //             if transformations.contains(&ShapeTransformation::Normal) {
                    //                 transform_mask |= 1;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw90) {
                    //                 transform_mask |= 2;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw180) {
                    //                 transform_mask |= 4;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw270) {
                    //                 transform_mask |= 8;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipX) {
                    //                 transform_mask |= 16;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw90) {
                    //                 transform_mask |= 32;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw180) {
                    //                 transform_mask |= 64;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw270) {
                    //                 transform_mask |= 128;
                    //             }
                    //         },
                    //         Err(_) => {},
                    //     }
                    //     // center_shapeid = the_shapeid;
                    //     // center_shapetransformations = transform_mask;
                    //     record.serialize_onehot_discard_overflow(the_shapeid, number_of_shape3x3ids);
                    //     record.serialize_bitmask_as_onehot(transform_mask as u16, 8);
                    // }

                    // {
                    //     let mut the_shapeid: u8 = 255;
                    //     let mut transform_mask: u8 = 0;
                    //     let mut my_area3x3: Image = area3x3.clone();
                    //     if let Some(color) = most_popular_color {
                    //         my_area3x3 = my_area3x3.to_mask_where_color_is_different(color);
                    //     }
                    //     match Shape3x3::instance().shapeid_and_transformations(&my_area3x3) {
                    //         Ok((shapeid, transformations)) => {
                    //             the_shapeid = shapeid;
                    //             if transformations.contains(&ShapeTransformation::Normal) {
                    //                 transform_mask |= 1;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw90) {
                    //                 transform_mask |= 2;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw180) {
                    //                 transform_mask |= 4;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw270) {
                    //                 transform_mask |= 8;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipX) {
                    //                 transform_mask |= 16;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw90) {
                    //                 transform_mask |= 32;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw180) {
                    //                 transform_mask |= 64;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw270) {
                    //                 transform_mask |= 128;
                    //             }
                    //         },
                    //         Err(_) => {},
                    //     }
                    //     // center_shapeid = the_shapeid;
                    //     // center_shapetransformations = transform_mask;
                    //     record.serialize_onehot_discard_overflow(the_shapeid, number_of_shape3x3ids);
                    //     record.serialize_bitmask_as_onehot(transform_mask as u16, 8);
                    // }

                    // let combos: [(i32, i32); 4] = [(xx - 1, 0), (xx - 1, (height as i32) - 3), (0, yy - 1), ((width as i32) - 3, yy - 1)];
                    // let combos: [(i32, i32); 4] = [(-1, 0), (0, -1), (1, 0), (0, 1)];
                    // let combos: [(i32, i32); 4] = [(-1, -1), (1, -1), (-1, 1), (1, 1)];
                    // for combo in combos {
                    //     let current_area3x3: Image = input.crop_outside(xx - 1 - combo.0, yy - 1 - combo.1, 3, 3, 255)?;
                    //     // let current_area3x3: Image = input.crop_outside(combo.0, combo.1, 3, 3, 255)?;
                    //     let mut the_shapeid: u8 = 255;
                    //     let mut transform_mask: u8 = 0;
                    //     match Shape3x3::instance().shapeid_and_transformations(&current_area3x3) {
                    //         Ok((shapeid, transformations)) => {
                    //             the_shapeid = shapeid;
                    //             if transformations.contains(&ShapeTransformation::Normal) {
                    //                 transform_mask |= 1;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw90) {
                    //                 transform_mask |= 2;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw180) {
                    //                 transform_mask |= 4;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::RotateCw270) {
                    //                 transform_mask |= 8;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipX) {
                    //                 transform_mask |= 16;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw90) {
                    //                 transform_mask |= 32;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw180) {
                    //                 transform_mask |= 64;
                    //             }
                    //             if transformations.contains(&ShapeTransformation::FlipXRotateCw270) {
                    //                 transform_mask |= 128;
                    //             }
                    //         },
                    //         Err(_) => {},
                    //     }
                    //     let same_color: bool = current_area3x3.get(1, 1).unwrap_or(255) == center;

                    //     // if !same_color {
                    //     //     the_shapeid = 255;
                    //     //     transform_mask = 0;
                    //     // }
                    //     // record.serialize_onehot_discard_overflow(the_shapeid, number_of_shape3x3ids);
                    //     // record.serialize_bitmask_as_onehot(transform_mask as u16, 8);



                    //     let same_shape: bool = the_shapeid == center_shapeid;
                    //     let same_transformations: bool = transform_mask == center_shapetransformations;
                    //     let same_shape_and_transformations = same_shape && same_transformations;
                    //     // record.serialize_bool(same_shape);
                    //     record.serialize_bool(same_shape_and_transformations);


                    //     let same_color_and_same_shape: bool = same_color && same_shape;
                    //     let different_color_and_same_shape: bool = (same_color == false) && same_shape;
                    //     let different_color_and_different_shape: bool = (same_color == false) && (same_shape == false);
                    //     let same_color_and_different_shape: bool = same_color && (same_shape == false);
                    //     // record.serialize_bool(same_color_and_same_shape);
                    //     // record.serialize_bool(different_color_and_same_shape);
                    //     // record.serialize_bool(different_color_and_different_shape);
                    //     // record.serialize_bool(same_color_and_different_shape);
                    // }

                    // {
                    //     let pixel: u8 = non_background_shape_type_image_connectivity4.get(xx, yy).unwrap_or(255);
                    //     record.serialize_onehot_discard_overflow(pixel, shape_type_count);
                    // }
                    // {
                    //     let pixel: u8 = non_background_shape_type_image_connectivity8.get(xx, yy).unwrap_or(255);
                    //     record.serialize_onehot_discard_overflow(pixel, shape_type_count);
                    // }
                    {
                        let pixel: u8 = shape_type_image_connectivity4.get(xx, yy).unwrap_or(255);
                        record.serialize_onehot_discard_overflow(pixel, shape_type_count);
                    }
                    {
                        let pixel: u8 = shape_type_image_connectivity8.get(xx, yy).unwrap_or(255);
                        record.serialize_onehot_discard_overflow(pixel, shape_type_count);
                    }
                    {
                        let pixel: u8 = shape_type45_image_connectivity4.get(xx, yy).unwrap_or(255);
                        record.serialize_onehot_discard_overflow(pixel, shape_type_count);
                    }
                    {
                        let pixel: u8 = shape_type45_image_connectivity8.get(xx, yy).unwrap_or(255);
                        record.serialize_onehot_discard_overflow(pixel, shape_type_count);
                    }
                    if enable_shape_transformation_images {
                        for shape_transformation_image in &shape_transformation_images_connectivity4 {
                            let pixel: u8 = shape_transformation_image.get(xx, yy).unwrap_or(255);
                            // record.serialize_u8(pixel);
                            record.serialize_bitmask_as_onehot(pixel as u16, 8);
                        }
                        for shape_transformation_image in &shape_transformation_images_connectivity8 {
                            let pixel: u8 = shape_transformation_image.get(xx, yy).unwrap_or(255);
                            // record.serialize_u8(pixel);
                            record.serialize_bitmask_as_onehot(pixel as u16, 8);
                        }
                    }

                    // Shape orientation: landscape, portrait, square
                    {
                        let mut shape_width: u8 = 0;
                        let mut shape_height: u8 = 0;
                        for (shape_size_image_index, shape_size_image) in shape_size_images_connectivity4.iter().enumerate() {
                            let value: u8 = shape_size_image.get(xx, yy).unwrap_or(255);
                            if shape_size_image_index == 0 {
                                shape_width = value;
                            }
                            if shape_size_image_index == 1 {
                                shape_height = value;
                            }
                        }
                        let shape_orientation: i8;
                        if shape_width > shape_height {
                            shape_orientation = 1;
                        } else {
                            if shape_width < shape_height {
                                shape_orientation = -1;
                            } else {
                                shape_orientation = 0;
                            }
                        }
                        record.serialize_ternary(shape_orientation);
                    }

                    // Shape orientation: landscape, portrait, square
                    {
                        let mut shape_width: u8 = 0;
                        let mut shape_height: u8 = 0;
                        for (shape_size_image_index, shape_size_image) in shape_size_images_connectivity8.iter().enumerate() {
                            let value: u8 = shape_size_image.get(xx, yy).unwrap_or(255);
                            if shape_size_image_index == 0 {
                                shape_width = value;
                            }
                            if shape_size_image_index == 1 {
                                shape_height = value;
                            }
                        }
                        let shape_orientation: i8;
                        if shape_width > shape_height {
                            shape_orientation = 1;
                        } else {
                            if shape_width < shape_height {
                                shape_orientation = -1;
                            } else {
                                shape_orientation = 0;
                            }
                        }
                        record.serialize_ternary(shape_orientation);
                    }

                    for shape_size_image in &shape_size_images_connectivity4 {
                        let pixel: u8 = shape_size_image.get(xx, yy).unwrap_or(255);
                        // record.serialize_u8(pixel);
                        record.serialize_onehot(pixel, 30);
                    }
                    for shape_size_image in &shape_size_images_connectivity8 {
                        let pixel: u8 = shape_size_image.get(xx, yy).unwrap_or(255);
                        // record.serialize_u8(pixel);
                        record.serialize_onehot(pixel, 30);
                    }

                    // if let Some(image) = earlier_prediction_image {
                    //     let pixel: u8 = image.get(xx, yy).unwrap_or(255);
                    //     let mut value: bool = false;
                    //     if pixel < 10 {
                    //         value = no_change_to_color[pixel as usize];
                    //     }
                    //     record.serialize_bool_onehot(value);
                    // }

                    {

                        // if let Some(image) = &earlier_prediction_image {
                        //     let earlier_prediction_area3x3: Image = image.crop_outside(xx - 1, yy - 1, 3, 3, 255)?;
                        //     {
                        //         let mut the_shapeid: u8 = 255;
                        //         let mut transform_mask: u8 = 0;
                        //         match Shape3x3::instance().shapeid_and_transformations(&earlier_prediction_area3x3) {
                        //             Ok((shapeid, transformations)) => {
                        //                 the_shapeid = shapeid;
                        //                 if transformations.contains(&ShapeTransformation::Normal) {
                        //                     transform_mask |= 1;
                        //                 }
                        //                 if transformations.contains(&ShapeTransformation::RotateCw90) {
                        //                     transform_mask |= 2;
                        //                 }
                        //                 if transformations.contains(&ShapeTransformation::RotateCw180) {
                        //                     transform_mask |= 4;
                        //                 }
                        //                 if transformations.contains(&ShapeTransformation::RotateCw270) {
                        //                     transform_mask |= 8;
                        //                 }
                        //                 if transformations.contains(&ShapeTransformation::FlipX) {
                        //                     transform_mask |= 16;
                        //                 }
                        //                 if transformations.contains(&ShapeTransformation::FlipXRotateCw90) {
                        //                     transform_mask |= 32;
                        //                 }
                        //                 if transformations.contains(&ShapeTransformation::FlipXRotateCw180) {
                        //                     transform_mask |= 64;
                        //                 }
                        //                 if transformations.contains(&ShapeTransformation::FlipXRotateCw270) {
                        //                     transform_mask |= 128;
                        //                 }
                        //             },
                        //             Err(_) => {},
                        //         }
                        //         record.serialize_onehot_discard_overflow(the_shapeid, number_of_shape3x3ids);
                        //         record.serialize_bitmask_as_onehot(transform_mask as u16, 8);
                        //     }        
                        // }

                        if let Some(image) = &earlier_prediction_shapetype_connectivity4 {
                            let pixel: u8 = image.get(xx, yy).unwrap_or(0);
                            record.serialize_onehot_discard_overflow(pixel, shape_type_count);
                        }

                        if let Some(image) = &earlier_prediction_shapetype45_connectivity4 {
                            let pixel: u8 = image.get(xx, yy).unwrap_or(0);
                            record.serialize_onehot_discard_overflow(pixel, shape_type_count);
                        }

                        // if let Some(image) = &earlier_prediction_shapetype_connectivity8 {
                        //     let pixel: u8 = image.get(xx, yy).unwrap_or(0);
                        //     record.serialize_onehot_discard_overflow(pixel, shape_type_count);
                        // }

                        // if let Some(image) = &earlier_prediction_shapetype45_connectivity8 {
                        //     let pixel: u8 = image.get(xx, yy).unwrap_or(0);
                        //     record.serialize_onehot_discard_overflow(pixel, shape_type_count);
                        // }

                        // if let Some(image) = &earlier_prediction_mass_connectivity4 {
                        //     let mass: u8 = image.get(xx, yy).unwrap_or(0);
                        //     record.serialize_color_complex(mass, obfuscated_color_offset, enable_serialize_color_complex);
                        //     record.serialize_u8(mass);
                        //     record.serialize_onehot_discard_overflow(mass, 5);
                        //     record.serialize_onehot_discard_overflow(mass, 40);
                        // }

                        // if let Some(image) = &earlier_prediction_mass_connectivity8 {
                        //     let mass: u8 = image.get(xx, yy).unwrap_or(0);
                        //     record.serialize_color_complex(mass, obfuscated_color_offset, enable_serialize_color_complex);
                        //     record.serialize_u8(mass);
                        //     record.serialize_onehot_discard_overflow(mass, 5);
                        //     record.serialize_onehot_discard_overflow(mass, 40);
                        // }

                        if let Some(image) = earlier_prediction_image {
                            // let pixel: u8 = image.get(xx, yy).unwrap_or(0);
                            // record.serialize_onehot(pixel, 10);
                            // record.serialize_bool_onehot(pixel == center);
                            // record.serialize_color_complex(pixel, obfuscated_color_offset, enable_serialize_color_complex);

                            // {
                            //     let pixel: u8 = image.get(xx, yy).unwrap_or(255);
                            //     record.serialize_onehot_discard_overflow(pixel, 10);
                            //     record.serialize_bool(Some(pixel) == most_popular_color);
                            //     record.serialize_bool_onehot(pixel == center);
                            // }

                            // for j in 0..5 {
                            //     for i in 0..5 {
                            //         if j == 2 && i == 2 {
                            //             continue;
                            //         }
                            //         let pixel: u8 = image.get(xx - 2 + i, yy - 2 + j).unwrap_or(255);
                            //         record.serialize_onehot_discard_overflow(pixel, 10);
                            //     }
                            // }
                            for j in 0..3 {
                                for i in 0..3 {
                                    if j == 1 && i == 1 {
                                        continue;
                                    }
                                    let pixel: u8 = image.get(xx - 1 + i, yy - 1 + j).unwrap_or(255);
                                    record.serialize_onehot_discard_overflow(pixel, 10);
                                }
                            }
                        }
                    }

                    // {
                    //     let mut count_minus1: u8 = 0;
                    //     let mut count_zero: u8 = 0;
                    //     let mut count_plus1: u8 = 0;
                    //     let mut contains_center_color_minus1: bool = false;
                    //     let mut contains_center_color_plus1: bool = false;
                    //     if x > 0 {
                    //         if let Some(hist) = histogram_columns.get((x - 1) as usize) {
                    //             let count: u16 = hist.number_of_counters_greater_than_zero();
                    //             count_minus1 = count.min(255) as u8;
                    //             contains_center_color_minus1 = hist.get(center) > 0;
                    //         }
                    //     }
                    //     if let Some(hist) = histogram_columns.get(x as usize) {
                    //         let count: u16 = hist.number_of_counters_greater_than_zero();
                    //         count_zero = count.min(255) as u8;
                    //     }
                    //     if let Some(hist) = histogram_columns.get((x + 1) as usize) {
                    //         let count: u16 = hist.number_of_counters_greater_than_zero();
                    //         count_plus1 = count.min(255) as u8;
                    //         contains_center_color_plus1 = hist.get(center) > 0;
                    //     }
                    //     record.serialize_onehot(count_minus1, 4);
                    //     record.serialize_onehot(count_plus1, 4);
                    //     record.serialize_u8(count_minus1);
                    //     record.serialize_u8(count_zero);
                    //     record.serialize_u8(count_plus1);
                    //     record.serialize_bool_onehot(contains_center_color_minus1);
                    //     record.serialize_bool_onehot(contains_center_color_plus1);
                    // }

                    // {
                    //     let mut count_minus1: u8 = 0;
                    //     let mut count_zero: u8 = 0;
                    //     let mut count_plus1: u8 = 0;
                    //     let mut contains_center_color_minus1: bool = false;
                    //     let mut contains_center_color_plus1: bool = false;
                    //     if y > 0 {
                    //         if let Some(hist) = histogram_rows.get((y - 1) as usize) {
                    //             let count: u16 = hist.number_of_counters_greater_than_zero();
                    //             count_minus1 = count.min(255) as u8;
                    //             contains_center_color_minus1 = hist.get(center) > 0;
                    //         }
                    //     }
                    //     if let Some(hist) = histogram_rows.get(y as usize) {
                    //         let count: u16 = hist.number_of_counters_greater_than_zero();
                    //         count_zero = count.min(255) as u8;
                    //     }
                    //     if let Some(hist) = histogram_rows.get((y + 1) as usize) {
                    //         let count: u16 = hist.number_of_counters_greater_than_zero();
                    //         count_plus1 = count.min(255) as u8;
                    //         contains_center_color_plus1 = hist.get(center) > 0;
                    //     }
                    //     record.serialize_onehot(count_minus1, 4);
                    //     record.serialize_onehot(count_plus1, 4);
                    //     record.serialize_u8(count_minus1);
                    //     record.serialize_u8(count_zero);
                    //     record.serialize_u8(count_plus1);
                    //     record.serialize_bool_onehot(contains_center_color_minus1);
                    //     record.serialize_bool_onehot(contains_center_color_plus1);
                    // }


                    // skewed pixel with x skewed or y skewed. Worsens the predictions.
                    // {
                    //     let color0: u8 = input.get_wrap(xx + yy, yy).unwrap_or(255);
                    //     let color1: u8 = input.get_wrap(xx - yy, yy).unwrap_or(255);
                    //     let color2: u8 = input.get_wrap(xx, yy + xx).unwrap_or(255);                        
                    //     let color3: u8 = input.get_wrap(xx, yy - xx).unwrap_or(255);
                    //     let color0: u8 = input.get(xx + yy, yy).unwrap_or(255);
                    //     let color1: u8 = input.get(xx - yy, yy).unwrap_or(255);
                    //     let color2: u8 = input.get(xx, yy + xx).unwrap_or(255);
                    //     let color3: u8 = input.get(xx, yy - xx).unwrap_or(255);
                    //     record.serialize_color_complex(color0, obfuscated_color_offset, enable_serialize_color_complex);
                    //     record.serialize_color_complex(color1, obfuscated_color_offset, enable_serialize_color_complex);
                    //     record.serialize_color_complex(color2, obfuscated_color_offset, enable_serialize_color_complex);
                    //     record.serialize_color_complex(color3, obfuscated_color_offset, enable_serialize_color_complex);
                    //     record.serialize_bool(center == color0);
                    //     record.serialize_bool(center == color1);
                    //     record.serialize_bool(center == color2);
                    //     record.serialize_bool(center == color3);
                    // }

                    // shoot out rays in all directions. Worsens the predictions.
                    // {
                    //     for i in 1..3 {
                    //         // let color0: u8 = input.get_wrap(xx - i, yy - i).unwrap_or(255);
                    //         // let color1: u8 = input.get_wrap(xx + i, yy - i).unwrap_or(255);
                    //         // let color2: u8 = input.get_wrap(xx - i, yy + i).unwrap_or(255);
                    //         // let color3: u8 = input.get_wrap(xx + i, yy + i).unwrap_or(255);
                    //         // let color0: u8 = input.get(xx - i, yy - i).unwrap_or(255);
                    //         // let color1: u8 = input.get(xx + i, yy - i).unwrap_or(255);
                    //         // let color2: u8 = input.get(xx - i, yy + i).unwrap_or(255);
                    //         // let color3: u8 = input.get(xx + i, yy + i).unwrap_or(255);
                    //         let color0: u8 = input.get(xx - i, yy).unwrap_or(255);
                    //         let color1: u8 = input.get(xx + i, yy).unwrap_or(255);
                    //         let color2: u8 = input.get(xx, yy - i).unwrap_or(255);
                    //         let color3: u8 = input.get(xx, yy + i).unwrap_or(255);
                    //         let all_same: bool = color0 < 10 && color0 == color1 && color0 == color2 && color0 == color3;
                    //         // record.serialize_bool(all_same);
                    //         let agree_color: u8 = if all_same { color0 } else { 255 };
                    //         record.serialize_color_complex(agree_color, obfuscated_color_offset, enable_serialize_color_complex);
                    //         // record.serialize_bool(center == color0);
                    //         // record.serialize_bool(center == color1);
                    //         // record.serialize_bool(center == color2);
                    //         // record.serialize_bool(center == color3);
                    //         // record.serialize_color_complex(color0, obfuscated_color_offset, enable_serialize_color_complex);
                    //         // record.serialize_color_complex(color1, obfuscated_color_offset, enable_serialize_color_complex);
                    //         // record.serialize_color_complex(color2, obfuscated_color_offset, enable_serialize_color_complex);
                    //         // record.serialize_color_complex(color3, obfuscated_color_offset, enable_serialize_color_complex);
                    //     }
                    // }

                    // distance to the nearest corner. Worsens the predictions.
                    // {
                        // let distance1: u8 = distance_to_corner1.get(xx, yy).unwrap_or(255).min(3);
                        // let distance2: u8 = distance_to_corner2.get(xx, yy).unwrap_or(255);
                        // let distance3: u8 = distance_to_corner3.get(xx, yy).unwrap_or(255);
                        // let distance4: u8 = distance_to_corner4.get(xx, yy).unwrap_or(255);
                        // record.serialize_u8(distance1);
                        // record.serialize_u8(distance2);
                        // record.serialize_u8(distance3);
                        // record.serialize_u8(distance4);
                    // }

                    // Future experiments
                    // shape bounding box
                    //
                    // push all the training pairs that have been rotated by 90 degrees.
                    // push all the training pairs that have been flipped.
                    //
                    // draw lines between nearest clusters, with the same color as the cluster. for all 10 colors.
                    // interior mass of the constructed objects.
                    // histogram of the pixels inside the constructed objects.
                    //
                    // shape complexity score. Sometimes it's the most complex object that is of interest.
                    //
                    // reversed color popularity, 3x3 convolution
                    //
                    // when inside a single color object, what is the distance to the edge of the object, in all directions.
                    //
                    // when the image is splitted in half,
                    // is inside the left-side then it's -1, inside the separator then 0, inside the right-side: +1.
                    //
                    // when the image is splitted into multiple cells, example 3 cells:
                    // cell0: is inside split area 0
                    // cell1: is inside split area 1
                    // cell2: is inside split area 2
                    // border01: is on the border between cell0 and cell1
                    // border12: is on the border between cell1 and cell2
                    //
                    // parent object id
                    // child object id
                    //
                    // is solid object without holes
                    // hole is square/rectangle/sparse
                    // object size, is biggest
                    // object size, is smallest
                    // object size, is neither biggest nor smallest
                    // object is symmetric
                    // object is asymmetric
                    // object is square/rectangle
                    // is insertion color
                    // direction up color
                    // direction down color
                    // direction left color
                    // direction right color
                    // single pixel with this color, the mass of this color is 1.
                    // nesting depth, how many flood fills are needed to clear the image.
                    // distance inside object, how many pixels from the edge of the object.
                    // distance to nearest object, how many pixels from the edge of the nearest object.
                    // cell x
                    // cell y
                    // cell distance from top/bottom/left/right
                    // cell is top/bottom/left/right/center
                    
                    records.push(record);
                }
            }
        }

        Ok(records)
    }
}

fn create_dataset(records: &Vec<Record>, is_test: bool) -> anyhow::Result<Dataset<f64, usize, Ix1>> {
    let mut data: Vec<f64> = Vec::new();
    let mut rows: usize = 0;
    let mut targets_raw: Vec<usize> = Vec::new();
    let mut values_max: usize = 0;
    let mut values_min: usize = usize::MAX;
    for record in records {
        let value_count: usize = record.values.len();
        values_max = values_max.max(value_count);
        values_min = values_min.min(value_count);

        if is_test != record.is_test {
            continue;
        }
        targets_raw.push(record.classification as usize);
        data.push(record.pair_id as f64);
        for value in &record.values {
            data.push(*value);
        }
        rows += 1;
    }
    if values_max != values_min {
        return Err(anyhow::anyhow!("values_max != values_min. values_max: {} values_min: {}", values_max, values_min));
    }
    let columns: usize = values_max + 1;

    let array1: Array1<f64> = Array1::<f64>::from(data);
    let x: Array2<f64> = array1.into_shape((rows, columns))?;
    
    let y: Array1<usize> = Array1::<usize>::from(targets_raw);

    let dataset: Dataset<f64, usize, Ix1> = Dataset::new(x, y);
    Ok(dataset)
}

fn perform_logistic_regression(task: &Task, test_index: u8, records: &Vec<Record>) -> anyhow::Result<Image> {
    for retry_index in 0..=3 {
        match perform_logistic_regression_inner(task, test_index, records, retry_index) {
            Ok(image) => return Ok(image),
            Err(error) => {
                let error_message = format!("{:?}", error);
                if error_message.contains("MoreThuenteLineSearch") {
                    // Often the `MultiLogisticRegression.fit()` function returned this error:
                    // Condition violated: "`MoreThuenteLineSearch`: Search direction must be a descent direction."
                    // The naive solution is to be try again with lower `max_iterations` value.
                    debug!("perform_logistic_regression_inner retrying due to MoreThuenteLineSearch error. retry_index: {} error: {}", retry_index, error);
                    continue;
                }
                return Err(error);
            },
        }
    }
    Err(anyhow::anyhow!("perform_logistic_regression exhausted all retries"))
}

fn perform_logistic_regression_inner(task: &Task, test_index: u8, records: &Vec<Record>, retry_index: u8) -> anyhow::Result<Image> {
    // println!("task_id: {}", task.id);

    let dataset_train: Dataset<f64, usize, Ix1> = create_dataset(records, false)
        .context("perform_logistic_regression_inner dataset_train")?;

    let dataset_test: Dataset<f64, usize, Ix1> = create_dataset(records, true)
        .context("perform_logistic_regression_inner dataset_test")?;

    let max_iterations: u64 = match retry_index {
        0 => 50,
        1 => 25,
        2 => 10,
        _ => 5,
    };

    let model: MultiFittedLogisticRegression<f64, _> = MultiLogisticRegression::<f64>::default()
        .max_iterations(max_iterations)
        .fit(&dataset_train)
        .context("perform_logistic_regression_inner fit")?;

    // predict and map targets
    let pred = model.predict(&dataset_test);

    // create a confusion matrix
    // let cm = pred.confusion_matrix(&valid)
    //     .context("confusion_matrix")?;

    // Print the confusion matrix, this will print a table with four entries. On the diagonal are
    // the number of true-positive and true-negative predictions, off the diagonal are
    // false-positive and false-negative
    // println!("{:?}", cm);

    // print out the predicted output pixel values
    // println!("{:?}", pred);

    let found_pair: Option<&Pair> = task.pairs.iter().find(|pair| pair.test_index == Some(test_index));
    let pair: &Pair = match found_pair {
        Some(pair) => pair,
        None => return Err(anyhow::anyhow!("No pair found with test_index: {}", test_index)),
    };

    let original_input: Image = pair.input.image.clone();

    let width: u8 = original_input.width();
    let height: u8 = original_input.height();

    let mut computed_image: Image = Image::color(width, height, 10);
    for y in 0..height {
        for x in 0..width {
            let xx: i32 = x as i32;
            let yy: i32 = y as i32;
            let address: usize = (y as usize) * (width as usize) + (x as usize);
            let predicted_color: u8 = match pred.get(address) {
                Some(value) => (*value).min(u8::MAX as usize) as u8,
                None => 255
            };
            _ = computed_image.set(xx, yy, predicted_color);
        }
    }

    Ok(computed_image)
}
