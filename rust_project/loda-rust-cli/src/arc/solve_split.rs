//! Solve `split-view` like tasks.
//! 
//! * With the public ARC 1 dataset. It can solve 17 tasks.
//! * With the hidden ARC 1 dataset. It can solve 0 tasks.
//! 
//! Known problem: Can only split into columns or rows, not both.
//! 
//! In tasks where the input images have splits, and the output images happens to have the exact same size as one of the split parts.
//! 
//! How does it work:
//! * The input image is splitted into two or more parts.
//! * Transformations is applied to the input parts, starting from simple operations, and ending with more complex operations.
//! * This may yield a formula for output images.
//! 
//! Future experiments:
//! * Return multiple predictions, up to 3 is allowed.
use super::arc_work_model::{Task, Input, PairType};
use super::{ImageLabel, SplitLabel, ImageSplit, ImageSplitDirection, ImageOverlay, ImageHistogram, ColorMap};
use super::{Image, ImageMaskBoolean, Histogram, ImageReplaceColor};
use super::{arcathon_solution_json, arc_json_model};
use super::arc_json_model::GridFromImage;
use super::HtmlLog;
use std::collections::HashMap;
use itertools::Itertools;

#[derive(Debug, Clone, Default)]
pub struct OperationState {
    /// One or more `Operation::Overlay` caused an overlap.
    pub operation_overlay_detected_overlap: bool,
}

#[derive(Debug, Clone)]
pub enum Operation {
    MaskAnd,
    MaskOr,
    MaskXor,
    Overlay { mask_color: u8 },

    // Future experiments
    // KeepColorIfSame { background_color: u8, color_diff0: u8, color_diff1: u8 },
}

impl Operation {
    pub fn execute(&self, image0: &Image, image1: &Image, state: &mut OperationState) -> anyhow::Result<Image> {
        match self {
            Self::MaskAnd => {
                image0.mask_and(image1)
            },
            Self::MaskOr => {
                image0.mask_or(image1)
            },
            Self::MaskXor => {
                image0.mask_xor(image1)
            },
            Self::Overlay { mask_color } => {
                let image_out: Image = image0.overlay_with_mask_color(image1, *mask_color)?;

                // Detect overlap by comparing histograms. 
                // The content is preserved if this histograms are the same. No overlap.
                // Some of the content was overdrawn if the histograms are different. Overlap detected.
                let mut histogram_input: Histogram = image0.histogram_all();
                histogram_input.add_histogram(&image1.histogram_all());
                histogram_input.set_counter_to_zero(*mask_color);

                let mut histogram_output: Histogram = image_out.histogram_all();
                histogram_output.set_counter_to_zero(*mask_color);

                if histogram_input != histogram_output {
                    state.operation_overlay_detected_overlap = true;
                }
                return Ok(image_out);
            },
        }
    }

    pub fn execute_with_images(&self, images: &Vec<Image>) -> anyhow::Result<Image> {
        let mut state = OperationState::default();
        self.execute_with_images_and_state(images, &mut state)
    }

    pub fn execute_with_images_and_state(&self, images: &Vec<Image>, state: &mut OperationState) -> anyhow::Result<Image> {
        let mut work_image = match images.first() {
            Some(value) => value.clone(),
            None => {
                return Err(anyhow::anyhow!("Cannot apply operation to empty list of images"));
            }
        };
        for (image_index, image) in images.iter().enumerate() {
            if image_index == 0 {
                continue;
            }
            work_image = self.execute(&work_image, image, state)?;
        }
        Ok(work_image)
    }

    pub fn execute_with_images_and_permutations(&self, images: &Vec<Image>, permutations: &Vec<&usize>) -> anyhow::Result<Image> {
        let mut state = OperationState::default();
        self.execute_with_images_and_permutations_and_state(images, permutations, &mut state)
    }

