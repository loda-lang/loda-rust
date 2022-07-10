//! The `loda-rust postmine` subcommand, checks the mined programs for correctness and performance.
use crate::config::Config;
use crate::common::find_asm_files_recursively;
use crate::postmine::find_pending_programs;
use std::error::Error;
use std::path::PathBuf;

pub fn subcommand_postmine() -> Result<(), Box<dyn Error>> {
    let config = Config::load();

    let mine_event_dir: PathBuf = config.mine_event_dir();
    let paths_all: Vec<PathBuf> = find_asm_files_recursively(&mine_event_dir);
    let paths_for_processing: Vec<PathBuf> = find_pending_programs(&paths_all, true)?;

    println!("Will process {} programs", paths_for_processing.len());

    Ok(())
}
