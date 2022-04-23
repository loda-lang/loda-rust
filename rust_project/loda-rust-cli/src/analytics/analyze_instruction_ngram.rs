use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{InstructionId, ParsedProgram};
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
Creates csv files with unigram/bigram/trigram/skipgram with LODA instructions.
https://en.wikipedia.org/wiki/N-gram

This script traverses all the programs inside the "loda-programs/oeis" dir.
It looks for all the LODA assembly programs there are.
This script determines the most frequent combinations of instructions.

---

This script outputs a `unigram.csv` file, with this format:

    count;word
    158474;mov
    123892;add
    69003;sub
    67443;mul
    61799;STOP
    61799;START
    40310;lpe
    40310;lpb
    35876;div
    21128;seq
    20770;pow
    9658;mod
    8240;cmp
    8112;bin
    6163;trn
    4077;max
    3405;dif
    3160;gcd
    1188;min
    30;clr

Learnings from this unigram with LODA programs:
Learning A: The `mov` instruction is the most used instruction.
Learning B: The `clr` instruction is the least used instruction.

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
pub struct AnalyzeInstructionNgram {
    config: Config,
    histogram_unigram: HashMap<String,u32>,
    histogram_bigram: HashMap<HistogramBigramKey,u32>,
    histogram_trigram: HashMap<HistogramTrigramKey,u32>,
    histogram_skipgram: HashMap<HistogramSkipgramKey,u32>,
}

impl AnalyzeInstructionNgram {
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
        let instruction_ids: Vec<InstructionId> = parsed_program.instruction_ids();
        for instruction_id in instruction_ids {
            let word: String = String::from(instruction_id.shortname());
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_instruction_unigram_file();
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_instruction_bigram_file();
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_instruction_trigram_file();
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
        let output_path: PathBuf = self.config.analytics_dir_histogram_instruction_skipgram_file();
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

impl BatchProgramAnalyzerPlugin for AnalyzeInstructionNgram {
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
