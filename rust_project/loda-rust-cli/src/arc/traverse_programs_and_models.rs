use super::{arc_json_model, ActionLabel};
use super::arc_work_model::{PairType, Task};
use super::{RunWithProgram, RunWithProgramResult};
use super::{Prediction, TestItem, TaskItem, Tasks};
use super::{ImageHistogram, ImageSize, Histogram, ExportTasks};
use super::human_readable_utc_timestamp;
use crate::analytics::{AnalyticsDirectory, Analytics};
use crate::config::Config;
use crate::common::{find_json_files_recursively, parse_csv_file, create_csv_file};
use crate::common::find_asm_files_recursively;
use crate::mine::{Genome, GenomeItem, ToGenomeItemVec, CreateGenomeMutateContextMode, create_genome_mutate_context, GenomeMutateContext};
use bloomfilter::*;
use anyhow::Context;
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::{ProgramSerializer, ProgramId, ProgramRunner};
use loda_rust_core::parser::ParsedProgram;
use std::fmt;
use std::time::{Duration, Instant};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{PathBuf, Path};
use std::rc::Rc;
use std::thread;
use console::Style;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};

#[allow(unused_imports)]
use super::ExperimentWithConvolution;

#[allow(unused_imports)]
#[cfg(feature = "linfa")]
use super::ExperimentWithLogisticRegression;

#[allow(unused_imports)]
use super::{HtmlLog, ImageToHTML, InputLabel, GridLabel};

static SOLUTIONS_FILENAME: &str = "solution_notXORdinary.json";

/// There is a penalty if the ARCathon executable is running longer than 24 hours.
/// Some of the solutions takes minutes to evaluate, so the executable cannot stop instantly. 
/// Thus the limit is several minutes shorter so we are sure that the executable has stopped.
/// Originally I ran for 23h30m. I had to wait an entire day for an answer.
/// 
/// I suspect that most of the discovered solutions happens within the first few minutes.
/// Lets try run for 10 hours. Then I can submit a solution before I go to bed and check status next morning.
/// Three days later. yes. The score is the same. The solutions gets found within the 10 hour time frame.
///
/// Can I lower the time limit even more? Let's try 4 hours.
/// The score is the same. The solutions gets found within the 4 hour time frame.
/// 
/// Can it be lowered even more? Let's try 2 hours.
static ARC_COMPETITION_EXECUTE_DURATION_SECONDS: u64 = (4 * 60) * 60;

static ARC_COMPETITION_INITIAL_RANDOM_SEED: u64 = 4;

static ARC_COMPETITION_IGNORE_PROGRAMS_TAKING_LONGER_THAN_MILLIS: u64 = 200;

pub struct TraverseProgramsAndModels {
    config: Config,
    arc_config: RunArcCompetitionConfig,
    context: GenomeMutateContext,
    model_item_vec: Vec<Rc<RefCell<ModelItem>>>,
    program_item_vec: Vec<Rc<RefCell<ProgramItem>>>,
    locked_instruction_hashset: HashSet<String>,
    dependency_manager: DependencyManager,
}

impl TraverseProgramsAndModels {
    pub fn arc_competition() -> anyhow::Result<()> {
        let mut instance = TraverseProgramsAndModels::new()?;
        instance.run_arc_competition()?;
        Ok(())
    }

    pub fn eval_single_task_with_all_existing_solutions(pattern: String) -> anyhow::Result<()> {
        let instance = TraverseProgramsAndModels::new()?;
        instance.eval_single_task_with_all_existing_solutions_inner(&pattern)?;
        Ok(())
    }

    pub fn experiment_with_convolution() -> anyhow::Result<()> {
        // let tpam = TraverseProgramsAndModels::new()?;
        // let task_vec: Vec<Task> = tpam.to_task_vec();
        // let mut instance = ExperimentWithConvolution::new(task_vec);
        // instance.run()?;
        #[cfg(feature = "linfa")]
        {
            let tpam = TraverseProgramsAndModels::new()?;
            let task_vec: Vec<Task> = tpam.to_task_vec();
            let mut instance = ExperimentWithLogisticRegression::new(task_vec);
            instance.run()?;
            return Ok(());
        }

        #[cfg(not(feature = "linfa"))]
        {
            // return anyhow::bail!("The 'linfa' feature is not enabled");
            anyhow::bail!("The 'linfa' feature is not enabled")
        }
    }

    pub fn export_dataset() -> anyhow::Result<()> {
        let instance = TraverseProgramsAndModels::new()?;
        let task_vec: Vec<Task> = instance.to_task_vec();
        ExportTasks::export(&task_vec)?;
        Ok(())
    }

    fn to_task_vec(&self) -> Vec<Task> {
        let mut task_vec: Vec<Task> = vec!();
        for model_item in &self.model_item_vec {
            let task: Task = model_item.borrow().task.clone();
            task_vec.push(task);
        }
        task_vec
    }

    pub fn check_all_existing_solutions() -> anyhow::Result<()> {
        let instance = TraverseProgramsAndModels::new()?;
        instance.check_all_existing_solutions_inner()?;
        Ok(())
    }

    /// Compare all puzzles with all solutions and output a CSV file
    pub fn generate_solution_csv() -> anyhow::Result<()> {
        let mut instance = TraverseProgramsAndModels::new()?;
        instance.generate_solution_csv_inner()?;
        Ok(())
    }

    fn check_predicted_output_size_for_tasks(task_vec: &Vec<Task>) -> HashSet<String> {
        let verbose = false;

        let mut task_ids_with_correct_prediction = HashSet::<String>::new();

        let mut count_good = 0;
        let mut count_undecided = 0;
        for task in task_vec {
            let estimate: String = task.estimated_output_size();
            if estimate == "Undecided" {
                count_undecided += 1;
                continue;
            }
            count_good += 1;
        }
        if verbose {
            println!("Estimated output size. good: {}  missing: {}", count_good, count_undecided);
        }
        
        // Compute the output size with the test data, and compare with the expected output
        let mut count_predict_pair_correct: usize = 0;
        let mut count_predict_pair_incorrect: usize = 0;
        let mut count_predict_task_correct: usize = 0;
        let mut count_predict_task_incorrect: usize = 0;
        for task in task_vec {
            let estimate: String = task.estimated_output_size();
            if estimate == "Undecided" {
                continue;
            }

            let mut all_correct = true;
            for pair in &task.pairs {
                let predicted: ImageSize = match pair.predicted_output_size() {
                    Some(value) => value,
                    None => {
                        if verbose {
                            println!("No predicted output size. Task: {} pair: {:?}", task.id, pair.pair_type);
                        }
                        count_predict_pair_incorrect += 1;
                        all_correct = false;
                        continue;
                    }
                };

                let expected: ImageSize = match pair.pair_type {
                    PairType::Train => pair.output.image.size(),
                    PairType::Test => pair.output.test_image.size(),
                };

                if predicted == expected {
                    count_predict_pair_correct += 1;
                } else {
                    if verbose {
                        println!("Wrong output size. Expected {:?}, but got {:?}. Task: {} pair: {:?}", expected, predicted, task.id, pair.pair_type);
                    }
                    count_predict_pair_incorrect += 1;
                    all_correct = false;
                }
            }
            if all_correct {
                count_predict_task_correct += 1;
                task_ids_with_correct_prediction.insert(task.id.clone());
            } else {
                // Self::inspect_task(buffer_task)?;
                count_predict_task_incorrect += 1;
            }

            // If all the pairs had their output size predicted correctly,
            // then save the predicted output sizes on the pair instances.
            // If one or more pairs were incorrectly predicted, 
            // then don't save the predicted output size on the pair instances.
        }
        if verbose {
            println!("count_predict_pair_correct: {}", count_predict_pair_correct);
            println!("count_predict_pair_incorrect: {}", count_predict_pair_incorrect);
            println!("count_predict_task_correct: {}", count_predict_task_correct);
            println!("count_predict_task_incorrect: {}", count_predict_task_incorrect);
        }
        {
            let percent: usize = (100 * count_predict_pair_correct) / (count_predict_pair_correct + count_predict_pair_incorrect).max(1);
            if verbose {
                println!("Predicted single-image: correct: {} incorrect: {} correct-percent: {}%", count_predict_pair_correct, count_predict_pair_incorrect, percent);
            }
        }
        {
            let percent: usize = (100 * count_predict_task_correct) / (count_predict_task_correct + count_predict_task_incorrect).max(1);
            if verbose {
                println!("Predicted task: correct: {} incorrect: {} correct-percent: {}%", count_predict_task_correct, count_predict_task_incorrect, percent);
            }
        }
        {
            let number_of_tasks: usize = task_vec.len();
            let percent: usize = (100 * count_predict_task_correct) / number_of_tasks.max(1);
            println!("Summary: Output size prediction. There are {} correct tasks of {} all tasks. Percent: {}%", count_predict_task_correct, number_of_tasks, percent);
        }

        task_ids_with_correct_prediction
    }

    fn check_predicted_output_palette_for_tasks(task_vec: &Vec<Task>) -> HashSet<String> {
        let verbose = false;

        let mut task_ids_with_correct_prediction = HashSet::<String>::new();

        let mut count_predict_pair_correct: usize = 0;
        let mut count_predict_pair_incorrect: usize = 0;
        let mut count_predict_task_correct: usize = 0;
        let mut count_predict_task_incorrect: usize = 0;
        for task in task_vec {

            let mut all_correct = true;
            for pair in &task.pairs {
                let predicted: Histogram = match pair.predicted_output_palette() {
                    Some(value) => value,
                    None => {
                        if verbose {
                            println!("No predicted output palette. Task: {} pair: {:?}", task.id, pair.pair_type);
                        }
                        all_correct = false;
                        count_predict_pair_incorrect += 1;
                        continue;
                    }
                };

                let expected_histogram: Histogram = match pair.pair_type {
                    PairType::Train => pair.output.image.histogram_all(),
                    PairType::Test => pair.output.test_image.histogram_all(),
                };
                let expected_count: u16 = expected_histogram.number_of_counters_greater_than_zero();

                let mut histogram: Histogram = predicted.clone();
                histogram.intersection_histogram(&expected_histogram);
                let predicted_count: u16 = histogram.number_of_counters_greater_than_zero();
                if expected_count == predicted_count {
                    count_predict_pair_correct += 1;
                } else  {
                    count_predict_pair_incorrect += 1;
                    all_correct = false;
                }
            }
            if all_correct {
                count_predict_task_correct += 1;
                task_ids_with_correct_prediction.insert(task.id.clone());
            } else {
                count_predict_task_incorrect += 1;
                if verbose {
                    println!("incorrect prediction. {:?}", task.id);
                }
            }
        }
        if verbose {
            println!("count_predict_pair_correct: {}", count_predict_pair_correct);
            println!("count_predict_pair_incorrect: {}", count_predict_pair_incorrect);
            println!("count_predict_task_correct: {}", count_predict_task_correct);
            println!("count_predict_task_incorrect: {}", count_predict_task_incorrect);
        }
        {
            let number_of_tasks: usize = task_vec.len();
            let percent: usize = (100 * count_predict_task_correct) / number_of_tasks.max(1);
            println!("Summary: Output palette prediction. There are {} correct tasks of {} all tasks. Percent: {}%", count_predict_task_correct, number_of_tasks, percent);
        }
        task_ids_with_correct_prediction
    }

