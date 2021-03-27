use super::DependencyManager;
use crate::config::Config;
use std::path::PathBuf;

pub fn subcommand_dependencies(program_id: u64) {
    let config = Config::load();
    let loda_program_rootdir: PathBuf = config.loda_program_rootdir();
    let mut dm = DependencyManager::new(
        loda_program_rootdir,
    );
    dm.load(program_id);
    dm.print_dependencies();
}
