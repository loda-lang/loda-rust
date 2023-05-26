use super::arc_work_model::{Task, PairType};
use super::{Image, ImageOverlay};
use super::{ActionLabel, InputLabel};
use super::{HtmlLog, ImageCrop, Rectangle, PixelConnectivity, ImageHistogram, Histogram, ImageEdge, ImageMask};
use super::{ImageNeighbour, ImageNeighbourDirection, ImageCornerAnalyze};
use anyhow::Context;
use serde::Serialize;
use std::collections::HashMap;
use linfa::prelude::*;
use linfa_logistic::MultiLogisticRegression;
use ndarray::prelude::*;

#[derive(Clone, Debug, Serialize)]
struct Record {
    classification: u8,
    is_test: u8,
    pair_id: u8,
    values: Vec<u8>,
}

impl Record {
    fn serialize_raw(&mut self, value: u8) {
        self.values.push(value);
    }

    fn serialize_color(&mut self, color: u8) {
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
        match color {
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
        self.values.push(color0);
        self.values.push(color1);
        self.values.push(color2);
        self.values.push(color3);
        self.values.push(color4);
        self.values.push(color5);
        self.values.push(color6);
        self.values.push(color7);
        self.values.push(color8);
        self.values.push(color9);
        self.values.push(color_other);
    }
}

pub struct ExperimentWithLogisticRegression {
    #[allow(dead_code)]
    tasks: Vec<Task>,
}

impl ExperimentWithLogisticRegression {
    #[allow(dead_code)]
    pub fn new(tasks: Vec<Task>) -> Self {
        println!("loaded {} tasks", tasks.len());
        Self {
            tasks,
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

        if !task.is_output_size_same_as_input_size() {
            HtmlLog::text(&format!("skipping task: {} because output size is not the same as input size", task.id));
            return Ok(());
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

            let mut grid_color: u8 = 255;
            let mut grid_mask: Image = Image::empty();
            if let Some(grid_pattern) = &pair.input.grid_pattern {
                grid_mask = grid_pattern.line_mask.clone();
                grid_color = grid_pattern.color;
            }

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
                    let object_top: u8 = enumerated_objects.get(xx, yy - 1).unwrap_or(255);
                    let object_bottom: u8 = enumerated_objects.get(xx, yy + 1).unwrap_or(255);
                    let object_left: u8 = enumerated_objects.get(xx - 1, yy).unwrap_or(255);
                    let object_right: u8 = enumerated_objects.get(xx + 1, yy).unwrap_or(255);
                    // let enumerated_object: u8 = enumerated_objects.get(xx, yy).unwrap_or(255);

                    let grid_mask_center: u8 = grid_mask.get(xx, yy).unwrap_or(0);
                    let grid_center: u8 = if grid_mask_center > 0 { grid_color } else { 255 };
                    let is_grid: u8 = if grid_mask_center > 0 { 1 } else { 0 };

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
                    let neighbour_upleft: u8 = image_neighbour_upleft.get(xx, yy).unwrap_or(255);
                    let neighbour_upright: u8 = image_neighbour_upright.get(xx, yy).unwrap_or(255);
                    let neighbour_downleft: u8 = image_neighbour_downleft.get(xx, yy).unwrap_or(255);
                    let neighbour_downright: u8 = image_neighbour_downright.get(xx, yy).unwrap_or(255);

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

                    let mut is_corner: u8 = 0;
                    let mut corner_top_left: u8 = 0;
                    let mut corner_top_right: u8 = 0;
                    let mut corner_bottom_left: u8 = 0;
                    let mut corner_bottom_right: u8 = 0;
                    if let Some(sco) = &pair.input.single_color_objects {
                        let corner_classification: u8 = sco.corner_classification(center, xx, yy);
                        if corner_classification > 0 {
                            is_corner = 1;
                        }
                        if corner_classification & 1 > 0 {
                            corner_top_left = 1;
                        }
                        if corner_classification & 2 > 0 {
                            corner_top_right = 1;
                        }
                        if corner_classification & 4 > 0 {
                            corner_bottom_left = 1;
                        }
                        if corner_classification & 8 > 0 {
                            corner_bottom_right = 1;
                        }
                    }

                    let mut inside_bounding_box: u8 = 0;
                    if let Some(sco) = &pair.input.single_color_objects {
                        if sco.is_inside_bounding_box(center, xx, yy) {
                            inside_bounding_box = 1;
                        }
                    }

                    let half_left: u8 = if xx * 2 < width as i32 { 1 } else { 0 };
                    let half_right: u8 = if xx * 2 > width as i32 { 1 } else { 0 };
                    let half_top: u8 = if yy * 2 < height as i32 { 1 } else { 0 };
                    let half_bottom: u8 = if yy * 2 > height as i32 { 1 } else { 0 };

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
                    record.serialize_raw(distance_top);
                    record.serialize_raw(distance_bottom);
                    record.serialize_raw(distance_left);
                    record.serialize_raw(distance_right);
                    record.serialize_raw(input_is_noise_color);
                    record.serialize_raw(input_is_most_popular_color);
                    record.serialize_raw(x_mod2);
                    record.serialize_raw(y_mod2);
                    record.serialize_raw(x_reverse_mod2);
                    record.serialize_raw(y_reverse_mod2);
                    record.serialize_raw(preserve_edge);
                    record.serialize_raw(full_row_and_column);
                    record.serialize_raw(full_row_xor_column);
                    record.serialize_raw(full_row_or_column);
                    record.serialize_raw(one_or_more_holes_connectivity4);
                    record.serialize_raw(one_or_more_holes_connectivity8);
                    record.serialize_color(the_holecount_connectivity4);
                    record.serialize_color(the_holecount_connectivity8);
                    record.serialize_raw(corners_center1);
                    record.serialize_raw(corners_center2);
                    record.serialize_raw(corners_center3);
                    record.serialize_raw(corners_center4);
                    for i in 0..10 {
                        let value: u8 = if no_change_to_color[i] { 1 } else { 0 };
                        record.serialize_raw(value);
                    }
                    for i in 0..10 {
                        let value: u8 = if input_histogram_intersection[i] { 1 } else { 0 };
                        record.serialize_raw(value);
                    }
                    record.serialize_raw(input_has_unambiguous_connectivity);
                    // record.serialize_raw(is_corner);
                    // record.serialize_raw(corner_top_left);
                    // record.serialize_raw(corner_top_right);
                    // record.serialize_raw(corner_bottom_left);
                    // record.serialize_raw(corner_bottom_right);
                    // record.serialize_raw(inside_bounding_box);
                    // record.serialize_raw(is_grid);
                    // record.serialize_color(grid_center);
                    // record.serialize_color(grid_color);
                    // record.serialize_raw(half_left);
                    // record.serialize_raw(half_right);
                    // record.serialize_raw(half_top);
                    // record.serialize_raw(half_bottom);
                    record.serialize_raw(v0);
                    record.serialize_raw(v1);
                    record.serialize_raw(v2);
                    record.serialize_raw(v3);
                    record.serialize_raw(v4);
                    record.serialize_raw(v5);
                    record.serialize_raw(v6);
                    record.serialize_raw(v7);

                    // Future experiments
                    // push all the training pairs that have been rotated by 90 degrees.
                    // push all the training pairs that have been flipped.
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

                    records.push(record);
                }
            }
        }

        perform_logistic_regression(task, &records)?;

        Ok(())
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
            data.push(*value as f64);
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
    println!("train: {} test: {} split_ratio: {}", n_below, n_above, split_ratio);

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

fn perform_logistic_regression(task: &Task, records: &Vec<Record>) -> anyhow::Result<()> {
    println!("task_id: {}", task.id);

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
            if task.occur_in_solutions_csv {
                HtmlLog::text(format!("{} - correct - already solved in asm", task.id));
            } else {
                HtmlLog::text(format!("{} - correct - no previous solution", task.id));
            }
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
