use super::arc_work_model::{Task, PairType};
use super::{Image, ImageOverlay};
use crate::arc::{HtmlLog, ImageCrop, Rectangle, PixelConnectivity, ActionLabel, ImageHistogram, Histogram, ImageEdge, ImageCorner, ImageMask};
use crate::config::Config;
use anyhow::Context;
use std::path::{PathBuf, Path};
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use csv::WriterBuilder;
use std::error::Error;
use linfa::prelude::*;
use linfa_logistic::MultiLogisticRegression;
use ndarray::prelude::*;
use std::fs::File;
use std::io::Read;
use csv::ReaderBuilder;
use ndarray_csv::{Array2Reader, ReadError};

#[derive(Clone, Debug)]
struct PixelColor {
    value: u8,
}

impl From<u8> for PixelColor {
    fn from(value: u8) -> Self {
        Self {
            value,
        }
    }
}

impl Serialize for PixelColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut color0: u8 = 0;
        let mut color1: u8 = 0;
        let mut color2: u8 = 0;
        let mut color3: u8 = 0;
        let mut color4: u8 = 0;
        let mut color5: u8 = 0;
        let mut color6: u8 = 0;
        let mut color7: u8 = 0;
        let mut color8: u8 = 0;
        let mut color9: u8 = 0;
        let mut color_other: u8 = 0;
        match self.value {
            0 => color0 = 1,
            1 => color1 = 1,
            2 => color2 = 1,
            3 => color3 = 1,
            4 => color4 = 1,
            5 => color5 = 1,
            6 => color6 = 1,
            7 => color7 = 1,
            8 => color8 = 1,
            9 => color9 = 1,
            _ => color_other = 1,
        };
        let mut s = serializer.serialize_struct("PixelColor", 11)?;
        s.serialize_field("color0", &color0)?;
        s.serialize_field("color1", &color1)?;
        s.serialize_field("color2", &color2)?;
        s.serialize_field("color3", &color3)?;
        s.serialize_field("color4", &color4)?;
        s.serialize_field("color5", &color5)?;
        s.serialize_field("color6", &color6)?;
        s.serialize_field("color7", &color7)?;
        s.serialize_field("color8", &color8)?;
        s.serialize_field("color9", &color9)?;
        s.serialize_field("color_other", &color_other)?;
        s.end()
    }
}

#[derive(Clone, Debug, Serialize)]
struct Record {
    classification: u8,
    is_test: u8,
    pair_id: u8,
    top_left: PixelColor,
    top: PixelColor,
    top_right: PixelColor,
    left: PixelColor,
    center: PixelColor,
    right: PixelColor,
    bottom_left: PixelColor,
    bottom: PixelColor,
    bottom_right: PixelColor,
    distance_top: u8,
    distance_bottom: u8,
    distance_left: u8,
    distance_right: u8,
    input_is_noise_color: u8,
    input_is_most_popular_color: u8,
    x_mod2: u8,
    y_mod2: u8,
    x_reverse_mod2: u8,
    y_reverse_mod2: u8,
    preserve_edge: u8,
    full_row_and_column: u8,
    full_row_xor_column: u8,
    full_row_or_column: u8,
    top0: PixelColor,
    top1: PixelColor,
    top2: PixelColor,
    top3: PixelColor,
    top4: PixelColor,
    left1: PixelColor,
    left2: PixelColor,
    left3: PixelColor,
    right1: PixelColor,
    right2: PixelColor,
    right3: PixelColor,
    bottom0: PixelColor,
    bottom1: PixelColor,
    bottom2: PixelColor,
    bottom3: PixelColor,
    bottom4: PixelColor,
    center_x_reversed: PixelColor,
    center_y_reversed: PixelColor,
    mass_connectivity4: PixelColor,
    mass_connectivity8: PixelColor,
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,

    // Future experiments
    // is insertion color
    // distance from center x: i8,
    // distance from center y: i8,
    // direction up color
    // direction down color
    // direction left color
    // direction right color
    
