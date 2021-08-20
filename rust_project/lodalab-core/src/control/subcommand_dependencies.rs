use super::{DependencyManager,DependencyManagerFileSystemMode};
use crate::config::Config;
use std::path::PathBuf;

pub fn subcommand_dependencies(program_id: u64) {
    let config = Config::load();
    let loda_program_rootdir: PathBuf = config.loda_program_rootdir();
    let mut dm = DependencyManager::new(
        DependencyManagerFileSystemMode::System,
        loda_program_rootdir,
    );
    if let Err(error) = dm.load(program_id) {
        panic!("Failure during loadin of program. error: {:?}", error);
    }
    dm.print_dependencies();
}
