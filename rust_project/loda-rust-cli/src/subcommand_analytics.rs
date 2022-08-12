//! The `loda-rust analytics` subcommand, populates histograms/bloomfilters.
use std::error::Error;
use crate::analytics::Analytics;

pub fn subcommand_analytics() -> Result<(), Box<dyn Error>> {
    Analytics::run()?;
    Ok(())
}
