use crate::common::create_csv_file;
use crate::common::RecordBigram;
use crate::common::RecordTrigram;
use crate::common::RecordSkipgram;
use crate::common::RecordUnigram;
use crate::config::Config;
use loda_rust_core;
use loda_rust_core::parser::{InstructionParameter, ParsedProgram};
use std::path::PathBuf;
use std::error::Error;
use std::collections::HashMap;
use super::{BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};

type HistogramBigramKey = (String,String);
type HistogramTrigramKey = (String,String,String);
type HistogramSkipgramKey = (String,String);

/// Creates [N-gram] csv files with LODA source parameters.
/// 
/// Traverses all the programs inside the `loda-programs/oeis` dir.
/// It looks for all the LODA assembly programs there are.
/// Determines the most frequent combinations of source parameters.
/// 
/// ---
/// 
/// This outputs a `histogram_source_unigram.csv` file, with this format:
/// 
/// ```csv
/// count;word
/// 188414;1
/// 144834;NONE
/// 125947;$0
/// 116255;$1
/// 107093;2
/// 101601;STOP
/// 101601;START
/// 87819;$2
/// 53412;$3
/// 39696;$4
/// ```
///
/// Learnings from this unigram with LODA programs:
/// 
/// Learning A: Source is most often a contant.
/// 
/// Learning B: Often there is no source parameter, such as for loop begin `lpb $3` or loop end `lpe`.
/// 
/// Learning C: The source register `$0` is most used.
/// 
/// ---
/// 
/// This outputs a `histogram_source_bigram.csv` file, with this format:
/// 
/// ```csv
/// count;word0;word1
/// 45608;NONE;1
/// 36119;$1;STOP
/// 35448;1;NONE
/// 31346;START;1
/// 28262;NONE;$1
/// 25464;START;$0
/// 20347;1;$0
/// 20234;1;1
/// 18594;$0;2
/// ```
/// 
/// Learnings from this bigram with LODA programs:
/// 
/// Learning A: Source constant is most likely to be followed by another source constant.
/// 
/// Learning B: Source register `$0` is most likely to be followed by a source constant.
/// 
/// Learning C: The program is most likely to start with a source constant.
/// 
/// Learning D: The program is most likely to stop with a source constant.
/// 
/// ---
/// 
/// This outputs a `histogram_source_trigram.csv` file, with this format:
/// 
/// ```csv
/// count;word0;word1;word2
/// 15775;1;NONE;1
/// 13470;NONE;$1;STOP
/// 9546;NONE;1;$1
/// 8145;START;1;1
/// 7357;START;1;NONE
/// 7331;NONE;1;$2
/// 7266;1;1;NONE
/// 6969;START;$0;2
/// 6467;START;1;$0
/// ```
/// 
/// Learnings from this trigram with LODA programs:
/// 
/// Learning A: Three source contants in succession are most popular.
/// 
/// Learning B: Programs often start with two source contants.
/// 
/// Learning C: The source register `$0` is often surrounded by two source constants.
/// 
/// ---
/// 
/// This outputs a `histogram_source_skipgram.csv` file, with this format:
/// 
/// ```csv
/// count;word0;word2
/// 30765;1;NONE
/// 28622;1;1
/// 28617;NONE;STOP
/// 24477;START;1
/// 20608;START;$0
/// 19542;$0;1
/// 18662;1;$0
/// 18579;$0;NONE
/// ```
/// 
/// Learnings from this skipgram with LODA programs:
/// 
/// Learning A: Constant and some junk is usually followed by another constant.
/// 
/// Learning B: Program start and some junk is usually followed a constant.
/// 
/// Learning C: The `$0` and some junk is usually followed by a constant.
/// 
/// [N-gram]: <https://en.wikipedia.org/wiki/N-gram>
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

    fn extract_words(parsed_program: &ParsedProgram) -> Vec<String> {
        let mut words: Vec<String> = vec!();
        words.push("START".to_string());
        for instruction in &parsed_program.instruction_vec {
            if instruction.parameter_vec.len() < 2 {
                words.push("NONE".to_string());
                continue;
            }
            let parameter: &InstructionParameter = &instruction.parameter_vec[1];
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_source_unigram_file();
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_source_bigram_file();
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_source_trigram_file();
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_source_skipgram_file();
        create_csv_file(&records, &output_path)
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeSourceNgram {
    fn plugin_name(&self) -> &'static str {
        "AnalyzeSourceNgram"
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
