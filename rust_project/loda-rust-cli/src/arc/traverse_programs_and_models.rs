use super::{Model, ImagePair};
use super::{RunWithProgram, RunWithProgramResult};
use crate::config::Config;
use crate::common::find_json_files_recursively;
use crate::common::find_asm_files_recursively;
use crate::mine::{Genome, GenomeItem, ToGenomeItemVec, create_genome_mutate_context, GenomeMutateContext};
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::{ProgramSerializer, ProgramId, ProgramRunner};
use loda_rust_core::parser::ParsedProgram;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};
use rand::SeedableRng;
use rand::rngs::StdRng;

pub struct TraverseProgramsAndModels {
    config: Config,
    model_item_vec: Vec<ModelItem>,
    program_item_vec: Vec<ProgramItem>,
}

impl TraverseProgramsAndModels {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let mut instance = Self { 
            config,
            model_item_vec: vec!(),
            program_item_vec: vec!(),
        };
        instance.load_arc_models()?;
        instance.load_programs()?;
        Ok(instance)
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
                enabled: true,
            };
            model_item_vec.push(item);
        }
        if model_item_vec.len() != paths.len() {
            error!("Skipped some models. paths.len()={}, but model_item_vec.len()={}", paths.len(), model_item_vec.len());
        }
        self.model_item_vec = model_item_vec;
        Ok(())
    }

    pub fn filter_model_item_vec_by_pattern(&mut self, pattern: &String) -> anyhow::Result<()> {
        for model_item in self.model_item_vec.iter_mut() {
            model_item.enabled = false;
        }
        let mut number_of_enabled: usize = 0;
        for model_item in self.model_item_vec.iter_mut() {
            match &model_item.id {
                ModelItemId::None => {},
                ModelItemId::Path { path } => {
                    let s: String = path.to_string_lossy().to_string();
                    if s.contains(pattern) {
                        model_item.enabled = true;
                        number_of_enabled += 1;
                    }
                }
            }
        }
        if number_of_enabled == 0 {
            return Err(anyhow::anyhow!("No files match the pattern: {}", pattern));
        }
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

    fn mutate_program(&self, program_index: usize, program_item: &ProgramItem) -> anyhow::Result<()> {
        println!("loading context");
        let start = Instant::now();
        let context: GenomeMutateContext = create_genome_mutate_context(&self.config);
        println!("loaded context. elapsed: {}", HumanDuration(start.elapsed()));

        let mut genome = Genome::new();

        let initial_random_seed: u64 = 0;
        let mut rng: StdRng = StdRng::seed_from_u64(initial_random_seed);

        let program_content: String;
        match program_item.program_type {
            ProgramType::Simple => {
                program_content = RunWithProgram::convert_simple_to_full(&program_item.program_string);
            },
            ProgramType::Advance => {
                program_content = program_item.program_string.clone();
            }
        }

        let initial_parsed_program: ParsedProgram = match ParsedProgram::parse_program(&program_content) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("cannot parse the program: {:?}", error));
            }
        };

        println!("; INPUT PROGRAM\n; program_index: {}\n; id: {:?}\n\n{}", program_index, program_item.id, initial_parsed_program);

        let genome_vec: Vec<GenomeItem> = initial_parsed_program.to_genome_item_vec();

        // locking rows
        // for i in 0..3 {
        //     genome_vec[i].set_mutation_locked(true);
        // }

        genome.set_genome_vec(genome_vec);

        
        let mut dependency_manager: DependencyManager = RunWithProgram::create_dependency_manager();

        let mut number_of_successful_mutations: usize = 0;
        for _ in 0..40 {
            let mutate_success: bool = genome.mutate(&mut rng, &context);
            if !mutate_success {
                continue;
            }

            let parsed_program: ParsedProgram = genome.to_parsed_program();
            let program_runner: ProgramRunner = dependency_manager.parse_stage2(ProgramId::ProgramWithoutId, &parsed_program).expect("ProgramRunner");
    
            number_of_successful_mutations += 1;

            let mut serializer = ProgramSerializer::new();
            serializer.append_comment(format!("MUTATION {}", number_of_successful_mutations));
            serializer.append_comment(format!("program_index {}", program_index));
            serializer.append_comment(format!("program id {:?}", program_item.id));
            serializer.append_empty_line();
            program_runner.serialize(&mut serializer);
            serializer.append_empty_line();
            for message in genome.message_vec() {
                serializer.append_comment(message);
            }
            serializer.append_empty_line();
            let candidate_program: String = serializer.to_string();
            println!("; ------\n\n{}", candidate_program);

        }

        Ok(())
    }

    fn genome_experiments(&self) -> anyhow::Result<()> {
        for (program_index, program_item) in self.program_item_vec.iter().enumerate() {
            self.mutate_program(program_index, &program_item)?;
            println!("break after first iteration");
            break;
        }
        Ok(())
    }

    pub fn run(&mut self, verbose: bool) -> anyhow::Result<()> {
        // self.genome_experiments()?;
        // return Ok(());

        let mut count_match: usize = 0;
        let mut count_mismatch: usize = 0;
        let mut found_program_indexes: Vec<usize> = vec!();

        let start = Instant::now();
        let pb = ProgressBar::new((self.model_item_vec.len()+1) as u64);
        for model_item in &self.model_item_vec {
            pb.inc(1);
            if !model_item.enabled {
                continue;
            }
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
                            Err(error) => {
                                if verbose {
                                    error!("model: {:?} simple-program: {:?} error: {:?}", model_item.id, program_item.id, error);
                                }
                                continue;
                            }
                        };
                    },
                    ProgramType::Advance => {
                        result = match instance.run_advanced(&program_item.program_string) {
                            Ok(value) => value,
                            Err(error) => {
                                if verbose {
                                    error!("model: {:?} advanced-program: {:?} error: {:?}", model_item.id, program_item.id, error);
                                }
                                continue;
                            }
                        };
                    }
                }

                if verbose {
                    println!("model: {:?} program: {:?} result: {:?}", model_item.id, program_item.id, result);
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

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum ModelItemId {
    None,
    Path { path: PathBuf },
}

#[derive(Clone, Debug)]
struct ModelItem {
    id: ModelItemId,
    model: Model,
    enabled: bool,
}

#[derive(Clone, Debug)]
enum ProgramType {
    Simple,
    Advance,
}

#[allow(dead_code)]
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