    // These are worsening the predictions.
    // input_is_removal_color: u8,
    // mass_connectivity8: u8,
    // distance_top: PixelColor,
    // distance_bottom: PixelColor,
    // distance_left: PixelColor,
    // distance_right: PixelColor,
    // y mod3: u8,
    // x mod3: u8,
    // preserve corner: u8,
}

pub struct ExperimentWithLogisticRegression {
    #[allow(dead_code)]
    tasks: Vec<Task>,

    config: Config,
}

impl ExperimentWithLogisticRegression {
    #[allow(dead_code)]
    pub fn new(tasks: Vec<Task>) -> Self {
        println!("loaded {} tasks", tasks.len());
        let config = Config::load();
        Self {
            tasks,
            config,
        }
    }

    #[allow(dead_code)]
    pub fn run(&mut self) -> anyhow::Result<()> {
        self.run_all()?;
        // self.run_specific()?;
        Ok(())
    }

    #[allow(dead_code)]
    fn run_all(&mut self) -> anyhow::Result<()> {
        let mut count: usize = 0;
        for task in &self.tasks {
            if count >= 0 && count <= 800 {
                self.export_task_debug(task);
            }
            count += 1;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn run_specific(&mut self) -> anyhow::Result<()> {
        let task_ids = [
            "3618c87e",
            "3aa6fb7a",
            "6f8cd79b",
            "95990924",
            "a699fb00",
            "a79310a0",
            "b6afb2da",
            "bb43febb",
            "d364b489",
        ];
        for task_id in task_ids {
            self.export_task_id(task_id)?;
        }
        Ok(())
    }

    fn export_task_id(&self, task_id: &str) -> anyhow::Result<()> {
        // println!("exporting task: {}", task_id);

        let mut found_task: Option<&Task> = None;
        for task in &self.tasks {
            if task.id != task_id {
                continue;
            }
            found_task = Some(task);
        }
        let task: &Task = match found_task {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("didn't find a task_id: {}", task_id));
            }
        };
        self.export_task(task)
    }

    fn export_task_debug(&self, task: &Task) {
        match self.export_task(task) {
            Ok(()) => {},
            Err(error) => {
                println!("export_task: {:?}", error);
            }
        }
    }

