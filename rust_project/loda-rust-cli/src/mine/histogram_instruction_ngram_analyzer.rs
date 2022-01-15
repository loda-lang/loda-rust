use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{InstructionId, ParsedProgram};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;
use std::fs;
use csv::WriterBuilder;
use serde::Serialize;
use std::time::Instant;
use super::find_asm_files_recursively;
use super::program_id_from_path;

type HistogramBigramKey = (String,String);
type HistogramTrigramKey = (String,String,String);
type HistogramSkipgramKey = (String,String);

/*
Creates csv files with bigram/trigram/skipgram with LODA instructions.
https://en.wikipedia.org/wiki/N-gram

This script traverses all the programs inside the "loda-programs/oeis" dir.
It looks for all the LODA assembly programs there are.
This script determines the most frequent combinations of instructions.

---

This script outputs a `bigram.csv` file, with this format:

    count;word0;word1
    18066;mov;mov
    16888;START;mov
    14712;mov;lpb
    13386;mov;sub
    13132;mov;add
    11776;add;mov
    10522;add;add
    9840;mul;add

Learnings from this bigram with LODA programs:
Learning A: The `mov` instruction is most likely to be followed by another `mov` instruction.
Learning B: The program is most likely to start with a `mov` instruction.
Learning C: The `mul` instruction is most likely to be followed by an `add` instruction.
Learning D: The `lpb` instruction is most likely to be followed by a `mov` instruction.

---

This script outputs a `trigram.csv` file, with this format:

    count;word0;word1;word2
    8776;mov;mov;lpb
    6709;lpb;mov;sub
    5717;START;mov;mov
    5386;mov;lpb;mov
    4321;mov;lpb;sub
    4310;mul;add;STOP

Learnings from this trigram with LODA programs:
Learning A: The `mov` and `mov` is usually followed by a `lpb` instruction.
Learning B: The `lpb` and `mov` is usually followed by a `sub` instruction.
Learning C: The `mov` and `lpb` is usually followed by a `mov` instruction.
Learning D: The `mul` and `add` is usually the last of the program.

---

This script outputs a `skipgram.csv` file, with this format:

    count;word0;word2
    17826;mov;add
    15585;mov;mov
    12458;mov;lpb
    11971;add;mov
    11942;mov;sub
    11662;sub;mov

Learnings from this skipgram with LODA programs:
Learning A: The `mov` and some junk is usually followed by the `add` instruction.
Learning B: The `add` and some junk is usually followed by the `mov` instruction.
Learning C: The `sub` and some junk is usually followed by the `mov` instruction.
*/
pub struct HistogramInstructionNgramAnalyzer {
    config: Config,
    histogram_bigram: HashMap<HistogramBigramKey,u32>,
    histogram_trigram: HashMap<HistogramTrigramKey,u32>,
    histogram_skipgram: HashMap<HistogramSkipgramKey,u32>,
    number_of_program_files_that_could_not_be_loaded: u32,
}

impl HistogramInstructionNgramAnalyzer {
    pub fn run() {
        let mut instance = Self {
            config: Config::load(),
            histogram_bigram: HashMap::new(),
            histogram_trigram: HashMap::new(),
            histogram_skipgram: HashMap::new(),
            number_of_program_files_that_could_not_be_loaded: 0,
        };
        instance.analyze_all_program_files();
        instance.save_bigram();
        instance.save_trigram();
        instance.save_skipgram();
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
        println!("number of items in bigram: {:?}", self.histogram_bigram.len());
        println!("number of items in trigram: {:?}", self.histogram_trigram.len());
        println!("number of items in skipgram: {:?}", self.histogram_skipgram.len());
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
        self.populate_trigram(&words);
        self.populate_skipgram(&words);
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
        let mut keys = Vec::<HistogramBigramKey>::new();
        let mut prev_word = String::new();
        for (index, word1) in words.iter().enumerate() {
            let word0: String = prev_word;
            prev_word = word1.clone();
            if index == 0 {
                continue;
            }
            let key: HistogramBigramKey = (word0, word1.clone());
            keys.push(key);
        }
        for key in keys {
            let counter = self.histogram_bigram.entry(key).or_insert(0);
            *counter += 1;
        }
    }

