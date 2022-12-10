use super::{Model, ImagePair};
use super::{RunWithProgram, RunWithProgramResult};
use crate::config::Config;
use crate::common::find_json_files_recursively;
use crate::common::find_asm_files_recursively;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

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
                number_of_models: 0,
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

        let mut count_match: usize = 0;
        let mut count_mismatch: usize = 0;
        let mut found_program_indexes: Vec<usize> = vec!();

        let start = Instant::now();
        let pb = ProgressBar::new((self.model_item_vec.len()+1) as u64);
        for model_item in &self.model_item_vec {
            pb.inc(1);
            let model: Model = model_item.model.clone();
            let instance = RunWithProgram::new(model).expect("RunWithProgram");

            let pairs: Vec<ImagePair> = model_item.model.images_all().expect("pairs");
    
            let mut found_one_or_more_solutions = false;
            for (program_index, program_item) in self.program_item_vec.iter_mut().enumerate() {

                let result: RunWithProgramResult;
                match program_item.program_type {
                    ProgramType::Simple => {
                        result = match instance.run_simple(&program_item.program_string) {
                            Ok(value) => value,
                            Err(_error) => {
                                continue;
                            }
                        };
                    },
                    ProgramType::Advance => {
                        result = match instance.run_advanced(&program_item.program_string) {
                            Ok(value) => value,
                            Err(_error) => {
                                continue;
                            }
                        };
                    }
                }

                let count: usize = result.count_train_correct() + result.count_test_correct();

                if count == pairs.len() {
                    found_one_or_more_solutions = true;
                    found_program_indexes.push(program_index);
                    let message = format!("program: {:?} is a solution for model: {:?}", program_item.id, model_item.id);
                    pb.println(message);

                    program_item.number_of_models += 1;
                }
            }

            if found_one_or_more_solutions {
                count_match += 1;
            } else {
                count_mismatch += 1;
            }
        }
        pb.finish_and_clear();
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} processing programs/models in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        found_program_indexes.sort();

        println!("number of matches: {} mismatches: {}", count_match, count_mismatch);
        println!("found_program_indexes: {:?}", found_program_indexes);

        for program in &self.program_item_vec {
            if program.number_of_models == 0 {
                println!("unused program {:?}, it doesn't solve any of the models, and can be removed", program.id);
            }
        }

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
    number_of_models: usize,
}
