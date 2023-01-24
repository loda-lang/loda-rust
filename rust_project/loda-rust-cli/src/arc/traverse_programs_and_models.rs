use super::{Model, ImagePair};
use super::{RunWithProgram, RunWithProgramResult};
use super::{Prediction, TestItem, TaskItem, Tasks};
use crate::analytics::{AnalyticsDirectory, Analytics};
use crate::config::Config;
use crate::common::{find_json_files_recursively, parse_csv_file, create_csv_file};
use crate::common::find_asm_files_recursively;
use crate::mine::{Genome, GenomeItem, ToGenomeItemVec, CreateGenomeMutateContextMode, create_genome_mutate_context, GenomeMutateContext};
use bloomfilter::*;
use anyhow::Context;
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::{ProgramSerializer, ProgramId, ProgramRunner};
use loda_rust_core::parser::ParsedProgram;
use chrono::prelude::*;
use std::time::{SystemTime, Duration};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{PathBuf, Path};
use std::rc::Rc;
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};

static SOLUTIONS_FILENAME: &str = "solution_notXORdinary.json";

pub struct TraverseProgramsAndModels {
    config: Config,
    context: GenomeMutateContext,
    model_item_vec: Vec<Rc<RefCell<ModelItem>>>,
    program_item_vec: Vec<Rc<RefCell<ProgramItem>>>,
    locked_instruction_hashset: HashSet<String>,
    path_solution_dir: PathBuf,
    path_solution_teamid_json: PathBuf,
}

impl TraverseProgramsAndModels {
    pub fn arc_competition() -> anyhow::Result<()> {
        let mut instance = TraverseProgramsAndModels::new()?;
        instance.run_arc_competition()?;
        Ok(())
    }

    pub fn eval_single_puzzle_with_all_existing_solutions(pattern: String) -> anyhow::Result<()> {
        let instance = TraverseProgramsAndModels::new()?;
        instance.eval_single_puzzle_with_all_existing_solutions_inner(&pattern)?;
        Ok(())
    }

    pub fn check_all_existing_solutions() -> anyhow::Result<()> {
        let instance = TraverseProgramsAndModels::new()?;
        instance.check_all_existing_solutions_inner()?;
        Ok(())
    }

    /// Compare all puzzles with all solutions and output a CSV file
    pub fn generate_solution_csv() -> anyhow::Result<()> {
        let mut instance = TraverseProgramsAndModels::new()?;
        instance.generate_solution_csv_inner()?;
        Ok(())
    }

    fn new() -> anyhow::Result<Self> {
        Analytics::arc_run_if_expired()?;

        let config = Config::load();

        println!("loading genome mutate context");
        let start = Instant::now();

        let analytics_directory = AnalyticsDirectory::new(
            config.analytics_arc_dir()
        ).with_context(||"unable to create AnalyticsDirectory instance")?;    

        let context: GenomeMutateContext = create_genome_mutate_context(CreateGenomeMutateContextMode::ARC, analytics_directory)?;
        println!("loaded genome mutate context. elapsed: {}", HumanDuration(start.elapsed()));

        let path_solution_dir: PathBuf = config.arc_repository_data().join(Path::new("solution"));
        let path_solution_teamid_json: PathBuf = path_solution_dir.join(Path::new(SOLUTIONS_FILENAME));

        let mut instance = Self { 
            config,
            context,
            model_item_vec: vec!(),
            program_item_vec: vec!(),
            locked_instruction_hashset: HashSet::new(),
            path_solution_dir,
            path_solution_teamid_json,
        };
        instance.load_puzzle_files()?;
        instance.load_solution_files()?;
        instance.init_locked_instruction_hashset()?;
        Ok(instance)
    }

    fn files_to_keep(path: &PathBuf) -> bool {
        if let Some(filename) = path.file_name() {
            if filename.to_string_lossy() == SOLUTIONS_FILENAME {
                debug!("ignoring the SOLUTIONS_FILENAME. path: {:?}", path);
                return false;
            }
        }
        true
    }