    /// Traverse all puzzles and classify each puzzle.
    pub fn label_all_puzzles() -> anyhow::Result<()> {
        let instance = TraverseProgramsAndModels::new()?;

        let mut task_vec: Vec<Task> = vec!();
        for model_item in &instance.model_item_vec {
            let task: Task = model_item.borrow().task.clone();
            task_vec.push(task);
        }

        let task_ids_with_correct_output_size: HashSet<String> = 
            Self::check_predicted_output_size_for_tasks(&task_vec);

        let task_ids_with_correct_output_palette: HashSet<String> = 
            Self::check_predicted_output_palette_for_tasks(&task_vec);

        let mut task_ids_intersection = HashSet::<String>::new();
        for task_id in task_ids_with_correct_output_size.intersection(&task_ids_with_correct_output_palette) {
            task_ids_intersection.insert(task_id.clone());
        }
        {
            let number_of_tasks: usize = task_vec.len();
            let percent: usize = (100 * task_ids_intersection.len()) / number_of_tasks.max(1);
            println!("tasks with size=ok  and palette=ok.   {}  Percent: {}%", task_ids_intersection.len(), percent);
        }

        let mut task_ids_only_size = HashSet::<String>::new();
        for task_id in task_ids_with_correct_output_size.difference(&task_ids_intersection) {
            task_ids_only_size.insert(task_id.clone());
        }
        println!("tasks with size=ok  and palette=bad.  {}", task_ids_only_size.len());

        let mut task_ids_only_palette = HashSet::<String>::new();
        for task_id in task_ids_with_correct_output_palette.difference(&task_ids_intersection) {
            task_ids_only_palette.insert(task_id.clone());
        }
        println!("tasks with size=bad and palette=ok.   {}", task_ids_only_palette.len());

        let mut count_tasks_without_predictions: usize = 0;
        for task in &task_vec {
            if task_ids_with_correct_output_size.contains(&task.id) {
                continue;
            }
            if task_ids_with_correct_output_palette.contains(&task.id) {
                continue;
            }
            count_tasks_without_predictions += 1;
        }
        println!("tasks with size=bad and palette=bad.  {}", count_tasks_without_predictions);

        // Self::inspect_task_id(&task_vec, "83302e8f")?;
        // Self::inspect_task_id(&task_vec, "95a58926")?;
        // Self::inspect_task_id(&task_vec, "e906de3d")?;
        // Self::inspect_task_id(&task_vec, "7837ac64")?;
        // Self::inspect_task_id(&task_vec, "8ee02e8f")?;
        // Self::inspect_task_id(&task_vec, "92e50de0")?;
        // Self::inspect_task_id(&task_vec, "c3202e5a")?;

        // repair and crop
        // Self::inspect_task_id(&task_vec, "0934a4d8")?;

        // advanced
        // Self::inspect_task_id(&task_vec, "3631a71a")?;
        // Self::inspect_task_id(&task_vec, "f9d67f8b")?;

        // // simple
        // Self::inspect_task_id(&task_vec, "dc0a314f")?;
        // Self::inspect_task_id(&task_vec, "9ecd008a")?;
        // Self::inspect_task_id(&task_vec, "b8825c91")?;
        // Self::inspect_task_id(&task_vec, "ff805c23")?;

        // Self::inspect_task_id(&task_vec, "332efdb3")?;
        // Self::inspect_task_id(&task_vec, "17cae0c1")?;
        // Self::inspect_task_id(&task_vec, "929ab4e9")?;
        // Self::inspect_task_id(&task_vec, "9ecd008a")?;
        // Self::inspect_task_id(&task_vec, "de493100")?;
        // Self::inspect_task_id(&task_vec, "af22c60d")?;
        // Self::inspect_task_id(&task_vec, "f15e1fac")?;
        // Self::inspect_task_id(&task_vec, "f9012d9b")?;
        // Self::inspect_task_id(&task_vec, "1b60fb0c")?;
        // Self::inspect_task_id(&task_vec, "67a423a3")?;
        // Self::inspect_task_id(&task_vec, "f9012d9b")?;
        // Self::inspect_task_id(&task_vec, "d2abd087")?;
        // Self::inspect_task_id(&task_vec, "b190f7f5")?;
        // Self::inspect_task_id(&task_vec, "ae4f1146")?;
        // Self::inspect_task_id(&task_vec, "6e82a1ae")?;
        // Self::inspect_task_id(&task_vec, "be94b721")?;
        // Self::inspect_task_id(&task_vec, "6e82a1ae")?;
        // Self::inspect_task_id(&task_vec, "776ffc46")?;
        // Self::inspect_task_id(&task_vec, "ddf7fa4f")?;
        // Self::inspect_task_id(&task_vec, "c3f564a4")?;
        // Self::inspect_task_id(&task_vec, "36d67576")?;
        // Self::inspect_task_id(&task_vec, "aedd82e4")?;
        // Self::inspect_task_id(&task_vec, "4c5c2cf0")?;
        // Self::inspect_task_id(&task_vec, "5c0a986e")?;
        Self::inspect_tasks_without_solution(&task_vec)?;
        // Self::inspect_undecided(&task_vec)?;
        // Self::inspect_decided(&task_vec)?;
        // Self::inspect_task_id(&task_vec, "72ca375d")?;
        // Self::inspect_task_id(&task_vec, "d56f2372")?;
        // Self::inspect_task_id(&task_vec, "a85d4709")?;
        // Self::inspect_task_id(&task_vec, "29ec7d0e")?;
        // Self::inspect_task_id(&task_vec, "ea959feb")?;
        // Self::inspect_tasks_with_single_repair_color(&task_vec)?;
        // Self::inspect_tasks_with_output_image_color(&task_vec)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn inspect_tasks_without_solution(task_vec: &Vec<Task>) -> anyhow::Result<()> {
        let mut indexes = HashSet::<usize>::new();
        for (index, task) in task_vec.iter().enumerate() {
            if task.occur_in_solutions_csv {
                continue;
            }
            if task.is_output_size_same_as_input_size() {
                continue;
            }
            // if task.input_histogram_union.number_of_counters_greater_than_zero() > 3 {
            //     continue;
            // }
            // if task.input_histogram_intersection.most_popular_color_disallow_ambiguous() == None {
            //     continue;
            // }
            let mut found: bool = false;
            found = true;
            // if task.has_removal_color() {
            //     found = true;
            // }
            // for action_label in &task.action_label_set_intersection {
            //     match action_label {
            //         ActionLabel::OutputSizeIsTheSameAsSingleColorObject { label } => {
            //             println!("output_size: {:?}", label);
            //             found = true;
            //         },
            //         _ => {}
            //     }
            // }
            // for action_label in &task.action_label_set_intersection {
            //     match action_label {
            //         ActionLabel::OutputImagePreserveInputImageEdge { edge: _ } => {
            //             found = true;
            //         },
            //         _ => {}
            //     }
            // }
            // for action_label in &task.action_label_set_intersection {
            //     match action_label {
            //         ActionLabel::OutputImagePreserveInputImageCorner { corner: _ } => {
            //             found = true;
            //         },
            //         _ => {}
            //     }
            // }
            // for action_label in &task.action_label_set_intersection {
            //     match action_label {
            //         ActionLabel::InputImageOccurInsideOutputImageSameNumberOfTimesAsColor { color: _ } => {
            //             found = true;
            //         },
            //         ActionLabel::InputImageOccurInsideOutputImageSameNumberOfTimesAsTheMostPopularColorOfInputImage => {
            //             found = true;
            //         },
            //         ActionLabel::InputImageOccurInsideOutputImageSameNumberOfTimesAsTheLeastPopularColorOfInputImage => {
            //             found = true;
            //         },
            //         _ => {}
            //     }
            // }
            // if let Some(count) = task.input_properties_intersection.get(&PropertyInput::InputUniqueColorCount) {
            //     if *count == 2 {
            //         found = true;
            //     }
            // }
            // if task.has_repaired_image() {
            //     found = true;
            // }
            // if task.has_predicted_single_color_image() {
            //     found = true;
            // }
            // if task.is_output_size_same_as_input_size() {
            //     found = true;
            // }
            // for input_label in &task.input_label_set_intersection {
            //     match input_label {
            //         InputLabel::InputUnambiguousConnectivityWithAllColors => {
            //             found = true;
            //         },
            //         _ => {}
            //     }
            // }
            // for input_label in &task.input_label_set_intersection {
            //     match input_label {
            //         InputLabel::InputNoiseWithColor { color: _ } => {
            //             found = true;
            //         },
            //         InputLabel::InputNoiseWithSomeColor => {
            //             found = true;
            //         },
            //         _ => {}
            //     }
            // }
            // for input_label in &task.input_label_set_intersection {
            //     let grid_label: GridLabel = match input_label {
            //         InputLabel::InputGrid { label } => label.clone(),
            //         _ => continue
            //     };
            //     match grid_label {
            //         GridLabel::GridColor { color: _ } => {
            //             // found = true;
            //         },
            //         GridLabel::GridWithSomeColor => {
            //             // found = true;
            //         },
            //         GridLabel::GridWithMismatchesAndColor { color: _ } => {
            //             // found = true;
            //         },
            //         GridLabel::GridWithMismatchesAndSomeColor => {
            //             // found = true;
            //         },
            //     }
            // }
            // if task.has_enumerated_objects() {
            //     found = true;
            // }
            // if task.has_substitution_rule_applied() {
            //     found = true;
            // }
            // if task.has_grid_pattern() {
                // found = true;
                // found = false;
            // }
            // if task.input_label_set_intersection.contains(&InputLabel::InputImageIsSymmetricYWithInset) {
            //     found = true;
            // }
            // if task.input_label_set_intersection.contains(&InputLabel::InputImageIsSymmetricXWithMismatches) && task.input_label_set_intersection.contains(&InputLabel::InputImageIsSymmetricYWithMismatches) {
            //     found = true;
            // }
            // if task.input_label_set_intersection.contains(&InputLabel::InputImageIsSymmetricXWithInset) || task.input_label_set_intersection.contains(&InputLabel::InputImageIsSymmetricYWithInset) {
            //     found = true;
            // }
            // if task.is_output_size_same_as_removed_rectangle_after_single_color_removal() {
            //     found = true;
            // }
            // if task.is_output_size_same_as_primary_object_after_single_color_removal() {
            //     found = true;
            // }
            // if task.action_label_set_intersection.contains(&ActionLabel::OutputImageHasSameStructureAsInputImage) {
            //     found = true;
            // }
            if found {
                indexes.insert(index);
            }
        }

        let mut task_ids = Vec::<String>::new();
        for (index, task) in task_vec.iter().enumerate() {
            if !indexes.contains(&index) {
                continue;
            }
            task_ids.push(task.id.clone());
        }
        task_ids.sort();

        let mut count = 0;
        for (index, task) in task_vec.iter().enumerate() {
            if !indexes.contains(&index) {
                continue;
            }
            if count > 0 {
                task.inspect()?;
            }
            count += 1;
            if count > 50 {
                break;
            }
        }
        HtmlLog::text(format!("tasks count: {}", indexes.len()));
        HtmlLog::text(format!("task ids: {:?}", task_ids));
        Ok(())
    }

