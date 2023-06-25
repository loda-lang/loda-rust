//! The `loda-rust arc` subcommands, perform experiments with the `Abstraction and Reasoning Corpus`.
use std::path::PathBuf;

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

    /// Exploring what a model may be like.
    ExperimentWithConvolution,

    /// Predict the output sizes for a single ARC task.
    PredictOutputSizesForSingleTask { task_json_file: PathBuf },
}

pub struct SubcommandARC;

impl SubcommandARC {
    pub fn run(mode: SubcommandARCMode) -> anyhow::Result<()> {
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
                return TraverseProgramsAndModels::export_dataset();
            },
            SubcommandARCMode::ExperimentWithConvolution => {
                return TraverseProgramsAndModels::experiment_with_convolution();
            },
            SubcommandARCMode::PredictOutputSizesForSingleTask { task_json_file } => {
                return SubcommandARCSize::run(&task_json_file);
            },
        }
    }

    pub async fn run_web_server() -> anyhow::Result<()> {
        SubcommandARCWeb::run_web_server().await
    }
}
