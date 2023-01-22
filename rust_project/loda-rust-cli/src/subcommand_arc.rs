//! The `loda-rust arc` subcommands, perform ARC Challenge experiments.
use crate::arc::TraverseProgramsAndModels;

#[derive(Debug)]
pub enum SubcommandARCMode {
    /// Check that all the existing solutions still works.
    CheckAllExistingSolutions,

    /// Eval a single puzzle or solution and see the internal state of what is going on.
    EvalByFilename { pattern: String },

    /// The code being executed inside the docker image submitted for the `ARCathon 1` contest.
    Competition,
}

pub struct SubcommandARC;

impl SubcommandARC {
    pub fn run(mode: SubcommandARCMode) -> anyhow::Result<()> {
        println!("mode: {:?}", mode);
        match mode {
            SubcommandARCMode::CheckAllExistingSolutions => {
                return TraverseProgramsAndModels::check_all_existing_solutions();
            },
            SubcommandARCMode::EvalByFilename { pattern } => {
                return TraverseProgramsAndModels::eval_by_filename(pattern);
            },
            SubcommandARCMode::Competition => {
                return TraverseProgramsAndModels::arc_competition();
            },
        }
    }
}
