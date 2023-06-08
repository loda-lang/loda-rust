//! Performs logistic regression of each input pixel with the corresponding classification for the output pixel.
//! 
//! This doesn't solve any of the tasks from the hidden dataset.
//!
//! This solves 41 of the 800 tasks in the public ARC dataset.
//! 009d5c81, 00d62c1b, 0a2355a6, 2281f1f4, 25d8a9c8, 32597951, 332efdb3, 3618c87e, 37d3e8b2, 
//! 4258a5f9, 50cb2852, 543a7ed5, 67385a82, 67a3c6ac, 69889d6e, 6c434453, 6d75e8bb, 6e82a1ae, 
//! 6f8cd79b, 810b9b61, 84f2aca1, 903d1b4a, 95990924, a699fb00, a9f96cdd, ae58858e, aedd82e4, 
//! b1948b0a, b2862040, b60334d2, b6afb2da, bb43febb, c0f76784, c8f0f002, ce039d91, ce22a75a, 
//! d2abd087, d364b489, d406998b, e0fb7511, e8593010
//! 
//! Weakness: The tasks that it solves doesn't involve object manipulation. 
//! It cannot move an object by a few pixels, the object must stay steady in the same position.
use super::arc_json_model::GridFromImage;
use super::arc_work_model::{Task, PairType};
use super::{Image, ImageOverlay, arcathon_solution_json, arc_json_model, ImageMix, MixMode};
use super::{ActionLabel, InputLabel};
use super::{HtmlLog, PixelConnectivity, ImageHistogram, Histogram, ImageEdge, ImageMask};
use super::{ImageNeighbour, ImageNeighbourDirection, ImageCornerAnalyze, ImageMaskGrow};
use super::human_readable_utc_timestamp;
use anyhow::Context;
use indicatif::ProgressBar;
use serde::Serialize;
use std::collections::HashMap;
use linfa::prelude::*;
use linfa_logistic::MultiLogisticRegression;
use ndarray::prelude::*;

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
    fn serialize_u8(&mut self, value: u8) {
        self.values.push(value as f64);
    }

    fn serialize_color(&mut self, color: u8) {
        self.serialize_onehot(color, 10);
    }

    /// Set the counter to 1 that are equal to the value.
    /// 
    /// Otherwise the counters are zero.
    /// 
    /// When the value overflows the capacity then set the `other` counter to 1.
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

    /// Set the counter to 1 that are equal to the value.
    /// 
    /// Otherwise the counters are zero.
    /// 
    /// When the value overflows then all the counters are set to zero.
    #[allow(dead_code)]
    fn serialize_onehot_discard_overflow(&mut self, value: u8, count: u8) {
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
}

pub struct ExperimentWithLogisticRegression {
    #[allow(dead_code)]
    tasks: Vec<Task>,
}