    /// Load all the ARC puzzle files into memory
    fn load_puzzle_files(&mut self) -> anyhow::Result<()> {
        let repo_path: PathBuf = self.config.arc_repository_data();
        let all_json_paths: Vec<PathBuf> = find_json_files_recursively(&repo_path);

        // Ignore the solutions json file, since it's not an ARC puzzle json file
        let paths: Vec<PathBuf> = all_json_paths
            .into_iter()
            .filter(Self::files_to_keep)
            .collect();
        debug!("arc_repository_data. number of json files: {}", paths.len());

        let mut model_item_vec: Vec<Rc<RefCell<ModelItem>>> = vec!();
        for path in &paths {
            let model = match Model::load_with_json_file(path) {
                Ok(value) => value,
                Err(error) => {
                    error!("Ignoring file. Cannot parse arc_json_model file. path: {:?} error: {:?}", path, error);
                    continue;
                }
            };
            let instance = ModelItem {
                id: ModelItemId::Path { path: path.clone() },
                model,
            };
            let item = Rc::new(RefCell::new(instance));
            model_item_vec.push(item);
        }
        if model_item_vec.len() != paths.len() {
            error!("Skipped some models. paths.len()={}, but model_item_vec.len()={}", paths.len(), model_item_vec.len());
        }
        self.model_item_vec = model_item_vec;
        Ok(())
    }

    /// Load all `.asm` programs into memory
    fn load_solution_files(&mut self) -> anyhow::Result<()> {
        let path: PathBuf = self.config.loda_arc_challenge_repository_programs();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&path);
        debug!("loda_arc_challenge_repository_programs. number of asm files: {}", paths.len());

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

    const INSTRUCTIONS_TO_LOCK: &'static str = r#"
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