    #[allow(dead_code)]
    fn inspect_tasks_with_single_repair_color(task_vec: &Vec<Task>) -> anyhow::Result<()> {
        let mut indexes = HashSet::<usize>::new();
        for (index, task) in task_vec.iter().enumerate() {
            let mut found = false;
            for label in &task.action_label_set_intersection {
                match label {
                    ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { .. } => {
                        found = true;
                        break;
                    },
                    _ => {}
                };
            }
            if found {
                continue;
            }
            indexes.insert(index);
        }
        let mut count = 0;
        for (index, task) in task_vec.iter().enumerate() {
            if !indexes.contains(&index) {
                continue;
            }
            if count > 0 {
                task.inspect()?;
            }
            count += 1;
            if count > 50 {
                break;
            }
        }
        HtmlLog::text(format!("tasks count: {}", indexes.len()));
        Ok(())
    }

    #[allow(dead_code)]
    fn inspect_tasks_with_output_image_color(task_vec: &Vec<Task>) -> anyhow::Result<()> {
        let mut indexes = HashSet::<usize>::new();
        for (index, task) in task_vec.iter().enumerate() {
            let mut found = false;
            for label in &task.action_label_set_intersection {
                match label {
                    ActionLabel::OutputImageUniqueColorCount { .. } => {
                        found = true;
                        break;
                    },
                    ActionLabel::OutputImageColorsComesFromInputImage => {
                        found = true;
                        break;
                    },
                    _ => {}
                };
            }
            if !found {
                continue;
            }
            indexes.insert(index);
        }
        let mut count = 0;
        for (index, task) in task_vec.iter().enumerate() {
            if !indexes.contains(&index) {
                continue;
            }
            if count > 0 {
                task.inspect()?;
            }
            count += 1;
            if count > 50 {
                break;
            }
        }
        HtmlLog::text(format!("tasks count: {}", indexes.len()));
        Ok(())
    }

