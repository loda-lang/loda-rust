use crate::config::Config;
use crate::common::find_json_files_recursively;
use crate::common::find_asm_files_recursively;
use std::path::PathBuf;

pub struct TraverseProgramsAndModels {}

impl TraverseProgramsAndModels {
    pub async fn run() -> anyhow::Result<()> {
        let mut instance = Self {};
        instance.run_inner()?;
        Ok(())
    }

    fn run_inner(&mut self) -> anyhow::Result<()> {

        let config = Config::load();
        {
            let path: PathBuf = config.arc_repository_data_training();
            let paths: Vec<PathBuf> = find_json_files_recursively(&path);
            println!("number of json files: {}", paths.len());
        }
        {
            let path: PathBuf = config.loda_arc_challenge_repository_programs();
            let paths: Vec<PathBuf> = find_asm_files_recursively(&path);
            println!("number of asm files: {}", paths.len());
        }

        Ok(())
    }
}
