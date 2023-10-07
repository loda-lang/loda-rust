//! Performs logistic regression of each input pixel with the corresponding classification for the output pixel.
//! 
//! This solves 1 of the tasks from the hidden ARC dataset.
//!
//! This solves 63 of the 800 tasks in the public ARC dataset.
//! 009d5c81, 00d62c1b, 00dbd492, 08ed6ac7, 0a2355a6, 0d3d703e, 178fcbfb, 1c0d0a4b, 21f83797, 2281f1f4,
//! 23581191, 253bf280, 25d8a9c8, 32597951, 332efdb3, 3618c87e, 37d3e8b2, 4258a5f9, 44d8ac46, 4612dd53,
//! 50cb2852, 543a7ed5, 6455b5f5, 67385a82, 694f12f3, 69889d6e, 6c434453, 6d75e8bb, 6ea4a07e, 6f8cd79b,
//! 810b9b61, 84f2aca1, 95990924, a5313dff, a61f2674, a699fb00, a8d7556c, a934301b, a9f96cdd, aa4ec2a5,
//! ae58858e, aedd82e4, af902bf9, b1948b0a, b2862040, b60334d2, b6afb2da, bb43febb, c0f76784, c8f0f002,
//! ce039d91, ce22a75a, d2abd087, d364b489, d37a1ef5, d406998b, d5d6de2d, dc433765, ded97339, e0fb7511,
//! e7dd8335, e9c9d9a1, ef135b50, 
//! 
//! This partially solves 3 of the 800 tasks in the public ARC dataset. Where one ore more `test` pairs is solved, but not all of the `test` pairs gets solved.
//! 25ff71a9, 794b24be, da2b0fe3
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
use super::arc_json_model::GridFromImage;
use super::arc_work_model::{Task, PairType, Pair};
use super::{Image, ImageOverlay, arcathon_solution_coordinator, arc_json_model, ImageMix, MixMode, ObjectsAndMass, ImageCrop, Rectangle, ImageExtractRowColumn, ImageDenoise, TaskGraph, ShapeType, ImageSize, ShapeTransformation, SingleColorObject, ShapeIdentificationFromSingleColorObject, ImageDetectHole, ImagePadding, ImageRepairPattern};
use super::{ActionLabel, ImageLabel, ImageMaskDistance, LineSpan, LineSpanDirection, LineSpanMode};
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
use std::collections::HashMap;
use linfa::prelude::*;
use linfa_logistic::MultiLogisticRegression;
use ndarray::prelude::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static WRITE_TO_HTMLLOG: bool = false;

#[derive(Clone, Debug, Serialize)]
struct Record {
    classification: u8,
    is_test: u8,
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

    fn serialize_color_complex(&mut self, color: u8, offset: f64) {
        self.serialize_complex_scaled(color as u16, 10, offset, 1.0)
    }

