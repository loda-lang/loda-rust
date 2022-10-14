use crate::common::create_csv_file;
use crate::common::RecordBigram;
use crate::common::RecordTrigram;
use crate::common::RecordSkipgram;
use crate::common::RecordUnigram;
use loda_rust_core;
use crate::config::Config;
use loda_rust_core::parser::{InstructionParameter, ParsedProgram};
use std::path::PathBuf;
use std::error::Error;
use std::collections::HashMap;
use super::{BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};

type HistogramBigramKey = (String,String);
type HistogramTrigramKey = (String,String,String);
type HistogramSkipgramKey = (String,String);

/// Creates [N-gram] csv files with LODA target parameters.
/// 
/// Traverses all the programs inside the `loda-programs/oeis` dir.
/// It looks for all the LODA assembly programs there are.
/// Determines the most frequent combinations of target parameters.
/// 
/// ---
/// 
/// This outputs a `histogram_target_unigram.csv` file, with this format:
/// 
/// ```csv
/// count;word
/// 374735;$0
/// 219113;$1
/// 175935;$2
/// 126541;$3
/// 101736;STOP
/// 101736;START
/// 72110;NONE
/// 69648;$4
/// ```
/// 
/// Learnings from this unigram with LODA programs:
/// 
/// Learning A: Target register `$0` is by far the most used.
/// 
/// Learning B: Target register `$1` and `$2` is on a shared 2nd place.
/// 
/// Learning C: Target register `NONE` is for the `lpe` instruction, that doesn't have any register.
/// 
/// ---
/// 
/// This outputs a `histogram_target_bigram.csv` file, with this format:
/// 
/// ```csv
/// count;word0;word1
/// 166137;$0;$0
/// 100158;$0;STOP
/// 83136;$1;$1
/// 68268;$2;$2
/// 57590;$3;$3
/// 56872;$1;$0
/// 47892;$0;$1
/// 47443;NONE;$0
/// 42897;START;$0
/// 35321;START;$1
/// 34262;$1;$2
/// ```
/// 
/// Learnings from this bigram with LODA programs:
/// 
/// Learning A: Target register `$0` is most likely to be followed by another `$0` target register.
/// 
/// Learning B: Target register `$1` is most likely to be followed by another `$1` target register.
/// 
/// Learning C: Target register `$2` is most likely to be followed by another `$2` target register.
/// 
/// Learning D: The program is most likely to start with a `$0` target register.
/// 
/// Learning E: The program is most likely to stop with a `$0` target register.
/// 
/// ---
/// 
/// This outputs a `histogram_target_trigram.csv` file, with this format:
/// 
/// ```csv
/// count;word0;word1;word2
/// 64171;$0;$0;$0
/// 52591;$0;$0;STOP
/// 31892;$1;$1;$1
/// 29623;$1;$1;$0
/// 28649;NONE;$0;STOP
/// 27618;$1;$0;$0
/// 26024;$3;$3;$3
/// 25285;$2;$2;$2
/// 24374;START;$0;$0
/// ```
/// 
/// Learnings from this trigram with LODA programs:
/// 
/// Learning A: Target register `$0` and `$0` is usually followed by a `$0` target register.
/// 
/// Learning B: Target register `$1` and `$1` is usually followed by a `$1` target register.
/// 
/// Learning C: Target register `$0` and `$0` is often the last two target registers in a program.
/// 
/// ---
/// 
/// This outputs a `histogram_target_skipgram.csv` file, with this format:
/// 
/// ```csv
/// count;word0;word2
/// 82135;$0;$0
/// 79498;$1;$0
/// 53440;$0;STOP
/// 46047;$2;$0
/// 44468;START;$0
/// 43954;$0;$1
/// 41945;$1;$1
/// 40304;$0;$2
/// ```
/// 
/// Learnings from this skipgram with LODA programs:
/// 
/// Learning A: The `$0` and some junk is usually followed by another `$0` target register.
/// 
/// Learning B: The `$1` and some junk is usually followed by a `$0` target register.
/// 
/// Learning C: The `$0` and some junk is usually followed by the end of the program.
/// 
/// [N-gram]: <https://en.wikipedia.org/wiki/N-gram>
pub struct AnalyzeTargetNgram {
    config: Config,
    histogram_unigram: HashMap<String,u32>,
    histogram_bigram: HashMap<HistogramBigramKey,u32>,
    histogram_trigram: HashMap<HistogramTrigramKey,u32>,
    histogram_skipgram: HashMap<HistogramSkipgramKey,u32>,
}

impl AnalyzeTargetNgram {
    pub fn new() -> Self {
        Self {
            config: Config::load(),
            histogram_unigram: HashMap::new(),
            histogram_bigram: HashMap::new(),
            histogram_trigram: HashMap::new(),
            histogram_skipgram: HashMap::new(),
        }
    }

    fn extract_words(parsed_program: &ParsedProgram) -> Vec<String> {
        let mut words: Vec<String> = vec!();
        words.push("START".to_string());
        for instruction in &parsed_program.instruction_vec {
            if instruction.parameter_vec.len() < 1 {
                words.push("NONE".to_string());
                continue;
            }
            let parameter: &InstructionParameter = &instruction.parameter_vec[0];
            words.push(parameter.to_string());
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_target_unigram_file();
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_target_bigram_file();
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_target_trigram_file();
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_target_skipgram_file();
        create_csv_file(&records, &output_path)
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeTargetNgram {
    fn plugin_name(&self) -> &'static str {
        "AnalyzeTargetNgram"
    }
    
    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> Result<(), Box<dyn Error>> {
        let words: Vec<String> = Self::extract_words(&context.parsed_program);
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
        ];
        rows.join(", ")
    }
}
