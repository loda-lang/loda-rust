//! The `loda-rust similar` subcommand, identifies similar programs.
use crate::similar::Similar;
use std::error::Error;

pub fn subcommand_similar() -> Result<(), Box<dyn Error>> {
    Similar::run()?;
    Ok(())
}