    pub fn execute_with_images_and_permutations_and_state(&self, images: &Vec<Image>, permutations: &Vec<&usize>, state: &mut OperationState) -> anyhow::Result<Image> {
        if images.len() != permutations.len() {
            return Err(anyhow::anyhow!("Length of images and permutations must be equal"));
        }
        let first_index: usize = match permutations.first() {
            Some(value) => **value,
            None => {
                return Err(anyhow::anyhow!("permutations is empty"));
            }
        };
        let mut work_image = match images.get(first_index) {
            Some(value) => value.clone(),
            None => {
                return Err(anyhow::anyhow!("first_index is out of bounds"));
            }
        };
        for (loop_index, permutation_index) in permutations.iter().enumerate() {
            if loop_index == 0 {
                continue;
            }
            let image: &Image = match images.get(**permutation_index) {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("permutation_index is out of bounds"));
                }
            };
    
            work_image = self.execute(&work_image, image, state)?;
        }
        Ok(work_image)
    }
}

#[derive(Debug, Clone)]
struct SplitRecord {
    part_count: u8,
    separator_size: u8,
}

impl SplitRecord {
    fn create_record_foreach_pair(task: &Task, is_horizontal_split: bool) -> anyhow::Result<Vec<SplitRecord>> {
        let mut record_vec = Vec::<SplitRecord>::new();
        for pair in &task.pairs {
            let input: &Input = &pair.input;
            let mut found_part_count: Option<u8> = None;
            let mut found_separator_size: Option<u8> = None;
            for image_label in &input.image_meta.image_label_set {
                let split_label: &SplitLabel = match image_label {
                    ImageLabel::Split { label } => label,
                    _ => continue
                };
                if is_horizontal_split {
                    match split_label {
                        SplitLabel::SplitPartCountX { count } => {
                            found_part_count = Some(*count);
                        },
                        SplitLabel::SplitSeparatorSizeX { size } => {
                            found_separator_size = Some(*size);
                        },
                        _ => continue
                    }
                } else {
                    match split_label {
                        SplitLabel::SplitPartCountY { count } => {
                            found_part_count = Some(*count);
                        },
                        SplitLabel::SplitSeparatorSizeY { size } => {
                            found_separator_size = Some(*size);
                        },
                        _ => continue
                    }
                }
            }
            let part_count: u8 = match found_part_count {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Unable to determine how many parts to split into"));
                }
            };
            let separator_size: u8 = match found_separator_size {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Unable to determine the separator size"));
                }
            };
            let record = SplitRecord {
                part_count,
                separator_size,
            };
            record_vec.push(record);
        }
        Ok(record_vec)
    }
}

pub struct SolveSplit {
    verbose: bool,
}

