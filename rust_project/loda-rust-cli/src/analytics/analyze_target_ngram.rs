use crate::common::create_csv_file;
use crate::common::RecordBigram;
use crate::common::RecordTrigram;
use crate::common::RecordSkipgram;
use crate::common::RecordUnigram;
use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{InstructionParameter, ParsedProgram};
use std::path::PathBuf;
use std::error::Error;
use std::collections::HashMap;
use super::{BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};

type HistogramBigramKey = (String,String);
type HistogramTrigramKey = (String,String,String);
type HistogramSkipgramKey = (String,String);

/*
Creates csv files with unigram/bigram/trigram/skipgram with LODA target registers.
https://en.wikipedia.org/wiki/N-gram

This script traverses all the programs inside the "loda-programs/oeis" dir.
It looks for all the LODA assembly programs there are.
This script determines the most frequent combinations of target registers.

---

This script outputs a `unigram.csv` file, with this format:

    count;word
    225985;0
    122853;1
    100247;2
    62531;3
    61799;STOP
    61799;START
    40310;NONE
    32944;4
    17332;5

Learnings from this unigram with LODA programs:
Learning A: Target register `$0` is by far the most used.
Learning B: Target register `$1` and `$2` is on a shared 2nd place.
Learning C: Target register `NONE` is for the `lpe` instruction, that doesn't have any register.

---

This script outputs a `bigram.csv` file, with this format:

    count;word0;word1
    91494;0;0
    60727;0;STOP
    41637;1;1
    34118;1;0
    30760;START;0
    28122;0;1
    27650;2;2
    25737;0;2
    24493;NONE;0
    23051;2;0

Learnings from this bigram with LODA programs:
Learning A: Target register `$0` is most likely to be followed by another `$0` target register.
Learning B: Target register `$1` is most likely to be followed by another `$1` target register.
Learning C: Target register `$2` is most likely to be followed by another `$2` target register.
Learning D: The program is most likely to start with a `$0` target register.
Learning E: The program is most likely to stop with a `$0` target register.

---

This script outputs a `trigram.csv` file, with this format:

    count;word0;word1;word2
    33031;0;0;0
    31497;0;0;STOP
    17270;1;1;1
    16090;START;0;0
    14132;1;0;STOP
    13901;1;1;0
    12383;NONE;0;STOP

Learnings from this trigram with LODA programs:
Learning A: Target register `$0` and `$0` is usually followed by a `$0` target register.
Learning B: Target register `$1` and `$1` is usually followed by a `$1` target register.
Learning C: Target register `$0` and `$0` is often the last two target registers in a program.

---

This script outputs a `skipgram.csv` file, with this format:

    count;word0;word2
    56263;0;0
    37328;1;0
    31837;0;STOP
    28951;1;1
    27690;START;0
    27089;2;0
    25911;0;1

Learnings from this skipgram with LODA programs:
Learning A: The `$0` and some junk is usually followed by another `$0` target register.
Learning B: The `$1` and some junk is usually followed by a `$0` target register.
Learning C: The `$0` and some junk is usually followed by the end of the program.
*/
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
            let parameter: &InstructionParameter = match instruction.parameter_vec.first() {
                Some(value) => &value,
                None => {
                    words.push("NONE".to_string());
                    continue;
                }
            };
            let parameter_value: i64 = parameter.parameter_value;
            words.push(format!("{:?}", parameter_value));
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
            format!("number of items in unigram: {:?}", self.histogram_unigram.len()),
            format!("number of items in bigram: {:?}", self.histogram_bigram.len()),
            format!("number of items in trigram: {:?}", self.histogram_trigram.len()),
            format!("number of items in skipgram: {:?}", self.histogram_skipgram.len()),
        ];
        rows.join("\n")
    }
}