    #[allow(dead_code)]
    fn inspect_undecided(task_vec: &Vec<Task>) -> anyhow::Result<()> {
        let mut count = 0;
        for task in task_vec {
            let estimate: String = task.estimated_output_size();
            if estimate != "Undecided" {
                continue;
            }
            if count > 0 {
                task.inspect()?;
            }
            count += 1;
            if count > 50 {
                break;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn inspect_decided(task_vec: &Vec<Task>) -> anyhow::Result<()> {
        let mut count = 0;
        for task in task_vec {
            let estimate: String = task.estimated_output_size();
            if estimate == "Undecided" {
                continue;
            }
            if count > 0 {
                task.inspect()?;
            }
            count += 1;
            if count > 50 {
                break;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn inspect_task_id(task_vec: &Vec<Task>, task_id: &str) -> anyhow::Result<()> {
        for task in task_vec {
            if task.id == task_id {
                task.inspect()?;
                break;
            }
        }
        Ok(())
    }

    fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let arc_config = RunArcCompetitionConfig::new(&config);
        let dependency_manager: DependencyManager = RunWithProgram::create_dependency_manager();

        let mut instance = Self { 
            config,
            arc_config,
            context: GenomeMutateContext::default(),
            model_item_vec: vec!(),
            program_item_vec: vec!(),
            locked_instruction_hashset: HashSet::new(),
            dependency_manager,
        };
        instance.load_puzzle_files()?;
        instance.load_solution_files()?;
        instance.init_locked_instruction_hashset()?;
        match instance.update_task_occur_in_solutions_csv() {
            Ok(()) => {},
            Err(error) => {
                println!("Couldn't update models with solution status. error: {:?}", error);
            }
        }
        Ok(instance)
    }

    fn files_to_keep(path: &PathBuf) -> bool {
        if let Some(filename) = path.file_name() {
            if filename.to_string_lossy() == SOLUTIONS_FILENAME {
                debug!("ignoring the SOLUTIONS_FILENAME. path: {:?}", path);
                return false;
            }
        }
        true
    }

    /// Load all the ARC puzzle files into memory
    fn load_puzzle_files(&mut self) -> anyhow::Result<()> {
        let repo_path: PathBuf = self.config.arc_repository_data();
        let all_json_paths: Vec<PathBuf> = find_json_files_recursively(&repo_path);

        // Ignore the solutions json file, since it's not an ARC puzzle json file
        let paths: Vec<PathBuf> = all_json_paths
            .into_iter()
            .filter(Self::files_to_keep)
            .collect();
        debug!("arc_repository_data. number of json files: {}", paths.len());

        let mut model_item_vec: Vec<Rc<RefCell<ModelItem>>> = vec!();
        for path in &paths {
            let json_task: arc_json_model::Task = match arc_json_model::Task::load_with_json_file(path) {
                Ok(value) => value,
                Err(error) => {
                    error!("Ignoring file. Cannot parse arc_json_model file. path: {:?} error: {:?}", path, error);
                    continue;
                }
            };
            let task: Task = match Task::try_from(&json_task) {
                Ok(value) => value,
                Err(error) => {
                    error!("Ignoring file. Cannot construct arc_work_model::Task from json model. path: {:?} error: {:?}", path, error);
                    continue;
                }
            };
            let instance = ModelItem {
                id: ModelItemId::Path { path: path.clone() },
                task,
            };
            let item = Rc::new(RefCell::new(instance));
            model_item_vec.push(item);
        }
        if model_item_vec.len() != paths.len() {
            error!("Skipped some models. paths.len()={}, but model_item_vec.len()={}", paths.len(), model_item_vec.len());
        }
        self.model_item_vec = model_item_vec;
        Ok(())
    }

    /// Load all `.asm` programs into memory
    fn load_solution_files(&mut self) -> anyhow::Result<()> {
        let path: PathBuf = self.config.loda_arc_challenge_repository_programs();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&path);
        debug!("loda_arc_challenge_repository_programs. number of asm files: {}", paths.len());

        let mut program_item_vec: Vec<Rc<RefCell<ProgramItem>>> = vec!();
        for path in &paths {

            let program_string: String = match fs::read_to_string(path) {
                Ok(value) => value,
                Err(error) => {
                    error!("cannot read the file: {:?} error: {:?}", path, error);
                    continue;
                }
            };

            let is_simple: bool = program_string.contains("Program Type: simple");
            let is_advanced: bool = program_string.contains("Program Type: advanced");
            let program_type: ProgramType;
            match (is_simple, is_advanced) {
                (false, false) => {
                    error!("Cannot find 'Program Type: simple' nor 'Program Type: advanced'. Skipping program. path: {:?}", path);
                    continue;
                },
                (false, true) => {
                    program_type = ProgramType::Advance;
                },
                (true, false) => {
                    program_type = ProgramType::Simple;
                },
                (true, true) => {
                    error!("Ambiguous use of 'Program Type'. Should be either 'Program Type: simple' or 'Program Type: advanced'. Skipping program. path: {:?}", path);
                    continue;
                }
            }

            let program_content: String;
            match program_type {
                ProgramType::Simple => {
                    program_content = RunWithProgram::convert_simple_to_full(&program_string);
                },
                ProgramType::Advance => {
                    program_content = program_string.clone();
                }
            }
            let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&program_content) {
                Ok(value) => value,
                Err(error) => {
                    error!("cannot parse the program. Skipping program. path: {:?} error: {:?}", path, error);
                    continue;
                }
            };

            let program_runner: ProgramRunner = match self.dependency_manager.parse_stage2(ProgramId::ProgramWithoutId, &parsed_program) {
                Ok(value) => value,
                Err(error) => {
                    error!("cannot create ProgramRunner. Skipping program. path: {:?} error: {:?}", path, error);
                    continue;
                }
            };

            let instance = ProgramItem {
                id: ProgramItemId::Path { path: path.clone() },
                program_string,
                program_type,
                parsed_program,
                program_runner,
                ignore_due_to_slowness: false,
            };
            let item = Rc::new(RefCell::new(instance));
            program_item_vec.push(item);
        }
        if program_item_vec.len() != paths.len() {
            error!("Skipped some programs. paths.len()={}, but program_item_vec.len()={}", paths.len(), program_item_vec.len());
        }
        self.program_item_vec = program_item_vec;
        Ok(())
    }

    const INSTRUCTIONS_TO_LOCK: &'static str = r#"
    mov $80,$97 ; set iteration counter = length of "train" vector
    mov $81,100 ; address of first training data train[0].input
    mov $82,101 ; address of first training data train[0].output
    lps $80
      mov $0,$$81 ; load train[x].input image
      mov $1,$$82 ; load train[x].output image
    
      ; do stuff
      
      ; next iteration
      add $81,10 ; jump to address of next training input image
      add $82,10 ; jump to address of next training output image
    lpe
    "#;

    fn init_locked_instruction_hashset(&mut self) -> anyhow::Result<()> {
        self.insert_program_into_locked_instruction_hashset(RunWithProgram::SIMPLE_PROGRAM_PRE)?;
        self.insert_program_into_locked_instruction_hashset(RunWithProgram::SIMPLE_PROGRAM_POST)?;
        self.insert_program_into_locked_instruction_hashset(Self::INSTRUCTIONS_TO_LOCK)?;
        Ok(())
    }

    fn insert_program_into_locked_instruction_hashset<S: AsRef<str>>(&mut self, program: S) -> anyhow::Result<()> {
        let program_str: &str = program.as_ref();
        let parsed_program: ParsedProgram = ParsedProgram::parse_program(program_str)
            .map_err(|e| anyhow::anyhow!("parse with program: {:?}. error: {:?}", program_str, e))?;
        for instruction in &parsed_program.instruction_vec {
            let s: String = instruction.to_string();
            self.locked_instruction_hashset.insert(s);
        }
        Ok(())
    }

    /// Create mutations of a single program.
    /// 
    /// The `bloom` parameter, helps ensure that the mutated programs are different than previously tried out programs.
    /// 
    /// Returns a vector with length `number_of_programs_to_generate`.
    fn create_mutations_of_program(
        &mut self, 
        program_item: RcProgramItem, 
        mutation_index: u64,
        number_of_programs_to_generate: usize, 
        bloom: &mut Bloom::<String>
    ) -> anyhow::Result<RcProgramItemVec> {
        let mut genome = Genome::new();
        genome.append_message(format!("template: {:?}", program_item.borrow().id.file_name()));

        let mut genome_vec: Vec<GenomeItem> = program_item.borrow().parsed_program.to_genome_item_vec();

        // locking rows that are not to be mutated
        for genome_item in genome_vec.iter_mut() {
            let program_line: String = genome_item.to_line_string();
            if self.locked_instruction_hashset.contains(&program_line) {
                genome_item.set_mutation_locked(true);
            }
        }

        genome.set_genome_vec(genome_vec);
        
        let mut result_program_item_vec: RcProgramItemVec = RcProgramItemVec::with_capacity(number_of_programs_to_generate);

        let max_number_of_iterations = 100;
        for iteration in 0..max_number_of_iterations {

            // Notes about random seed.
            //
            // Originally the random generator was initialized once before entering the loop.
            // The initial random seed was based on datetime.
            // It was non-deterministic, and would yield different results.
            // When new files got added to the solutions repo, then the random seed would change.
            //
            // Lesson learned: Reproducibility is highly valuable. 
            // Reproduce the same results under the same circumstances, makes it possible to compare algorithms.
            // In order to make the code deterministic:
            // The random seed is unaffected of how many files there are. When a new file gets added, it's still the same random_seed.
            // The random generator is reinitialized for every iteration.
            // The random seed is unaffected by how many threads are running in parallel.
            // However there are still several non-deterministic things that may affect the outcome,
            // Such as the analytics file on disk, how are the rows arranged in the csv file. Bloomfilter.
            // Such as the way the Genome::mutate() picks a mutation strategy.
            let random_seed: u64 = mutation_index * 0x10000 + iteration + ARC_COMPETITION_INITIAL_RANDOM_SEED;
            let mut rng: StdRng = StdRng::seed_from_u64(random_seed);

            let mutate_success: bool = genome.mutate(&mut rng, &self.context);
            if !mutate_success {
                continue;
            }

            let parsed_program: ParsedProgram = genome.to_parsed_program();
            let bloom_key: String = parsed_program.to_string();
            if bloom.check(&bloom_key) {
                // It's likely that this program mutation has already has been explored in the past. Ignore it.
                // debug!("skip program mutation that already have been tried out");
                continue;                
            }

            // This program mutation is not contained in the bloomfilter.

            // This ensures that we don't try out this mutation again.
            bloom.set(&bloom_key);
            
            // Proceed making a program out of it.
            let program_runner: ProgramRunner = match self.dependency_manager.parse_stage2(ProgramId::ProgramWithoutId, &parsed_program) {
                Ok(value) => value,
                Err(error) => {
                    error!("ignoring program mutation. parse_stage2 with program: {:?}. error: {:?}", genome.to_string(), error);
                    continue;
                }
            };
    
            // println!("program: {:?} random_seed: {:#x}", program_item.borrow().id.file_name(), random_seed);
            let mut serializer = ProgramSerializer::new();
            serializer.append_comment("Submitted by Simon Strandgaard");
            serializer.append_comment("Program Type: advanced");
            serializer.append_empty_line();
            program_runner.serialize(&mut serializer);
            serializer.append_empty_line();
            for message in genome.message_vec() {
                serializer.append_comment(message);
            }
            serializer.append_empty_line();
            let candidate_program: String = serializer.to_string();
            // println!("; ------\n\n{}", candidate_program);

            let mutated_program_item = ProgramItem {
                id: ProgramItemId::None,
                program_string: candidate_program,
                program_type: ProgramType::Advance,
                parsed_program,
                program_runner,
                ignore_due_to_slowness: false,
            };
            result_program_item_vec.push(Rc::new(RefCell::new(mutated_program_item)));
            if result_program_item_vec.len() >= number_of_programs_to_generate {
                return Ok(result_program_item_vec);
            }
        }
        if result_program_item_vec.is_empty() {
            return Err(anyhow::anyhow!("unable to mutate in {} attempts, {:?}", max_number_of_iterations, program_item.borrow().id.file_name()));
        }
        Ok(result_program_item_vec)
    }

    /// Create mutations of all the existing programs.
    /// 
    /// The `bloom` parameter, helps ensure that the mutated programs are different than previously tried out programs.
    /// 
    /// Returns a vector with length `number_of_programs_to_generate` x number of available programs.
    fn create_mutations_of_all_programs(
        &mut self,
        mutation_index: u64, 
        number_of_programs_to_generate: usize, 
        bloom: &mut Bloom::<String>
    ) -> RcProgramItemVec {
        let mut result_program_item_vec: RcProgramItemVec = RcProgramItemVec::new();
        for program_item in self.program_item_vec.clone() {
            match self.create_mutations_of_program(program_item, mutation_index, number_of_programs_to_generate, bloom) {
                Ok(mut mutated_programs) => {
                    result_program_item_vec.append(&mut mutated_programs);
                },
                Err(error) => {
                    debug!("Skipping mutation. {:?}", error);
                }
            }
        }
        result_program_item_vec
    }

    fn read_solutions_json(&self) -> anyhow::Result<Tasks> {
        let path: &Path = &self.arc_config.path_solution_teamid_json;
        let solution_teamid_json_string: String = match fs::read_to_string(path) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("something went wrong reading the file: {:?} error: {:?}", path, error));
            }
        };
        let tasks: Tasks = match serde_json::from_str(&solution_teamid_json_string) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Could not parse archaton_solution_json file, path: {:?} error: {:?} json: {:?}", path, error, solution_teamid_json_string));
            }
        };
        Ok(tasks)
    }

    fn eval_single_task_with_all_existing_solutions_inner(&self, pattern: &String) -> anyhow::Result<()> {
        let verbose = false;
        let verify_test_output = true;

        // Extract the puzzle model
        let mut candidate_model_items = Vec::<ModelItem>::new();
        for model_item in &self.model_item_vec {
            let file_stem: String = model_item.borrow().id.file_stem();
            if file_stem.contains(pattern) {
                candidate_model_items.push(model_item.borrow().clone());
            }
        }
        // There is supposed to be exactly 1 puzzle with this name.
        if candidate_model_items.len() >= 2 {
            return Err(anyhow::anyhow!("There are {} puzzles that matches the pattern, please specify a longer pattern: {:?}", candidate_model_items.len(), pattern));
        }
        let model_item: ModelItem = match candidate_model_items.pop() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("No puzzle matches the specified pattern: {:?}", pattern));
            }
        };

        let count_train: usize = model_item.task.count_train();
        let count_test: usize = model_item.task.count_test();
        println!("Evaluating the puzzle: {:?} train-pairs: {} test-pairs: {}", model_item.id, count_train, count_test);

        let mut count_ok: usize = 0;
        let mut count_error_compute: usize = 0;
        let mut count_error_incorrect: usize = 0;
        let mut count_partial_match: usize = 0;
        let mut count_dangerous_false_positive: usize = 0;

        let pb = ProgressBar::new(self.program_item_vec.len() as u64);
        pb.tick();
        for (program_index, program_item) in self.program_item_vec.iter().enumerate() {
            if program_index > 0 {
                pb.inc(1);
            }

            let instance = RunWithProgram::new(model_item.task.clone(), verify_test_output);

            let result: RunWithProgramResult;
            match program_item.borrow().program_type {
                ProgramType::Simple => {
                    result = match instance.run_simple(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            if verbose {
                                pb.println(format!("ERROR: in row {}. program: {:?}. Run failed with error {:?}", program_index, program_item, error));
                            }
                            continue;
                        }
                    };
                },
                ProgramType::Advance => {
                    result = match instance.run_advanced(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            if verbose {
                                pb.println(format!("ERROR: in row {}. program: {:?}. Run failed with error {:?}", program_index, program_item, error));
                            }
                            continue;
                        }
                    };
                }
            }

            if verbose {
                let s = format!("model: {:?} program: {:?} result: {:?}", model_item.id, program_item.borrow().id, result);
                pb.println(s);
            }

            if !result.all_train_pairs_and_test_pairs_are_correct() {
                let expected = format!("({},{})", count_train, count_test);
                let actual = format!("({},{})", result.count_train_correct(), result.count_test_correct());

                if result.all_train_pairs_are_correct() && !result.all_test_pairs_are_correct() {
                    pb.println(format!("Dangerous false positive. Expected {} but got {}. {:?}", expected, actual, program_item.borrow().id.file_name()));
                    count_dangerous_false_positive += 1;
                } else {
                    let count_correct = result.count_train_correct() + result.count_test_correct();
                    if count_correct > 0 {
                        count_partial_match += 1;
                        pb.println(format!("Partial solution. Expected {} but got {}. {:?}", expected, actual, program_item.borrow().id.file_name()));
                    }
                }
                if verbose {
                    pb.println(format!("ERROR: in row {}. program: {:?}. Expected {}, but got {}", program_index, program_item, expected, actual));
                }
                count_error_incorrect += 1;
                continue;
            }

            count_ok += 1;
            pb.println(format!("Solution: {:?}", program_item.borrow().id.file_name()));
        }
        pb.finish_and_clear();

        debug!("STATS:");
        debug!("count_partial_match: {}", count_partial_match);
        debug!("count_error_compute: {}", count_error_compute);
        debug!("count_error_incorrect: {}", count_error_incorrect);
        if count_dangerous_false_positive > 0 {
            error!("Encountered {} dangerous false positive solutions. These are unwanted.", count_dangerous_false_positive);
        }

        if count_ok > 0 {
            let green_bold = Style::new().green().bold();        
            let s = format!("Status: Found {} solutions", count_ok);
            println!("{}", green_bold.apply_to(&s));
        } else {
            let green_bold = Style::new().red().bold();        
            println!("{}", green_bold.apply_to("Status: Found no solutions among the existing programs"));
        }
        Ok(())
    }

    fn update_task_occur_in_solutions_csv(&self) -> anyhow::Result<()> {
        let path_solutions_csv = self.config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));
        if !path_solutions_csv.is_file() {
            return Err(anyhow::anyhow!("update_task_occur_in_solutions_csv: there is no existing solutions.csv file, so the solutions cannot be checked. path_solutions_csv: {:?}", path_solutions_csv));
        }

        let record_vec: Vec<Record> = Record::load_record_vec(&path_solutions_csv)?;
        debug!("update_task_occur_in_solutions_csv: solutions.csv: number of rows: {}", record_vec.len());

        let mut task_id_set = HashSet::<String>::new();
        for record in &record_vec {
            let filename_with_json_suffix: String = record.model_filename.clone();
            let task_id = filename_with_json_suffix.replace(".json", "");
            task_id_set.insert(task_id);
        }

        for model_item in &self.model_item_vec {
            let mut model_item_mut = model_item.borrow_mut();
            let has_solution: bool = task_id_set.contains(&model_item_mut.task.id);
            model_item_mut.task.occur_in_solutions_csv = has_solution;
        }

        let mut count_has_solution_true: usize = 0;
        for model_item in &self.model_item_vec {
            let has_solution: bool = model_item.borrow().task.occur_in_solutions_csv;
            if has_solution {
                count_has_solution_true += 1;
            }
        }
        debug!("update_task_occur_in_solutions_csv: tasks with one or more solutions: {}", count_has_solution_true);
        Ok(())
    }

    fn check_all_existing_solutions_inner(&self) -> anyhow::Result<()> {
        let verbose = false;
        let verify_test_output = true;

        let path_solutions_csv = self.config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));
        if !path_solutions_csv.is_file() {
            return Err(anyhow::anyhow!("there is no existing solutions.csv file, so the solutions cannot be checked. path_solutions_csv: {:?}", path_solutions_csv));
        }

        let record_vec: Vec<Record> = Record::load_record_vec(&path_solutions_csv)?;
        debug!("solutions.csv: number of rows: {}", record_vec.len());

        let mut count_ok: usize = 0;
        let mut count_error_other: usize = 0;
        let mut count_error_duplicate: usize = 0;
        let mut count_error_compute: usize = 0;
        let mut count_error_incorrect: usize = 0;

        let mut unique_records = HashSet::<Record>::new();

        let pb = ProgressBar::new(record_vec.len() as u64);
        for (record_index, record) in record_vec.iter().enumerate() {
            if record_index > 0 {
                pb.inc(1);
            }

            // The rows are supposed to be unique
            if unique_records.contains(&record) {
                pb.println(format!("ERROR: in row {}. Expected unique rows, but this is a duplicate.", record_index));
                count_error_duplicate += 1;
                continue;
            }
            unique_records.insert(record.clone());

            // Extract the puzzle model
            let mut candidate_model_items = Vec::<ModelItem>::new();
            for model_item in &self.model_item_vec {
                let file_name: String = model_item.borrow().id.file_name();
                if file_name == record.model_filename {
                    candidate_model_items.push(model_item.borrow().clone());
                }
            }
            // There is supposed to be exactly 1 puzzle with this name.
            if candidate_model_items.len() >= 2 {
                pb.println(format!("ERROR: in row {}. Expected 1 puzzle for row in csv file, but got multiple.", record_index));
                count_error_other += 1;
                continue;
            }
            let model_item: ModelItem = match candidate_model_items.pop() {
                Some(value) => value,
                None => {
                    pb.println(format!("ERROR: in row {}. Missing puzzle.", record_index));
                    count_error_other += 1;
                    continue;
                }
            };

            // Extract the solution model
            let mut candidate_programs = Vec::<Rc::<RefCell::<ProgramItem>>>::new();
            let program_filename: String = record.program_filename.clone();
            for program_item in &self.program_item_vec {
                let this_file_name: String = program_item.borrow_mut().id.file_name();
                if this_file_name == program_filename {
                    candidate_programs.push(program_item.clone());
                }
            }
            // There is supposed to be exactly 1 solution with this name.
            if candidate_programs.len() >= 2 {
                pb.println(format!("ERROR: in row {}. Expected 1 solution for row in csv file, but got multiple.", record_index));
                count_error_other += 1;
                continue;
            }
            let program_item: Rc<RefCell<ProgramItem>> = match candidate_programs.pop() {
                Some(value) => value,
                None => {
                    pb.println(format!("ERROR: in row {}. record: {:?}. Missing solution.", record_index, record));
                    count_error_other += 1;
                    continue;
                }
            };
    
            let instance: RunWithProgram = RunWithProgram::new(model_item.task.clone(), verify_test_output);
            let count_train: usize = model_item.task.count_train();
            let count_test: usize = model_item.task.count_test();

            let result: RunWithProgramResult;
            match program_item.borrow().program_type {
                ProgramType::Simple => {
                    result = match instance.run_simple(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            pb.println(format!("ERROR: in row {}. record: {:?}. Run failed with error {:?}", record_index, record, error));
                            continue;
                        }
                    };
                },
                ProgramType::Advance => {
                    result = match instance.run_advanced(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            pb.println(format!("ERROR: in row {}. record: {:?}. Run failed with error {:?}", record_index, record, error));
                            continue;
                        }
                    };
                }
            }

            if verbose {
                let s = format!("model: {:?} program: {:?} result: {:?}", model_item.id, program_item.borrow().id, result);
                pb.println(s);
            }

            if !result.all_train_pairs_and_test_pairs_are_correct() {
                let expected = format!("({},{})", count_train, count_test);
                let actual = format!("({},{})", result.count_train_correct(), result.count_test_correct());
                    pb.println(format!("ERROR: in row {}. record: {:?}. Expected {}, but got {}", record_index, record, expected, actual));
                count_error_incorrect += 1;
                continue;
            }

            count_ok += 1;
        }
        pb.finish_and_clear();

        if count_ok == record_vec.len() {
            let green_bold = Style::new().green().bold();        
            println!("{}", green_bold.apply_to("Status: All solutions passes ok"));
        } else {
            println!("count_ok: {}", count_ok);
            println!("count_error_other: {}", count_error_other);
            println!("count_error_duplicate: {}", count_error_duplicate);
            println!("count_error_compute: {}", count_error_compute);
            println!("count_error_incorrect: {}", count_error_incorrect);
            let sum: usize = count_error_other + count_error_duplicate + count_error_compute + count_error_incorrect;
            error!("There are {} errors that needs to be resolved. csv file: {:?}", sum, path_solutions_csv);
        }
        Ok(())
    }

    fn generate_solution_csv_inner(&mut self) -> anyhow::Result<()> {
        let verbose = false;
        let verify_test_output = true;

        let path_solutions_csv = self.config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));
        
        let mut unique_records = HashSet::<Record>::new();
        Record::save_solutions_csv(&unique_records, &path_solutions_csv);
        
        let start = Instant::now();
        
        let mut visited_program_paths = HashSet::<PathBuf>::new();
        let mut count_ok: usize = 0;
        let mut count_dangerous_false_positive: usize = 0;
        let mut count_partial_match: usize = 0;
        let mut count_incorrect: usize = 0;
        let mut count_compute_error: usize = 0;

        let multi_progress = MultiProgress::new();
        let progress_style: ProgressStyle = ProgressStyle::with_template(
            "{prefix} [{elapsed_precise}] {wide_bar} {pos:>5}/{len:5} {msg}",
        )?;

        let pb = multi_progress.add(ProgressBar::new(self.model_item_vec.len() as u64));
        pb.set_style(progress_style.clone());
        pb.set_prefix("Task    ");
        pb.tick();

        for (model_index, model_item) in self.model_item_vec.iter_mut().enumerate() {
            if model_index > 0 {
                pb.inc(1);
            }

            let print_prefix_task_id: String = format!("Task {:?}", model_item.borrow().id.file_stem());

            let task: Task = model_item.borrow().task.clone();
            let count_train: usize = task.count_train();
            let count_test: usize = task.count_test();
            let instance: RunWithProgram = RunWithProgram::new(task.clone(), verify_test_output);
    
            let pb2 = multi_progress.insert_after(&pb, ProgressBar::new( self.program_item_vec.len() as u64));
            pb2.set_style(progress_style.clone());
            pb2.set_prefix("Solution");
            pb2.tick();
            for (program_index, program_item) in self.program_item_vec.iter_mut().enumerate() {
                if program_index > 0 {
                    pb2.inc(1);
                }

                let result: RunWithProgramResult;
                match program_item.borrow().program_type {
                    ProgramType::Simple => {
                        result = match instance.run_simple(&program_item.borrow().program_string) {
                            Ok(value) => value,
                            Err(error) => {
                                count_compute_error += 1;
                                if verbose {
                                    error!("model: {:?} simple-program: {:?} error: {:?}", task.id, program_item.borrow().id, error);
                                }
                                continue;
                            }
                        };
                    },
                    ProgramType::Advance => {
                        result = match instance.run_advanced(&program_item.borrow().program_string) {
                            Ok(value) => value,
                            Err(error) => {
                                count_compute_error += 1;
                                if verbose {
                                    error!("model: {:?} advanced-program: {:?} error: {:?}", task.id, program_item.borrow().id, error);
                                }
                                continue;
                            }
                        };
                    }
                }

                let program_id: ProgramItemId = program_item.borrow().id.clone();

                if verbose {
                    let s = format!("model: {:?} program: {:?} result: {:?}", task.id, program_id, result);
                    pb.println(s);
                }

                if !result.all_train_pairs_and_test_pairs_are_correct() {
                    let expected = format!("({},{})", count_train, count_test);
                    let actual = format!("({},{})", result.count_train_correct(), result.count_test_correct());
                    if result.all_train_pairs_are_correct() && !result.all_test_pairs_are_correct() {
                        pb.println(format!("{} - {} - Dangerous false positive. Expected {} but got {}. Solution {:?}", human_readable_utc_timestamp(), print_prefix_task_id, expected, actual, program_id.file_name()));
                        count_dangerous_false_positive += 1;
                        continue;
                    }
                    let count_correct = result.count_train_correct() + result.count_test_correct();
                    if count_correct > 0 {
                        count_partial_match += 1;
                        pb.println(format!("{} - {} - Partial solution. Expected {} but got {}. Solution {:?}", human_readable_utc_timestamp(), print_prefix_task_id, expected, actual, program_id.file_name()));
                        continue;
                    }
                    if verbose {
                        pb.println(format!("{} - ERROR: in row {}. program: {:?}. Expected {}, but got {}", human_readable_utc_timestamp(), program_index, program_item, expected, actual));
                    }
                    count_incorrect += 1;
                    continue;
                }
    
                pb.println(format!("{} - {} - Solution: {:?}", human_readable_utc_timestamp(), print_prefix_task_id, program_id.file_name()));
                count_ok += 1;
                match program_id {
                    ProgramItemId::Path { path } => {
                        visited_program_paths.insert(path.clone());
                    },
                    ProgramItemId::None => {
                        pb.println(format!("{} - Encountered a solution without a path.", print_prefix_task_id));
                    }
                }

                let model_filename: String = model_item.borrow().id.file_name();
                let program_filename: String = program_item.borrow().id.file_name();
                let record = Record {
                    model_filename: model_filename,
                    program_filename,
                };
                unique_records.insert(record);
                Record::save_solutions_csv(&unique_records, &path_solutions_csv);
            }

            pb2.finish_and_clear();
        }
        pb.finish_and_clear();
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} compared all tasks with all solutions in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        // Print out names of unused programs that serves no purpose and can be removed
        let mut unused_programs = Vec::<String>::new();
        for program_item in &self.program_item_vec {
            let program_id: ProgramItemId = program_item.borrow().id.clone();
            let path: PathBuf = match program_id {
                ProgramItemId::Path { ref path } => path.clone(),
                ProgramItemId::None => {
                    continue;
                }
            };
            if !visited_program_paths.contains(&path) {
                unused_programs.push(program_id.file_name());
            }
        }
        if !unused_programs.is_empty() {
            error!("There are {} unused programs. These doesn't solve any of the models, and can be removed.", unused_programs.len());
            for filename in unused_programs {
                println!("UNUSED {:?}", filename);
            }
        }
    
        // Stats
        println!("row count in solutions csv file: {}", unique_records.len());
        println!("count_ok: {}", count_ok);
        println!("count_incorrect: {}", count_incorrect);
        println!("count_compute_error: {}", count_compute_error);
        println!("count_partial_match: {}", count_partial_match);
        println!("count_dangerous_false_positive: {}", count_dangerous_false_positive);
        Ok(())
    }

    /// Eliminate duplicates in the program_item_vec
    fn dedup_program_item_vec(&mut self) {
        let count_before: usize = self.program_item_vec.len();
        let mut uniques = HashSet::<ProgramItemId>::new();
        self.program_item_vec.retain(|program_item| {
            let program_id: ProgramItemId = program_item.borrow().id.clone();
            uniques.insert(program_id)
        });
        let count_after: usize = self.program_item_vec.len();
        if count_before != count_after {
            println!("Removed duplicates from program_item_vec. count_before: {} count_after: {}", count_before, count_after);
        } else {
            println!("Great, no duplicates found");
        }
    }

    fn reload_analytics_dir(&mut self) -> anyhow::Result<()> {
        println!("loading genome mutate context");
        let start = Instant::now();

        Analytics::arc_run_force()?;

        let analytics_directory = AnalyticsDirectory::new(
            self.arc_config.path_analytics_arc_dir.clone()
        ).with_context(||"unable to create AnalyticsDirectory instance")?;    

        let context: GenomeMutateContext = create_genome_mutate_context(CreateGenomeMutateContextMode::ARC, analytics_directory)?;
        self.context = context;
        println!("loaded genome mutate context. elapsed: {}", HumanDuration(start.elapsed()));
        Ok(())
    }

    /// Print out lots of useful info.
    /// 
    /// I have tried submitting a docker image built with the wrong architecture. I don't want to repeat that.
    fn print_system_info() {
        println!("env::consts::ARCH: {}", std::env::consts::ARCH);
        println!("env::consts::OS: {}", std::env::consts::OS);
        println!("thread::current(): {:?}", thread::current());

        const VERSION: &str = env!("CARGO_PKG_VERSION");
        let build_mode: &str;
        if cfg!(debug_assertions) {
            build_mode = "DEBUG (terrible performance!)";
        } else {
            build_mode = "RELEASE";
        }
        println!("LODA-RUST version: {}, build: {}", VERSION, build_mode);
    }

    fn run_arc_competition(&mut self) -> anyhow::Result<()> {
        let execute_start_time: Instant = Instant::now();
        let execute_time_limit: Duration = Duration::from_secs(ARC_COMPETITION_EXECUTE_DURATION_SECONDS);

        // When participating in the contest, then we want first to try out the existing solutions.
        // This may be a solution to one of the hidden puzzles.
        // However it's slow, so it's disabled while developing, where we only want to explore mutations.
        let try_existing_solutions = true;
        let try_logistic_regression = false;

        let number_of_programs_to_generate: usize = 3;

        println!("{} - Start of program", human_readable_utc_timestamp());
        Self::print_system_info();

        println!("initial random seed: {}", ARC_COMPETITION_INITIAL_RANDOM_SEED);
        println!("ignore programs taking longer than millis: {}", ARC_COMPETITION_IGNORE_PROGRAMS_TAKING_LONGER_THAN_MILLIS);

        println!("initial number of solutions: {}", self.program_item_vec.len());
        println!("initial number of tasks: {}", self.model_item_vec.len());

        self.dedup_program_item_vec();
        self.reload_analytics_dir()?;

        let mut scheduled_model_item_vec: Vec<Rc<RefCell<ModelItem>>> = self.model_item_vec.clone();

        let initial_tasks: Tasks = match self.read_solutions_json() {
            Ok(value) => value,
            Err(error) => {
                error!("Starting out with zero tasks. Unable to load existing solutions file: {:?}", error);
                vec!()
            }
        };
        println!("initial_tasks.len: {}", initial_tasks.len());

        let mut puzzle_names_to_ignore = HashSet::<String>::new();
        for task in &initial_tasks {
            puzzle_names_to_ignore.insert(task.task_name.clone());
        }

        let mut unique_records = HashSet::<Record>::new();

        let ignore_puzzles_with_a_solution: bool = self.arc_config.path_solutions_csv.is_file();
        if ignore_puzzles_with_a_solution {
            let record_vec = Record::load_record_vec(&self.arc_config.path_solutions_csv)?;
            debug!("solutions.csv: number of rows: {}", record_vec.len());
    
            for record in &record_vec {
                unique_records.insert(record.clone());
            }

            for record in &record_vec {
                let puzzle_filename_with_json_suffix: String = record.model_filename.clone();
                let puzzle_filename = puzzle_filename_with_json_suffix.replace(".json", "");
                puzzle_names_to_ignore.insert(puzzle_filename);
            }
        }
        debug!("puzzle_names_to_ignore: {:?}", puzzle_names_to_ignore);

        scheduled_model_item_vec = ModelItem::remove_model_items_where_filestem_contains(
            &scheduled_model_item_vec, 
            &puzzle_names_to_ignore
        );

        // println!("scheduled_model_item_vec.len(): {}", scheduled_model_item_vec.len());

        // Summary of what puzzles are to be solved
        {
            let mut number_of_solved_puzzles: usize = 0;
            let mut number_of_unsolved_puzzles: usize = 0;
            for model_item in &self.model_item_vec {
                let mut is_same = false;
                for model_item2 in &scheduled_model_item_vec {
                    if Rc::ptr_eq(&model_item, &model_item2) {
                        is_same = true;
                        break;
                    }
                }
                if is_same {
                    number_of_unsolved_puzzles += 1;
                } else {
                    number_of_solved_puzzles += 1;
                }
            }
            println!("puzzles solved: {}", number_of_solved_puzzles);
            println!("puzzles unsolved: {}", number_of_unsolved_puzzles);
        }

        let current_tasks: Tasks = initial_tasks;
        save_solutions_json(
            &self.arc_config.path_solution_dir,
            &self.arc_config.path_solution_teamid_json,
            &current_tasks
        );

        let bloom_items_count = 1000000;
        let false_positive_rate = 0.01;
        let mut bloom = Bloom::<String>::new_for_fp_rate(bloom_items_count, false_positive_rate);

        // Register the existing programs in the bloomfilter, so that these never gets suggested as a candidate solution
        for program_item in &self.program_item_vec {
            match program_item.borrow().bloom_key() {
                Ok(bloom_key) => {
                    bloom.set(&bloom_key);
                },
                Err(error) => {
                    error!("unable to create bloom_key for program: {:?}", error);
                }
            }
        }

        let plan = BatchPlan {
            execute_start_time,
            execute_time_limit,
            scheduled_model_item_vec,
            scheduled_program_item_vec: self.program_item_vec.clone(),
        };
        
        let mut state = BatchState {
            remove_model_items: vec!(),
            discovered_program_item_vec: vec!(),
            unique_records,
            current_tasks,
            terminate_due_to_timeout: false,
        };

        let mut runner = BatchRunner {
            config: self.arc_config.clone(),
            plan,
        };

        if try_existing_solutions {
            println!("{} - Run existing solutions without mutations", human_readable_utc_timestamp());
            runner.run_one_batch(&mut state)?;
            self.transfer_discovered_programs(&mut state)?;
        }

        if try_logistic_regression {
            #[cfg(feature = "linfa")]
            {
                let number_of_tasks: u64 = runner.plan.scheduled_model_item_vec.len() as u64;
                println!("{} - Run logistic regression with {} tasks", human_readable_utc_timestamp(), number_of_tasks);
                let pb = ProgressBar::new(number_of_tasks as u64);
                let verbose_logistic_regression = false;
                let verify_test_output = false;
                for model_item in &runner.plan.scheduled_model_item_vec {
                    let task: Task = model_item.borrow().task.clone();
                    
                    let predictions: Vec<Prediction> = match ExperimentWithLogisticRegression::process_task(&task, verify_test_output) {
                        Ok(value) => value,
                        Err(error) => {
                            if verbose_logistic_regression {
                                println!("task: {} - could not make predictions. error: {:?}", task.id, error);
                            }
                            pb.inc(1);
                            continue;
                        }
                    };
                    if verbose_logistic_regression {
                        println!("task: {} - predictions.len(): {}", task.id, predictions.len());
                    }
    
                    let model_id: ModelItemId = model_item.borrow().id.clone(); 
    
                    let test_item = TestItem { 
                        output_id: 0,
                        number_of_predictions: predictions.len() as u8,
                        predictions: predictions,
                    };
                    let task_name: String = model_id.file_stem();
                    let task_item = TaskItem {
                        task_name: task_name,
                        test_vec: vec![test_item],
                    };
                    // TODO: don't add if already exists
                    state.current_tasks.push(task_item);        
                    pb.inc(1);
                }
                pb.finish_and_clear();
                save_solutions_json(
                    &self.arc_config.path_solution_dir,
                    &self.arc_config.path_solution_teamid_json,
                    &state.current_tasks
                );
                println!("{} - Executable elapsed: {}.", human_readable_utc_timestamp(), HumanDuration(execute_start_time.elapsed()));
    
                println!("Done!");
                return Ok(());
            }

            #[cfg(not(feature = "linfa"))]
            {
                error!("{} - Logistic regression is not enabled. Please enable the 'linfa' feature.", human_readable_utc_timestamp());
            }
        }

        // loop until all puzzles have been solved
        let mut mutation_index: u64 = 0;
        loop {
            if runner.plan.scheduled_model_item_vec.is_empty() {
                println!("{} - It seems all the puzzles have been solved.", human_readable_utc_timestamp());
                break;
            }
            if state.terminate_due_to_timeout {
                println!("{} - Terminating due to timeout.", human_readable_utc_timestamp());
                break;
            }
            println!("{} - Mutation: {}", human_readable_utc_timestamp(), mutation_index);

            // Create new mutated programs in every iteration
            runner.plan.scheduled_program_item_vec = self.create_mutations_of_all_programs(
                mutation_index, 
                number_of_programs_to_generate, 
                &mut bloom
            );

            // Evaluate all puzzles with all candidate programs
            runner.run_one_batch(&mut state)?;
            self.transfer_discovered_programs(&mut state)?;
            
            mutation_index += 1;
        }
        println!("{} - Executable elapsed: {}.", human_readable_utc_timestamp(), HumanDuration(execute_start_time.elapsed()));

        println!("Done!");
        Ok(())
    }

    /// Move discovered programs to the original programs vector
    fn transfer_discovered_programs(&mut self, state: &mut BatchState) -> anyhow::Result<()> {
        if state.discovered_program_item_vec.is_empty() {
            return Ok(());
        }
        println!("transferred {:?} solutions", state.discovered_program_item_vec.len());

        self.program_item_vec.append(&mut state.discovered_program_item_vec);
        if !state.discovered_program_item_vec.is_empty() {
            error!("Expected state.discovered_program_item_vec to be empty after moving the elements");
        }

        // When a program solves multiple puzzles, 
        // then the program gets appended multiple times. 
        // This eliminates the duplicates.
        self.dedup_program_item_vec();

        // Regenerate analytics when new programs have been mined
        self.reload_analytics_dir()?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct RunArcCompetitionConfig {
    path_analytics_arc_dir: PathBuf,
    path_solutions_csv: PathBuf,
    path_programs: PathBuf,
    path_solution_dir: PathBuf,
    path_solution_teamid_json: PathBuf,
}

impl RunArcCompetitionConfig {
    fn new(config: &Config) -> Self {
        let path_solutions_csv = config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));

        let path_solution_dir: PathBuf = config.arc_repository_data().join(Path::new("solution"));
        let path_solution_teamid_json: PathBuf = path_solution_dir.join(Path::new(SOLUTIONS_FILENAME));

        RunArcCompetitionConfig {
            path_analytics_arc_dir: config.analytics_arc_dir(),
            path_solutions_csv,
            path_programs: config.loda_arc_challenge_repository_programs(),
            path_solution_dir,
            path_solution_teamid_json,
        }
    }
}

