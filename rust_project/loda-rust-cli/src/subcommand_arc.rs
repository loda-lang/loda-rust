//! The `loda-rust arc` subcommand, perform ARC Challenge experiments.
use crate::arc::TraverseProgramsAndModels;

#[derive(Debug)]
pub enum SubcommandARCMode {
    RunVerboseTestsOnlyForFilename { pattern: String },
    RunAllTests,
}

pub struct SubcommandARC;

impl SubcommandARC {
    pub fn run(mode: SubcommandARCMode) -> anyhow::Result<()> {
        let mut instance = TraverseProgramsAndModels::new()?;
        let mut verbose = false;
        match mode {
            SubcommandARCMode::RunAllTests => {

            },
            SubcommandARCMode::RunVerboseTestsOnlyForFilename { pattern } => {
                instance.filter_model_item_vec_by_pattern(&pattern)?;
                verbose = true;
            }
        }
        instance.run(verbose)?;
        Ok(())
    }
}
