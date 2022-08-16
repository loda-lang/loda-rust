//! The `loda-rust names` subcommand, insert sequence names into the mined programs.
use crate::postmine::InsertNames;
use std::error::Error;

pub fn subcommand_insertnames() -> Result<(), Box<dyn Error>> {
    InsertNames::run()
}