#[derive(Debug)]
struct BatchPlan {
    execute_start_time: Instant,
    execute_time_limit: Duration,
    scheduled_model_item_vec: Vec<Rc<RefCell<ModelItem>>>,
    scheduled_program_item_vec: Vec<Rc<RefCell<ProgramItem>>>,
}

impl BatchPlan {
    /// Outer loop traverses the unsolved puzzles.
    /// 
    /// Inner loop traverses the candidate solutions.
    fn run_one_batch(
        &self, 
        config: &RunArcCompetitionConfig,
        state: &mut BatchState,
    ) -> anyhow::Result<()> {
        let verify_test_output = false;
        let verbose = false;
        let max_duration_seconds: u64 = 60;

        let mut start_time = Instant::now();
        let mut slowest_program_elapsed = Duration::ZERO;
        let mut slowest_program_name = String::new();

        let multi_progress = MultiProgress::new();
        let progress_style: ProgressStyle = ProgressStyle::with_template(
            "{prefix} [{elapsed_precise}] {wide_bar} {pos:>5}/{len:5} {msg}",
        )?;

        let pb = multi_progress.add(ProgressBar::new(self.scheduled_model_item_vec.len() as u64));
        pb.set_style(progress_style.clone());
        pb.set_prefix("Unsolved puzzle   ");
        for (model_index, model_item) in self.scheduled_model_item_vec.iter().enumerate() {
            if model_index > 0 {
                pb.inc(1);
            }
    
            let task: Task = model_item.borrow().task.clone();
            if verbose {
                let count_train: usize = task.count_train();
                let count_test: usize = task.count_test();
                pb.println(format!("puzzle: {} train: {} test: {}", task.id, count_train, count_test));
            }

            let instance: RunWithProgram = RunWithProgram::new(task, verify_test_output);

            let pb2 = multi_progress.insert_after(&pb, ProgressBar::new(self.scheduled_program_item_vec.len() as u64));
            pb2.set_style(progress_style.clone());
            pb2.set_prefix("Candidate solution");
            for program_index in 0..self.scheduled_program_item_vec.len() {
                if program_index > 0 {
                    pb.tick();
                    pb2.inc(1);
                }

                if self.execute_start_time.elapsed() >= self.execute_time_limit {
                    state.terminate_due_to_timeout = true;
                    let message = format!(
                        "{} - Exceeded time limit for executable", 
                        human_readable_utc_timestamp(), 
                    );
                    pb.println(message);
                    pb2.finish_and_clear();
                    pb.finish_and_clear();
                    return Ok(());
                }

                let elapsed: Duration = start_time.elapsed();
                if elapsed.as_secs() >= max_duration_seconds {
                    let total_number_of_solutions: usize = state.current_tasks.len();
                    let message = format!(
                        "{} - Status.  Total number of solutions: {}  Slowest program: {:?} {}", 
                        human_readable_utc_timestamp(), 
                        total_number_of_solutions,
                        slowest_program_name,
                        HumanDuration(slowest_program_elapsed)
                    );
                    pb.println(message);
                    start_time = Instant::now();
                    slowest_program_elapsed = Duration::ZERO;
                    slowest_program_name = String::new();
                }
    
                let program_item: &Rc<RefCell<ProgramItem>> = &self.scheduled_program_item_vec[program_index];
                {
                    if program_item.borrow().ignore_due_to_slowness {
                        if verbose {
                            pb.println("skip slow program");
                        }
                        continue;
                    }
                }
                
                let before_run_program = Instant::now();
                let run_program_runner_result = {
                    let program_runner: &ProgramRunner = &program_item.borrow().program_runner;
                    instance.run_program_runner(program_runner)
                };
                
                {
                    let program_run_elapsed: Duration = before_run_program.elapsed();
                    if program_run_elapsed > slowest_program_elapsed {
                        slowest_program_elapsed = program_run_elapsed;
                        slowest_program_name = program_item.borrow().id.file_name();
                    }

                    if program_run_elapsed > Duration::from_millis(ARC_COMPETITION_IGNORE_PROGRAMS_TAKING_LONGER_THAN_MILLIS) {
                        let s = format!("Ignoring slow program. Elapsed: {}", HumanDuration(program_run_elapsed));
                        pb.println(s);
                        program_item.borrow_mut().ignore_due_to_slowness = true;
                        continue;
                    }
                }
    
                let run_with_program_result: RunWithProgramResult;
                {
                    run_with_program_result = match run_program_runner_result {
                        Ok(value) => value,
                        Err(error) => {
                            if verbose {
                                error!("run_program_runner model: {:?} program: {:?} error: {:?}", model_item.borrow().id, program_item.borrow().id, error);
                            }
                            continue;
                        }
                    };
                    if verbose {
                        let s = format!("model: {:?} program: {:?} result: {:?}", model_item.borrow().id, program_item.borrow().id, run_with_program_result);
                        pb.println(s);
                    }
                }

                if !run_with_program_result.all_train_pairs_are_correct() {
                    // Something is not satisfied about the training pairs.
                    // Usually it's one or more of of the training pairs that doesn't match the expected output.
                    // This is not a solution. Proceed to the next candidate solution.
                    // pb.println(format!("Task {:?}, the training pairs is not correct. Ignoring.", model_item.borrow().id));
                    continue;
                }

                let count_test_empty: usize = run_with_program_result.count_test_empty();
                if count_test_empty > 0 {
                    // No task in ARC outputs an empty image.
                    // Thus all the "test" output images must be non-empty. 
                    // This is not a solution. Proceed to the next candidate solution.
                    pb.println(format!("{} - Task {:?}, ignoring test images that are empty.", human_readable_utc_timestamp(), model_item.borrow().id));
                    continue;
                }

                // All the train pairs are correct.
                // The test pairs are unverified, and have a size of 1x1 or bigger.
                // This may be a solution.
                pb.println(format!("{} - Task {:?}, possible solution", human_readable_utc_timestamp(), model_item.borrow().id));

                let save_result = state.save_solution(
                    config, 
                    Rc::clone(model_item), 
                    Rc::clone(program_item), 
                    run_with_program_result, 
                    &pb
                );

                match save_result {
                    Ok(()) => {
                        // This is a solution to this puzzle. No need to loop through the remaining programs.
                        break;
                    },
                    Err(error) => {
                        error!("Unable to save solution. model: {:?} error: {:?}", model_item.borrow().id, error);
                        // Something went wrong saving this solution. Consider this puzzle as still being unsolved.
                        // Loop through the remaining programs to check for another solution.
                        continue;
                    }
                }
            }
            pb2.finish_and_clear();
        }
        pb.finish_and_clear();

        Ok(())
    }

