use super::{Model, ImagePair};
use super::{RunWithProgram, RunWithProgramResult};
use super::{Prediction, TestItem, TaskItem, Tasks};
use crate::analytics::{AnalyticsDirectory, Analytics};
use crate::config::Config;
use crate::common::{find_json_files_recursively, parse_csv_file, create_csv_file};
use crate::common::find_asm_files_recursively;
use crate::mine::{Genome, GenomeItem, ToGenomeItemVec, CreateGenomeMutateContextMode, create_genome_mutate_context, GenomeMutateContext};
use anyhow::Context;
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::{ProgramSerializer, ProgramId, ProgramRunner};
use loda_rust_core::parser::ParsedProgram;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{PathBuf, Path};
use std::rc::Rc;
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};

static SOLUTIONS_FILENAME: &str = "solution_notXORdinary.json";

pub struct TraverseProgramsAndModels {
    config: Config,
    context: GenomeMutateContext,
    model_item_vec: Vec<ModelItem>,
    program_item_vec: Vec<Rc<RefCell<ProgramItem>>>,
    locked_instruction_hashset: HashSet<String>,
    path_solution_teamid_json: PathBuf,
}

impl TraverseProgramsAndModels {
    pub fn new() -> anyhow::Result<Self> {
        Analytics::arc_run_if_expired()?;

        let config = Config::load();

        println!("loading genome mutate context");
        let start = Instant::now();

        let analytics_directory = AnalyticsDirectory::new(
            config.analytics_arc_dir()
        ).with_context(||"unable to create AnalyticsDirectory instance")?;    

        let context: GenomeMutateContext = create_genome_mutate_context(CreateGenomeMutateContextMode::ARC, analytics_directory)?;
        println!("loaded genome mutate context. elapsed: {}", HumanDuration(start.elapsed()));

        let path_solution_teamid_json: PathBuf = config.arc_repository_data().join(Path::new(SOLUTIONS_FILENAME));

        let mut instance = Self { 
            config,
            context,
            model_item_vec: vec!(),
            program_item_vec: vec!(),
            locked_instruction_hashset: HashSet::new(),
            path_solution_teamid_json: path_solution_teamid_json,
        };
        instance.load_arc_models()?;
        instance.load_programs()?;
        instance.init_locked_instruction_hashset()?;
        Ok(instance)
    }

