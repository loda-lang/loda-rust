use std::error::Error;
use std::io::BufReader;
use serde::Deserialize;
use std::path::PathBuf;
use std::fs::File;
use loda_rust_core::config::Config;
use super::parse_csv_data;

pub trait SkipgramVec {
    fn skipgram_vec(&self) -> Result<Vec<RecordSkipgram>, Box<dyn Error>>;
}

#[allow(dead_code)]
pub struct HistogramInstructionNgram {
    path_bigram: PathBuf,
    path_trigram: PathBuf,
    path_skipgram: PathBuf,
}

impl HistogramInstructionNgram {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let config = Config::load(); 
        Self {
            path_bigram: config.cache_dir_histogram_instruction_bigram_file(),
            path_trigram: config.cache_dir_histogram_instruction_trigram_file(),
            path_skipgram: config.cache_dir_histogram_instruction_skipgram_file(),
        }
    }
}

impl SkipgramVec for HistogramInstructionNgram {
    fn skipgram_vec(&self) -> Result<Vec<RecordSkipgram>, Box<dyn Error>> {
        let file = File::open(&self.path_skipgram)?;
        let mut reader = BufReader::new(file);
        let records: Vec<RecordSkipgram> = parse_csv_data(&mut reader)?;
        Ok(records)
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RecordSkipgram {
    pub count: u32,
    pub word0: String,
    pub word2: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_30000_skipgram_parse_csv_data() {
        let data = "\
count;word0;word2
24181;add;mov
24069;mov;mov
22644;mov;add
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordSkipgram> = parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {}", record.count, record.word0, record.word2)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "24181 add mov,24069 mov mov,22644 mov add");
    }
}
