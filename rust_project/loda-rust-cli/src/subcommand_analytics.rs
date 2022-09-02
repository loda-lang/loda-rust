//! The `loda-rust analytics` subcommand, populates histograms/bloomfilters.
use crate::analytics::Analytics;
use std::error::Error;

pub fn subcommand_analytics() -> Result<(), Box<dyn Error>> {
    Analytics::run()
}