impl SolveSplit {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
        }
    }

    pub fn solve_and_verify(&self, task: &Task, verify_test_pairs: bool) -> anyhow::Result<SolveSplitFoundSolution> {
        let mut solution = self.solve(task)?;
        solution.verify(task, verify_test_pairs);
        Ok(solution)
    }

    /// Can only split into columns or rows, not both.
    fn solve(&self, task: &Task) -> anyhow::Result<SolveSplitFoundSolution> {
        let is_split_x: bool = task.is_output_size_same_as_input_splitview_x();
        let is_split_y: bool = task.is_output_size_same_as_input_splitview_y();
        let is_horizontal_split: bool;
        let split_direction: ImageSplitDirection;
        match (is_split_x, is_split_y) {
            (true, true) => {
                return Err(anyhow::anyhow!("Cannot split both horizontally and vertically"));
            },
            (false, false) => {
                return Err(anyhow::anyhow!("Not a split in this task"));
            },
            (true, false) => {
                is_horizontal_split = true;
                split_direction = ImageSplitDirection::IntoColumns;
            },
            (false, true) => {
                is_horizontal_split = false;
                split_direction = ImageSplitDirection::IntoRows;
            }
        }

        let record_vec: Vec<SplitRecord> = SplitRecord::create_record_foreach_pair(task, is_horizontal_split)?;
        if record_vec.len() != task.pairs.len() {
            return Err(anyhow::anyhow!("task: {} mismatch in number of records and number of pairs", task.id));
        }

        if self.verbose {
            let s: String = format!("task: {} parts: {:?}", task.id, record_vec);
            HtmlLog::text(s);
        }

        let mut pair_splitted_images = Vec::<Vec::<Image>>::new();
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            let record: &SplitRecord = &record_vec[pair_index];
            let part_count: u8 = record.part_count;
            let separator_size: u8 = record.separator_size;

            let input_image: &Image = &pair.input.image;
            let images: Vec<Image> = match input_image.split(part_count, separator_size, split_direction) {
                Ok(value) => value,
                Err(error) => {
                    return Err(anyhow::anyhow!("task: {} Unable to split image: {}", task.id, error));
                }
            };
            // println!("task: {} split into {} parts", task.id, images.len());
            pair_splitted_images.push(images);
        }

        // Is the output always the same as one of the inputs
        // Is the output sometimes the same as one of the inputs
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.pair_type != PairType::Train {
                continue;
            }
            let images: &Vec<Image> = &pair_splitted_images[pair_index];

            // let mut number_of_matches: usize = 0;
            for (image_index, image) in images.iter().enumerate() {
                if *image == pair.output.image {
                    // number_of_matches += 1;
                    if self.verbose {
                        HtmlLog::text(format!("task: {} output is the same as image: {}", task.id, image_index));
                        HtmlLog::image(&image);
                    }
                }
            }
        }

        // The output image have only 2 colors
        let operations = [
            Operation::MaskAnd,
            Operation::MaskOr,
            Operation::MaskXor,
        ];
        let mut simple_candidates = Vec::<SimpleOperationCandidate>::new();
        for operation in &operations {
            // if self.verbose {
            //     HtmlLog::text(&format!("task: {} operation: {:?}", task.id, operation));
            // }
            let mut image_comparison = Vec::<Image>::new();
            for (pair_index, _pair) in task.pairs.iter().enumerate() {
                let images: &Vec<Image> = &pair_splitted_images[pair_index];

                let work_image: Image = match operation.execute_with_images(images) {
                    Ok(value) => value,
                    Err(error) => {
                        debug!("task: {} Unable to execute operation: {}", task.id, error);
                        continue;
                    }
                };

                image_comparison.push(work_image);
            }

            if image_comparison.len() != task.pairs.len() {
                return Err(anyhow::anyhow!("task: {} mismatch in number of images and number of pairs", task.id));
            }

            // The colors of the predicted image rarely have the same colors as the output image.
            // However the histograms may have the same counters, indicating that it maybe is a match, and needs recoloring.
            // Compare histograms
            let mut count_train_ok: u32 = 0;
            let mut count_train_bad: u32 = 0;
            let mut color_map_vec = Vec::<ColorMap>::new();
            for (pair_index, pair) in task.pairs.iter().enumerate() {
                if pair.pair_type != PairType::Train {
                    color_map_vec.push(ColorMap::empty());
                    continue;
                }
                let predicted_output_image: &Image = &image_comparison[pair_index];

                let color_map: ColorMap = ColorMap::analyze(&predicted_output_image, &pair.output.image)?;
                // println!("color_map: {:?} ambiguous: {}", color_map.to_vec(), color_map.is_ambiguous());
                let is_ambiguous: bool = color_map.is_ambiguous();
                color_map_vec.push(color_map);

                if is_ambiguous {
                    count_train_bad += 1;
                } else {
                    count_train_ok += 1;
                }
            }

            // if self.verbose && count_train_ok > 0 {
            //     HtmlLog::text(format!("task: {} operation: {:?} train: {}-{}", task.id, operation, count_train_ok, count_train_bad));
            //     HtmlLog::compare_images(image_comparison.clone());
            // }

            if count_train_ok > 0 {
                let candidate = SimpleOperationCandidate {
                    operation: operation.clone(),
                    predicted_output_images_stage1: image_comparison,
                    predicted_output_images_stage2: vec!(),
                    count_train_histogram_ok: count_train_ok,
                    count_train_histogram_bad: count_train_bad,
                    color_map_vec,
                    count_train_stage2_ok: 0,
                    count_train_stage2_bad: 0,
                };
                simple_candidates.push(candidate);
            }
        }

        // Determine how to recolor the predicted image so it corresponds to the expected output image
        if !simple_candidates.is_empty() {
            for (candidate_index, candidate) in simple_candidates.iter_mut().enumerate() {
                if candidate.color_map_vec.len() != task.pairs.len() {
                    return Err(anyhow::anyhow!("task: {} candidate: {} color_map_vec.len() != task.pairs.len()", task.id, candidate_index));
                }
                if candidate.predicted_output_images_stage1.len() != task.pairs.len() {
                    return Err(anyhow::anyhow!("task: {} candidate: {} predicted_output_images.len() != task.pairs.len()", task.id, candidate_index));
                }
                let mut found_replacements = HashMap::<u8, u8>::new();
                let mut agrees_on_replacements = true;
                for (pair_index, pair) in task.pairs.iter().enumerate() {
                    if pair.pair_type != PairType::Train {
                        continue;
                    }

                    let color_map: &ColorMap = &candidate.color_map_vec[pair_index];
                    if color_map.is_empty() {
                        continue;
                    }

                    // Loop over all the color maps.
                    // Are they the same source -> target, for all pairs.
                    // If so, then we can use that color map.
                    // If not, then it more tricky, and I have no solution for that yet.

                    // let predicted_output_image: &Image = &candidate.predicted_output_images_stage1[pair_index];

                    let mut replacements = HashMap::<u8, u8>::new();
                    for (source_color, target_color, _count) in color_map.to_vec() {
                        replacements.insert(source_color, target_color);
                    }

                    // let recolored_image: Image = predicted_output_image.replace_colors_with_hashmap(&replacements)?;
                    // if self.verbose {
                    //     HtmlLog::image(&recolored_image);
                    // }

                    if found_replacements.is_empty() {
                        found_replacements = replacements;
                    } else {
                        if replacements != found_replacements {
                            agrees_on_replacements = false;
                            break;
                        }
                    }

                    // determine how to recolor
                    // let color_map: ColorMap = ColorMap::analyze(&predicted_output_image, &pair.output.image)?;
                    // println!("color_map: {:?} ambiguous: {}", color_map.to_vec(), color_map.is_ambiguous());

                    // candiates for color
                    // color[N], 
                    // most popular color of input image
                    // least popular color of input image
                    // most popular color of input intersection
                    // least popular color of input intersection
                    // most popular color of output intersection
                    // least popular color of output intersection
                    // insert color
                }

                if agrees_on_replacements && !found_replacements.is_empty() {
                    let mut predicted_output_images_stage2 = Vec::<Image>::new();
                    let mut count_train_ok: u32 = 0;
                    let mut count_train_bad: u32 = 0;
                    for (pair_index, pair) in task.pairs.iter().enumerate() {
                        let predicted_output_image: &Image = &candidate.predicted_output_images_stage1[pair_index];
                        let recolored_image: Image = predicted_output_image.replace_colors_with_hashmap(&found_replacements)?;

                        // Measure number of correct images for the `train` pairs.
                        if pair.pair_type == PairType::Train {
                            let is_correct: bool = pair.output.image == recolored_image;
                            if is_correct {
                                count_train_ok += 1;
                            } else {
                                count_train_bad += 1;
                            }
                        }

                        predicted_output_images_stage2.push(recolored_image);
                    }
                    // if self.verbose {
                    //     HtmlLog::text(format!("task: {} operation: {:?}", task.id, candidate.operation));
                    //     HtmlLog::compare_images(predicted_output_images_stage2.clone());
                    // }
                    candidate.predicted_output_images_stage2 = predicted_output_images_stage2;
                    candidate.count_train_stage2_ok = count_train_ok;
                    candidate.count_train_stage2_bad = count_train_bad;
                }
            }
        }

        let mut best_solution_so_far: Option<SolveSplitFoundSolution> = None;
        if !simple_candidates.is_empty() {
            let mut highest_score: u32 = 0;
            let mut best_candidate_index: usize = 0;
            for (candidate_index, candidate) in simple_candidates.iter().enumerate() {
                if candidate.count_train_histogram_ok > highest_score {
                    highest_score = candidate.count_train_histogram_ok;
                    best_candidate_index = candidate_index;
                }
            }
            let candidate = &simple_candidates[best_candidate_index];

            if self.verbose {
                if !candidate.predicted_output_images_stage2.is_empty() {
                    HtmlLog::text(format!("task: {} operation: {:?} train: {}-{}", task.id, candidate.operation, candidate.count_train_stage2_ok, candidate.count_train_stage2_bad));
                    HtmlLog::compare_images(candidate.predicted_output_images_stage2.clone());
                } else {
                    HtmlLog::text(format!("task: {} operation: {:?} train: {}-{}", task.id, candidate.operation, candidate.count_train_histogram_ok, candidate.count_train_histogram_bad));
                    HtmlLog::compare_images(candidate.predicted_output_images_stage1.clone());
                }
            }

            if !candidate.predicted_output_images_stage2.is_empty() {
                // pretty print recoloring
                let instance = SolveSplitFoundSolution {
                    task_id: task.id.clone(),
                    explanation: format!("{:?}", candidate.operation),
                    predicted_output_images: candidate.predicted_output_images_stage2.clone(),
                    verified_status: None,
                };
                if candidate.count_train_stage2_bad == 0 {
                    return Ok(instance);
                }
                best_solution_so_far = Some(instance);
            }
        }

        let mut shared_part_count: u8 = 0;
        let mut same_part_count_for_all_pairs = true;
        for (record_index, record) in record_vec.iter().enumerate() {
            if record_index == 0 {
                shared_part_count = record.part_count;
                continue;
            }
            if record.part_count != shared_part_count {
                same_part_count_for_all_pairs = false;
                break;
            }
        }

        // Is the input images overlayed on top of each other in a z-order.
        // Overlay images, by permuting the indexes of the images, if count <=5 then it's not too many permutations.
        if same_part_count_for_all_pairs && shared_part_count > 0 && shared_part_count <= 5 {
            // Future experiments:
            // * preserve color
            // * consider background color being transparent
            // Eliminate hard coded background color
            let operation = Operation::Overlay { mask_color: 0 };

            let mut candidate_vec = Vec::<PermutationCandidate>::new();
            let mut abort_permutations = false;
            for (pair_index, pair) in task.pairs.iter().enumerate() {
                if pair.pair_type != PairType::Train {
                    continue;
                }
                let images: &Vec<Image> = &pair_splitted_images[pair_index];
                if images.len() != shared_part_count as usize {
                    return Err(anyhow::anyhow!("task: {} mismatch in number of images and number of parts", task.id));
                }

                // println!("task: {} trying permutations: {}", task.id, shared_part_count);
                let indices: Vec<usize> = (0..shared_part_count as usize).collect();
                let mut count: usize = 0;
                for perm in indices.iter().permutations(shared_part_count as usize) {
                    // println!("{:?}", perm);
                    let mut state = OperationState::default();
                    let image: Image = operation.execute_with_images_and_permutations_and_state(images, &perm, &mut state)?;
                    // detect overlap when overlaying images
                    if state.operation_overlay_detected_overlap {
                        // HtmlLog::text(format!("task: {} permutation: {:?} detected overlap", task.id, perm));
                        // HtmlLog::image(&image);
                        // store it in the candidate
                    }
                    if image == pair.output.image {
                        // HtmlLog::text(format!("task: {} permutation: {:?} same as training output", task.id, perm));
                        // HtmlLog::image(&image);

                        let permutation: Vec<usize> = perm.iter().map(|x| **x).collect();
                        if let Some(candidate) = candidate_vec.iter_mut().find(|x| x.permutation == permutation) {
                            candidate.score += 1;
                        } else {
                            let candidate = PermutationCandidate {
                                permutation,
                                score: 1,
                            };
                            candidate_vec.push(candidate);
                        }
                    }
                    count += 1;
                    if count > 200 {
                        abort_permutations = true;
                        break;
                    }
                }
                if abort_permutations {
                    debug!("task: {} too many permutations. Aborting.", task.id);
                    break;
                }
            }
            // HtmlLog::text(format!("task: {} best permutation candidates: {:?}", task.id, candidate_vec));

            // Find candidate with the highest score
            // if there is a clear winner, then use it
            let mut highest_score: u32 = 0;
            let mut best_candidate_index: usize = 0;
            for (candidate_index, candidate) in candidate_vec.iter().enumerate() {
                if candidate.score > highest_score {
                    highest_score = candidate.score;
                    best_candidate_index = candidate_index;
                }
            }

            if highest_score > 0 {
                if let Some(candidate) = candidate_vec.get(best_candidate_index) {
                    // HtmlLog::text(format!("task: {} best permutation: {:?}", task.id, candidate.permutation));

                    let permutations: Vec<&usize> = candidate.permutation.iter().collect();

                    let mut computed_images = Vec::<Image>::new();
                    for (pair_index, _pair) in task.pairs.iter().enumerate() {
                        let images: &Vec<Image> = &pair_splitted_images[pair_index];
                        if images.len() != shared_part_count as usize {
                            return Err(anyhow::anyhow!("task: {} mismatch in number of images and number of parts", task.id));
                        }

                        let image: Image = operation.execute_with_images_and_permutations(images, &permutations)?;
                        computed_images.push(image);
                    }        
                    // HtmlLog::compare_images(computed_images.clone());

                    let instance = SolveSplitFoundSolution {
                        task_id: task.id.clone(),
                        explanation: format!("overlay {:?}", permutations),
                        predicted_output_images: computed_images,
                        verified_status: None,
                    };
                    return Ok(instance);
                }
            }
        }
        
        if let Some(solution) = best_solution_so_far {
            return Ok(solution);
        }

        Err(anyhow::anyhow!("task: {} no solution found", task.id))
    }
}

