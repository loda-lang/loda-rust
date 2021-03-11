use super::{DependencyManager, Settings};

pub fn subcommand_dependencies(settings: &Settings, program_id: u64) {
    let mut dm = DependencyManager::new(
        settings.loda_program_rootdir.clone(),
    );
    dm.load(program_id);
    dm.print_dependencies();
}
