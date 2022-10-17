use crate::common::create_csv_file;
use crate::common::RecordBigram;
use crate::common::RecordTrigram;
use crate::common::RecordSkipgram;
use crate::common::RecordUnigram;
use crate::config::Config;
use loda_rust_core;
use loda_rust_core::parser::ParameterType;
use loda_rust_core::parser::{InstructionId, ParsedProgram};
use std::path::PathBuf;
use std::error::Error;
use std::collections::HashMap;
use super::{BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};

type HistogramBigramKey = (String,String);
type HistogramTrigramKey = (String,String,String);
type HistogramSkipgramKey = (String,String);

/// Creates [N-gram] csv files with lines of LODA code.
/// 
/// Traverses all the programs inside the `loda-programs/oeis` dir.
/// It looks for all the LODA assembly programs there are.
/// Determines the most frequent combinations of LODA code lines.
/// 
/// ---
/// 
/// This outputs a `histogram_line_unigram.csv` file, with this format:
/// 
/// ```csv
/// count;word
/// 96597;STOP
/// 96597;START
/// 69080;lpe
/// 43970;lpb $0
/// 38553;mov $0,$1
/// 37459;add $0,1
/// 35716;sub $0,1
/// 21085;mov $1,$0
/// ```
///
/// Learnings from this unigram with LODA programs:
/// 
/// Learning A: Loop instructions are the most used instructions.
/// 
/// Learning B: As of 2022-oct-16 there are 102k LODA programs.
/// When ignoring values greater than 100, 
/// then there are 9k rows that 2 or more times and 12k rows that occurs once.
/// 
/// ---
/// 
/// This outputs a `histogram_line_bigram.csv` file, with this format:
/// 
/// ```csv
/// count;word0;word1
/// 25134;mov $0,$1;STOP
/// 20623;lpb $0;sub $0,1
/// 19807;lpe;mov $0,$1
/// 14278;START;mov $1,$0
/// 11410;add $0,1;STOP
/// 10803;START;add $0,1
/// 10034;START;mov $1,1
/// 8582;START;lpb $0
/// ```
/// 
/// Learnings from this bigram with LODA programs:
/// 
/// Learning A: Many programs ends with a `mov $0,$1` instruction.
/// Many programs starts with a `mov $1,$0` instruction, which is the opposite.
/// 
/// ---
/// 
/// This outputs a `histogram_line_trigram.csv` file, with this format:
/// 
/// ```csv
/// count;word0;word1;word2
/// 12576;lpe;mov $0,$1;STOP
/// 4255;mov $4,$0;max $4,0;cmp $4,$0
/// 4022;lpe;mov $0,$2;STOP
/// 3823;add $0,1;lpb $0;sub $0,1
/// 3607;max $4,0;cmp $4,$0;mul $2,$4
/// 3464;lpe;mov $0,$3;STOP
/// 3288;sub $2,1;lpe;mov $0,$1
/// 3197;mul $2,$4;sub $2,1;lpe
/// ```
/// 
/// ---
/// 
/// This outputs a `histogram_line_skipgram.csv` file, with this format:
/// 
/// ```csv
/// count;word0;word2
/// 27776;lpe;STOP
/// 11992;START;lpb $0
/// 9280;mov $0,$1;STOP
/// 5704;START;mov $2,$0
/// 5403;START;add $0,1
/// 5131;lpe;add $0,1
/// 4397;mov $4,$0;cmp $4,$0
/// 4249;add $0,1;sub $0,1
/// ```
/// 
/// [N-gram]: <https://en.wikipedia.org/wiki/N-gram>
pub struct AnalyzeLineNgram {
    config: Config,
    histogram_unigram: HashMap<String,u32>,
    histogram_bigram: HashMap<HistogramBigramKey,u32>,
    histogram_trigram: HashMap<HistogramTrigramKey,u32>,
    histogram_skipgram: HashMap<HistogramSkipgramKey,u32>,
    ignore_count: usize,
}

impl AnalyzeLineNgram {
    const IGNORE_CONSTANTS_GREATER_THAN: i64 = 100;

    pub fn new() -> Self {
        Self {
            config: Config::load(),
            histogram_unigram: HashMap::new(),
            histogram_bigram: HashMap::new(),
            histogram_trigram: HashMap::new(),
            histogram_skipgram: HashMap::new(),
            ignore_count: 0,
        }
    }

    fn extract_words(&mut self, parsed_program: &ParsedProgram) -> Vec<String> {
        let mut words: Vec<String> = vec!();
        words.push("START".to_string());
        for instruction in &parsed_program.instruction_vec {
            // junk data
            // If there are magic constants, then entirely ignore the program.
            // For the `seq` instruction allow constants.
            if instruction.instruction_id != InstructionId::EvalSequence {
                if instruction.parameter_vec.len() == 2 {
                    if let Some(parameter) = instruction.parameter_vec.last() {
                        if parameter.parameter_type == ParameterType::Constant {
                            if parameter.parameter_value.abs() > Self::IGNORE_CONSTANTS_GREATER_THAN {
                                self.ignore_count += 1;
                                return vec!();
                            }
                        }
                    }
                }
            }
            // Data seems good, append it.
            let word: String = format!("{}", instruction);
            words.push(word);
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

    fn save_unigram(&self) -> Result<(), Box<dyn Error>> {
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_line_unigram_file();
        create_csv_file(&records, &output_path)
    }

    fn save_bigram(&self) -> Result<(), Box<dyn Error>> {
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_line_bigram_file();
        create_csv_file(&records, &output_path)
    }

    fn save_trigram(&self) -> Result<(), Box<dyn Error>> {
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_line_trigram_file();
        create_csv_file(&records, &output_path)
    }

    fn save_skipgram(&self) -> Result<(), Box<dyn Error>> {
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_line_skipgram_file();
        create_csv_file(&records, &output_path)
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeLineNgram {
    fn plugin_name(&self) -> &'static str {
        "AnalyzeLineNgram"
    }
    
    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> Result<(), Box<dyn Error>> {
        let words: Vec<String> = self.extract_words(&context.parsed_program);
        self.populate_unigram(&words);
        self.populate_bigram(&words);
        self.populate_trigram(&words);
        self.populate_skipgram(&words);
        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn Error>> {
        self.save_unigram()?;
        self.save_bigram()?;
        self.save_trigram()?;
        self.save_skipgram()?;
        Ok(())
    }

    fn human_readable_summary(&self) -> String {
        let rows: Vec<String> = vec![
            format!("unigram: {:?}", self.histogram_unigram.len()),
            format!("bigram: {:?}", self.histogram_bigram.len()),
            format!("trigram: {:?}", self.histogram_trigram.len()),
            format!("skipgram: {:?}", self.histogram_skipgram.len()),
            format!("ignore count: {:?}", self.ignore_count),
        ];
        rows.join(", ")
    }
}
