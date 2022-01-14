use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{Instruction, InstructionId, InstructionParameter, ParameterType, ParsedProgram};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;
use std::fs;
use csv::WriterBuilder;
use serde::Serialize;
use std::time::Instant;
use super::find_asm_files_recursively;
use super::program_id_from_path;

pub struct NgramGenerator {
    config: Config,
    bigram: HashMap<String,u32>,
    number_of_program_files_that_could_not_be_loaded: u32,
}

impl NgramGenerator {
    pub fn run() {
        let mut instance = Self {
            config: Config::load(),
            bigram: HashMap::new(),
            number_of_program_files_that_could_not_be_loaded: 0,
        };
        instance.analyze_all_program_files();
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
        println!("number of items in bigram: {:?}", self.bigram.len());
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
        let words: Vec<String> = Self::extract_words(&parsed_program);
        self.populate_bigram(&words);
    }

    fn extract_words(parsed_program: &ParsedProgram) -> Vec<String> {
        let mut words: Vec<String> = vec!();
        words.push("START".to_string());
        let instruction_ids: Vec<InstructionId> = parsed_program.instruction_ids();
        for instruction_id in instruction_ids {
            let word: String = String::from(instruction_id.shortname());
            words.push(word);
        }
        words.push("STOP".to_string());
        words
    }

    fn populate_bigram(&mut self, words: &Vec<String>) {
        let mut combo_vec = Vec::<String>::new();
        let mut last_word = String::new();
        for (index, word1) in words.iter().enumerate() {
            let word0: String = last_word;
            last_word = word1.clone();
            if index == 0 {
                continue;
            }
            let combo = format!("{};{}", word0, word1);
            combo_vec.push(combo);
        }
        for combo in combo_vec {
            let counter = self.bigram.entry(combo).or_insert(0);
            *counter += 1;
        }
    }
}