    #[allow(dead_code)]
    fn serialize_cluster_id(&mut self, color: u8, cluster_id: u8) {
        let mut value: u16 = u16::MAX;
        if cluster_id < 41 && color < 10 {
            value = (cluster_id as u16) * 10 + (color as u16);
        }
        self.serialize_complex(value, 410);
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

    #[allow(dead_code)]
    pub fn run(&mut self) -> anyhow::Result<()> {
        let verbose = false;
        let verify_test_output = true;
        let number_of_tasks: u64 = self.tasks.len() as u64;
        println!("{} - run start - will process {} tasks with logistic regression", human_readable_utc_timestamp(), number_of_tasks);
        let count_solved = AtomicUsize::new(0);
        let pb = ProgressBar::new(number_of_tasks as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-")
        );
        self.tasks.par_iter_mut().for_each(|task| {
            match Self::process_task(task, verify_test_output) {
                Ok(_predictions) => {
                    count_solved.fetch_add(1, Ordering::Relaxed);
                    let count: usize = count_solved.load(Ordering::Relaxed);
                    pb.set_message(format!("Solved: {}", count));
                    pb.println(format!("task {} - solved", task.id));
                },
                Err(error) => {
                    if verbose {
                        pb.println(format!("task {} - error: {:?}", task.id, error));
                    }
                }
            }
            pb.inc(1);
        });
        pb.finish_and_clear();
        let count_solved: usize = count_solved.load(Ordering::Relaxed);
        println!("{} - run - end", human_readable_utc_timestamp());
        println!("{} - solved {} of {} tasks", human_readable_utc_timestamp(), count_solved, number_of_tasks);
        Ok(())
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

    pub fn process_task(task: &Task, verify_test_output: bool) -> anyhow::Result<Vec::<arcathon_solution_coordinator::Prediction>> {
        if !task.is_output_size_same_as_input_size() {
            // if WRITE_TO_HTMLLOG {
            //     HtmlLog::text(&format!("skipping task: {} because output size is not the same as input size", task.id));
            // }
            return Err(anyhow::anyhow!("skipping task: {} because output size is not the same as input size", task.id));
        }

        let count_test: u8 = task.count_test().min(255) as u8;
        if count_test < 1 {
            return Err(anyhow::anyhow!("skipping task: {} because it has no test pairs", task.id));
        }    

        let mut computed_images = Vec::<Image>::new();
        for test_index in 0..count_test {
            // println!("task: {} test_index: {} before", task.id, test_index);
            let computed_image: Image = match Self::process_task_with_one_test_pair(task, test_index) {
                Ok(value) => value,
                Err(error) => {
                    // println!("task: {} test_index: {} error: {:?}", task.id, test_index, error);
                    return Err(error);
                }
            };
            // println!("task: {} test_index: {} after", task.id, test_index);
            computed_images.push(computed_image);
        }
        // println!("task: {} computed_images.len(): {}", task.id, computed_images.len());

        let mut result_predictions = Vec::<arcathon_solution_coordinator::Prediction>::new();
        for (test_index, computed_image) in computed_images.iter().enumerate() {
            let grid: arc_json_model::Grid = arc_json_model::Grid::from_image(computed_image);
            let prediction = arcathon_solution_coordinator::Prediction {
                output_id: test_index.min(255) as u8,
                output: grid,
                prediction_type: arcathon_solution_coordinator::PredictionType::SolveLogisticRegression,
            };
            result_predictions.push(prediction);
        }

        if verify_test_output {
            let mut count_correct: usize = 0;
            let mut count_incorrect: usize = 0;
            for (test_index, computed_image) in computed_images.iter().enumerate() {
                let test_index_u8: u8 = test_index.min(255) as u8;
                let found_pair: Option<&Pair> = task.pairs.iter().find(|pair| pair.test_index == Some(test_index_u8));
                let pair: &Pair = match found_pair {
                    Some(pair) => pair,
                    None => return Err(anyhow::anyhow!("No pair found with test_index: {}", test_index)),
                };
                let expected_output: Image = pair.output.test_image.clone();
                let is_correct: bool = computed_image == &expected_output;
                if is_correct {
                    count_correct += 1;
                } else {
                    count_incorrect += 1;
                }

                if WRITE_TO_HTMLLOG {
                    if is_correct {
                        if task.occur_in_solutions_csv {
                            HtmlLog::text(format!("{} - correct - already solved in asm", task.id));
                        } else {
                            HtmlLog::text(format!("{} - correct - no previous solution", task.id));
                        }
                        HtmlLog::image(computed_image);
                    } else {
                        HtmlLog::text(format!("{} - incorrect", task.id));
                        let images: Vec<Image> = vec![
                            pair.input.image.clone(),
                            expected_output,
                            computed_image.clone(),
                        ];
                        HtmlLog::compare_images(images);
                    }
                }
            
            }
        
            if count_correct == 0 {
                return Err(anyhow::anyhow!("The predicted output doesn't match with the expected output"));
            }
            if count_incorrect > 0 {
                println!("task: {} partial match. correct: {} incorrect: {}", task.id, count_correct, count_incorrect);
            }
        }
    
        if result_predictions.len() != (count_test as usize) {
            return Err(anyhow::anyhow!("task: {} predictions.len() != task.count_test()", task.id));
        }
        Ok(result_predictions)
    }

    fn process_task_with_one_test_pair(task: &Task, test_index: u8) -> anyhow::Result<Image> {
        let number_of_iterations: usize = 5;
        let mut computed_images = Vec::<Image>::new();
        let mut last_computed_image: Option<Image> = None;
        for iteration_index in 0..number_of_iterations {
            let records = Self::process_task_iteration(task, iteration_index, test_index, last_computed_image)?;
            let computed_image: Image = perform_logistic_regression(task, test_index, &records)?;
            last_computed_image = Some(computed_image.clone());
            computed_images.push(computed_image);
        }
        if WRITE_TO_HTMLLOG {
            HtmlLog::compare_images(computed_images.clone());
        }

        let computed_image: Image = match last_computed_image {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Unable to get last computed image"));
            }
        };
        Ok(computed_image)
    }

    fn object_id_image(task_graph: &TaskGraph, pair_index: u8, width: u8, height: u8, connectivity: PixelConnectivity) -> anyhow::Result<Image> {
        let mut image: Image = Image::zero(width, height);
        for y in 0..height {
            for x in 0..width {
                let object_id: usize = task_graph.get_objectid_for_input_pixel(pair_index, x, y, connectivity)?;
                let color: u8 = object_id.min(255) as u8;
                _ = image.set(x as i32, y as i32, color);
            }
        }
        Ok(image)
    }

    fn relative_position_images(task_graph: &TaskGraph, pair_index: u8, width: u8, height: u8, connectivity: PixelConnectivity) -> anyhow::Result<Vec<Image>> {
        let mut image_x: Image = Image::zero(width, height);
        let mut image_y: Image = Image::zero(width, height);
        for y in 0..height {
            for x in 0..width {
                let (position_x, position_y) = task_graph.get_objectposition_for_input_pixel(pair_index, x, y, connectivity)?;
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
                    false => task_graph.get_shapetype_for_input_pixel(pair_index, x, y, connectivity)?,
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

    fn process_task_iteration(task: &Task, process_task_iteration_index: usize, test_index: u8, computed_image: Option<Image>) -> anyhow::Result<Vec::<Record>> {
        // println!("exporting task: {}", task.id);

        // let obfuscated_color_offset: f64 = 0.2;
        let obfuscated_color_offset: f64 = (process_task_iteration_index as f64 * 0.7333 + 0.2) % 1.0;

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

        let mut earlier_prediction_image_vec = Vec::<Image>::new();
        if let Some(computed_image) = computed_image {
            let random_seed: u64 = process_task_iteration_index as u64;
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
                            let strategy_value: &u8 = &strategy_vec.choose_weighted(&mut rng, |item| item.1).unwrap().0;
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

            let is_test: u8;
            let original_output: Image;
            match pair.pair_type {
                PairType::Train => {
                    is_test = 0;
                    original_output = pair.output.image.clone();
                },
                PairType::Test => {
                    is_test = 1;
                    original_output = Image::empty();
                },
            }
            let original_input: Image = pair.input.image.clone();

            let width: u8 = original_input.width().max(original_output.width()).min(253);
            let height: u8 = original_input.height().max(original_output.height()).min(253);

            let background: Image = Image::color(width, height, 10);
            let input: Image = background.overlay_with_position(&original_input, 0, 0)?;
            let output: Image = background.overlay_with_position(&original_output, 0, 0)?;

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

            let object_id_image_connectivity4: Image = Self::object_id_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4)?;
            _ = object_id_image_connectivity4;
            let object_id_image_connectivity8: Image = Self::object_id_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8)?;
            _ = object_id_image_connectivity8;

            let relative_position_images_connectivity4: Vec<Image> = Self::relative_position_images(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4)?;
            _ = relative_position_images_connectivity4;
            let relative_position_images_connectivity8: Vec<Image> = Self::relative_position_images(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8)?;
            _ = relative_position_images_connectivity8;

            let shape_type_count: u8 = ShapeType::len();
            let shape_type_image_connectivity4: Image = Self::shape_type_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4, false)?;
            let shape_type_image_connectivity8: Image = Self::shape_type_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8, false)?;
            let shape_type45_image_connectivity4: Image = Self::shape_type_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4, true)?;
            let shape_type45_image_connectivity8: Image = Self::shape_type_image(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8, true)?;
            
            // let shape_transformation_images_connectivity4: Vec<Image> = Self::shape_transformation_images(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity4)?;
            // let shape_transformation_images_connectivity8: Vec<Image> = Self::shape_transformation_images(&task_graph, pair_index_u8, width, height, PixelConnectivity::Connectivity8)?;

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

            // let histogram_diagonal_a: DiagonalHistogram = DiagonalHistogram::diagonal_a(&input)?;
            // let histogram_diagonal_b: DiagonalHistogram = DiagonalHistogram::diagonal_b(&input)?;

            let histogram_columns: Vec<Histogram> = input.histogram_columns();
            let histogram_rows: Vec<Histogram> = input.histogram_rows();

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
                let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
                for connectivity in connectivity_vec {
                    for color in 0..=9 {
                        match sco.enumerate_clusters(color, connectivity) {
                            Ok(image) => {
                                enumerated_clusters.insert((color, connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut enumerated_clusters_filled_holes_mask = HashMap::<(u8, PixelConnectivity), Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
                for connectivity in connectivity_vec {
                    for color in 0..=9 {
                        match sco.filled_holes_mask(color, connectivity) {
                            Ok(image) => {
                                enumerated_clusters_filled_holes_mask.insert((color, connectivity), image);
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

            let mut cluster_distance1 = HashMap::<(u8, PixelConnectivity), Image>::new();
            for color in 0..=9u8 {
                let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
                for connectivity in connectivity_vec {
                    let image: Image = input.to_mask_where_color_is_different(color);
                    let a: Image = match image.mask_distance(connectivity) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };
                    cluster_distance1.insert((color, connectivity), a);
                }
            }
            let mut cluster_distance2 = HashMap::<(u8, PixelConnectivity), Image>::new();
            for color in 0..=9u8 {
                let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
                for connectivity in connectivity_vec {
                    let image: Image = input.to_mask_where_color_is(color);
                    let a: Image = match image.mask_distance(connectivity) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };
                    cluster_distance2.insert((color, connectivity), a);
                }
            }
            let mut cluster_distance3 = HashMap::<(u8, PixelConnectivity), Image>::new();
            let mut cluster_distance4 = HashMap::<(u8, PixelConnectivity), Image>::new();
            for color in 0..=9u8 {
                let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
                for connectivity in connectivity_vec {
                    // let image: Image = input.to_mask_where_color_is_different(color);
                    let image: Image = input.to_mask_where_color_is(color);
                    let a: Image = match image.mask_distance(connectivity) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };
                    let b: Image = image.select_from_color_and_image(0, &a)?;
                    cluster_distance3.insert((color, connectivity), b);
                    let c: Image = image.select_from_image_and_color(&a, 0)?;
                    cluster_distance4.insert((color, connectivity), c);
                }
            }
            let mut cluster_distance5 = HashMap::<(u8, PixelConnectivity), Image>::new();
            for ((color, connectivity), image) in &enumerated_clusters {
                let a: Image = match image.mask_distance(*connectivity) {
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

            let mut color_to_linespan_images = HashMap::<u8, Vec::<Image>>::new();
            {
                for color in 0..=9u8 {

                    let mut images = Vec::<Image>::new();
                    {
                        let mask: Image = input.to_mask_where_color_is(color);
                        let draw_mass: bool = true;
                        let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::Before, draw_mass })?;
                        images.push(image);
                    }
                    {
                        let mask: Image = input.to_mask_where_color_is(color);
                        let draw_mass: bool = true;
                        let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Horizontal { mode: LineSpanMode::After, draw_mass })?;
                        images.push(image);
                    }
                    {
                        let mask: Image = input.to_mask_where_color_is(color);
                        let draw_mass: bool = true;
                        let image: Image = LineSpan::draw(&mask, &LineSpanDirection::Vertical { mode: LineSpanMode::Before, draw_mass })?;
                        images.push(image);
                    }
                    {
                        let mask: Image = input.to_mask_where_color_is(color);
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
                let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
                for connectivity in connectivity_vec {
                    for color in 0..=9 {
                        match sco.squares(color, connectivity) {
                            Ok(image) => {
                                squares.insert((color, connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            // let mut nonsquares = HashMap::<(u8, PixelConnectivity), Image>::new();
            // if let Some(sco) = &pair.input.image_meta.single_color_object {
            //     let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
            //     for connectivity in connectivity_vec {
            //         for color in 0..=9 {
            //             match sco.non_squares(color, connectivity) {
            //                 Ok(image) => {
            //                     nonsquares.insert((color, connectivity), image);
            //                 },
            //                 Err(_) => {},
            //             }
            //         }
            //     }
            // }

            let mut rectangles = HashMap::<(u8, PixelConnectivity), Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
                for connectivity in connectivity_vec {
                    for color in 0..=9 {
                        match sco.rectangles(color, connectivity) {
                            Ok(image) => {
                                rectangles.insert((color, connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut boxes = HashMap::<(u8, PixelConnectivity), Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
                for connectivity in connectivity_vec {
                    for color in 0..=9 {
                        match sco.boxes(color, connectivity) {
                            Ok(image) => {
                                boxes.insert((color, connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            let mut lines = HashMap::<(u8, PixelConnectivity), Image>::new();
            if let Some(sco) = &pair.input.image_meta.single_color_object {
                let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
                for connectivity in connectivity_vec {
                    for color in 0..=9 {
                        match sco.lines(color, connectivity) {
                            Ok(image) => {
                                lines.insert((color, connectivity), image);
                            },
                            Err(_) => {},
                        }
                    }
                }
            }

            // horizontal symmetry is worsening the prediction.
            // let mut horizontal_symmetry = HashMap::<(u8, PixelConnectivity), Image>::new();
            // if let Some(sco) = &pair.input.image_meta.single_color_object {
            //     let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
            //     for connectivity in connectivity_vec {
            //         for color in 0..=9 {
            //             match sco.horizontal_symmetry_mask(color, connectivity) {
            //                 Ok(image) => {
            //                     horizontal_symmetry.insert((color, connectivity), image);
            //                 },
            //                 Err(_) => {},
            //             }
            //         }
            //     }
            // }

            // vertical symmetry is worsening the prediction.
            // let mut vertical_symmetry = HashMap::<(u8, PixelConnectivity), Image>::new();
            // if let Some(sco) = &pair.input.image_meta.single_color_object {
            //     let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
            //     for connectivity in connectivity_vec {
            //         for color in 0..=9 {
            //             match sco.vertical_symmetry_mask(color, connectivity) {
            //                 Ok(image) => {
            //                     vertical_symmetry.insert((color, connectivity), image);
            //                 },
            //                 Err(_) => {},
            //             }
            //         }
            //     }
            // }

            let mut image_neighbour_up: Image = Image::color(width, height, 255);
            let mut image_neighbour_down: Image = Image::color(width, height, 255);
            let mut image_neighbour_left: Image = Image::color(width, height, 255);
            let mut image_neighbour_right: Image = Image::color(width, height, 255);
            #[allow(unused_variables)]
            #[allow(unused_assignments)]
            let mut image_neighbour_upleft: Image = Image::color(width, height, 255);
            #[allow(unused_variables)]
            #[allow(unused_assignments)]
            let mut image_neighbour_upright: Image = Image::color(width, height, 255);
            #[allow(unused_variables)]
            #[allow(unused_assignments)]
            let mut image_neighbour_downleft: Image = Image::color(width, height, 255);
            #[allow(unused_variables)]
            #[allow(unused_assignments)]
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
            _ = image_neighbour_upleft;
            _ = image_neighbour_upright;
            _ = image_neighbour_downleft;
            _ = image_neighbour_downright;

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

            // let mut horizontal_symmetry_connectivity4 = HashMap::<u8, Image>::new();
            // let mut horizontal_symmetry_connectivity8 = HashMap::<u8, Image>::new();
            // if let Some(sco) = &pair.input.image_meta.single_color_object {
            //     for color in 0..=9 {
            //         let image: Image = match sco.horizontal_symmetry_mask(color, PixelConnectivity::Connectivity4) {
            //             Ok(value) => value,
            //             Err(_) => {
            //                 continue;
            //             }
            //         };
            //         horizontal_symmetry_connectivity4.insert(color, image);
            //     }
            //     for color in 0..=9 {
            //         let image: Image = match sco.horizontal_symmetry_mask(color, PixelConnectivity::Connectivity8) {
            //             Ok(value) => value,
            //             Err(_) => {
            //                 continue;
            //             }
            //         };
            //         horizontal_symmetry_connectivity8.insert(color, image);
            //     }
            // }

            // let mut vertical_symmetry_connectivity4 = HashMap::<u8, Image>::new();
            // let mut vertical_symmetry_connectivity8 = HashMap::<u8, Image>::new();
            // if let Some(sco) = &pair.input.image_meta.single_color_object {
            //     for color in 0..=9 {
            //         let image: Image = match sco.vertical_symmetry_mask(color, PixelConnectivity::Connectivity4) {
            //             Ok(value) => value,
            //             Err(_) => {
            //                 continue;
            //             }
            //         };
            //         vertical_symmetry_connectivity4.insert(color, image);
            //     }
            //     for color in 0..=9 {
            //         let image: Image = match sco.vertical_symmetry_mask(color, PixelConnectivity::Connectivity8) {
            //             Ok(value) => value,
            //             Err(_) => {
            //                 continue;
            //             }
            //         };
            //         vertical_symmetry_connectivity8.insert(color, image);
            //     }
            // }

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
                        let shape_type: ShapeType = color_and_shape.shape_identification.shape_type;
                        let color: u8 = Self::color_from_shape_type(shape_type);
                        let mode = MixMode::PickColor1WhenColor0IsZero { color };
                        shapetype_image = color_and_shape.shape_identification.mask_uncropped.mix(&shapetype_image, mode)?;
                    }
                    earlier_prediction_shapetype_connectivity4 = Some(shapetype_image);

                    let mut shapetype45_image: Image = ep_image.clone_zero();
                    for (_color_and_shape_index, color_and_shape) in sifsco.color_and_shape_vec.iter().enumerate() {
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
            if width > height {
                input_orientation = 1;
            } else if width < height {
                input_orientation = -1;
            } else {
                input_orientation = 0;
            }

            let number_of_shape3x3ids: u8 = Shape3x3::instance().number_of_shapes();

            for y in 0..height {
                let yy: i32 = y as i32;
                let y_reverse: u8 = ((height as i32) - 1 - yy).max(0) as u8;

                // let area_top: Image = if y > 2 {
                //     input.top_rows(y - 1)?
                // } else {
                //     Image::empty()
                // };

                // let area_bottom: Image = if y_reverse > 2 {
                //     input.bottom_rows(y_reverse - 1)?
                // } else {
                //     Image::empty()
                // };

                // let mut area_top = Image::empty();
                // let mut area_bottom = Image::empty();
                // if let Some(image) = earlier_prediction_image {
                //     if y > 2 {
                //         area_top = image.top_rows(y - 1)?;
                //     };
                //     if y_reverse > 2 {
                //         area_bottom = image.bottom_rows(y_reverse - 1)?;
                //     }
                // }

                // let area_top_histogram_columns: Vec<Histogram> = area_top.histogram_columns();
                // let area_bottom_histogram_columns: Vec<Histogram> = area_bottom.histogram_columns();

                for x in 0..width {
                    let xx: i32 = x as i32;
                    let x_reverse: u8 = ((width as i32) - 1 - xx).max(0) as u8;
                    let output_color: u8 = output.get(xx, yy).unwrap_or(255);

                    let area3x3: Image = input.crop_outside(xx - 1, yy - 1, 3, 3, 255)?;
                    let area5x5: Image = input.crop_outside(xx - 2, yy - 2, 5, 5, 255)?;
                    let center: u8 = area5x5.get(2, 2).unwrap_or(255);

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
                    // let area_left_histogram_rows: Vec<Histogram> = area_left.histogram_rows();
                    // let area_right_histogram_rows: Vec<Histogram> = area_right.histogram_rows();
            
                    // let preserve_center_color: bool = histogram_preserve.get(center) > 0;

                    // let nonbackground_area3x3: Image = non_background_mask.crop_outside(xx - 1, yy - 1, 3, 3, 255)?;

                    let image_top: u8 = input.get(xx, 0).unwrap_or(255);
                    let image_bottom: u8 = input.get(xx, original_input.height() as i32 - 1).unwrap_or(255);
                    let image_left: u8 = input.get(0, yy).unwrap_or(255);
                    let image_right: u8 = input.get(original_input.width() as i32 - 1, yy).unwrap_or(255);

                    let center_x_reversed: u8 = input.get(x_reverse as i32, yy).unwrap_or(255);
                    let center_y_reversed: u8 = input.get(xx, y_reverse as i32).unwrap_or(255);
                    let center_xy_reversed: u8 = input.get(x_reverse as i32, y_reverse as i32).unwrap_or(255);
                    _ = center_xy_reversed;
                    
                    let center_denoise_type1: u8 = input_denoise_type1.get(xx, yy).unwrap_or(255);
                    // let center_denoise_type1_border: u8 = input_denoise_type1_border.get(xx, yy).unwrap_or(255);

                    let object_center: u8 = enumerated_objects.get(xx, yy).unwrap_or(255);
                    // let object_top: u8 = enumerated_objects.get(xx, yy - 1).unwrap_or(255);
                    // let object_bottom: u8 = enumerated_objects.get(xx, yy + 1).unwrap_or(255);
                    // let object_left: u8 = enumerated_objects.get(xx - 1, yy).unwrap_or(255);
                    // let object_right: u8 = enumerated_objects.get(xx + 1, yy).unwrap_or(255);
                    // let enumerated_object: u8 = enumerated_objects.get(xx, yy).unwrap_or(255);

                    let grid_mask_center: u8 = grid_mask.get(xx, yy).unwrap_or(0);
                    let grid_center: u8 = if grid_mask_center > 0 { grid_color } else { 255 };
                    let is_grid: bool = grid_mask_center > 0;

                    // let repair_center: u8 = repair_mask.get(xx, yy).unwrap_or(255);

                    let neighbour_up: u8 = image_neighbour_up.get(xx, yy).unwrap_or(255);
                    let neighbour_down: u8 = image_neighbour_down.get(xx, yy).unwrap_or(255);
                    let neighbour_left: u8 = image_neighbour_left.get(xx, yy).unwrap_or(255);
                    let neighbour_right: u8 = image_neighbour_right.get(xx, yy).unwrap_or(255);
                    // let neighbour_upleft: u8 = image_neighbour_upleft.get(xx, yy).unwrap_or(255);
                    // let neighbour_upright: u8 = image_neighbour_upright.get(xx, yy).unwrap_or(255);
                    // let neighbour_downleft: u8 = image_neighbour_downleft.get(xx, yy).unwrap_or(255);
                    // let neighbour_downright: u8 = image_neighbour_downright.get(xx, yy).unwrap_or(255);

                    let corners_center: u8 = corners.get(xx, yy).unwrap_or(255);
                    let corners_center1: bool = corners_center == 1;
                    let corners_center2: bool = corners_center == 2;
                    let corners_center3: bool = corners_center == 3;
                    let corners_center4: bool = corners_center == 4;
                    // let corners_center5: bool = corners_center >= 3;

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
                    let center_column: Image = match input.crop(Rectangle::new(x, 0, 1, height)) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    let center_row: Image = match input.crop(Rectangle::new(0, y, width, 1)) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    let center_column_top: Image = match center_column.top_rows(y) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    let center_column_bottom: Image = match center_column.bottom_rows(y_reverse) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    let center_row_left: Image = match center_row.left_columns(x) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    let center_row_right: Image = match center_row.right_columns(y_reverse) {
                        Ok(value) => value,
                        Err(_) => Image::empty()
                    };
                    



                    // let max_distance: u8 = 3;
                    // let distance_top: u8 = y.min(max_distance) + 1;
                    // let distance_bottom: u8 = y_reverse.min(max_distance) + 1;
                    // let distance_left: u8 = x.min(max_distance) + 1;
                    // let distance_right: u8 = x_reverse.min(max_distance) + 1;

                    let input_is_noise_color: bool = noise_color == Some(center);
                    // let input_is_removal_color: u8 = if removal_color == Some(center) { 1 } else { 0 };

                    let mass_connectivity4: u8 = image_mass_connectivity4.get(xx, yy).unwrap_or(0);
                    let mass_connectivity8: u8 = image_mass_connectivity8.get(xx, yy).unwrap_or(0);

                    let input_is_most_popular_color: bool = most_popular_color == Some(center);
                
                    let x_mod2: u8 = x % 2;
                    let y_mod2: u8 = y % 2;
                    let x_reverse_mod2: u8 = x_reverse % 2;
                    let y_reverse_mod2: u8 = y_reverse % 2;

                    let x_mod3: u8 = x % 3;
                    let y_mod3: u8 = y % 3;
                    let x_reverse_mod3: u8 = x_reverse % 3;
                    let y_reverse_mod3: u8 = y_reverse % 3;
                    _ = x_mod3;
                    _ = y_mod3;
                    _ = x_reverse_mod3;
                    _ = y_reverse_mod3;

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

                    let full_row_and_column: bool = is_full_row & is_full_column;
                    let full_row_xor_column: bool = is_full_row ^ is_full_column;
                    let full_row_or_column: bool = is_full_row | is_full_column;

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

                    let mut noise_color_in_outline1_connectivity4: u8 = 255;
                    let mut noise_color_in_outline1_connectivity8: u8 = 255;
                    // let mut noise_color_in_outline2_connectivity4: u8 = 0;
                    // let mut noise_color_in_outline2_connectivity8: u8 = 0;
                    if let Some(color) = noise_color {
                        if let Some(mask) = outline1_connectivity4.get(&color) {
                            noise_color_in_outline1_connectivity4 = mask.get(xx, yy).unwrap_or(0);
                        }
                        if let Some(mask) = outline1_connectivity8.get(&color) {
                            noise_color_in_outline1_connectivity8 = mask.get(xx, yy).unwrap_or(0);
                        }
                        // if let Some(mask) = outline2_connectivity4.get(&color) {
                        //     noise_color_in_outline2_connectivity4 = mask.get(xx, yy).unwrap_or(0);
                        // }
                        // if let Some(mask) = outline2_connectivity8.get(&color) {
                        //     noise_color_in_outline2_connectivity8 = mask.get(xx, yy).unwrap_or(0);
                        // }
                    }


                    // let mut the_horizontal_symmetry_connectivity4: u8 = 0;
                    // if let Some(mask) = horizontal_symmetry_connectivity4.get(&center) {
                    //     the_horizontal_symmetry_connectivity4 = mask.get(xx, yy).unwrap_or(0);
                    // }
                    // let mut the_horizontal_symmetry_connectivity8: u8 = 0;
                    // if let Some(mask) = horizontal_symmetry_connectivity8.get(&center) {
                    //     the_horizontal_symmetry_connectivity8 = mask.get(xx, yy).unwrap_or(0);
                    // }
                    // let mut the_vertical_symmetry_connectivity4: u8 = 0;
                    // if let Some(mask) = vertical_symmetry_connectivity4.get(&center) {
                    //     the_vertical_symmetry_connectivity4 = mask.get(xx, yy).unwrap_or(0);
                    // }
                    // let mut the_vertical_symmetry_connectivity8: u8 = 0;
                    // if let Some(mask) = vertical_symmetry_connectivity8.get(&center) {
                    //     the_vertical_symmetry_connectivity8 = mask.get(xx, yy).unwrap_or(0);
                    // }

                    // let mut is_corner: u8 = 0;
                    // let mut corner_top_left: u8 = 0;
                    // let mut corner_top_right: u8 = 0;
                    // let mut corner_bottom_left: u8 = 0;
                    // let mut corner_bottom_right: u8 = 0;
                    // if let Some(sco) = &pair.input.single_color_objects {
                    //     let corner_classification: u8 = sco.corner_classification(center, xx, yy);
                    //     if corner_classification > 0 {
                    //         is_corner = 1;
                    //     }
                    //     if corner_classification & 1 > 0 {
                    //         corner_top_left = 1;
                    //     }
                    //     if corner_classification & 2 > 0 {
                    //         corner_top_right = 1;
                    //     }
                    //     if corner_classification & 4 > 0 {
                    //         corner_bottom_left = 1;
                    //     }
                    //     if corner_classification & 8 > 0 {
                    //         corner_bottom_right = 1;
                    //     }
                    // }

                    #[allow(unused_variables)]
                    #[allow(unused_assignments)]
                    let mut inside_bounding_box: bool = false;
                    if let Some(sco) = &pair.input.image_meta.single_color_object {
                        if sco.is_inside_bounding_box(center, xx, yy) {
                            inside_bounding_box = true;
                        }
                    }
                    _ = inside_bounding_box;

                    let half_horizontal: i8;
                    if xx * 2 == width as i32 { 
                        half_horizontal = 0;
                    } else {
                        if xx * 2 < width as i32 { 
                            half_horizontal = -1;
                        } else { 
                            half_horizontal = 1;
                        };
                    }
                    let half_vertical: i8;
                    if yy * 2 == height as i32 { 
                        half_vertical = 0;
                    } else {
                        if yy * 2 < height as i32 { 
                            half_vertical = -1;
                        } else { 
                            half_vertical = 1;
                        };
                    }

                    let input_has_unambiguous_connectivity: bool = input_unambiguous_connectivity_histogram.get(center) > 0;

                    let mut record = Record {
                        classification: output_color,
                        is_test,
                        pair_id,
                        values: vec!(),
                    };
                    for area_y in 0..area5x5.height() {
                        for area_x in 0..area5x5.width() {
                            let color: u8 = area5x5.get(area_x as i32, area_y as i32).unwrap_or(255);
                            record.serialize_color_complex(color, obfuscated_color_offset);
                        }
                    }

                    // record.serialize_bool_onehot(preserve_center_color);
                    // {
                    //     let color: u8 = if preserve_center_color { center } else { 255 };
                    //     record.serialize_onehot(color, 11);
                    // }

                    // for color in 0..9u8 {
                    //     let maybe_color_is_present: bool = histogram_predicted_palette.get(color) > 0;
                    //     record.serialize_bool_onehot(maybe_color_is_present);
                    // }

                    record.serialize_color_complex(center_x_reversed, obfuscated_color_offset);
                    record.serialize_color_complex(center_y_reversed, obfuscated_color_offset);
                    // record.serialize_color_complex(center_xy_reversed, obfuscated_color_offset);
                    record.serialize_color_complex(mass_connectivity4, obfuscated_color_offset);
                    record.serialize_color_complex(mass_connectivity8, obfuscated_color_offset);
                    // record.serialize_u8(mass_connectivity4);
                    // record.serialize_u8(mass_connectivity8);
                    // record.serialize_onehot_discard_overflow(mass_connectivity4, 40);
                    // record.serialize_onehot_discard_overflow(mass_connectivity8, 40);
                    record.serialize_ternary(input_orientation);
                    // record.serialize_u8(distance_top);
                    // record.serialize_u8(distance_bottom);
                    // record.serialize_u8(distance_left);
                    // record.serialize_u8(distance_right);
                    record.serialize_ternary(half_horizontal);
                    record.serialize_ternary(half_vertical);
                    record.serialize_bool_onehot(input_is_noise_color);
                    record.serialize_bool_onehot(input_is_most_popular_color);
                    // record.serialize_bool(input_is_removal_color == 1);

                    // record.serialize_u8(x + 2);
                    // record.serialize_u8(y + 2);
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

                    record.serialize_onehot_discard_overflow(x_mod2, 2);
                    record.serialize_onehot_discard_overflow(y_mod2, 2);
                    record.serialize_onehot_discard_overflow(x_reverse_mod2, 2);
                    record.serialize_onehot_discard_overflow(y_reverse_mod2, 2);
                    // record.serialize_onehot_discard_overflow(x_mod3, 3);
                    // record.serialize_onehot_discard_overflow(y_mod3, 3);
                    // record.serialize_onehot_discard_overflow(x_reverse_mod3, 3);
                    // record.serialize_onehot_discard_overflow(y_reverse_mod3, 3);
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
                    // record.serialize_onehot_discard_overflow((x_mod2 + y_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x_mod2 + y_reverse_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x_reverse_mod2 + y_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x_reverse_mod2 + y_reverse_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x2_mod2 + y2_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x2_mod2 + y2_reverse_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x2_reverse_mod2 + y2_mod2) & 1, 2);
                    // record.serialize_onehot_discard_overflow((x2_reverse_mod2 + y2_reverse_mod2) & 1, 2);
                    record.serialize_bool_onehot(preserve_edge);
                    record.serialize_bool(full_row_and_column);
                    record.serialize_bool(full_row_xor_column);
                    record.serialize_bool(full_row_or_column);
                    record.serialize_bool(one_or_more_holes_connectivity4);
                    record.serialize_bool(one_or_more_holes_connectivity8);
                    record.serialize_color_complex(the_holecount_connectivity4, obfuscated_color_offset);
                    record.serialize_color_complex(the_holecount_connectivity8, obfuscated_color_offset);
                    // record.serialize_u8(the_holecount_connectivity4);
                    // record.serialize_u8(the_holecount_connectivity8);
                    record.serialize_onehot_discard_overflow(the_holecount_connectivity4, 2);
                    record.serialize_onehot_discard_overflow(the_holecount_connectivity8, 2);
                    // record.serialize_onehot_discard_overflow(the_holecount_connectivity4.min(9), 10);
                    // record.serialize_onehot_discard_overflow(the_holecount_connectivity8.min(9), 10);
                    record.serialize_bool(corners_center1);
                    record.serialize_bool(corners_center2);
                    record.serialize_bool(corners_center3);
                    record.serialize_bool(corners_center4);
                    // record.serialize_bool_onehot(corners_center5);
                    for i in 0..10 {
                        // let value: u8 = if no_change_to_color[i] { 1 } else { 0 };
                        // record.serialize_u8(value);
                        let value2: u8 = if no_change_to_color[i] { i as u8 } else { 255 };
                        record.serialize_color_complex(value2, obfuscated_color_offset);
                    }
                    for i in 0..10 {
                        // let value: u8 = if input_histogram_intersection[i] { 1 } else { 0 };
                        // record.serialize_u8(value);
                        let value2: u8 = if input_histogram_intersection[i] { i as u8 } else { 255 };
                        record.serialize_color_complex(value2, obfuscated_color_offset);
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
                    record.serialize_color_complex(noise_color_in_outline1_connectivity4, obfuscated_color_offset);
                    record.serialize_color_complex(noise_color_in_outline1_connectivity8, obfuscated_color_offset);
                    // record.serialize_u8(noise_color_in_outline2_connectivity4); // worsens the prediction
                    // record.serialize_u8(noise_color_in_outline2_connectivity8); // worsens the prediction

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



                    {
                        // let count: u16 = histogram_columns[x as usize].number_of_counters_greater_than_zero();
                        // record.serialize_bool_onehot(count > 0);
                        // record.serialize_f64(count as f64);
                        // record.serialize(histogram_columns.num, count)
                        for color in 0..=9u8 {
                            record.serialize_bool(histogram_columns[x as usize].get(color) > 0);
                        }

                        // for color in 0..=9u8 {
                        //     let x_minus1: i32 = (x as i32) - 2;
                        //     let mut value: bool = false;
                        //     if x_minus1 >= 0 {
                        //         value = histogram_columns[x_minus1 as usize].get(color) > 0;
                        //     }
                        //     record.serialize_bool(value);
                        // }
                        // for color in 0..=9u8 {
                        //     let x_minus1: i32 = (x as i32) - 1;
                        //     let mut value: bool = false;
                        //     if x_minus1 >= 0 {
                        //         value = histogram_columns[x_minus1 as usize].get(color) > 0;
                        //     }
                        //     record.serialize_bool(value);
                        // }

                        // for color in 0..=9u8 {
                        //     let x_plus1: u16 = (x as u16) + 1;
                        //     let mut value: bool = false;
                        //     if x_plus1 < (width as u16) {
                        //         value = histogram_columns[x_plus1 as usize].get(color) > 0;
                        //     }
                        //     record.serialize_bool(value);
                        // }
                        // for color in 0..=9u8 {
                        //     let x_plus1: u16 = (x as u16) + 2;
                        //     let mut value: bool = false;
                        //     if x_plus1 < (width as u16) {
                        //         value = histogram_columns[x_plus1 as usize].get(color) > 0;
                        //     }
                        //     record.serialize_bool(value);
                        // }
                    }

                    {
                        // let count: u16 = histogram_rows[y as usize].number_of_counters_greater_than_zero();
                        // record.serialize_bool_onehot(count > 0);
                        // record.serialize_f64(count as f64);
                        // record.serialize(histogram_columns.num, count)
                        for color in 0..=9u8 {
                            record.serialize_bool(histogram_rows[y as usize].get(color) > 0);
                        }

                        // for color in 0..=9u8 {
                        //     let y_minus1: i32 = (y as i32) - 2;
                        //     let mut value: bool = false;
                        //     if y_minus1 >= 0 {
                        //         value = histogram_rows[y_minus1 as usize].get(color) > 0;
                        //     }
                        //     record.serialize_bool(value);
                        // }
                        // for color in 0..=9u8 {
                        //     let y_minus1: i32 = (y as i32) - 1;
                        //     let mut value: bool = false;
                        //     if y_minus1 >= 0 {
                        //         value = histogram_rows[y_minus1 as usize].get(color) > 0;
                        //     }
                        //     record.serialize_bool(value);
                        // }

                        // for color in 0..=9u8 {
                        //     let y_plus1: u16 = (y as u16) + 1;
                        //     let mut value: bool = false;
                        //     if y_plus1 < (height as u16) {
                        //         value = histogram_rows[y_plus1 as usize].get(color) > 0;
                        //     }
                        //     record.serialize_bool(value);
                        // }
                        // for color in 0..=9u8 {
                        //     let y_plus1: u16 = (y as u16) + 2;
                        //     let mut value: bool = false;
                        //     if y_plus1 < (height as u16) {
                        //         value = histogram_rows[y_plus1 as usize].get(color) > 0;
                        //     }
                        //     record.serialize_bool(value);
                        // }
                    }

                    // {
                    //     for color in 0..=9u8 {
                    //         let mut found = false;
                    //         if let Some(histogram) = histogram_diagonal_a.get(x as i32, y as i32) {
                    //             if histogram.get(color) > 0 {
                    //                 found = true;
                    //             }
                    //         }
                    //         record.serialize_bool_onehot(found);
                    //     }
                    // }

                    // {
                    //     for color in 0..=9u8 {
                    //         let mut found = false;
                    //         if let Some(histogram) = histogram_diagonal_b.get(x as i32, y as i32) {
                    //             if histogram.get(color) > 0 {
                    //                 found = true;
                    //             }
                    //         }
                    //         record.serialize_bool_onehot(found);
                    //     }
                    // }

                    {
                        // let mut count: u8 = 0;
                        // if let Some(histogram) = histogram_diagonal_a.get(x as i32, y as i32) {
                        //     count = histogram.number_of_counters_greater_than_zero().min(255) as u8;
                        // }
                        // record.serialize_onehot(count + 1, 4);
                        // record.serialize_onehot(count, 20);
                        // record.serialize_u8(count);
                        // record.serialize_bool_onehot(count > 1);
                    }

                    {
                        // let mut count: u8 = 0;
                        // if let Some(histogram) = histogram_diagonal_b.get(x as i32, y as i32) {
                        //     count = histogram.number_of_counters_greater_than_zero().min(255) as u8;
                        // }
                        // record.serialize_onehot(count + 1, 4);
                        // record.serialize_onehot(count, 20);
                        // record.serialize_u8(count);
                        // record.serialize_bool_onehot(count > 1);
                    }
                    
                    // {
                    //     for color in 0..=9u8 {
                    //         let mut mass : u8 = 0;
                    //         if let Some(histogram) = histogram_diagonal_a.get(x as i32, y as i32) {
                    //             mass = histogram.get(color).min(255) as u8;
                    //         }
                    //         record.serialize_onehot(mass, 4);
                    //         // record.serialize_u8(mass);
                    //     }
                    //     for color in 0..=9u8 {
                    //         let mut mass : u8 = 0;
                    //         if let Some(histogram) = histogram_diagonal_b.get(x as i32, y as i32) {
                    //             mass = histogram.get(color).min(255) as u8;
                    //         }
                    //         record.serialize_onehot(mass, 4);
                    //         // record.serialize_u8(mass);
                    //     }
                    // }

                    let mut color_hole_type1: u8 = 255;
                    if let Some(image) = color_to_hole_type1.get(&center) {
                        color_hole_type1 = image.get(xx, yy).unwrap_or(0);
                    }
                    record.serialize_color_complex(color_hole_type1, obfuscated_color_offset);

                    let mut color_repair: u8 = 255;
                    if let Some(image) = color_to_repair.get(&center) {
                        color_repair = image.get(xx, yy).unwrap_or(0);
                    }
                    record.serialize_color_complex(color_repair, obfuscated_color_offset);

                    // for color in 0..=9u8 {
                    //     let mut color_repair: u8 = 255;
                    //     if let Some(image) = color_to_hole_type1.get(&color) {
                    //         color_repair = image.get(xx, yy).unwrap_or(0);
                    //     }
                    //     record.serialize_color_complex(color_repair, obfuscated_color_offset);
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
                            record.serialize_color_complex(neighbour_color, obfuscated_color_offset);
                        }
                    }

                    // Future experiment
                    // for all 10 colors.
                    // look in the diagonal direction, skip the first 2 colors, and pick the 2nd color

    
                    // Cluster id
                    {
                        let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let cluster_id: u8 = match enumerated_clusters.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                record.serialize_cluster_id(color, cluster_id);
                                // record.serialize_cluster_id(color, 255 - cluster_id);
                                // record.serialize_complex(cluster_id as u16, 41);
                            }
                        }
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let cluster_id: u8 = match small_medium_big.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                record.serialize_complex(cluster_id as u16, 4);
                                // record.serialize_onehot_discard_overflow(cluster_id, 4);
                            }
                        }
                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let cluster_id: u8 = match sort2_small_big.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(255)
                                    }
                                    None => 255
                                };
                                // record.serialize_complex(cluster_id as u16, 3);
                                record.serialize_onehot_discard_overflow(cluster_id, 3);
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
                                // record.serialize_onehot(distance, 20);
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
                                // record.serialize_split_zeros_ones(distance, 5);
                                // record.serialize_split_zeros_ones(distance, 8);
                                // record.serialize_onehot(distance, 20);
                                record.serialize_bool(distance % 2 == 0);
                            }
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
                            }
                        }

                        // non-squares are worsening the prediction.
                        // for connectivity in &connectivity_vec {
                        //     for color in 0..=9 {
                        //         let is_square: bool = match nonsquares.get(&(color, *connectivity)) {
                        //             Some(value) => {
                        //                 value.get(xx, yy).unwrap_or(0) > 0
                        //             }
                        //             None => false
                        //         };
                        //         record.serialize_bool(is_square);
                        //     }
                        // }

                        for connectivity in &connectivity_vec {
                            for color in 0..=9 {
                                let is_rectangle: bool = match rectangles.get(&(color, *connectivity)) {
                                    Some(value) => {
                                        value.get(xx, yy).unwrap_or(0) > 0
                                    }
                                    None => false
                                };
                                record.serialize_bool(is_rectangle);
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

                        // horizontal symmetry is worsening the prediction.
                        // for connectivity in &connectivity_vec {
                        //     for color in 0..=9 {
                        //         let is_symmetric: bool = match horizontal_symmetry.get(&(color, *connectivity)) {
                        //             Some(value) => {
                        //                 value.get(xx, yy).unwrap_or(0) > 0
                        //             }
                        //             None => false
                        //         };
                        //         record.serialize_bool(is_symmetric);
                        //     }
                        // }

                        // vertical symmetry is worsening the prediction.
                        // for connectivity in &connectivity_vec {
                        //     for color in 0..=9 {
                        //         let is_symmetric: bool = match vertical_symmetry.get(&(color, *connectivity)) {
                        //             Some(value) => {
                        //                 value.get(xx, yy).unwrap_or(0) > 0
                        //             }
                        //             None => false
                        //         };
                        //         record.serialize_bool(is_symmetric);
                        //     }
                        // }

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
                        //             record.serialize_cluster_id(color, cluster_id);
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
                            record.serialize_color_complex(most_popular.unwrap_or(255), obfuscated_color_offset);
                            record.serialize_color_complex(least_popular.unwrap_or(255), obfuscated_color_offset);
                            // let count: u16 = h.number_of_counters_greater_than_zero();
                            // record.serialize_f64((count+1) as f64);
                            // record.serialize_bool(count < 2);
                        }
                    }

                    record.serialize_color_complex(center_denoise_type1, obfuscated_color_offset);
                    // record.serialize_color_complex(center_denoise_type1_border, obfuscated_color_offset);

                    // let is_border_most_popular_color: bool = Some(center) == border_most_popular_color;
                    // let is_border_least_popular_color: bool = Some(center) == border_least_popular_color;
                    // record.serialize_bool_onehot(is_border_most_popular_color);
                    // record.serialize_bool_onehot(is_border_least_popular_color);
                    // let border_histogram_count: u32 = histogram_border.get(center);
                    // record.serialize_bool_onehot(border_histogram_count > 0);

                    for color in 0..=9 {
                        let mut is_inside_bounding_box: bool = false;
                        if let Some(sco) = &pair.input.image_meta.single_color_object {
                            is_inside_bounding_box = sco.is_inside_bounding_box(color, xx, yy);
                        }
                        record.serialize_bool(is_inside_bounding_box);
                        // record.serialize_bool_onehot(is_inside_bounding_box)
                    }

                    // for linespan_image in &linespan_images {
                        // let pixel: u8 = linespan_image.get(xx, yy).unwrap_or(255);
                        // let is_line: bool = pixel > 0;
                        // record.serialize_bool(is_line);
                    // }

                    if let Some(images) = color_to_linespan_images.get(&center) {
                        for linespan_image in images {
                            let pixel: u8 = linespan_image.get(xx, yy).unwrap_or(255);
                            record.serialize_u8(pixel);
                            // record.serialize_onehot(pixel, 4);
                            // record.serialize_onehot_discard_overflow(pixel, 2);
                        }
                    }
                    
                    // {
                    //     let pixel: u8 = object_id_image_connectivity4.get(xx, yy).unwrap_or(255);
                    //     record.serialize_onehot(pixel, 255);
                    //     record.serialize_u8(pixel);
                    //     record.serialize_complex(pixel as u16, 256);
                    // }
                    // {
                    //     let pixel: u8 = object_id_image_connectivity8.get(xx, yy).unwrap_or(255);
                    //     record.serialize_onehot(pixel, 255);
                    //     record.serialize_u8(pixel);
                    //     record.serialize_complex(pixel as u16, 256);
                    // }

                    // for relative_position_image in &relative_position_images_connectivity4 {
                    //     let pixel: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                    //     record.serialize_u8(pixel);
                    //     // record.serialize_onehot(pixel, 30);
                    // }
                    // for relative_position_image in &relative_position_images_connectivity8 {
                    //     let pixel: u8 = relative_position_image.get(xx, yy).unwrap_or(255);
                    //     record.serialize_u8(pixel);
                    //     // record.serialize_onehot(pixel, 30);
                    // }

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
                    // for shape_transformation_image in &shape_transformation_images_connectivity4 {
                    //     let pixel: u8 = shape_transformation_image.get(xx, yy).unwrap_or(255);
                    //     record.serialize_u8(pixel);
                    //     record.serialize_bitmask_as_onehot(pixel as u16, 8);
                    // }
                    // for shape_transformation_image in &shape_transformation_images_connectivity8 {
                    //     let pixel: u8 = shape_transformation_image.get(xx, yy).unwrap_or(255);
                    //     record.serialize_u8(pixel);
                    //     record.serialize_bitmask_as_onehot(pixel as u16, 8);
                    // }

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
                        //     record.serialize_color_complex(mass, obfuscated_color_offset);
                        //     record.serialize_u8(mass);
                        //     record.serialize_onehot_discard_overflow(mass, 40);
                        // }

                        // if let Some(image) = &earlier_prediction_mass_connectivity8 {
                        //     let mass: u8 = image.get(xx, yy).unwrap_or(0);
                        //     record.serialize_color_complex(mass, obfuscated_color_offset);
                        //     record.serialize_u8(mass);
                        //     record.serialize_onehot_discard_overflow(mass, 40);
                        // }

                        if let Some(image) = earlier_prediction_image {
                            // let pixel: u8 = image.get(xx, yy).unwrap_or(0);
                            // record.serialize_onehot(pixel, 10);
                            // record.serialize_bool_onehot(pixel == center);
                            // record.serialize_color_complex(pixel, obfuscated_color_offset);

                            {
                                let pixel: u8 = image.get(xx - 1, yy - 1).unwrap_or(255);
                                record.serialize_onehot_discard_overflow(pixel, 10);
                            }
                            {
                                let pixel: u8 = image.get(xx, yy - 1).unwrap_or(255);
                                record.serialize_onehot_discard_overflow(pixel, 10);
                            }
                            {
                                let pixel: u8 = image.get(xx + 1, yy - 1).unwrap_or(255);
                                record.serialize_onehot_discard_overflow(pixel, 10);
                            }
                            {
                                let pixel: u8 = image.get(xx - 1, yy).unwrap_or(255);
                                record.serialize_onehot_discard_overflow(pixel, 10);
                            }
                            // {
                            //     let pixel: u8 = image.get(xx, yy).unwrap_or(255);
                            //     record.serialize_onehot_discard_overflow(pixel, 10);
                            //     record.serialize_bool(Some(pixel) == most_popular_color);
                            // }
                            {
                                let pixel: u8 = image.get(xx + 1, yy).unwrap_or(255);
                                record.serialize_onehot_discard_overflow(pixel, 10);
                            }
                            {
                                let pixel: u8 = image.get(xx - 1, yy + 1).unwrap_or(255);
                                record.serialize_onehot_discard_overflow(pixel, 10);
                            }
                            {
                                let pixel: u8 = image.get(xx, yy + 1).unwrap_or(255);
                                record.serialize_onehot_discard_overflow(pixel, 10);
                            }
                            {
                                let pixel: u8 = image.get(xx + 1, yy + 1).unwrap_or(255);
                                record.serialize_onehot_discard_overflow(pixel, 10);
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
                    //     record.serialize_color_complex(color0, obfuscated_color_offset);
                    //     record.serialize_color_complex(color1, obfuscated_color_offset);
                    //     record.serialize_color_complex(color2, obfuscated_color_offset);
                    //     record.serialize_color_complex(color3, obfuscated_color_offset);
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
                    //         record.serialize_color_complex(agree_color, obfuscated_color_offset);
                    //         // record.serialize_bool(center == color0);
                    //         // record.serialize_bool(center == color1);
                    //         // record.serialize_bool(center == color2);
                    //         // record.serialize_bool(center == color3);
                    //         // record.serialize_color_complex(color0, obfuscated_color_offset);
                    //         // record.serialize_color_complex(color1, obfuscated_color_offset);
                    //         // record.serialize_color_complex(color2, obfuscated_color_offset);
                    //         // record.serialize_color_complex(color3, obfuscated_color_offset);
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
                    
                    // These are worsening the predictions.
                    // input_is_removal_color: u8,
                    // distance_top: PixelColor,
                    // distance_bottom: PixelColor,
                    // distance_left: PixelColor,
                    // distance_right: PixelColor,
                    // y mod3: u8,
                    // x mod3: u8,
                    // preserve corner: u8,
                    // x_distance_from_center: i16,
                    // y_distance_from_center: i16,
                    // record.serialize_u8(the_horizontal_symmetry_connectivity4);
                    // record.serialize_u8(the_horizontal_symmetry_connectivity8);
                    // record.serialize_u8(the_vertical_symmetry_connectivity4);
                    // record.serialize_u8(the_vertical_symmetry_connectivity8);
                    // record.serialize_u8(is_corner);
                    // record.serialize_u8(corner_top_left);
                    // record.serialize_u8(corner_top_right);
                    // record.serialize_u8(corner_bottom_left);
                    // record.serialize_u8(corner_bottom_right);
                    record.serialize_bool_onehot(is_grid);
                    record.serialize_color_complex(grid_center, obfuscated_color_offset);
                    record.serialize_color_complex(grid_color, obfuscated_color_offset);
                    // record.serialize_bool(inside_bounding_box);
                    // record.serialize_complex(object_center, 20);

                    records.push(record);
                }
            }
        }

        Ok(records)
    }
}

struct MyDataset {
    dataset: Dataset<f64, usize, Ix1>,
    split_ratio: f32,
}

fn dataset_from_records(records: &Vec<Record>) -> anyhow::Result<MyDataset> {
    let mut data: Vec<f64> = Vec::new();
    let mut values_max: usize = 0;
    let mut values_min: usize = usize::MAX;
    for record in records {
        data.push(record.classification as f64);
        data.push(record.is_test as f64);
        data.push(record.pair_id as f64);
        for value in &record.values {
            data.push(*value);
        }
        let value_count: usize = record.values.len();
        values_max = values_max.max(value_count);
        values_min = values_min.min(value_count);
    }
    if values_max != values_min {
        return Err(anyhow::anyhow!("values_max != values_min. values_max: {} values_min: {}", values_max, values_min));
    }
    // println!("values_max: {}", values_max);
    let columns: usize = values_max + 3;

    let array1: Array1<f64> = Array1::<f64>::from(data);
    let array: Array2<f64> = array1.into_shape((records.len(), columns))?;

    // split using the "is_test" column
    // the "is_test" column, determine where the split point is
    let col1 = array.column(1);
    let mut n_above: usize = 0;
    let mut n_below: usize = 0;
    for item in col1.iter() {
        if *item > 0.01 {
            n_above += 1;
        } else {
            n_below += 1;
        }
    }
    let split_ratio: f32 = (n_below as f32) / ((n_above + n_below) as f32);
    // println!("train: {} test: {} split_ratio: {}", n_below, n_above, split_ratio);

    let (data, targets) = (
        array.slice(s![.., 2..]).to_owned(),
        array.column(0).to_owned(),
    );

    let dataset = Dataset::new(data, targets)
        .map_targets(|x| *x as usize);

    let instance = MyDataset {
        dataset,
        split_ratio,
    };
    Ok(instance)
}

fn perform_logistic_regression(task: &Task, test_index: u8, records: &Vec<Record>) -> anyhow::Result<Image> {
    // println!("task_id: {}", task.id);

    // Future experiment:
    // Deal with ARC tasks that have 2 or more `test` pairs.
    // If there are multiple `test` pairs, then the `test` pairs should be split into multiple `valid` pairs.
    // Currently assumes that there is only 1 `test` pair. So the `pred.get(address)` behaves the same for all the `test` pairs.
    //
    // Run logistic regression on the `train` pairs. By adding the `train` pairs to the `valid` pairs. 
    // If all the `train` inputs yields the correct output.
    // Then run logistic regression on the `test` pairs.

    let dataset: Dataset<f64, usize, Ix1>;
    let ratio: f32;
    {
        let my_dataset: MyDataset = dataset_from_records(records)?;
        ratio = my_dataset.split_ratio;
        dataset = my_dataset.dataset;
    }

    // split using the "is_test" column
    // let (train, valid) = dataset.split_with_ratio(0.9);
    let (train, valid) = dataset.split_with_ratio(ratio);

    // println!(
    //     "Fit Multinomial Logistic Regression classifier with #{} training points",
    //     train.nsamples()
    // );

    // fit a Logistic regression model with 150 max iterations
    let model = MultiLogisticRegression::default()
        .max_iterations(50)
        .fit(&train)
        .context("MultiLogisticRegression")?;

    // predict and map targets
    let pred = model.predict(&valid);

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