    fn load_arc_models(&mut self) -> anyhow::Result<()> {
        let path: PathBuf = self.config.arc_repository_data();
        let paths: Vec<PathBuf> = find_json_files_recursively(&path);
        println!("arc_repository_data. number of json files: {}", paths.len());

        let mut model_item_vec = Vec::<ModelItem>::new();
        for path in &paths {
            let model = match Model::load_with_json_file(path) {
                Ok(value) => value,
                Err(error) => {
                    error!("Ignoring file. Cannot parse arc_json_model file. path: {:?} error: {:?}", path, error);
                    continue;
                }
            };
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

    pub const INSTRUCTIONS_TO_LOCK: &'static str = r#"
    mov $80,$97 ; set iteration counter = length of "train" vector
    mov $81,100 ; address of first training data train[0].input
    mov $82,101 ; address of first training data train[0].output
    lps $80
      mov $0,$$81 ; load train[x].input image
      mov $1,$$82 ; load train[x].output image
    
      ; do stuff
      
      ; next iteration
      add $81,10 ; jump to address of next training input image
      add $82,10 ; jump to address of next training output image
    lpe
    "#;

    fn init_locked_instruction_hashset(&mut self) -> anyhow::Result<()> {
        self.insert_program_into_locked_instruction_hashset(RunWithProgram::SIMPLE_PROGRAM_PRE)?;
        self.insert_program_into_locked_instruction_hashset(RunWithProgram::SIMPLE_PROGRAM_POST)?;
        self.insert_program_into_locked_instruction_hashset(Self::INSTRUCTIONS_TO_LOCK)?;
        Ok(())
    }

    fn insert_program_into_locked_instruction_hashset<S: AsRef<str>>(&mut self, program: S) -> anyhow::Result<()> {
        let program_str: &str = program.as_ref();
        let parsed_program: ParsedProgram = ParsedProgram::parse_program(program_str)
            .map_err(|e| anyhow::anyhow!("parse with program: {:?}. error: {:?}", program_str, e))?;
        for instruction in &parsed_program.instruction_vec {
            let s: String = instruction.to_string();
            self.locked_instruction_hashset.insert(s);
        }
        Ok(())
    }

    fn mutate_program(&self, program_item: &ProgramItem, random_seed: u64, mutation_count: usize) -> anyhow::Result<ProgramItem> {
        let mut genome = Genome::new();
        genome.append_message(format!("template: {:?}", program_item.id.file_name()));

        let mut rng: StdRng = StdRng::seed_from_u64(random_seed);

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

        // println!("; INPUT PROGRAM\n; filename: {:?}\n\n{}", program_item.id.file_name(), initial_parsed_program);

        let mut genome_vec: Vec<GenomeItem> = initial_parsed_program.to_genome_item_vec();

        // locking rows that are not to be mutated
        for genome_item in genome_vec.iter_mut() {
            let program_line: String = genome_item.to_line_string();
            if self.locked_instruction_hashset.contains(&program_line) {
                genome_item.set_mutation_locked(true);
            }
        }

        genome.set_genome_vec(genome_vec);

        
        let mut dependency_manager: DependencyManager = RunWithProgram::create_dependency_manager();

        let max_number_of_retries = 40;
        let mut number_of_mutations: usize = 0;
        for _ in 0..max_number_of_retries {
            let mutate_success: bool = genome.mutate(&mut rng, &self.context);
            if !mutate_success {
                continue;
            }
            number_of_mutations += 1;

            if number_of_mutations < mutation_count {
                continue;
            }

            let parsed_program: ParsedProgram = genome.to_parsed_program();
            let program_runner: ProgramRunner = dependency_manager.parse_stage2(ProgramId::ProgramWithoutId, &parsed_program)
                .map_err(|e| anyhow::anyhow!("parse_stage2 with program: {:?}. error: {:?}", genome.to_string(), e))?;
    
            let mut serializer = ProgramSerializer::new();
            serializer.append_comment("Submitted by Simon Strandgaard");
            serializer.append_comment("Program Type: advanced");
            serializer.append_empty_line();
            program_runner.serialize(&mut serializer);
            serializer.append_empty_line();
            for message in genome.message_vec() {
                serializer.append_comment(message);
            }
            serializer.append_empty_line();
            let candidate_program: String = serializer.to_string();
            println!("; ------\n\n{}", candidate_program);

            let mutated_program_item = ProgramItem {
                id: ProgramItemId::None,
                program_string: candidate_program,
                program_type: ProgramType::Advance,
                number_of_models: 0,
            };

            return Ok(mutated_program_item);
        }

        Err(anyhow::anyhow!("unable to create a mutation in {} attempts", max_number_of_retries))
    }

    #[allow(dead_code)]
    fn genome_experiments(&self) -> anyhow::Result<()> {
        for program_item in &self.program_item_vec {
            let random_seed: u64 = 0;
            let mutation_count: usize = 1;
            let _ = self.mutate_program(&program_item.borrow(), random_seed, mutation_count)?;
            println!("break after first iteration");
            break;
        }
        Ok(())
    }

    fn read_solutions_json(&self) -> anyhow::Result<Tasks> {
        let solution_teamid_json_string: String = match fs::read_to_string(&self.path_solution_teamid_json) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("something went wrong reading the file: {:?} error: {:?}", self.path_solution_teamid_json, error));
            }
        };
        let tasks: Tasks = match serde_json::from_str(&solution_teamid_json_string) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Could not parse archaton_solution_json file, path: {:?} error: {:?} json: {:?}", self.path_solution_teamid_json, error, solution_teamid_json_string));
            }
        };
        Ok(tasks)
    }

    fn write_solutions_json(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn run(&mut self, verbose: bool) -> anyhow::Result<()> {
        // self.genome_experiments()?;
        // return Ok(());

        let verify_test_output = false;
        println!("verify_test_output: {:?}", verify_test_output);

        println!("initial model_item_vec.len: {:?}", self.model_item_vec.len());


        let initial_tasks: Tasks = match self.read_solutions_json() {
            Ok(value) => value,
            Err(error) => {
                error!("Starting out with zero tasks. Unable to load existing solutions file: {:?}", error);
                vec!()
            }
        };
        println!("initial_tasks.len: {}", initial_tasks.len());

        let mut task_names_to_ignore = HashSet::<String>::new();
        for task in &initial_tasks {
            task_names_to_ignore.insert(task.task_name.clone());
        }

        let mut number_of_disabled_model_items: usize = 0;
        for model_item in self.model_item_vec.iter_mut() {
            let file_stem: String = model_item.id.file_stem();
            if task_names_to_ignore.contains(&file_stem) {
                model_item.enabled = false;
                number_of_disabled_model_items += 1;
            }
        }
        println!("number_of_disabled_model_items: {:?}", number_of_disabled_model_items);


        // return Ok(());

        let path_solutions_csv = self.config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));
        let path_programs = self.config.loda_arc_challenge_repository_programs();

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

        if scheduled_program_item_vec.is_empty() {
            let number_of_mutations: u64 = 40;

            for program_item in &self.program_item_vec {
                for i in 0..number_of_mutations {
                    let random_seed: u64 = i + 200;
                    let mutation_count: usize = ((i % 4) + 1) as usize;
                    match self.mutate_program(&program_item.borrow(), random_seed, mutation_count) {
                        Ok(mutated_program) => {
                            scheduled_program_item_vec.push(Rc::new(RefCell::new(mutated_program)));
                        },
                        Err(error) => {
                            debug!("Skipping this mutation. The original program cannot be mutated. {:?}", error);
                            break;
                        }
                    }
                }
            }
            println!("scheduled_program_item_vec.len: {}", scheduled_program_item_vec.len());
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

        let mut current_tasks: Tasks = initial_tasks;

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
            let instance = RunWithProgram::new(model, verify_test_output).expect("RunWithProgram");

            let pairs: Vec<ImagePair> = model_item.model.images_all().expect("pairs");
    
            let mut found_one_or_more_solutions = false;
            for (_program_index, program_item) in scheduled_program_item_vec.iter_mut().enumerate() {

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

                    let model_filename: String = model_item.id.file_name();
                    let mut program_filename: String = program_item.borrow().id.file_name();

                    let is_mutation: bool = program_item.borrow().id == ProgramItemId::None;
                    let is_first: bool = program_item.borrow().number_of_models == 0;
                    if is_mutation && is_first {
                        let content: String = program_item.borrow().program_string.clone();
                        let mut s: String = model_filename.clone();
                        s = s.replace(".json", "-x.asm");
                        program_filename = s.clone();
                        let path = path_programs.join(Path::new(&s));
                        let mut file = File::create(&path)?;
                        file.write_all(content.as_bytes())?;
                    }

                    let record = Record {
                        model_filename: model_filename,
                        program_filename,
                    };
                    record_vec.push(record);

                    program_item.borrow_mut().number_of_models += 1;

                    let message = format!("program: {:?} is a solution for model: {:?}", program_item.borrow().id, model_item.id);
                    pb.println(message);
                }
            }

            if found_one_or_more_solutions {
                count_match += 1;
            } else {
                count_mismatch += 1;
            }

            // found_one_or_more_solutions = true;
            if found_one_or_more_solutions {
                let json: String = match serde_json::to_string(&current_tasks) {
                    Ok(value) => value,
                    Err(error) => {
                        error!("unable to serialize tasks to json: {:?}", error);
                        continue;
                    }
                };
                match fs::write(&self.path_solution_teamid_json, json) {
                    Ok(()) => {},
                    Err(error) => {
                        error!("unable to save solutions file. path: {:?} error: {:?}", self.path_solution_teamid_json, error);
                        continue;
                    }
                }
                println!("updated solutions file: tasks.len(): {}", current_tasks.len());
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
            if program_item.borrow().id == ProgramItemId::None {
                continue;
            }
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

    fn file_stem(&self) -> String {
        match self {
            ModelItemId::None => {
                return "None".to_string();
            },
            ModelItemId::Path { path } => {
                match path.file_stem() {
                    Some(value) => {
                        return value.to_string_lossy().to_string();
                    },
                    None => {
                        return "Path without a file_stem".to_string();
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
#[derive(Clone, Debug, PartialEq)]
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