    /// The `bloom` parameter, helps ensure that the mutated programs are different than previously tried out programs.
    fn mutate_program(&self, program_item: &ProgramItem, random_seed: u64, mutation_count: usize, bloom: &mut Bloom::<String>) -> anyhow::Result<ProgramItem> {
        let mut genome = Genome::new();
        genome.append_message(format!("template: {:?}", program_item.id.file_name()));

        let mut rng: StdRng = StdRng::seed_from_u64(random_seed);

        let initial_parsed_program: ParsedProgram = program_item.parsed_program()?;

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

        let max_number_of_retries = 100;
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
            let bloom_key: String = parsed_program.to_string();
            if bloom.check(&bloom_key) {
                // It's likely that this program mutation has already has been explored in the past. Ignore it.
                // debug!("skip program mutation that already have been tried out");
                continue;                
            }

            // This program mutation is not contained in the bloomfilter.
            // Proceed making a program out of it.

            bloom.set(&bloom_key);
    
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
            // println!("; ------\n\n{}", candidate_program);

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

    fn eval_single_puzzle_with_all_existing_solutions_inner(&self, pattern: &String) -> anyhow::Result<()> {
        let verbose = false;
        let verify_test_output = true;

        // Extract the puzzle model
        let mut candidate_model_items = Vec::<ModelItem>::new();
        for model_item in &self.model_item_vec {
            let file_stem: String = model_item.borrow().id.file_stem();
            if file_stem.contains(pattern) {
                candidate_model_items.push(model_item.borrow().clone());
            }
        }
        // There is supposed to be exactly 1 puzzle with this name.
        if candidate_model_items.len() >= 2 {
            return Err(anyhow::anyhow!("There are {} puzzles that matches the pattern, please specify a longer pattern: {:?}", candidate_model_items.len(), pattern));
        }
        let model_item: ModelItem = match candidate_model_items.pop() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("No puzzle matches the specified pattern: {:?}", pattern));
            }
        };

        let pairs_train: Vec<ImagePair> = model_item.model.images_train().expect("pairs");
        let pairs_test: Vec<ImagePair> = model_item.model.images_test().expect("pairs");
        println!("Evaluating the puzzle: {:?} train-pairs: {} test-pairs: {}", model_item.id, pairs_train.len(), pairs_test.len());

        let mut count_ok: usize = 0;
        let mut count_error_compute: usize = 0;
        let mut count_error_incorrect: usize = 0;
        let mut count_partial_match: usize = 0;
        let mut count_dangerous_false_positive: usize = 0;

        let pb = ProgressBar::new(self.program_item_vec.len() as u64 + 1);
        for (program_index, program_item) in self.program_item_vec.iter().enumerate() {
            pb.inc(1);

            let instance = RunWithProgram::new(model_item.model.clone(), verify_test_output).expect("RunWithProgram");

            let result: RunWithProgramResult;
            match program_item.borrow().program_type {
                ProgramType::Simple => {
                    result = match instance.run_simple(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            if verbose {
                                pb.println(format!("ERROR: in row {}. program: {:?}. Run failed with error {:?}", program_index, program_item, error));
                            }
                            continue;
                        }
                    };
                },
                ProgramType::Advance => {
                    result = match instance.run_advanced(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            if verbose {
                                pb.println(format!("ERROR: in row {}. program: {:?}. Run failed with error {:?}", program_index, program_item, error));
                            }
                            continue;
                        }
                    };
                }
            }

            if verbose {
                let s = format!("model: {:?} program: {:?} result: {:?}", model_item.id, program_item.borrow().id, result);
                pb.println(s);
            }

            let expected = format!("({},{})", pairs_train.len(), pairs_test.len());
            let actual = format!("({},{})", result.count_train_correct(), result.count_test_correct());
            if actual != expected {
                if result.count_train_correct() == pairs_train.len() && result.count_test_correct() != pairs_test.len() {
                    pb.println(format!("Dangerous false positive. Expected {} but got {}. {:?}", expected, actual, program_item.borrow().id.file_name()));
                    count_dangerous_false_positive += 1;
                } else {
                    let count_correct = result.count_train_correct() + result.count_test_correct();
                    if count_correct > 0 {
                        count_partial_match += 1;
                        pb.println(format!("Partial solution. Expected {} but got {}. {:?}", expected, actual, program_item.borrow().id.file_name()));
                    }
                }
                if verbose {
                    pb.println(format!("ERROR: in row {}. program: {:?}. Expected {}, but got {}", program_index, program_item, expected, actual));
                }
                count_error_incorrect += 1;
                continue;
            }

            count_ok += 1;
            pb.println(format!("Found a solution: {:?}", program_item.borrow().id.file_name()));
        }
        pb.finish_and_clear();

        debug!("STATS:");
        debug!("count_partial_match: {}", count_partial_match);
        debug!("count_error_compute: {}", count_error_compute);
        debug!("count_error_incorrect: {}", count_error_incorrect);
        if count_dangerous_false_positive > 0 {
            error!("Encountered {} dangerous false positive solutions. These are unwanted.", count_dangerous_false_positive);
        }

        if count_ok > 0 {
            let green_bold = Style::new().green().bold();        
            let s = format!("Status: Found {} solutions", count_ok);
            println!("{}", green_bold.apply_to(&s));
        } else {
            let green_bold = Style::new().red().bold();        
            println!("{}", green_bold.apply_to("Status: Found no solutions among the existing programs"));
        }
        Ok(())
    }

    fn check_all_existing_solutions_inner(&self) -> anyhow::Result<()> {
        let verbose = false;
        let verify_test_output = true;

        let path_solutions_csv = self.config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));
        if !path_solutions_csv.is_file() {
            return Err(anyhow::anyhow!("there is no existing solutions.csv file, so the solutions cannot be checked. path_solutions_csv: {:?}", path_solutions_csv));
        }

        let record_vec: Vec<Record> = Record::load_record_vec(&path_solutions_csv)?;
        debug!("solutions.csv: number of rows: {}", record_vec.len());

        let mut count_ok: usize = 0;
        let mut count_error_other: usize = 0;
        let mut count_error_duplicate: usize = 0;
        let mut count_error_compute: usize = 0;
        let mut count_error_incorrect: usize = 0;

        let mut unique_records = HashSet::<Record>::new();

        let pb = ProgressBar::new(record_vec.len() as u64);
        for (record_index, record) in record_vec.iter().enumerate() {
            if record_index > 0 {
                pb.inc(1);
            }

            // The rows are supposed to be unique
            if unique_records.contains(&record) {
                pb.println(format!("ERROR: in row {}. Expected unique rows, but this is a duplicate.", record_index));
                count_error_duplicate += 1;
                continue;
            }
            unique_records.insert(record.clone());

            // Extract the puzzle model
            let mut candidate_model_items = Vec::<ModelItem>::new();
            for model_item in &self.model_item_vec {
                let file_name: String = model_item.borrow().id.file_name();
                if file_name == record.model_filename {
                    candidate_model_items.push(model_item.borrow().clone());
                }
            }
            // There is supposed to be exactly 1 puzzle with this name.
            if candidate_model_items.len() >= 2 {
                pb.println(format!("ERROR: in row {}. Expected 1 puzzle for row in csv file, but got multiple.", record_index));
                count_error_other += 1;
                continue;
            }
            let model_item: ModelItem = match candidate_model_items.pop() {
                Some(value) => value,
                None => {
                    pb.println(format!("ERROR: in row {}. Missing puzzle.", record_index));
                    count_error_other += 1;
                    continue;
                }
            };

            // Extract the solution model
            let mut candidate_programs = Vec::<Rc::<RefCell::<ProgramItem>>>::new();
            let program_filename: String = record.program_filename.clone();
            for program_item in &self.program_item_vec {
                let this_file_name: String = program_item.borrow_mut().id.file_name();
                if this_file_name == program_filename {
                    candidate_programs.push(program_item.clone());
                }
            }
            // There is supposed to be exactly 1 solution with this name.
            if candidate_programs.len() >= 2 {
                pb.println(format!("ERROR: in row {}. Expected 1 solution for row in csv file, but got multiple.", record_index));
                count_error_other += 1;
                continue;
            }
            let program_item: Rc<RefCell<ProgramItem>> = match candidate_programs.pop() {
                Some(value) => value,
                None => {
                    pb.println(format!("ERROR: in row {}. Missing solution.", record_index));
                    count_error_other += 1;
                    continue;
                }
            };
    
            let instance = RunWithProgram::new(model_item.model.clone(), verify_test_output).expect("RunWithProgram");
            let pairs_train: Vec<ImagePair> = model_item.model.images_train().expect("pairs");
            let pairs_test: Vec<ImagePair> = model_item.model.images_test().expect("pairs");

            let result: RunWithProgramResult;
            match program_item.borrow().program_type {
                ProgramType::Simple => {
                    result = match instance.run_simple(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            pb.println(format!("ERROR: in row {}. record: {:?}. Run failed with error {:?}", record_index, record, error));
                            continue;
                        }
                    };
                },
                ProgramType::Advance => {
                    result = match instance.run_advanced(&program_item.borrow().program_string) {
                        Ok(value) => value,
                        Err(error) => {
                            count_error_compute += 1;
                            pb.println(format!("ERROR: in row {}. record: {:?}. Run failed with error {:?}", record_index, record, error));
                            continue;
                        }
                    };
                }
            }

            if verbose {
                let s = format!("model: {:?} program: {:?} result: {:?}", model_item.id, program_item.borrow().id, result);
                pb.println(s);
            }

            let expected = format!("({},{})", pairs_train.len(), pairs_test.len());
            let actual = format!("({},{})", result.count_train_correct(), result.count_test_correct());
            if actual != expected {
                pb.println(format!("ERROR: in row {}. record: {:?}. Expected {}, but got {}", record_index, record, expected, actual));
                count_error_incorrect += 1;
                continue;
            }

            count_ok += 1;
        }
        pb.finish_and_clear();

        if count_ok == record_vec.len() {
            let green_bold = Style::new().green().bold();        
            println!("{}", green_bold.apply_to("Status: All solutions passes ok"));
        } else {
            println!("count_ok: {}", count_ok);
            println!("count_error_other: {}", count_error_other);
            println!("count_error_duplicate: {}", count_error_duplicate);
            println!("count_error_compute: {}", count_error_compute);
            println!("count_error_incorrect: {}", count_error_incorrect);
            let sum: usize = count_error_other + count_error_duplicate + count_error_compute + count_error_incorrect;
            error!("There are {} errors that needs to be resolved. csv file: {:?}", sum, path_solutions_csv);
        }
        Ok(())
    }

    fn generate_solution_csv_inner(&mut self) -> anyhow::Result<()> {
        let verbose = false;
        let verify_test_output = true;

        let path_solutions_csv = self.config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));

        let mut record_vec = Vec::<Record>::new();
        Record::save_solutions_csv(&record_vec, &path_solutions_csv);

        let start = Instant::now();

        let mut count_ok: usize = 0;
        let mut count_dangerous_false_positive: usize = 0;
        let mut count_partial_match: usize = 0;
        let mut count_incorrect: usize = 0;
        let mut count_compute_error: usize = 0;

        let multi_progress = MultiProgress::new();
        let progress_style: ProgressStyle = ProgressStyle::with_template(
            "{prefix} [{elapsed_precise}] {wide_bar} {pos:>5}/{len:5} {msg}",
        )?;

        let pb = multi_progress.add(ProgressBar::new(self.model_item_vec.len() as u64));
        pb.set_style(progress_style.clone());
        pb.set_prefix("Puzzle  ");
        pb.tick();

        for (model_index, model_item) in self.model_item_vec.iter_mut().enumerate() {
            if model_index > 0 {
                pb.inc(1);
            }

            let print_prefix_puzzle_id: String = format!("Puzzle#{} {:?}", model_index, model_item.borrow().id.file_name());

            let model: Model = model_item.borrow().model.clone();
            let pairs_train: Vec<ImagePair> = model.images_train().expect("pairs");
            let pairs_test: Vec<ImagePair> = model.images_test().expect("pairs");

            let instance = RunWithProgram::new(model, verify_test_output).expect("RunWithProgram");
    
            let pb2 = multi_progress.insert_after(&pb, ProgressBar::new( self.program_item_vec.len() as u64));
            pb2.set_style(progress_style.clone());
            pb2.set_prefix("Solution");
            pb2.tick();
            for (program_index, program_item) in self.program_item_vec.iter_mut().enumerate() {
                if program_index > 0 {
                    pb2.inc(1);
                }

                let result: RunWithProgramResult;
                match program_item.borrow().program_type {
                    ProgramType::Simple => {
                        result = match instance.run_simple(&program_item.borrow().program_string) {
                            Ok(value) => value,
                            Err(error) => {
                                count_compute_error += 1;
                                if verbose {
                                    error!("model: {:?} simple-program: {:?} error: {:?}", model_item.borrow().id, program_item.borrow().id, error);
                                }
                                continue;
                            }
                        };
                    },
                    ProgramType::Advance => {
                        result = match instance.run_advanced(&program_item.borrow().program_string) {
                            Ok(value) => value,
                            Err(error) => {
                                count_compute_error += 1;
                                if verbose {
                                    error!("model: {:?} advanced-program: {:?} error: {:?}", model_item.borrow().id, program_item.borrow().id, error);
                                }
                                continue;
                            }
                        };
                    }
                }

                if verbose {
                    let s = format!("model: {:?} program: {:?} result: {:?}", model_item.borrow().id, program_item.borrow().id, result);
                    pb.println(s);
                }

                let expected = format!("({},{})", pairs_train.len(), pairs_test.len());
                let actual = format!("({},{})", result.count_train_correct(), result.count_test_correct());
                if actual != expected {
                    if result.count_train_correct() == pairs_train.len() && result.count_test_correct() != pairs_test.len() {
                        pb.println(format!("{} - Dangerous false positive. Expected {} but got {}. {:?}", print_prefix_puzzle_id, expected, actual, program_item.borrow().id.file_name()));
                        count_dangerous_false_positive += 1;
                        continue;
                    }
                    let count_correct = result.count_train_correct() + result.count_test_correct();
                    if count_correct > 0 {
                        count_partial_match += 1;
                        pb.println(format!("{} - Partial solution. Expected {} but got {}. {:?}", print_prefix_puzzle_id, expected, actual, program_item.borrow().id.file_name()));
                        continue;
                    }
                    if verbose {
                        pb.println(format!("ERROR: in row {}. program: {:?}. Expected {}, but got {}", program_index, program_item, expected, actual));
                    }
                    count_incorrect += 1;
                    continue;
                }
    
                pb.println(format!("{} - Found a solution: {:?}", print_prefix_puzzle_id, program_item.borrow().id.file_name()));
                count_ok += 1;
                program_item.borrow_mut().number_of_models += 1;

                let model_filename: String = model_item.borrow().id.file_name();
                let program_filename: String = program_item.borrow().id.file_name();
                let record = Record {
                    model_filename: model_filename,
                    program_filename,
                };
                record_vec.push(record);
                Record::save_solutions_csv(&record_vec, &path_solutions_csv);
            }

            pb2.finish_and_clear();
        }
        pb.finish_and_clear();
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} processing all puzzles with all solutions in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        Record::save_solutions_csv(&record_vec, &path_solutions_csv);

        // Print out names of unused programs that serves no purpose and can be removed
        let mut unused_programs = Vec::<String>::new();
        for program_item in &self.program_item_vec {
            if program_item.borrow().id == ProgramItemId::None {
                continue;
            }
            if program_item.borrow().number_of_models == 0 {
                let filename: String = program_item.borrow().id.file_name();
                unused_programs.push(filename);
            }
        }
        if !unused_programs.is_empty() {
            error!("There are {} unused programs. These doesn't solve any of the models, and can be removed.", unused_programs.len());
            for filename in unused_programs {
                println!("UNUSED {:?}", filename);
            }
        }
    
        // Stats
        println!("row count in solutions csv file: {}", record_vec.len());
        println!("count_ok: {}", count_ok);
        println!("count_incorrect: {}", count_incorrect);
        println!("count_compute_error: {}", count_compute_error);
        println!("count_partial_match: {}", count_partial_match);
        if count_dangerous_false_positive > 0 {
            error!("count_dangerous_false_positive: {}", count_dangerous_false_positive);
        } else {
            println!("count_dangerous_false_positive: {}", count_dangerous_false_positive);
        }
        Ok(())
    }

    /// The `bloom` parameter, helps ensure that the mutated programs are different than previously tried out programs.
    fn create_mutated_programs(&self, random_seed: u64, mutation_count: usize, bloom: &mut Bloom::<String>) -> Vec<Rc<RefCell<ProgramItem>>> {
        let mut result_program_item_vec: Vec<Rc<RefCell<ProgramItem>>> = Vec::<Rc<RefCell<ProgramItem>>>::new();
        for program_item in &self.program_item_vec {
            match self.mutate_program(&program_item.borrow(), random_seed, mutation_count, bloom) {
                Ok(mutated_program) => {
                    result_program_item_vec.push(Rc::new(RefCell::new(mutated_program)));
                },
                Err(error) => {
                    debug!("Skipping this mutation. The original program cannot be mutated. {:?}", error);
                    break;
                }
            }
        }
        result_program_item_vec
    }

    fn run_arc_competition(&mut self) -> anyhow::Result<()> {
        let duration: Duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Duration since UNIX_EPOCH failed");
        let initial_random_seed: u64 = duration.as_secs();

        println!("initial model_item_vec.len: {:?}", self.model_item_vec.len());
        let mut scheduled_model_item_vec: Vec<Rc<RefCell<ModelItem>>> = self.model_item_vec.clone();

        let initial_tasks: Tasks = match self.read_solutions_json() {
            Ok(value) => value,
            Err(error) => {
                error!("Starting out with zero tasks. Unable to load existing solutions file: {:?}", error);
                vec!()
            }
        };
        println!("initial_tasks.len: {}", initial_tasks.len());

        let mut puzzle_names_to_ignore = HashSet::<String>::new();
        for task in &initial_tasks {
            puzzle_names_to_ignore.insert(task.task_name.clone());
        }

        let path_solutions_csv = self.config.loda_arc_challenge_repository().join(Path::new("solutions.csv"));
        let path_programs = self.config.loda_arc_challenge_repository_programs();

        let mut record_vec = Vec::<Record>::new();

        let ignore_puzzles_with_a_solution: bool = path_solutions_csv.is_file();
        if ignore_puzzles_with_a_solution {
            record_vec = Record::load_record_vec(&path_solutions_csv)?;
            debug!("solutions.csv: number of rows: {}", record_vec.len());
    
            for record in &record_vec {
                let puzzle_filename_with_json_suffix: String = record.model_filename.clone();
                let puzzle_filename = puzzle_filename_with_json_suffix.replace(".json", "");
                puzzle_names_to_ignore.insert(puzzle_filename);
            }
        }
        debug!("puzzle_names_to_ignore: {:?}", puzzle_names_to_ignore);

        scheduled_model_item_vec = ModelItem::remove_model_items_where_filestem_contains(
            &scheduled_model_item_vec, 
            &puzzle_names_to_ignore
        );

        // println!("scheduled_model_item_vec.len(): {}", scheduled_model_item_vec.len());

        // Summary of what puzzles are to be solved
        {
            let mut number_of_solved_puzzles: usize = 0;
            let mut number_of_unsolved_puzzles: usize = 0;
            for model_item in &self.model_item_vec {
                let mut is_same = false;
                for model_item2 in &scheduled_model_item_vec {
                    if Rc::ptr_eq(&model_item, &model_item2) {
                        is_same = true;
                        break;
                    }
                }
                if is_same {
                    number_of_unsolved_puzzles += 1;
                } else {
                    number_of_solved_puzzles += 1;
                }
            }
            println!("puzzles solved: {}", number_of_solved_puzzles);
            println!("puzzles unsolved: {}", number_of_unsolved_puzzles);
        }

        let current_tasks: Tasks = initial_tasks;
        save_solutions(
            &self.path_solution_dir,
            &self.path_solution_teamid_json,
            &current_tasks
        );

        let bloom_items_count = 1000000;
        let false_positive_rate = 0.01;
        let mut bloom = Bloom::<String>::new_for_fp_rate(bloom_items_count, false_positive_rate);

        // Register the existing programs in the bloomfilter, so that these never gets suggested as a candidate solution
        for program_item in &self.program_item_vec {
            match program_item.borrow().bloom_key() {
                Ok(bloom_key) => {
                    bloom.set(&bloom_key);
                },
                Err(error) => {
                    error!("unable to create bloom_key for program: {:?}", error);
                }
            }
        }

        let mut state = BatchState {
            path_solutions_csv,
            path_programs,
            path_solution_dir: self.path_solution_dir.clone(),
            path_solution_teamid_json: self.path_solution_teamid_json.clone(),
            scheduled_model_item_vec,
            scheduled_program_item_vec: vec!(),
            record_vec,
            current_tasks,
        };

        // loop until all puzzles have been solved
        let mut mutation_index: u64 = 0;
        while !state.scheduled_model_item_vec.is_empty() {

            let datetime: DateTime<Utc> = Utc::now();
            let timestamp = datetime.to_rfc3339_opts(SecondsFormat::Secs, true).to_string();
        
            println!("{} - Mutation: {}", timestamp, mutation_index);

            let random_seed: u64 = (initial_random_seed * 0x1000000) + mutation_index;
            // debug!("random_seed: {:#x}", random_seed);


            // Create new mutated programs in every iteration
            let mutation_count: usize = ((random_seed % 4) + 1) as usize;
            state.scheduled_program_item_vec = self.create_mutated_programs(random_seed, mutation_count, &mut bloom);

            // Evaluate all puzzles with all candidate programs
            state.run_one_batch()?;

            mutation_index += 1;
        }

        println!("Done!");

        Ok(())
    }
}