impl ExperimentWithLogisticRegression {
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
        let mut count_solved: usize = 0;
        let pb = ProgressBar::new(number_of_tasks as u64);
        for task in &self.tasks {
            match Self::process_task(task, verify_test_output) {
                Ok(_predictions) => {
                    count_solved += 1;
                    pb.println(format!("task {} - solved", task.id));
                },
                Err(error) => {
                    if verbose {
                        pb.println(format!("task {} - error: {:?}", task.id, error));
                    }
                }
            }
            pb.inc(1);
        }
        pb.finish_and_clear();
        println!("{} - run - end", human_readable_utc_timestamp());
        println!("{} - solved {} of {} tasks", human_readable_utc_timestamp(), count_solved, number_of_tasks);
        Ok(())
    }

    pub fn process_task(task: &Task, verify_test_output: bool) -> anyhow::Result<Vec::<arcathon_solution_json::Prediction>> {
        // println!("exporting task: {}", task.id);

        if !task.is_output_size_same_as_input_size() {
            if WRITE_TO_HTMLLOG {
                HtmlLog::text(&format!("skipping task: {} because output size is not the same as input size", task.id));
            }
            return Err(anyhow::anyhow!("skipping task: {} because output size is not the same as input size", task.id));
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
        for label in &task.input_label_set_intersection {
            match label {
                InputLabel::InputUnambiguousConnectivityWithColor { color } => {
                    input_unambiguous_connectivity_histogram.increment(*color);
                },
                _ => {}
            }
        }

        let mut records = Vec::<Record>::new();
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            let pair_id: u8 = pair_index.min(255) as u8;

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

            let mut enumerated_objects: Image = Image::zero(width, height);
            if let Some(image) = &pair.input.enumerated_objects {
                enumerated_objects = enumerated_objects.overlay_with_position(image, 0, 0)?;
            }

            // let mut grid_color: u8 = 255;
            // let mut grid_mask: Image = Image::empty();
            // if let Some(grid_pattern) = &pair.input.grid_pattern {
            //     grid_mask = grid_pattern.line_mask.clone();
            //     grid_color = grid_pattern.color;
            // }

            // let mut repair_mask: Image = Image::zero(width, height);
            // if let Some(mask) = &pair.input.repair_mask {
            //     repair_mask = repair_mask.overlay_with_position(mask, 0, 0)?;
            // }

            let noise_color: Option<u8> = pair.input.single_pixel_noise_color;
            let most_popular_color: Option<u8> = pair.input.most_popular_intersection_color;
            // let removal_color: Option<u8> = pair.input.removal_color;

            let mut image_mass_connectivity4: Image = Image::zero(width, height);
            let mut image_mass_connectivity8: Image = Image::zero(width, height);
            if let Some(sco) = &pair.input.single_color_objects {
                if let Ok(image) = sco.mass_as_image(PixelConnectivity::Connectivity4) {
                    image_mass_connectivity4 = image_mass_connectivity4.overlay_with_position(&image, 0, 0)?;
                }
                if let Ok(image) = sco.mass_as_image(PixelConnectivity::Connectivity8) {
                    image_mass_connectivity8 = image_mass_connectivity8.overlay_with_position(&image, 0, 0)?;
                }
            }

            let histogram_columns: Vec<Histogram> = input.histogram_columns();
            let histogram_rows: Vec<Histogram> = input.histogram_rows();

            let mut image_neighbour_up: Image = Image::color(width, height, 255);
            let mut image_neighbour_down: Image = Image::color(width, height, 255);
            let mut image_neighbour_left: Image = Image::color(width, height, 255);
            let mut image_neighbour_right: Image = Image::color(width, height, 255);
            // let mut image_neighbour_upleft: Image = Image::color(width, height, 255);
            // let mut image_neighbour_upright: Image = Image::color(width, height, 255);
            // let mut image_neighbour_downleft: Image = Image::color(width, height, 255);
            // let mut image_neighbour_downright: Image = Image::color(width, height, 255);
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
                // match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpLeft, 255) {
                //     Ok(image) => {
                //         image_neighbour_upleft = image;
                //     },
                //     Err(_) => {},
                // }
                // match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpRight, 255) {
                //     Ok(image) => {
                //         image_neighbour_upright = image;
                //     },
                //     Err(_) => {},
                // }
                // match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownLeft, 255) {
                //     Ok(image) => {
                //         image_neighbour_downleft = image;
                //     },
                //     Err(_) => {},
                // }
                // match input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownRight, 255) {
                //     Ok(image) => {
                //         image_neighbour_downright = image;
                //     },
                //     Err(_) => {},
                // }
            }

            let mut holes_connectivity4 = HashMap::<u8, Image>::new();
            let mut holes_connectivity8 = HashMap::<u8, Image>::new();
            if let Some(sco) = &pair.input.single_color_objects {
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

            let mut holecount_connectivity4 = HashMap::<u8, Image>::new();
            let mut holecount_connectivity8 = HashMap::<u8, Image>::new();
            if let Some(sco) = &pair.input.single_color_objects {
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
            // if let Some(sco) = &pair.input.single_color_objects {
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
            // if let Some(sco) = &pair.input.single_color_objects {
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

            for y in 0..height {
                for x in 0..width {
                    let xx: i32 = x as i32;
                    let yy: i32 = y as i32;
                    let x_reverse: u8 = ((width as i32) - 1 - xx).max(0) as u8;
                    let y_reverse: u8 = ((height as i32) - 1 - yy).max(0) as u8;
                    let output_color: u8 = output.get(xx, yy).unwrap_or(255);

                    let top_left: u8 = input.get(xx -1, yy - 1).unwrap_or(255);
                    let top: u8 = input.get(xx, yy - 1).unwrap_or(255);
                    let top_right: u8 = input.get(xx + 1, yy - 1).unwrap_or(255);
                    let left: u8 = input.get(xx - 1, yy).unwrap_or(255);
                    let center: u8 = input.get(xx, yy).unwrap_or(255);
                    let right: u8 = input.get(xx + 1, yy).unwrap_or(255);
                    let bottom_left: u8 = input.get(xx - 1, yy + 1).unwrap_or(255);
                    let bottom: u8 = input.get(xx, yy + 1).unwrap_or(255);
                    let bottom_right: u8 = input.get(xx + 1, yy + 1).unwrap_or(255);

                    let image_top: u8 = input.get(xx, 0).unwrap_or(255);
                    let image_bottom: u8 = input.get(xx, original_input.height() as i32 - 1).unwrap_or(255);
                    let image_left: u8 = input.get(0, yy).unwrap_or(255);
                    let image_right: u8 = input.get(original_input.width() as i32 - 1, yy).unwrap_or(255);

                    let center_x_reversed: u8 = input.get(x_reverse as i32, yy).unwrap_or(255);
                    let center_y_reversed: u8 = input.get(xx, y_reverse as i32).unwrap_or(255);
                    
                    let object_center: u8 = enumerated_objects.get(xx, yy).unwrap_or(255);
                    // let object_top: u8 = enumerated_objects.get(xx, yy - 1).unwrap_or(255);
                    // let object_bottom: u8 = enumerated_objects.get(xx, yy + 1).unwrap_or(255);
                    // let object_left: u8 = enumerated_objects.get(xx - 1, yy).unwrap_or(255);
                    // let object_right: u8 = enumerated_objects.get(xx + 1, yy).unwrap_or(255);
                    // let enumerated_object: u8 = enumerated_objects.get(xx, yy).unwrap_or(255);

                    // let grid_mask_center: u8 = grid_mask.get(xx, yy).unwrap_or(0);
                    // let grid_center: u8 = if grid_mask_center > 0 { grid_color } else { 255 };
                    // let is_grid: u8 = if grid_mask_center > 0 { 1 } else { 0 };

                    // let repair_center: u8 = repair_mask.get(xx, yy).unwrap_or(255);

                    let t: i32 = 2;
                    let top0: u8 = input.get(xx - t, yy - t).unwrap_or(255);
                    let top1: u8 = input.get(xx - 1, yy - t).unwrap_or(255);
                    let top2: u8 = input.get(xx, yy - t).unwrap_or(255);
                    let top3: u8 = input.get(xx + 1, yy - t).unwrap_or(255);
                    let top4: u8 = input.get(xx + t, yy - t).unwrap_or(255);
                    let left1: u8 = input.get(xx - t, yy - 1).unwrap_or(255);
                    let left2: u8 = input.get(xx - t, yy).unwrap_or(255);
                    let left3: u8 = input.get(xx - t, yy + 1).unwrap_or(255);
                    let right1: u8 = input.get(xx + t, yy - 1).unwrap_or(255);
                    let right2: u8 = input.get(xx + t, yy).unwrap_or(255);                                    
                    let right3: u8 = input.get(xx + t, yy + 1).unwrap_or(255);
                    let bottom0: u8 = input.get(xx - t, yy + t).unwrap_or(255);
                    let bottom1: u8 = input.get(xx - 1, yy + t).unwrap_or(255);
                    let bottom2: u8 = input.get(xx, yy + t).unwrap_or(255);
                    let bottom3: u8 = input.get(xx + 1, yy + t).unwrap_or(255);
                    let bottom4: u8 = input.get(xx + t, yy + t).unwrap_or(255);

                    let neighbour_up: u8 = image_neighbour_up.get(xx, yy).unwrap_or(255);
                    let neighbour_down: u8 = image_neighbour_down.get(xx, yy).unwrap_or(255);
                    let neighbour_left: u8 = image_neighbour_left.get(xx, yy).unwrap_or(255);
                    let neighbour_right: u8 = image_neighbour_right.get(xx, yy).unwrap_or(255);
                    // let neighbour_upleft: u8 = image_neighbour_upleft.get(xx, yy).unwrap_or(255);
                    // let neighbour_upright: u8 = image_neighbour_upright.get(xx, yy).unwrap_or(255);
                    // let neighbour_downleft: u8 = image_neighbour_downleft.get(xx, yy).unwrap_or(255);
                    // let neighbour_downright: u8 = image_neighbour_downright.get(xx, yy).unwrap_or(255);

                    let corners_center: u8 = corners.get(xx, yy).unwrap_or(255);
                    let corners_center1: u8 = if corners_center == 1 { 1 } else { 0 };
                    let corners_center2: u8 = if corners_center == 2 { 1 } else { 0 };
                    let corners_center3: u8 = if corners_center == 3 { 1 } else { 0 };
                    let corners_center4: u8 = if corners_center == 4 { 1 } else { 0 };

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

                    let max_distance: u8 = 3;
                    let distance_top: u8 = y.min(max_distance);
                    let distance_bottom: u8 = y_reverse.min(max_distance);
                    let distance_left: u8 = x.min(max_distance);
                    let distance_right: u8 = x_reverse.min(max_distance);

                    let input_is_noise_color: u8 = if noise_color == Some(center) { 1 } else { 0 };
                    // let input_is_removal_color: u8 = if removal_color == Some(center) { 1 } else { 0 };

                    let mass_connectivity4: u8 = image_mass_connectivity4.get(xx, yy).unwrap_or(0);
                    let mass_connectivity8: u8 = image_mass_connectivity8.get(xx, yy).unwrap_or(0);

                    let input_is_most_popular_color: u8 = if most_popular_color == Some(center) { 1 } else { 0 };
                
                    let x_mod2: u8 = x % 2;
                    let y_mod2: u8 = y % 2;
                    let x_reverse_mod2: u8 = x_reverse % 2;
                    let y_reverse_mod2: u8 = y_reverse % 2;

                    // let x_mod3: u8 = x % 3;
                    // let y_mod3: u8 = y % 3;
                    // let x_reverse_mod3: u8 = x_reverse % 3;
                    // let y_reverse_mod3: u8 = y_reverse % 3;

                    let mut preserve_edge: u8 = 0;

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
                                            preserve_edge = 1;
                                        }
                                    },
                                    ImageEdge::Bottom => {
                                        if y_reverse == 0 {
                                            preserve_edge = 1;
                                        }
                                    },
                                    ImageEdge::Left => {
                                        if x == 0 {
                                            preserve_edge = 1;
                                        }
                                    },
                                    ImageEdge::Right => {
                                        if x_reverse == 0 {
                                            preserve_edge = 1;
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

                    let full_row_and_column: u8 = if is_full_row & is_full_column { 1 } else { 0 };
                    let full_row_xor_column: u8 = if is_full_row ^ is_full_column { 1 } else { 0 };
                    let full_row_or_column: u8 = if is_full_row | is_full_column { 1 } else { 0 };

                    let mut one_or_more_holes_connectivity4: u8 = 0;
                    if let Some(hole_mask) = holes_connectivity4.get(&center) {
                        if hole_mask.get(xx, yy).unwrap_or(0) > 0 {
                            one_or_more_holes_connectivity4 = 1;
                        }
                    }
                    let mut one_or_more_holes_connectivity8: u8 = 0;
                    if let Some(hole_mask) = holes_connectivity8.get(&center) {
                        if hole_mask.get(xx, yy).unwrap_or(0) > 0 {
                            one_or_more_holes_connectivity8 = 1;
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

                    let mut noise_color_in_outline1_connectivity4: u8 = 0;
                    let mut noise_color_in_outline1_connectivity8: u8 = 0;
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

                    // let mut inside_bounding_box: u8 = 0;
                    // if let Some(sco) = &pair.input.single_color_objects {
                    //     if sco.is_inside_bounding_box(center, xx, yy) {
                    //         inside_bounding_box = 1;
                    //     }
                    // }

                    // let half_left: u8 = if xx * 2 < width as i32 { 1 } else { 0 };
                    // let half_right: u8 = if xx * 2 > width as i32 { 1 } else { 0 };
                    // let half_top: u8 = if yy * 2 < height as i32 { 1 } else { 0 };
                    // let half_bottom: u8 = if yy * 2 > height as i32 { 1 } else { 0 };

                    let input_has_unambiguous_connectivity: u8 = if input_unambiguous_connectivity_histogram.get(center) > 0 { 1 } else { 0 };

                    let mut record = Record {
                        classification: output_color,
                        is_test,
                        pair_id,
                        values: vec!(),
                    };
                    record.serialize_color(top_left);
                    record.serialize_color(top);
                    record.serialize_color(top_right);
                    record.serialize_color(left);
                    record.serialize_color(center);
                    record.serialize_color(right);
                    record.serialize_color(bottom_left);
                    record.serialize_color(bottom);
                    record.serialize_color(bottom_right);
                    record.serialize_color(top0);
                    record.serialize_color(top1);
                    record.serialize_color(top2);
                    record.serialize_color(top3);
                    record.serialize_color(top4);
                    record.serialize_color(left1);
                    record.serialize_color(left2);
                    record.serialize_color(left3);
                    record.serialize_color(right1);
                    record.serialize_color(right2);
                    record.serialize_color(right3);
                    record.serialize_color(bottom0);
                    record.serialize_color(bottom1);
                    record.serialize_color(bottom2);
                    record.serialize_color(bottom3);
                    record.serialize_color(bottom4);
                    record.serialize_color(center_x_reversed);
                    record.serialize_color(center_y_reversed);
                    record.serialize_color(mass_connectivity4);
                    record.serialize_color(mass_connectivity8);
                    record.serialize_u8(distance_top);
                    record.serialize_u8(distance_bottom);
                    record.serialize_u8(distance_left);
                    record.serialize_u8(distance_right);
                    record.serialize_u8(input_is_noise_color);
                    record.serialize_u8(input_is_most_popular_color);
                    record.serialize_u8(x_mod2);
                    record.serialize_u8(y_mod2);
                    record.serialize_u8(x_reverse_mod2);
                    record.serialize_u8(y_reverse_mod2);
                    record.serialize_u8(preserve_edge);
                    record.serialize_u8(full_row_and_column);
                    record.serialize_u8(full_row_xor_column);
                    record.serialize_u8(full_row_or_column);
                    record.serialize_u8(one_or_more_holes_connectivity4);
                    record.serialize_u8(one_or_more_holes_connectivity8);
                    record.serialize_color(the_holecount_connectivity4);
                    record.serialize_color(the_holecount_connectivity8);
                    record.serialize_u8(corners_center1);
                    record.serialize_u8(corners_center2);
                    record.serialize_u8(corners_center3);
                    record.serialize_u8(corners_center4);
                    for i in 0..10 {
                        let value: u8 = if no_change_to_color[i] { 1 } else { 0 };
                        record.serialize_u8(value);
                    }
                    for i in 0..10 {
                        let value: u8 = if input_histogram_intersection[i] { 1 } else { 0 };
                        record.serialize_u8(value);
                    }
                    record.serialize_u8(input_has_unambiguous_connectivity);
                    record.serialize_u8(v0);
                    record.serialize_u8(v1);
                    record.serialize_u8(v2);
                    record.serialize_u8(v3);
                    record.serialize_u8(v4);
                    record.serialize_u8(v5);
                    record.serialize_u8(v6);
                    record.serialize_u8(v7);
                    record.serialize_u8(noise_color_in_outline1_connectivity4);
                    record.serialize_u8(noise_color_in_outline1_connectivity8);
                    // record.serialize_u8(noise_color_in_outline2_connectivity4); // worsens the prediction
                    // record.serialize_u8(noise_color_in_outline2_connectivity8); // worsens the prediction

                    let mut row_contains_noise_color: u8 = 0;
                    let mut column_contains_noise_color: u8 = 0;
                    if let Some(color) = noise_color {
                        if histogram_rows[y as usize].get(color) > 0 {
                            row_contains_noise_color = 1;
                        }
                        if histogram_columns[x as usize].get(color) > 0 {
                            column_contains_noise_color = 1;
                        }
                    }
                    record.serialize_u8(row_contains_noise_color);
                    record.serialize_u8(column_contains_noise_color);


                    // Future experiments
                    // push all the training pairs that have been rotated by 90 degrees.
                    // push all the training pairs that have been flipped.
                    //
                    // when the image is splitted into multiple cells, example 3 cells:
                    // cell0: is inside split area 0
                    // cell1: is inside split area 1
                    // cell2: is inside split area 2
                    // border01: is on the border between cell0 and cell1
                    // border12: is on the border between cell1 and cell2
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
                    // record.serialize_u8(inside_bounding_box);
                    // record.serialize_u8(is_grid);
                    // record.serialize_color(grid_center);
                    // record.serialize_color(grid_color);
                    // record.serialize_u8(half_left);
                    // record.serialize_u8(half_right);
                    // record.serialize_u8(half_top);
                    // record.serialize_u8(half_bottom);

                    records.push(record);
                }
            }
        }

        let predictions: Vec::<arcathon_solution_json::Prediction> = perform_logistic_regression(
            task, 
            &records, 
            verify_test_output,
        )?;

        Ok(predictions)
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
        return Err(anyhow::anyhow!("values_max != values_min"));
    }
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

fn perform_logistic_regression(task: &Task, records: &Vec<Record>, verify_test_output: bool) -> anyhow::Result<Vec::<arcathon_solution_json::Prediction>> {
    // println!("task_id: {}", task.id);

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

    let mut predictions = Vec::<arcathon_solution_json::Prediction>::new();
    let mut count_test: usize = 0;
    for pair in &task.pairs {
        if pair.pair_type != PairType::Test {
            continue;
        }

        let index: usize = count_test;
        count_test += 1;
        
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

        {
            let grid: arc_json_model::Grid = arc_json_model::Grid::from_image(&computed_image);
            let prediction = arcathon_solution_json::Prediction {
                prediction_id: index as u8,
                output: grid,
            };
            predictions.push(prediction);
        }

        if WRITE_TO_HTMLLOG {
            let expected_output: Image = pair.output.test_image.clone();
            if computed_image == expected_output {
                if task.occur_in_solutions_csv {
                    HtmlLog::text(format!("{} - correct - already solved in asm", task.id));
                } else {
                    HtmlLog::text(format!("{} - correct - no previous solution", task.id));
                }
                HtmlLog::image(&computed_image);
            } else {
                // HtmlLog::text(format!("{} - incorrect", task.id));
                // let images: Vec<Image> = vec![
                //     original_input,
                //     expected_output,
                //     computed_image.clone(),
                // ];
                // HtmlLog::compare_images(images);
            }
        }

        if verify_test_output {
            let expected_output: Image = pair.output.test_image.clone();
            if computed_image != expected_output {
                return Err(anyhow::anyhow!("The predicted output doesn't match with the expected output"));
            }
        }
    }

    // Calculate the accuracy and Matthew Correlation Coefficient (cross-correlation between
    // predicted and targets)
    // println!("accuracy {}, MCC {}", cm.accuracy(), cm.mcc());
    // HtmlLog::text(format!("accuracy {}, MCC {}", cm.accuracy(), cm.mcc()));
    Ok(predictions)
}
