use std::error::Error;
use std::io::BufReader;
use serde::Deserialize;
use std::path::Path;
use std::fs::File;
use super::parse_csv_data;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RecordBigram {
    pub count: u32,
    pub word0: String,
    pub word1: String,
}

impl RecordBigram {
    #[allow(dead_code)]
    pub fn parse_csv(path: &Path) -> Result<Vec<RecordBigram>, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let records: Vec<RecordBigram> = parse_csv_data(&mut reader)?;
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_parse_ok() {
        let data = "\
count;word0;word1
29843;START;mov
28868;add;mov
24764;mov;STOP
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordBigram> = parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {}", record.count, record.word0, record.word1)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "29843 START mov,28868 add mov,24764 mov STOP");
    }

    #[test]
    fn test_10001_parse_ok() {
        let data = "\
count;word0;word1
91494;0;0
60727;0;STOP
41637;1;1
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordBigram> = parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {}", record.count, record.word0, record.word1)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "91494 0 0,60727 0 STOP,41637 1 1");
    }
}
