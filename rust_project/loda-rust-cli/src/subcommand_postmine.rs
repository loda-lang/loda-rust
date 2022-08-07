//! The `loda-rust postmine` subcommand, checks the mined programs for correctness and performance.
use crate::postmine::PostMine;
use std::error::Error;

pub fn subcommand_postmine() -> Result<(), Box<dyn Error>> {
    PostMine::run()
}
