//! The `loda-rust arc` subcommands, perform ARC Challenge experiments.
use crate::arc::TraverseProgramsAndModels;

#[derive(Debug)]
pub enum SubcommandARCMode {
    /// Check that all the existing solutions still works.
    CheckAllExistingSolutions,

    /// Eval a single puzzle with all the existing solutions.
    EvalSinglePuzzle { pattern: String },

    /// The code being executed inside the docker image submitted for the `ARCathon 1` contest.
    Competition,
}

pub struct SubcommandARC;

impl SubcommandARC {
    pub fn run(mode: SubcommandARCMode) -> anyhow::Result<()> {
        match mode {
            SubcommandARCMode::CheckAllExistingSolutions => {
                return TraverseProgramsAndModels::check_all_existing_solutions();
            },
            SubcommandARCMode::EvalSinglePuzzle { pattern } => {
                return TraverseProgramsAndModels::eval_single_puzzle_with_all_existing_solutions(pattern);
            },
            SubcommandARCMode::Competition => {
                return TraverseProgramsAndModels::arc_competition();
            },
        }
    }
}
