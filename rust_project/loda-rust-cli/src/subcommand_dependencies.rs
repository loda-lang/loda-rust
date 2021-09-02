use loda_rust_core;
use super::{DependencyManager,DependencyManagerFileSystemMode};
use loda_rust_core::config::Config;
use std::path::PathBuf;

pub fn subcommand_dependencies(program_id: u64) {
    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let mut dm = DependencyManager::new(
        DependencyManagerFileSystemMode::System,
        loda_programs_oeis_dir,
    );
    if let Err(error) = dm.load(program_id) {
        panic!("Failure during loadin of program. error: {:?}", error);
    }
    dm.print_dependencies();
}
