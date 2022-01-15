use std::error::Error;
use std::io::{BufRead, BufReader};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use std::fs::File;
use loda_rust_core::config::Config;

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

    #[allow(dead_code)]
    pub fn loda_bigram(&self) -> Result<Vec<RecordBigram>, Box<dyn Error>> {
        let file = File::open(&self.path_bigram)?;
        let mut reader = BufReader::new(file);
        let records: Vec<RecordBigram> = Self::parse_csv_data(&mut reader)?;
        Ok(records)
    }

    #[allow(dead_code)]
    pub fn loda_trigram(&self) -> Result<Vec<RecordTrigram>, Box<dyn Error>> {
        let file = File::open(&self.path_trigram)?;
        let mut reader = BufReader::new(file);
        let records: Vec<RecordTrigram> = Self::parse_csv_data(&mut reader)?;
        Ok(records)
    }

    #[allow(dead_code)]
    pub fn loda_skipgram(&self) -> Result<Vec<RecordSkipgram>, Box<dyn Error>> {
        let file = File::open(&self.path_skipgram)?;
        let mut reader = BufReader::new(file);
        let records: Vec<RecordSkipgram> = Self::parse_csv_data(&mut reader)?;
        Ok(records)
    }

    #[allow(dead_code)]
    fn parse_csv_data<D: DeserializeOwned>(reader: &mut dyn BufRead) 
        -> Result<Vec<D>, Box<dyn Error>> 
    {
        let mut records = Vec::<D>::new();
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(true)
            .from_reader(reader);
        for result in csv_reader.deserialize() {
            let record: D = result?;
            records.push(record);
        }
        Ok(records)
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RecordBigram {
    pub count: u32,
    pub word0: String,
    pub word1: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RecordTrigram {
    pub count: u32,
    pub word0: String,
    pub word1: String,
    pub word2: String,
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
    fn test_10000_bigram_parse_csv_data() {
        let data = "\
count;word0;word1
29843;START;mov
28868;add;mov
24764;mov;STOP
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordBigram> = HistogramInstructionNgram::parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {}", record.count, record.word0, record.word1)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "29843 START mov,28868 add mov,24764 mov STOP");
    }

    #[test]
    fn test_20000_trigram_parse_csv_data() {
        let data = "\
count;word0;word1;word2
10976;lpe;mov;STOP
10556;mov;lpb;sub
10224;add;lpe;mov
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordTrigram> = HistogramInstructionNgram::parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {} {}", record.count, record.word0, record.word1, record.word2)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "10976 lpe mov STOP,10556 mov lpb sub,10224 add lpe mov");
    }

    #[test]
    fn test_30000_skipgram_parse_csv_data() {
        let data = "\
count;word0;word2
24181;add;mov
24069;mov;mov
22644;mov;add
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordSkipgram> = HistogramInstructionNgram::parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {}", record.count, record.word0, record.word2)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "24181 add mov,24069 mov mov,22644 mov add");
    }
}
