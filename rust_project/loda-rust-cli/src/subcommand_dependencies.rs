//! The `loda-rust dependencies` subcommand, prints dependencies of a program.
use super::{DependencyManager,DependencyManagerFileSystemMode};
use loda_rust_core::execute::UnofficialFunctionRegistry;
use crate::config::Config;
use std::path::PathBuf;

pub fn subcommand_dependencies(program_id: u64) {
    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let mut dm = DependencyManager::new(
        DependencyManagerFileSystemMode::System,
        loda_programs_oeis_dir,
        UnofficialFunctionRegistry::new(),
    );
    if let Err(error) = dm.load(program_id) {
        panic!("Failure during loadin of program. error: {:?}", error);
    }
    dm.print_dependencies();
}
