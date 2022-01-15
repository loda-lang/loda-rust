use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use serde::Deserialize;
use rand::Rng;
use rand::seq::SliceRandom;
use loda_rust_core::parser::InstructionId;

pub struct HistogramInstructionNgram {
}

impl HistogramInstructionNgram {
}

#[derive(Debug, Deserialize)]
struct Record {
    count: u32,
    word0: String,
    word1: String,
}

impl Record {
    fn parse_csv_data(reader: &mut dyn BufRead) -> Result<Vec<Record>, Box<dyn Error>> {
        let mut records = Vec::<Record>::new();
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(true)
            .from_reader(reader);
        for result in csv_reader.deserialize() {
            let record: Record = result?;
            records.push(record);
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_10000_bigram_parse_csv_data() {
        let data = "\
count;word0;word1
29843;START;mov
28868;add;mov
24764;mov;STOP
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<Record> = Record::parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {}", record.count, record.word0, record.word1)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "29843 START mov,28868 add mov,24764 mov STOP");
    }
}
