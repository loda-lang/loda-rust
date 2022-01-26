use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::ParsedProgram;
use std::path::PathBuf;
use std::fs;
use std::time::Instant;
use std::rc::Rc;
use core::cell::RefCell;
use super::find_asm_files_recursively;
use super::program_id_from_path;

pub struct ProgramIteratorContext {
    pub program_id: u32,
    pub parsed_program: ParsedProgram,
}

pub trait ProgramIteratorPlugin {
    fn process(&mut self, context: &ProgramIteratorContext) -> bool;
    fn save(&self);
}

pub type ProgramIteratorPluginItem = Rc<RefCell<dyn ProgramIteratorPlugin>>;

pub struct ProgramIterator {
    config: Config,
    number_of_program_files_that_could_not_be_loaded: u32,
    plugin_vec: Vec<ProgramIteratorPluginItem>,
}

impl ProgramIterator {
    pub fn new() -> Self {
        Self {
            config: Config::load(),
            number_of_program_files_that_could_not_be_loaded: 0,
            plugin_vec: vec!(),
        }
    }

    pub fn install_plugin(&mut self, plugin: ProgramIteratorPluginItem) {
        self.plugin_vec.push(plugin);
    }

    pub fn run(&mut self) {
        self.analyze_all_program_files();
        self.save_result_files();
    }

    fn analyze_all_program_files(&mut self) {
        let dir_containing_programs: PathBuf = self.config.loda_programs_oeis_dir();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_programs);
        let number_of_paths = paths.len();
        if number_of_paths <= 0 {
            error!("Expected 1 or more programs, but there are no programs to analyze");
            return;
        }
        let max_index: usize = number_of_paths - 1;
        println!("number of programs for the ngram generator: {:?}", paths.len());
        let mut progress_time = Instant::now();
        for (index, path) in paths.iter().enumerate() {
            let elapsed: u128 = progress_time.elapsed().as_millis();
            let is_last: bool = index == max_index;
            if elapsed >= 1000 || is_last {
                let percent: usize = (index * 100) / max_index;
                println!("progress: {}%  {} of {}", percent, index + 1, number_of_paths);
                progress_time = Instant::now();
            }
            self.analyze_program_file(&path);
        }
        println!("number of program files that could not be loaded: {:?}", self.number_of_program_files_that_could_not_be_loaded);
    }

    fn analyze_program_file(&mut self, path_to_program: &PathBuf) {
        let program_id: u32 = match program_id_from_path(&path_to_program) {
            Some(program_id) => program_id,
            None => {
                debug!("Unable to extract program_id from {:?}", path_to_program);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return;
            }
        };
        let contents: String = match fs::read_to_string(&path_to_program) {
            Ok(value) => value,
            Err(error) => {
                debug!("loading program_id: {:?}, something went wrong reading the file: {:?}", program_id, error);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return;
            }
        };
        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&contents) {
            Ok(value) => value,
            Err(error) => {
                debug!("loading program_id: {:?}, something went wrong parsing the program: {:?}", program_id, error);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return;
            }
        };
        let context = ProgramIteratorContext {
            program_id: program_id,
            parsed_program: parsed_program,
        };
        for plugin in self.plugin_vec.iter() {
            let ok: bool = plugin.borrow_mut().process(&context);
            if !ok {
                break;
            }
        }
    }

    fn save_result_files(&self) {
        for plugin in self.plugin_vec.iter() {
            plugin.borrow().save();
        }
    }
}