struct BatchState {
    path_solutions_csv: PathBuf,
    path_programs: PathBuf,
    path_solution_dir: PathBuf,
    path_solution_teamid_json: PathBuf,
    scheduled_model_item_vec: Vec<Rc<RefCell<ModelItem>>>,
    scheduled_program_item_vec: Vec<Rc<RefCell<ProgramItem>>>,
    record_vec: Vec::<Record>,
    current_tasks: Tasks,
}

impl BatchState {
    fn run_one_batch(&mut self) -> anyhow::Result<()> {
        let verify_test_output = false;
        let verbose = false;

        let mut remove_model_items: Vec<Rc<RefCell<ModelItem>>> = vec!();

        let multi_progress = MultiProgress::new();
        let progress_style: ProgressStyle = ProgressStyle::with_template(
            "{prefix} [{elapsed_precise}] {wide_bar} {pos:>5}/{len:5} {msg}",
        )?;

        let pb = multi_progress.add(ProgressBar::new(self.scheduled_model_item_vec.len() as u64));
        pb.set_style(progress_style.clone());
        pb.set_prefix("Puzzle  ");
        for (model_index, model_item) in self.scheduled_model_item_vec.iter_mut().enumerate() {
            if model_index > 0 {
                pb.inc(1);
            }
    
            let model: Model = model_item.borrow().model.clone();
            let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
            let instance = RunWithProgram::new(model, verify_test_output).expect("RunWithProgram");
    
            let pb2 = multi_progress.insert_after(&pb, ProgressBar::new(self.scheduled_program_item_vec.len() as u64));
            pb2.set_style(progress_style.clone());
            pb2.set_prefix("Solution");
            for (program_index, program_item) in self.scheduled_program_item_vec.iter_mut().enumerate() {
                if program_index > 0 {
                    pb.tick();
                    pb2.inc(1);
                }

                let result: RunWithProgramResult;
                match program_item.borrow().program_type {
                    ProgramType::Simple => {
                        result = match instance.run_simple(&program_item.borrow().program_string) {
                            Ok(value) => value,
                            Err(error) => {
                                if verbose {
                                    error!("model: {:?} simple-program: {:?} error: {:?}", model_item.borrow().id, program_item.borrow().id, error);
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
                                    error!("model: {:?} advanced-program: {:?} error: {:?}", model_item.borrow().id, program_item.borrow().id, error);
                                }
                                continue;
                            }
                        };
                    }
                }

                if verbose {
                    let s = format!("model: {:?} program: {:?} result: {:?}", model_item.borrow().id, program_item.borrow().id, result);
                    pb.println(s);
                }

                let count: usize = result.count_train_correct() + result.count_test_correct();
                if count != pairs.len() {
                    // This is not a solution. Proceed to the next candidate solution.
                    continue;
                }

                // This may be a solution.

                let model_filename: String = model_item.borrow().id.file_name();

                let program_filename: String;
                {
                    let content: String = program_item.borrow().program_string.clone();
                    let mut s: String = model_filename.clone();
                    s = s.replace(".json", "-x.asm");
                    program_filename = s.clone();
                    let path = self.path_programs.join(Path::new(&s));
                    let mut file = File::create(&path)?;
                    file.write_all(content.as_bytes())?;
                }

                let record = Record {
                    model_filename: model_filename,
                    program_filename,
                };
                self.record_vec.push(record);
                Record::save_solutions_csv(&self.record_vec, &self.path_solutions_csv);

                let message = format!("program: {:?} is a solution for model: {:?}", program_item.borrow().id, model_item.borrow().id);
                pb.println(message);

                let predictions: Vec<Prediction> = result.predictions().clone();
                let test_item = TestItem { 
                    output_id: 0,
                    number_of_predictions: predictions.len() as u8,
                    predictions: predictions,
                };

                let task_name: String = model_item.borrow().id.file_stem();
                let task_item = TaskItem {
                    task_name: task_name,
                    test_vec: vec![test_item],
                };
                self.current_tasks.push(task_item);
                save_solutions(
                    &self.path_solution_dir,
                    &self.path_solution_teamid_json,
                    &self.current_tasks
                );

                remove_model_items.push(Rc::clone(model_item));

                // This is a solution to this puzzle. No need to loop through the remaining programs.
                break;
            }
            pb2.finish_and_clear();
        }
        pb.finish_and_clear();

        // Remove solved puzzles from the scheduled_model_item_vec
        if !remove_model_items.is_empty() {
            self.scheduled_model_item_vec = ModelItem::remove_model_items(
                &self.scheduled_model_item_vec, 
                &remove_model_items
            );
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
}

impl ModelItem {
    fn remove_model_items_where_filestem_contains(
        model_item_vec: &Vec<Rc<RefCell<ModelItem>>>,
        names_for_removal: &HashSet<String>
    ) -> Vec<Rc<RefCell<ModelItem>>> {
        let mut result_items: Vec<Rc<RefCell<ModelItem>>> = vec!();
        for model_item in model_item_vec {
            let file_stem: String = model_item.borrow().id.file_stem();
            if !names_for_removal.contains(&file_stem) {
                result_items.push(Rc::clone(model_item));
            }
        }
        result_items
    }

    fn remove_model_items(
        model_item_vec: &Vec<Rc<RefCell<ModelItem>>>,
        model_item_vec_for_removal: &Vec<Rc<RefCell<ModelItem>>>
    ) -> Vec<Rc<RefCell<ModelItem>>> {
        if model_item_vec_for_removal.is_empty() {
            return model_item_vec.clone();
        }
        let count_before: usize = model_item_vec.len();
        let mut result_model_item_vec: Vec<Rc<RefCell<ModelItem>>> = vec!();
        for model_item in model_item_vec {
            let mut keep = true;
            for remove_model_item in model_item_vec_for_removal {
                if Rc::ptr_eq(&remove_model_item, &model_item) {
                    keep = false;
                    break;
                }
            }
            if keep {
                result_model_item_vec.push(Rc::clone(model_item));
            }
        }
        let count_after: usize = result_model_item_vec.len();
        if count_after > count_before {
            error!("Expected removal to shrink vector, but it grows. {} != {} + {}", count_before, count_after, model_item_vec_for_removal.len());
        }
        result_model_item_vec
    }
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

impl ProgramItem {
    fn parsed_program(&self) -> anyhow::Result<ParsedProgram> {
        let program_content: String;
        match self.program_type {
            ProgramType::Simple => {
                program_content = RunWithProgram::convert_simple_to_full(&self.program_string);
            },
            ProgramType::Advance => {
                program_content = self.program_string.clone();
            }
        }
        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&program_content) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("cannot parse the program: {:?}", error));
            }
        };
        Ok(parsed_program)
    }

    /// Returns a compacted version of the program, that is only intended for use in the bloomfilter.
    /// Inserts header/footer if it's a simple program. Keeps the program if it's an adavanced program.
    /// There are no comments or unneccessary spacing.
    fn bloom_key(&self) -> anyhow::Result<String> {
        let pp: ParsedProgram = self.parsed_program()?;
        let compact_program_string: String = pp.to_string();
        Ok(compact_program_string)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Serialize, PartialEq)]
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

    fn save_solutions_csv(record_vec: &Vec<Record>, path_csv: &Path) {
        let mut record_vec: Vec<Record> = record_vec.clone();
        record_vec.sort_unstable_by_key(|item| (item.model_filename.clone(), item.program_filename.clone()));
        match create_csv_file(&record_vec, &path_csv) {
            Ok(()) => {},
            Err(error) => {
                error!("Unable to save csv file: {:?}", error);
            }
        }
    }
}

fn save_solutions(path_solution_dir: &Path, path_solution_teamid_json: &Path, tasks: &Tasks) {
    if !path_solution_dir.exists() {
            match fs::create_dir(path_solution_dir) {
            Ok(_) => {},
            Err(err) => {
                panic!("Unable to create solution directory: {:?}, error: {:?}", path_solution_dir, err);
            }
        }
    }
    let json: String = match serde_json::to_string(&tasks) {
        Ok(value) => value,
        Err(error) => {
            error!("unable to serialize tasks to json: {:?}", error);
            return;
        }
    };
    match fs::write(&path_solution_teamid_json, json) {
        Ok(()) => {},
        Err(error) => {
            error!("unable to save solutions file. path: {:?} error: {:?}", path_solution_teamid_json, error);
            return;
        }
    }
    debug!("updated solutions file: tasks.len(): {}", tasks.len());
}
