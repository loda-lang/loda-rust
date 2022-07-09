//! The `loda-rust postmine` subcommand, checks the mined programs for correctness and performance.
use crate::config::Config;
use std::error::Error;

pub fn subcommand_postmine() -> Result<(), Box<dyn Error>> {
    let _config = Config::load();
    Ok(())
}
