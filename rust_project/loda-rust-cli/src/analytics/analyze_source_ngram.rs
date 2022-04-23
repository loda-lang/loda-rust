use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{InstructionParameter, ParsedProgram, ParameterType};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;
use csv::WriterBuilder;
use serde::Serialize;
use super::{BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};

type HistogramBigramKey = (String,String);
type HistogramTrigramKey = (String,String,String);
type HistogramSkipgramKey = (String,String);

/*
Creates csv files with unigram/bigram/trigram/skipgram with LODA source register/constants.
https://en.wikipedia.org/wiki/N-gram

This script traverses all the programs inside the "loda-programs/oeis" dir.
It looks for all the LODA assembly programs there are.
This script determines the most frequent combinations of source register/constants.

---

This script outputs a `unigram.csv` file, with this format:

    count;word
    291363;CONST
    80741;NONE
    74575;0
    61967;STOP
    61967;START
    57089;1
    50819;2
    29725;3

Learnings from this unigram with LODA programs:
Learning A: Source is most often a contant.
Learning B: Often there is no source parameter, such as for loop begin `lpb $3` or loop end `lpe`.
Learning C: The source register `$0` is most used.

---

This script outputs a `bigram.csv` file, with this format:

    count;word0;word1
    107662;CONST;CONST
    43147;0;CONST
    42103;START;CONST
    40569;CONST;NONE
    38239;CONST;0
    33260;NONE;CONST
    31547;CONST;STOP
    25079;CONST;1
    21977;1;STOP
    20578;CONST;2

Learnings from this bigram with LODA programs:
Learning A: Source constant is most likely to be followed by another source constant.
Learning B: Source register `$0` is most likely to be followed by a source constant.
Learning C: The program is most likely to start with a source constant.
Learning D: The program is most likely to stop with a source constant.

---

This script outputs a `trigram.csv` file, with this format:

    count;word0;word1;word2
    36000;CONST;CONST;CONST
    23556;START;CONST;CONST
    19935;CONST;NONE;CONST
    19921;CONST;0;CONST
    19376;0;CONST;CONST
    17677;CONST;CONST;NONE
    16321;CONST;CONST;STOP
    11998;START;CONST;0
    11679;START;0;CONST
    11052;CONST;CONST;0

Learnings from this trigram with LODA programs:
Learning A: Three source contants in succession are most popular.
Learning B: Programs often start with two source contants.
Learning C: The source register `$0` is often surrounded by two source constants.

---

This script outputs a `skipgram.csv` file, with this format:

    count;word0;word2
    99972;CONST;CONST
    38355;START;CONST
    35538;NONE;CONST
    35059;CONST;NONE
    30733;0;CONST
    28197;CONST;STOP
    24782;CONST;1
    24108;CONST;2
    21690;CONST;0

Learnings from this skipgram with LODA programs:
Learning A: Constant and some junk is usually followed by another constant.
Learning B: Program start and some junk is usually followed a constant.
Learning C: The `$0` and some junk is usually followed by a constant.
*/
pub struct AnalyzeSourceNgram {
    config: Config,
    histogram_unigram: HashMap<String,u32>,
    histogram_bigram: HashMap<HistogramBigramKey,u32>,
    histogram_trigram: HashMap<HistogramTrigramKey,u32>,
    histogram_skipgram: HashMap<HistogramSkipgramKey,u32>,
}

impl AnalyzeSourceNgram {
    pub fn new() -> Self {
        Self {
            config: Config::load(),
            histogram_unigram: HashMap::new(),
            histogram_bigram: HashMap::new(),
            histogram_trigram: HashMap::new(),
            histogram_skipgram: HashMap::new(),
        }
    }

    fn save_inner(&self) {
        println!("number of items in unigram: {:?}", self.histogram_unigram.len());
        println!("number of items in bigram: {:?}", self.histogram_bigram.len());
        println!("number of items in trigram: {:?}", self.histogram_trigram.len());
        println!("number of items in skipgram: {:?}", self.histogram_skipgram.len());
        self.save_unigram();
        self.save_bigram();
        self.save_trigram();
        self.save_skipgram();
    }

    fn extract_words(parsed_program: &ParsedProgram) -> Vec<String> {
        let mut words: Vec<String> = vec!();
        words.push("START".to_string());
        for instruction in &parsed_program.instruction_vec {
            if instruction.parameter_vec.len() < 2 {
                words.push("NONE".to_string());
                continue;
            }
            let parameter: &InstructionParameter = &instruction.parameter_vec[1];
            match parameter.parameter_type {
                ParameterType::Constant => {
                    words.push("CONST".to_string());
                },
                ParameterType::Register => {
                    let parameter_value: i64 = parameter.parameter_value;
                    words.push(format!("{:?}", parameter_value));
                }
            }
        }
        words.push("STOP".to_string());
        words
    }

    fn populate_unigram(&mut self, words: &Vec<String>) {
        let keys: Vec<String> = words.clone();
        for key in keys {
            let counter = self.histogram_unigram.entry(key).or_insert(0);
            *counter += 1;
        }
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

    fn save_unigram(&self) {
        // Convert from dictionary to array
        let mut records = Vec::<RecordUnigram>::new();
        for (histogram_key, histogram_count) in &self.histogram_unigram {
            let record = RecordUnigram {
                count: *histogram_count,
                word: histogram_key.clone(),
            };
            records.push(record);
        }

        // Move the most frequently occuring items to the top
        // Move the lesser used items to the bottom
        records.sort_unstable_by_key(|item| (item.count, item.word.clone()));
        records.reverse();

        // Save as a CSV file
        let output_path: PathBuf = self.config.cache_dir_histogram_source_unigram_file();
        match Self::create_csv_file(&records, &output_path) {
            Ok(_) => {
                println!("saved unigram.csv");
            },
            Err(error) => {
                println!("cannot save unigram.csv error: {:?}", error);
            }
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
        let output_path: PathBuf = self.config.cache_dir_histogram_source_bigram_file();
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
        let output_path: PathBuf = self.config.cache_dir_histogram_source_trigram_file();
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
        let output_path: PathBuf = self.config.cache_dir_histogram_source_skipgram_file();
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

impl BatchProgramAnalyzerPlugin for AnalyzeSourceNgram {
    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> bool {
        let words: Vec<String> = Self::extract_words(&context.parsed_program);
        self.populate_unigram(&words);
        self.populate_bigram(&words);
        self.populate_trigram(&words);
        self.populate_skipgram(&words);
        true
    }

    fn save(&self) {
        self.save_inner();
    }
}

#[derive(Serialize)]
struct RecordUnigram {
    count: u32,
    word: String,
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
