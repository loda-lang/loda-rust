use super::{Model, Image, GridToImage, ImagePair};
use crate::config::Config;
use crate::common::find_json_files_recursively;
use crate::common::find_asm_files_recursively;
use std::fs;
use std::path::PathBuf;

pub struct TraverseProgramsAndModels {
    config: Config,
    model_item_vec: Vec<ModelItem>,
    program_item_vec: Vec<ProgramItem>,
}

impl TraverseProgramsAndModels {
    pub async fn run() -> anyhow::Result<()> {
        let config = Config::load();
        let mut instance = Self { 
            config,
            model_item_vec: vec!(),
            program_item_vec: vec!(),
        };
        instance.load_arc_models()?;
        instance.load_programs()?;
        instance.run_inner()?;
        Ok(())
    }

    fn load_arc_models(&mut self) -> anyhow::Result<()> {
        let path: PathBuf = self.config.arc_repository_data_training();
        let paths: Vec<PathBuf> = find_json_files_recursively(&path);
        println!("arc_repository_data_training. number of json files: {}", paths.len());

        let mut model_item_vec = Vec::<ModelItem>::new();
        for path in &paths {
            let model = Model::load_with_json_file(path).expect("model");
            let item = ModelItem {
                id: ModelItemId::Path { path: path.clone() },
                model,
            };
            model_item_vec.push(item);
        }
        if model_item_vec.len() != paths.len() {
            error!("Skipped some models. paths.len()={}, but model_item_vec.len()={}", paths.len(), model_item_vec.len());
        }
        self.model_item_vec = model_item_vec;
        Ok(())
    }

    fn load_programs(&mut self) -> anyhow::Result<()> {
        let path: PathBuf = self.config.loda_arc_challenge_repository_programs();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&path);
        println!("loda_arc_challenge_repository_programs. number of asm files: {}", paths.len());

        let mut program_item_vec: Vec<ProgramItem> = vec!();
        for path in &paths {

            let program_string: String = match fs::read_to_string(path) {
                Ok(value) => value,
                Err(error) => {
                    error!("cannot read the file: {:?} error: {:?}", path, error);
                    continue;
                }
            };

            let is_simple: bool = program_string.contains("Program Type: simple");
            let is_advanced: bool = program_string.contains("Program Type: advanced");
            let program_type: ProgramType;
            match (is_simple, is_advanced) {
                (false, false) => {
                    error!("Cannot find 'Program Type: simple' nor 'Program Type: advanced'. Skipping program. path: {:?}", path);
                    continue;
                },
                (false, true) => {
                    program_type = ProgramType::Advance;
                },
                (true, false) => {
                    program_type = ProgramType::Simple;
                },
                (true, true) => {
                    error!("Ambiguous use of 'Program Type'. Should be either 'Program Type: simple' or 'Program Type: advanced'. Skipping program. path: {:?}", path);
                    continue;
                }
            }

            let item = ProgramItem {
                id: ProgramItemId::Path { path: path.clone() },
                program_string,
                program_type,
            };
            program_item_vec.push(item);
        }
        if program_item_vec.len() != paths.len() {
            error!("Skipped some programs. paths.len()={}, but program_item_vec.len()={}", paths.len(), program_item_vec.len());
        }
        self.program_item_vec = program_item_vec;
        Ok(())
    }

    fn run_inner(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum ModelItemId {
    None,
    Path { path: PathBuf },
}

#[derive(Clone, Debug)]
struct ModelItem {
    id: ModelItemId,
    model: Model,
}

#[derive(Clone, Debug)]
enum ProgramType {
    Simple,
    Advance,
}

#[derive(Clone, Debug)]
enum ProgramItemId {
    None,
    Path { path: PathBuf },
}

#[derive(Clone, Debug)]
struct ProgramItem {
    id: ProgramItemId,
    program_string: String,
    program_type: ProgramType,
}
