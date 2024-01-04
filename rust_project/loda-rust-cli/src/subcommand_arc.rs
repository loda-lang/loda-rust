//! The `loda-rust arc` subcommands, perform experiments with the `Abstraction and Reasoning Corpus`.
use std::path::PathBuf;

#[cfg(feature = "loda-rust-arc")]
use crate::arc::{SubcommandARCSize, SubcommandARCWeb, TraverseProgramsAndModels};

#[derive(Debug)]
pub enum SubcommandARCMode {
    /// Check that all the existing solutions still works.
    CheckAllExistingSolutions,

    /// Populate the `solutions.csv` file by trying out all puzzles with all solutions.
    GenerateSolutionCSV,

    /// Eval a single task with all the existing solutions.
    EvalSingleTask { pattern: String },

    /// The code being executed inside the docker image submitted for the `ARCathon` contest.
    Competition,

    /// Traverse all puzzles and classify each puzzle.
    LabelAllPuzzles,

    /// Create a file with training data.
    ExportDataset,

    /// Run all tasks using the specified solver.
    /// 
    /// where `name_of_solver` is one of:
    /// - `lr` is logistic regression.
    /// - `one` is `SolveOneColor`.
    SolveWithSpecificSolver { name_of_solver: String },
    
    /// Predict the output sizes for a single ARC task.
    PredictOutputSizesForSingleTask { task_json_file: PathBuf },
    
    /// Traverse the task json files, and assign a number of histogram comparisons.
    MetadataHistogram { count: u16, task_json_directory: PathBuf },
}

pub struct SubcommandARC;

impl SubcommandARC {
    #[cfg(not(feature = "loda-rust-arc"))]
    pub fn run(_mode: SubcommandARCMode) -> anyhow::Result<()> {
        panic!("loda-rust-arc feature is not enabled");
    }

    #[cfg(feature = "loda-rust-arc")]
    pub fn run(mode: SubcommandARCMode) -> anyhow::Result<()> {
        #[allow(unused_imports)]
        use crate::arc::GenerateDataset;

        match mode {
            SubcommandARCMode::CheckAllExistingSolutions => {
                return TraverseProgramsAndModels::check_all_existing_solutions();
            },
            SubcommandARCMode::GenerateSolutionCSV => {
                return TraverseProgramsAndModels::generate_solution_csv();
            },
            SubcommandARCMode::EvalSingleTask { pattern } => {
                return TraverseProgramsAndModels::eval_single_task_with_all_existing_solutions(pattern);
            },
            SubcommandARCMode::Competition => {
                return TraverseProgramsAndModels::arc_competition();
            },
            SubcommandARCMode::LabelAllPuzzles => {
                return TraverseProgramsAndModels::label_all_puzzles();
            },
            SubcommandARCMode::ExportDataset => {
                // let path: PathBuf = PathBuf::from("/Users/neoneye/Downloads/histogram-comparisons.jsonl");
                // GenerateDataset::generate_dataset_huge(&path)?;
                // GenerateDataset::generate_dataset_small(&path)?;
                // return Ok(());
                return TraverseProgramsAndModels::export_dataset();
            },
            SubcommandARCMode::SolveWithSpecificSolver { name_of_solver } => {
                return TraverseProgramsAndModels::solve_with_specific_solver(&name_of_solver);
            },
            SubcommandARCMode::PredictOutputSizesForSingleTask { task_json_file } => {
                return SubcommandARCSize::run(&task_json_file);
            },
            SubcommandARCMode::MetadataHistogram { count, task_json_directory } => {
                debug!("arc-metadata-histograms count: {}", count);
                debug!("arc-metadata-histograms directory: {:?}", task_json_directory);
                if !task_json_directory.is_dir() {
                    anyhow::bail!("arc-metadata-histograms. Expected directory to be a directory, but it's not. path: {:?}", task_json_directory);
                }
                if count == 0 {
                    anyhow::bail!("arc-metadata-histograms. Expected count to be greater than zero, but it's not. count: {}", count);
                }
                if count > 1000 {
                    anyhow::bail!("arc-metadata-histograms. Expected count to be less than or equal to 1000. count: {}", count);
                }
                return Ok(());
            },
        }
    }

    #[cfg(not(feature = "loda-rust-arc"))]
    pub async fn run_web_server() -> anyhow::Result<()> {
        panic!("loda-rust-arc feature is not enabled");
    }

    #[cfg(feature = "loda-rust-arc")]
    pub async fn run_web_server() -> anyhow::Result<()> {
        SubcommandARCWeb::run_web_server().await
    }
}