    fn export_task(&self, task: &Task) -> anyhow::Result<()> {
        println!("exporting task: {}", task.id);
        let path: PathBuf = self.config.analytics_arc_dir().join(format!("{}.csv", task.id));

        let mut records = Vec::<Record>::new();
        let mut pair_id: u8 = 0;
        for pair in &task.pairs {
            let is_test: u8;
            let original_output: Image;
            match pair.pair_type {
                PairType::Train => {
                    is_test = 0;
                    original_output = pair.output.image.clone();
                },
                PairType::Test => {
                    is_test = 1;
                    original_output = pair.output.test_image.clone();
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

            // let mut grid_mask: Image = Image::zero(width, height);
            // if let Some(grid_pattern) = &pair.input.grid_pattern {
            //     grid_mask = grid_mask.overlay_with_position(&grid_pattern.line_mask, 0, 0)?;
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

                    let center_x_reversed: u8 = input.get(x_reverse as i32, yy).unwrap_or(255);
                    let center_y_reversed: u8 = input.get(xx, y_reverse as i32).unwrap_or(255);
                    
                    let object_center: u8 = enumerated_objects.get(xx, yy).unwrap_or(255);
                    let object_top: u8 = enumerated_objects.get(xx, yy - 1).unwrap_or(255);
                    let object_bottom: u8 = enumerated_objects.get(xx, yy + 1).unwrap_or(255);
                    let object_left: u8 = enumerated_objects.get(xx - 1, yy).unwrap_or(255);
                    let object_right: u8 = enumerated_objects.get(xx + 1, yy).unwrap_or(255);
                    // let enumerated_object: u8 = enumerated_objects.get(xx, yy).unwrap_or(255);

                    // let grid_center: u8 = grid_mask.get(xx, yy).unwrap_or(255);
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
                    // v2 = grid_center;
                    // v2 = repair_center;
                    // v2 = x_mod3;
                    // v3 = y_mod3;
                    // v4 = x_reverse_mod3;
                    // v5 = y_reverse_mod3;

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
                                    v2 = 1;
                                }
                                if noise_color == Some(*color) {
                                    v3 = 1;
                                }
                                if most_popular_color == Some(*color) {
                                    v4 = 1;
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
                        if histogram.number_of_counters_greater_than_zero() == 1 {
                            is_full_column = true;
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
                        if histogram.number_of_counters_greater_than_zero() == 1 {
                            is_full_row = true;
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

                    let record = Record {
                        classification: output_color,
                        is_test,
                        pair_id,
                        top_left: PixelColor::from(top_left),
                        top: PixelColor::from(top),
                        top_right: PixelColor::from(top_right),
                        left: PixelColor::from(left),
                        center: PixelColor::from(center),
                        right: PixelColor::from(right),
                        bottom_left: PixelColor::from(bottom_left),
                        bottom: PixelColor::from(bottom),
                        bottom_right: PixelColor::from(bottom_right),
                        distance_top,
                        distance_bottom,
                        distance_left,
                        distance_right,
                        input_is_noise_color,
                        input_is_most_popular_color,
                        x_mod2,
                        y_mod2,
                        x_reverse_mod2,
                        y_reverse_mod2,
                        preserve_edge,
                        full_row_and_column,
                        full_row_xor_column,
                        full_row_or_column,
                        top0: PixelColor::from(top0),
                        top1: PixelColor::from(top1),
                        top2: PixelColor::from(top2),
                        top3: PixelColor::from(top3),
                        top4: PixelColor::from(top4),
                        left1: PixelColor::from(left1),
                        left2: PixelColor::from(left2),
                        left3: PixelColor::from(left3),
                        right1: PixelColor::from(right1),
                        right2: PixelColor::from(right2),
                        right3: PixelColor::from(right3),
                        bottom0: PixelColor::from(bottom0),
                        bottom1: PixelColor::from(bottom1),
                        bottom2: PixelColor::from(bottom2),
                        bottom3: PixelColor::from(bottom3),
                        bottom4: PixelColor::from(bottom4),
                        center_x_reversed: PixelColor::from(center_x_reversed),
                        center_y_reversed: PixelColor::from(center_y_reversed),
                        mass_connectivity4: PixelColor::from(mass_connectivity4),
                        mass_connectivity8: PixelColor::from(mass_connectivity8),
                        v0,
                        v1,
                        v2,
                        v3,
                        v4,
                        v5,
                    };

                    records.push(record);
                }
            }

            pair_id += 1;
        }

        println!("saving file: {:?}", path);
        match create_csv_file_without_header(&records, &path) {
            Ok(()) => {},
            Err(error) => {
                return Err(anyhow::anyhow!("could not save: {:?}", error));
            }
        }

        match perform_logistic_regression(task, &path) {
            Ok(()) => {},
            Err(error) => {
                return Err(anyhow::anyhow!("perform_logistic_regression: {:?}", error));
            }
        }

        Ok(())
    }
}

fn create_csv_file_without_header<S: Serialize>(records: &Vec<S>, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_path(output_path)?;
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}

fn array_from_csv<R: Read>(
    csv: R,
    has_headers: bool,
    separator: u8,
) -> Result<Array2<f64>, ReadError> {
    // parse CSV
    let mut reader = ReaderBuilder::new()
        .has_headers(has_headers)
        .delimiter(separator)
        .from_reader(csv);

    // extract ndarray
    reader.deserialize_array2_dynamic()
}

struct MyDataset {
    dataset: Dataset<f64, usize, Ix1>,
    split_ratio: f32,
}

fn load_dataset(path: &Path) -> Result<MyDataset, Box<dyn Error>> {
    let file = File::open(path)?;
    let array: Array2<f64> = array_from_csv(file, false, b';')?;
    // println!("{:?}", array);

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
    println!("train: {} test: {} split_ratio: {}", n_below, n_above, split_ratio);

    let (data, targets) = (
        array.slice(s![.., 2..]).to_owned(),
        array.column(0).to_owned(),
    );

    let feature_names = vec![
        "pair_id",
        "center_color0",
        "center_color1",
        "center_color2",
        "center_color3",
        "center_color4",
        "center_color5",
        "center_color6",
        "center_color7",
        "center_color8",
        "center_color9",
        "center_color_padding",
        "top_color0",
        "top_color1",
        "top_color2",
        "top_color3",
        "top_color4",
        "top_color5",
        "top_color6",
        "top_color7",
        "top_color8",
        "top_color9",
        "top_color_padding",
    ];

    let dataset = Dataset::new(data, targets)
        .map_targets(|x| *x as usize)
        .with_feature_names(feature_names);

    let instance = MyDataset {
        dataset,
        split_ratio,
    };
    Ok(instance)
}

fn perform_logistic_regression(task: &Task, path: &Path) -> Result<(), Box<dyn Error>> {
    println!("task_id: {}", task.id);

    let dataset: Dataset<f64, usize, Ix1>;
    let ratio: f32;
    {
        let my_dataset: MyDataset = load_dataset(&path)?;
        ratio = my_dataset.split_ratio;
        dataset = my_dataset.dataset;
    }

    // split using the "is_test" column
    // let (train, valid) = dataset.split_with_ratio(0.9);
    let (train, valid) = dataset.split_with_ratio(ratio);

    println!(
        "Fit Multinomial Logistic Regression classifier with #{} training points",
        train.nsamples()
    );

    // fit a Logistic regression model with 150 max iterations
    let model = MultiLogisticRegression::default()
        .max_iterations(50)
        .fit(&train)
        .context("MultiLogisticRegression")?;

    // predict and map targets
    let pred = model.predict(&valid);

    // create a confusion matrix
    let cm = pred.confusion_matrix(&valid)
        .context("confusion_matrix")?;

    // Print the confusion matrix, this will print a table with four entries. On the diagonal are
    // the number of true-positive and true-negative predictions, off the diagonal are
    // false-positive and false-negative
    println!("{:?}", cm);

    // print out the predicted output pixel values
    // println!("{:?}", pred);

    for pair in &task.pairs {

        let expected_output: Image;
        match pair.pair_type {
            PairType::Train => {
                continue;
            },
            PairType::Test => {
                expected_output = pair.output.test_image.clone();
            },
        }
        let original_input: Image = pair.input.image.clone();

        let width: u8 = original_input.width().max(expected_output.width()).min(253);
        let height: u8 = original_input.height().max(expected_output.height()).min(253);

        let mut result_image: Image = Image::color(width, height, 10);
        for y in 0..height {
            for x in 0..width {
                let xx: i32 = x as i32;
                let yy: i32 = y as i32;
                let address: usize = (y as usize) * (width as usize) + (x as usize);
                let predicted_color: u8 = match pred.get(address) {
                    Some(value) => (*value).min(u8::MAX as usize) as u8,
                    None => 255
                };
                _ = result_image.set(xx, yy, predicted_color);
            }
        }
        result_image = result_image.crop(Rectangle::new(0, 0, expected_output.width(), expected_output.height()))?;

        if result_image == expected_output {
            HtmlLog::text(format!("{} - correct", task.id));
            HtmlLog::image(&result_image);
        } else {
            HtmlLog::text(format!("{} - incorrect", task.id));
            // let images: Vec<Image> = vec![
            //     original_input,
            //     expected_output,
            //     result_image,
            // ];
            // HtmlLog::compare_images(images);
        }
    }


    // Calculate the accuracy and Matthew Correlation Coefficient (cross-correlation between
    // predicted and targets)
    println!("accuracy {}, MCC {}", cm.accuracy(), cm.mcc());
    Ok(())
}
