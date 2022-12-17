use super::{Model, ImagePair};
use super::{RunWithProgram, RunWithProgramResult};
use crate::config::Config;
use crate::common::{find_json_files_recursively, parse_csv_file, create_csv_file};
use crate::common::find_asm_files_recursively;
use crate::mine::{Genome, GenomeItem, ToGenomeItemVec, create_genome_mutate_context, GenomeMutateContext};
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::{ProgramSerializer, ProgramId, ProgramRunner};
use loda_rust_core::parser::ParsedProgram;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::path::{PathBuf, Path};
use std::rc::Rc;
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};

pub struct TraverseProgramsAndModels {
    config: Config,
    context: GenomeMutateContext,
    model_item_vec: Vec<ModelItem>,
    program_item_vec: Vec<Rc<RefCell<ProgramItem>>>,
}

impl TraverseProgramsAndModels {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();

        println!("loading genome mutate context");
        let start = Instant::now();
        let context: GenomeMutateContext = create_genome_mutate_context(&config);
        println!("loaded genome mutate context. elapsed: {}", HumanDuration(start.elapsed()));

        let mut instance = Self { 
            config,
            context,
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

        let mut program_item_vec: Vec<Rc<RefCell<ProgramItem>>> = vec!();
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

            let instance = ProgramItem {
                id: ProgramItemId::Path { path: path.clone() },
                program_string,
                program_type,
                number_of_models: 0,
            };
            let item = Rc::new(RefCell::new(instance));
            program_item_vec.push(item);
        }
        if program_item_vec.len() != paths.len() {
            error!("Skipped some programs. paths.len()={}, but program_item_vec.len()={}", paths.len(), program_item_vec.len());
        }
        self.program_item_vec = program_item_vec;
        Ok(())
    }

    fn mutate_program(&self, program_index: usize, program_item: &ProgramItem) -> anyhow::Result<()> {
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
            let mutate_success: bool = genome.mutate(&mut rng, &self.context);
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

            if number_of_successful_mutations > 5 {
                break;
            }
        }

        Ok(())
    }

    fn genome_experiments(&self) -> anyhow::Result<()> {
        for (program_index, program_item) in self.program_item_vec.iter().enumerate() {
            self.mutate_program(program_index, &program_item.borrow())?;
            println!("break after first iteration");
            break;
        }
        Ok(())
    }

    pub fn run(&mut self, verbose: bool) -> anyhow::Result<()> {
        // self.genome_experiments()?;
        // return Ok(());

        let path_solutions_csv = self.config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));

        let mut record_vec = Vec::<Record>::new();

        let ignore_models_with_a_solution: bool = path_solutions_csv.is_file();
        if ignore_models_with_a_solution {
            record_vec = Record::load_record_vec(&path_solutions_csv)?;
            println!("solutions.csv: number of rows: {}", record_vec.len());
    
            let mut file_names_to_ignore = HashSet::<String>::new();
            for record in &record_vec {
                file_names_to_ignore.insert(record.model_filename.clone());

                let program_filename: String = record.program_filename.clone();
                for program_item in &self.program_item_vec {
                    // let item: &mut ProgramItem = &program_item.borrow_mut();
                    if program_item.borrow().id.file_name() == program_filename {
                        program_item.borrow_mut().number_of_models += 1;
                    }
                }
            }
            for model_item in self.model_item_vec.iter_mut() {
                let file_name: String = model_item.id.file_name();
                if file_names_to_ignore.contains(&file_name) {
                    model_item.enabled = false;
                }
            }
        }

        let mut scheduled_program_item_vec: Vec<Rc<RefCell<ProgramItem>>> = Vec::<Rc<RefCell<ProgramItem>>>::new();
        for program_item in self.program_item_vec.iter_mut() {
            if program_item.borrow().number_of_models == 0 {
                scheduled_program_item_vec.push(program_item.clone());
            }
        }

        let mut number_of_models_for_processing: u64 = 0;
        let mut number_of_models_ignored: u64 = 0;
        for model_item in &self.model_item_vec {
            if model_item.enabled {
                number_of_models_for_processing += 1;
            } else {
                number_of_models_ignored += 1;
            }
        }
        println!("number of models for processing: {}", number_of_models_for_processing);
        println!("number of models being ignored: {}", number_of_models_ignored);

        let mut count_match: usize = 0;
        let mut count_mismatch: usize = 0;
        let start = Instant::now();
        let pb = ProgressBar::new(number_of_models_for_processing+1);
        for model_item in &self.model_item_vec {
            if !model_item.enabled {
                continue;
            }
            pb.inc(1);
            let model: Model = model_item.model.clone();
            let instance = RunWithProgram::new(model).expect("RunWithProgram");

            let pairs: Vec<ImagePair> = model_item.model.images_all().expect("pairs");
    
            let mut found_one_or_more_solutions = false;
            for (program_index, program_item) in scheduled_program_item_vec.iter_mut().enumerate() {

                let result: RunWithProgramResult;
                match program_item.borrow().program_type {
                    ProgramType::Simple => {
                        result = match instance.run_simple(&program_item.borrow().program_string) {
                            Ok(value) => value,
                            Err(error) => {
                                if verbose {
                                    error!("model: {:?} simple-program: {:?} error: {:?}", model_item.id, program_item.borrow().id, error);
                                }
                                continue;
                            }
                        };
                    },
                    ProgramType::Advance => {
                        result = match instance.run_advanced(&program_item.borrow().program_string) {
                            Ok(value) => value,
                            Err(error) => {
                                if verbose {
                                    error!("model: {:?} advanced-program: {:?} error: {:?}", model_item.id, program_item.borrow().id, error);
                                }
                                continue;
                            }
                        };
                    }
                }

                if verbose {
                    println!("model: {:?} program: {:?} result: {:?}", model_item.id, program_item.borrow().id, result);
                }

                let count: usize = result.count_train_correct() + result.count_test_correct();

                if count == pairs.len() {
                    found_one_or_more_solutions = true;
                    let message = format!("program: {:?} is a solution for model: {:?}", program_item.borrow().id, model_item.id);
                    pb.println(message);

                    let model_filename: String = model_item.id.file_name();
                    let program_filename: String = program_item.borrow().id.file_name();
                    let record = Record {
                        model_filename,
                        program_filename,
                    };
                    record_vec.push(record);

                    program_item.borrow_mut().number_of_models += 1;
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

        println!("number of matches: {} mismatches: {}", count_match, count_mismatch);

        for program_item in &scheduled_program_item_vec {
            if program_item.borrow().number_of_models == 0 {
                println!("unused program {:?}, it doesn't solve any of the models, and can be removed", program_item.borrow().id);
            }
        }

        record_vec.sort_unstable_by_key(|item| (item.model_filename.clone(), item.program_filename.clone()));
        match create_csv_file(&record_vec, &path_solutions_csv) {
            Ok(()) => {},
            Err(error) => {
                error!("Unable to save csv file: {:?}", error);
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

impl ModelItemId {
    fn file_name(&self) -> String {
        match self {
            ModelItemId::None => {
                return "None".to_string();
            },
            ModelItemId::Path { path } => {
                match path.file_name() {
                    Some(value) => {
                        return value.to_string_lossy().to_string();
                    },
                    None => {
                        return "Path without a file_name".to_string();
                    }
                }
            }
        }
    }
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

impl ProgramItemId {
    fn file_name(&self) -> String {
        match self {
            ProgramItemId::None => {
                return "None".to_string();
            },
            ProgramItemId::Path { path } => {
                match path.file_name() {
                    Some(value) => {
                        return value.to_string_lossy().to_string();
                    },
                    None => {
                        return "Path without a file_name".to_string();
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct ProgramItem {
    id: ProgramItemId,
    program_string: String,
    program_type: ProgramType,
    number_of_models: usize,
}

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    #[serde(rename = "model filename")]
    model_filename: String,
    #[serde(rename = "program filename")]
    program_filename: String,
}

impl Record {
    fn load_record_vec(csv_path: &Path) -> anyhow::Result<Vec<Record>> {
        let record_vec: Vec<Record> = parse_csv_file(csv_path)
            .map_err(|e| anyhow::anyhow!("unable to parse csv file. error: {:?}", e))?;
        Ok(record_vec)
    }
}