    fn reschedule(&mut self, state: &mut BatchState) -> anyhow::Result<()> {
        if state.remove_model_items.is_empty() {
            return Ok(());
        }
        
        // Remove solved puzzles from the scheduled_model_item_vec
        self.scheduled_model_item_vec = ModelItem::remove_model_items(
            &self.scheduled_model_item_vec, 
            &state.remove_model_items
        );
        state.remove_model_items.clear();

        Ok(())
    }
}

struct BatchState {
    remove_model_items: Vec<Rc<RefCell<ModelItem>>>,
    discovered_program_item_vec: Vec<Rc<RefCell<ProgramItem>>>,
    unique_records: HashSet::<Record>,
    current_tasks: Tasks,
    terminate_due_to_timeout: bool,
}

impl BatchState {
    fn save_solution(
        &mut self, 
        config: &RunArcCompetitionConfig, 
        model_item: Rc<RefCell<ModelItem>>, 
        program_item: Rc<RefCell<ProgramItem>>,
        run_with_program_result: RunWithProgramResult,
        progress_bar: &ProgressBar,
    ) -> anyhow::Result<()> {
        let model_id: ModelItemId = model_item.borrow().id.clone(); 

        // Save the program to disk.
        //
        // Don't save the program when it already exist in the file system.
        // On launch of the miner, then first try out all the existing programs with the puzzles. This may yield a match.
        // In which case we don't want to save the already existing program to disk.
        let is_new_program: bool = program_item.borrow().id == ProgramItemId::None;
        if is_new_program {
            let name: String = model_id.file_stem();
            let program_filename: String = match ProgramItem::unique_name_for_saving(&config.path_programs, &name) {
                Ok(filename) => filename,
                Err(error) => {
                    return Err(anyhow::anyhow!("cannot save file, because of error: {:?}", error));
                }
            };
            let program_path: PathBuf = config.path_programs.join(Path::new(&program_filename));
            let mut file = File::create(&program_path)?;
            let content: String = program_item.borrow().program_string.clone();
            file.write_all(content.as_bytes())?;
            program_item.borrow_mut().id = ProgramItemId::Path { path: program_path };
        }

        let program_id: ProgramItemId = program_item.borrow().id.clone(); 
        if program_id == ProgramItemId::None {
            return Err(anyhow::anyhow!("Expected ProgramItem.id to be a Path, but got None. {:?}", program_item));
        }

        // Print that the puzzle has been solved using a new/existing program
        let solution_type: &str;
        if is_new_program {
            solution_type = "a new";
        } else {
            solution_type = "an existing";
        }
        let message = format!("Puzzle {:?} solved with {} program: {:?}", model_id.file_stem(), solution_type, program_id.file_name());
        progress_bar.println(message);

        // Update CSV file
        let record = Record {
            model_filename: model_id.file_name(),
            program_filename: program_id.file_name(),
        };
        self.unique_records.insert(record);
        Record::save_solutions_csv(&self.unique_records, &config.path_solutions_csv);
        
        // Update JSON file
        let predictions: Vec<Prediction> = run_with_program_result.predictions().clone();
        let test_item = TestItem { 
            output_id: 0,
            number_of_predictions: predictions.len() as u8,
            predictions: predictions,
        };
        let task_name: String = model_id.file_stem();
        let task_item = TaskItem {
            task_name: task_name,
            test_vec: vec![test_item],
        };
        self.current_tasks.push(task_item);
        save_solutions_json(
            &config.path_solution_dir,
            &config.path_solution_teamid_json,
            &self.current_tasks
        );

        // Append the puzzle to the solved puzzles
        self.remove_model_items.push(Rc::clone(&model_item));

        // Append new programs to discovered programs
        // Ignore existing programs
        if is_new_program {
            self.discovered_program_item_vec.push(program_item);
        }

        Ok(())
    }
}

