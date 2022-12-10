//! The `loda-rust arc` subcommand, perform ARC Challenge experiments.
use crate::arc::TraverseProgramsAndModels;

pub struct SubcommandARC;

impl SubcommandARC {
    pub async fn run() -> anyhow::Result<()> {
        TraverseProgramsAndModels::run().await?;
        Ok(())
    }
}