    fn populate_trigram(&mut self, words: &Vec<String>) {
        let mut keys = Vec::<HistogramTrigramKey>::new();
        let mut prev_prev_word = String::new();
        let mut prev_word = String::new();
        for (index, word2) in words.iter().enumerate() {
            let word0: String = prev_prev_word;
            let word1: String = prev_word.clone();
            prev_prev_word = prev_word;
            prev_word = word2.clone();
            if index < 2 {
                continue;
            }
            let key: HistogramTrigramKey = (word0, word1, word2.clone());
            keys.push(key);
        }
        for key in keys {
            let counter = self.histogram_trigram.entry(key).or_insert(0);
            *counter += 1;
        }
    }

    fn populate_skipgram(&mut self, words: &Vec<String>) {
        let mut keys = Vec::<HistogramSkipgramKey>::new();
        let mut prev_prev_word = String::new();
        let mut prev_word = String::new();
        for (index, word2) in words.iter().enumerate() {
            let word0: String = prev_prev_word;
            prev_prev_word = prev_word;
            prev_word = word2.clone();
            if index < 2 {
                continue;
            }
            let key: HistogramSkipgramKey = (word0, word2.clone());
            keys.push(key);
        }
        for key in keys {
            let counter = self.histogram_skipgram.entry(key).or_insert(0);
            *counter += 1;
        }
    }

    fn save_bigram(&self) {
        // Convert from dictionary to array
        let mut records = Vec::<RecordBigram>::new();
        for (histogram_key, histogram_count) in &self.histogram_bigram {
            let record = RecordBigram {
                count: *histogram_count,
                word0: histogram_key.0.clone(),
                word1: histogram_key.1.clone()
            };
            records.push(record);
        }

        // Move the most frequently occuring items to the top
        // Move the lesser used items to the bottom
        records.sort_unstable_by_key(|item| (item.count, item.word0.clone(), item.word1.clone()));
        records.reverse();

        // Save as a CSV file
        let output_path: PathBuf = self.config.cache_dir_histogram_instruction_bigram_file();
        match Self::create_csv_file(&records, &output_path) {
            Ok(_) => {
                println!("saved bigram.csv");
            },
            Err(error) => {
                println!("cannot save bigram.csv error: {:?}", error);
            }
        }
    }

    fn save_trigram(&self) {
        // Convert from dictionary to array
        let mut records = Vec::<RecordTrigram>::new();
        for (histogram_key, histogram_count) in &self.histogram_trigram {
            let record = RecordTrigram {
                count: *histogram_count,
                word0: histogram_key.0.clone(),
                word1: histogram_key.1.clone(),
                word2: histogram_key.2.clone()
            };
            records.push(record);
        }

        // Move the most frequently occuring items to the top
        // Move the lesser used items to the bottom
        records.sort_unstable_by_key(|item| (item.count, item.word0.clone(), item.word1.clone(), item.word2.clone()));
        records.reverse();

        // Save as a CSV file
        let output_path: PathBuf = self.config.cache_dir_histogram_instruction_trigram_file();
        match Self::create_csv_file(&records, &output_path) {
            Ok(_) => {
                println!("saved trigram.csv");
            },
            Err(error) => {
                println!("cannot save trigram.csv error: {:?}", error);
            }
        }
    }

    fn save_skipgram(&self) {
        // Convert from dictionary to array
        let mut records = Vec::<RecordSkipgram>::new();
        for (histogram_key, histogram_count) in &self.histogram_skipgram {
            let record = RecordSkipgram {
                count: *histogram_count,
                word0: histogram_key.0.clone(),
                word2: histogram_key.1.clone()
            };
            records.push(record);
        }

        // Move the most frequently occuring items to the top
        // Move the lesser used items to the bottom
        records.sort_unstable_by_key(|item| (item.count, item.word0.clone(), item.word2.clone()));
        records.reverse();

        // Save as a CSV file
        let output_path: PathBuf = self.config.cache_dir_histogram_instruction_skipgram_file();
        match Self::create_csv_file(&records, &output_path) {
            Ok(_) => {
                println!("saved skipgram.csv");
            },
            Err(error) => {
                println!("cannot save skipgram.csv error: {:?}", error);
            }
        }
    }

    fn create_csv_file<S: Serialize>(records: &Vec<S>, output_path: &Path) -> Result<(), Box<dyn Error>> {
        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_path(output_path)?;
        for record in records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

#[derive(Serialize)]
struct RecordBigram {
    count: u32,
    word0: String,
    word1: String,
}

#[derive(Serialize)]
struct RecordTrigram {
    count: u32,
    word0: String,
    word1: String,
    word2: String,
}

#[derive(Serialize)]
struct RecordSkipgram {
    count: u32,
    word0: String,
    word2: String,
}