struct BatchRunner {
    config: RunArcCompetitionConfig,
    plan: BatchPlan,
}

impl BatchRunner {
    fn run_one_batch(&mut self, state: &mut BatchState) -> anyhow::Result<()> {
        self.plan.run_one_batch(&self.config, state)?;
        self.plan.reschedule(state)?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum ModelItemId {
    None,
    Path { path: PathBuf },
}

impl ModelItemId {
    fn file_name(&self) -> String {
        match self {
            ModelItemId::None => {
                return "None".to_string();
            },
            ModelItemId::Path { path } => {
                match path.file_name() {
                    Some(value) => {
                        return value.to_string_lossy().to_string();
                    },
                    None => {
                        return "Path without a file_name".to_string();
                    }
                }
            }
        }
    }

    fn file_stem(&self) -> String {
        match self {
            ModelItemId::None => {
                return "None".to_string();
            },
            ModelItemId::Path { path } => {
                match path.file_stem() {
                    Some(value) => {
                        return value.to_string_lossy().to_string();
                    },
                    None => {
                        return "Path without a file_stem".to_string();
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct ModelItem {
    id: ModelItemId,
    task: Task,
}

impl ModelItem {
    fn remove_model_items_where_filestem_contains(
        model_item_vec: &Vec<Rc<RefCell<ModelItem>>>,
        names_for_removal: &HashSet<String>
    ) -> Vec<Rc<RefCell<ModelItem>>> {
        let mut result_items: Vec<Rc<RefCell<ModelItem>>> = vec!();
        for model_item in model_item_vec {
            let file_stem: String = model_item.borrow().id.file_stem();
            if !names_for_removal.contains(&file_stem) {
                result_items.push(Rc::clone(model_item));
            }
        }
        result_items
    }

    fn remove_model_items(
        model_item_vec: &Vec<Rc<RefCell<ModelItem>>>,
        model_item_vec_for_removal: &Vec<Rc<RefCell<ModelItem>>>
    ) -> Vec<Rc<RefCell<ModelItem>>> {
        if model_item_vec_for_removal.is_empty() {
            return model_item_vec.clone();
        }
        let count_before: usize = model_item_vec.len();
        let mut result_model_item_vec: Vec<Rc<RefCell<ModelItem>>> = vec!();
        for model_item in model_item_vec {
            let mut keep = true;
            for remove_model_item in model_item_vec_for_removal {
                if Rc::ptr_eq(&remove_model_item, &model_item) {
                    keep = false;
                    break;
                }
            }
            if keep {
                result_model_item_vec.push(Rc::clone(model_item));
            }
        }
        let count_after: usize = result_model_item_vec.len();
        if count_after > count_before {
            error!("Expected removal to shrink vector, but it grows. {} != {} + {}", count_before, count_after, model_item_vec_for_removal.len());
        }
        result_model_item_vec
    }
}

#[derive(Clone, Debug)]
enum ProgramType {
    Simple,
    Advance,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum ProgramItemId {
    None,
    Path { path: PathBuf },
}

impl ProgramItemId {
    fn file_name(&self) -> String {
        match self {
            ProgramItemId::None => {
                return "None".to_string();
            },
            ProgramItemId::Path { path } => {
                match path.file_name() {
                    Some(value) => {
                        return value.to_string_lossy().to_string();
                    },
                    None => {
                        return "Path without a file_name".to_string();
                    }
                }
            }
        }
    }
}

type RcProgramItem = Rc<RefCell<ProgramItem>>;
type RcProgramItemVec = Vec<RcProgramItem>;

struct ProgramItem {
    id: ProgramItemId,
    program_string: String,
    program_type: ProgramType,
    parsed_program: ParsedProgram,
    program_runner: ProgramRunner,
    ignore_due_to_slowness: bool,
}

impl ProgramItem {
    /// Returns a compacted version of the program, that is only intended for use in the bloomfilter.
    /// Inserts header/footer if it's a simple program. Keeps the program if it's an adavanced program.
    /// There are no comments or unneccessary spacing.
    fn bloom_key(&self) -> anyhow::Result<String> {
        let compact_program_string: String = self.parsed_program.to_string();
        Ok(compact_program_string)
    }

    fn unique_name_for_saving(dir_path: &Path, name: &str) -> anyhow::Result<String> {
        assert!(dir_path.is_dir());
        assert!(dir_path.is_absolute());
        let max_number_of_variants: usize = 30;
        for variant_index in 1..max_number_of_variants {
            let filename = format!("{}-{}.asm", name, variant_index);
            let file_path: PathBuf = dir_path.join(&filename);
            if !file_path.is_file() {
                return Ok(filename);
            }
        }
        Err(anyhow::anyhow!("ProgramItem: Cannot construct unique filename for {:?} inside dir: {:?}", name, dir_path))
    }
}

impl fmt::Debug for ProgramItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ProgramItem {:?} program {:?}", self.id, self.program_string)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Serialize, PartialEq)]
struct Record {
    #[serde(rename = "model filename")]
    model_filename: String,
    #[serde(rename = "program filename")]
    program_filename: String,
}

impl Record {
    fn load_record_vec(csv_path: &Path) -> anyhow::Result<Vec<Record>> {
        let record_vec: Vec<Record> = parse_csv_file(csv_path)
            .map_err(|e| anyhow::anyhow!("unable to parse csv file. error: {:?}", e))?;
        Ok(record_vec)
    }

    fn save_solutions_csv(unique_records: &HashSet<Record>, path_csv: &Path) {
        let mut record_vec: Vec<Record> = unique_records.iter().map(|record| record.clone()).collect();
        record_vec.sort_unstable_by_key(|item| (item.model_filename.clone(), item.program_filename.clone()));
        match create_csv_file(&record_vec, &path_csv) {
            Ok(()) => {},
            Err(error) => {
                error!("Unable to save csv file: {:?}", error);
            }
        }
    }
}

fn save_solutions_json(path_solution_dir: &Path, path_solution_teamid_json: &Path, tasks: &Tasks) {
    if !path_solution_dir.exists() {
            match fs::create_dir(path_solution_dir) {
            Ok(_) => {},
            Err(err) => {
                panic!("Unable to create solution directory: {:?}, error: {:?}", path_solution_dir, err);
            }
        }
    }
    let json: String = match serde_json::to_string(&tasks) {
        Ok(value) => value,
        Err(error) => {
            error!("unable to serialize tasks to json: {:?}", error);
            return;
        }
    };
    match fs::write(&path_solution_teamid_json, json) {
        Ok(()) => {},
        Err(error) => {
            error!("unable to save solutions file. path: {:?} error: {:?}", path_solution_teamid_json, error);
            return;
        }
    }
    debug!("updated solutions file: tasks.len(): {}", tasks.len());
}