#[derive(Debug, Clone)]
struct SimpleOperationCandidate {
    count_train_histogram_ok: u32,
    count_train_histogram_bad: u32,
    operation: Operation,
    predicted_output_images_stage1: Vec<Image>,
    predicted_output_images_stage2: Vec<Image>,
    color_map_vec: Vec<ColorMap>,
    count_train_stage2_ok: u32,
    count_train_stage2_bad: u32,
}

#[derive(Debug, Clone)]
struct PermutationCandidate {
    permutation: Vec<usize>,
    score: u32,
}

#[derive(Debug, Clone)]
pub struct SolveSplitFoundSolution {
    task_id: String,
    explanation: String,
    predicted_output_images: Vec<Image>,
    verified_status: Option<String>,
}

impl SolveSplitFoundSolution {
    fn verify(&mut self, task: &Task, verify_test_pairs: bool) {
        if self.predicted_output_images.len() != task.pairs.len() {
            self.verified_status = Some("predicted_output_images.len() != task.pairs.len()".to_string());
            return;
        }

        let mut count_train_ok: usize = 0;
        let mut count_train_bad: usize = 0;
        let mut count_test_ok: usize = 0;
        let mut count_test_bad: usize = 0;
        let mut count_test_unverified: usize = 0;
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            let predicted_output_image: &Image = &self.predicted_output_images[pair_index];
            match pair.pair_type {
                PairType::Train => {
                    let is_correct: bool = predicted_output_image == &pair.output.image;
                    if is_correct {
                        count_train_ok += 1;
                    } else {
                        count_train_bad += 1;
                    }
                }
                PairType::Test => {
                    if !verify_test_pairs {
                        count_test_unverified += 1;
                        continue;
                    }
                    let is_correct: bool = predicted_output_image == &pair.output.test_image;
                    if is_correct {
                        count_test_ok += 1;
                    } else {
                        count_test_bad += 1;
                    }
                }
            }
        }

