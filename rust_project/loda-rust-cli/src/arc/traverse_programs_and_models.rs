use super::{Model, Image, GridToImage, ImagePair};
use crate::config::Config;
use crate::common::find_json_files_recursively;
use crate::common::find_asm_files_recursively;
use std::path::PathBuf;

pub struct TraverseProgramsAndModels {
    config: Config,
    model_item_vec: Vec<ModelItem>,
}

impl TraverseProgramsAndModels {
    pub async fn run() -> anyhow::Result<()> {
        let config = Config::load();
        let mut instance = Self { 
            config,
            model_item_vec: vec!(),
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
        println!("model_item_vec.len(): {}", model_item_vec.len());
        self.model_item_vec = model_item_vec;
        Ok(())
    }

    fn load_programs(&mut self) -> anyhow::Result<()> {
        let path: PathBuf = self.config.loda_arc_challenge_repository_programs();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&path);
        println!("loda_arc_challenge_repository_programs. number of asm files: {}", paths.len());
        Ok(())
    }

    fn run_inner(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum ModelItemId {
    None,
    Path { path: PathBuf },
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct ModelItem {
    id: ModelItemId,
    model: Model,
}