        // Format the `train` string. 
        // Example: `train5`, means that all 5 training pairs are correct.
        // Example: `train3(-2)` means that 3 training pairs are correct, but 2 are wrong.
        let status_train: String = if count_train_bad == 0 {
            format!("train{}", count_train_ok)
        } else {
            format!("train{}(-{})", count_train_ok, count_train_bad)
        };

        // Format the `test` string. 
        // Example: `test5unverified`, means that none of the 5 test pairs have been verified. This is when working on the hidden dataset, there is no access to the output.
        // Example: `test5`, means that all 5 test pairs are correct. This is when working on the public dataset, there is access to the output.
        // Example: `test3(-2)` means that 3 test pairs are correct, but 2 are wrong. This is when working on the public dataset, there is access to the output.
        let status_test: String;
        if count_test_unverified > 0 {
            status_test = format!("test{}unverified", count_test_unverified);
        } else {
            if count_test_bad == 0 {
                status_test = format!("test{}", count_test_ok);
           } else {
                status_test = format!("test{}(-{})", count_test_ok, count_test_bad);
           }
        }

        let status: String;
        if count_train_bad == 0 && count_test_bad == 0 {
            status = format!("ok {} {}", status_train, status_test);
        } else {
            status = format!("error {} {}", status_train, status_test);
        }
        self.verified_status = Some(status);
    }

    fn status(&self) -> String {
        if let Some(status) = &self.verified_status {
            return status.clone();
        } else {
            return "Unverified".to_string();
        }
    }

    pub fn testitems_from_test_pairs(&self, task: &Task) -> anyhow::Result<Vec<arcathon_solution_json::TestItem>> {
        if self.predicted_output_images.len() != task.pairs.len() {
            return Err(anyhow::anyhow!("task: {} self.predicted_output_images.len() != task.pairs.len()", task.id));
        }
        let mut testitem_vec = Vec::<arcathon_solution_json::TestItem>::new();
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.pair_type != PairType::Test {
                continue;
            }
            let predicted_output_image: &Image = &self.predicted_output_images[pair_index];

            let grid: arc_json_model::Grid = arc_json_model::Grid::from_image(predicted_output_image);
            let prediction = arcathon_solution_json::Prediction {
                prediction_id: 0,
                output: grid,
            };

            let mut predictions: Vec<arcathon_solution_json::Prediction> = Vec::new();
            predictions.push(prediction);

            let output_id: u8 = testitem_vec.len().min(255) as u8;
            let testitem = arcathon_solution_json::TestItem {
                output_id,
                number_of_predictions: predictions.len().min(255) as u8,
                predictions: predictions,
            };
            testitem_vec.push(testitem);
        }
        if testitem_vec.len() != task.count_test() {
            return Err(anyhow::anyhow!("task: {} testitem_vec.len() != task.count_test()", task.id));
        }
        Ok(testitem_vec)
    }

    /// Pretty print the predicted images to the HTML console.
    pub fn inspect(&self) {
        HtmlLog::text(format!("task: {} status: {} explanation: {}", self.task_id, self.status(), self.explanation));
        HtmlLog::compare_images(self.predicted_output_images.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use crate::arc::arc_json_model;
    use crate::arc::arc_work_model::Task;

    #[test]
    fn test_10000_mask_or() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 1, 1, 1,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(4, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0,
            1, 1, 1, 1,
            0, 0, 0, 0,
        ];
        let input1: Image = Image::try_create(4, 3, pixels1).expect("image");

        let pixels2: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            1, 1, 1, 1,
        ];
        let input2: Image = Image::try_create(4, 3, pixels2).expect("image");

        let images = vec![input0, input1, input2];

        // Act
        let actual: Image = Operation::MaskOr.execute_with_images(&images).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_overlay_without_overlap() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 1, 1, 1,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(4, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0,
            2, 2, 2, 2,
            0, 0, 0, 0,
        ];
        let input1: Image = Image::try_create(4, 3, pixels1).expect("image");

        let pixels2: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            3, 3, 3, 3,
        ];
        let input2: Image = Image::try_create(4, 3, pixels2).expect("image");

        let images = vec![input0, input1, input2];

        let operation = Operation::Overlay { mask_color: 0 };

        // Act
        let mut state = OperationState::default();
        let actual: Image = operation.execute_with_images_and_state(&images, &mut state).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            2, 2, 2, 2,
            3, 3, 3, 3,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
        assert_eq!(state.operation_overlay_detected_overlap, false);
    }

    #[test]
    fn test_20001_overlay_with_overlap() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 1, 1, 1,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(4, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0,
            2, 2, 2, 2,
            0, 0, 0, 0,
        ];
        let input1: Image = Image::try_create(4, 3, pixels1).expect("image");

        let pixels2: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 9, 0,
            3, 3, 3, 3,
        ];
        let input2: Image = Image::try_create(4, 3, pixels2).expect("image");

        let images = vec![input0, input1, input2];

        let operation = Operation::Overlay { mask_color: 0 };

        // Act
        let mut state = OperationState::default();
        let actual: Image = operation.execute_with_images_and_state(&images, &mut state).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            2, 2, 9, 2,
            3, 3, 3, 3,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
        assert_eq!(state.operation_overlay_detected_overlap, true);
    }

    fn solve(name: &str, inspect: bool) -> anyhow::Result<SolveSplitFoundSolution> {
        let json_task: arc_json_model::Task = arc_json_model::Task::load_testdata(name)?;
        let task: Task = Task::try_from(&json_task)?;

        let verify_test_pairs = true;
        let solve_split = SolveSplit::new(inspect);
        let solution: SolveSplitFoundSolution = solve_split.solve_and_verify(&task, verify_test_pairs)?;

        if inspect {
            solution.inspect();
        }

        Ok(solution)
    }

    #[test]
    fn test_90000_overlay_cf98881b() {
        let actual: SolveSplitFoundSolution = solve("cf98881b", false).expect("ok");
        assert_eq!(actual.explanation, "overlay [2, 1, 0]");
        assert_eq!(actual.status(), "ok train5 test1");
    }

    #[test]
    fn test_90001_overlay_281123b4() {
        let actual: SolveSplitFoundSolution = solve("281123b4", false).expect("ok");
        assert_eq!(actual.explanation, "overlay [1, 0, 3, 2]");
        assert_eq!(actual.status(), "ok train6 test1");
    }

    #[test]
    fn test_90002_overlay_e98196ab() {
        let actual: SolveSplitFoundSolution = solve("e98196ab", false).expect("ok");
        assert_eq!(actual.explanation, "overlay [0, 1]");
        assert_eq!(actual.status(), "ok train3 test1");
    }

    #[test]
    fn test_91000_xor_3428a4f5() {
        let actual: SolveSplitFoundSolution = solve("3428a4f5", false).expect("ok");
        assert_eq!(actual.explanation, "MaskXor");
        assert_eq!(actual.status(), "ok train4 test2");
    }

    #[test]
    fn test_92000_or_f2829549() {
        let actual: SolveSplitFoundSolution = solve("f2829549", false).expect("ok");
        assert_eq!(actual.explanation, "MaskOr");
        assert_eq!(actual.status(), "ok train5 test1");
    }

    #[test]
    fn test_93000_and_0520fde7() {
        let actual: SolveSplitFoundSolution = solve("0520fde7", false).expect("ok");
        assert_eq!(actual.explanation, "MaskAnd");
        assert_eq!(actual.status(), "ok train3 test1");
    }
}
