use std::error::Error;
use serde::{Deserialize, Serialize};
use std::path::Path;
use super::parse_csv_file;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct RecordTrigram {
    pub count: u32,
    pub word0: String,
    pub word1: String,
    pub word2: String,
}

impl RecordTrigram {
    pub fn parse_csv(path: &Path) -> Result<Vec<RecordTrigram>, Box<dyn Error>> {
        parse_csv_file::parse_csv_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::parse_csv_data;

    #[test]
    fn test_10000_parse_ok() {
        let data = "\
count;word0;word1;word2
10976;lpe;mov;STOP
10556;mov;lpb;sub
10224;add;lpe;mov
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordTrigram> = parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {} {}", record.count, record.word0, record.word1, record.word2)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "10976 lpe mov STOP,10556 mov lpb sub,10224 add lpe mov");
    }

    #[test]
    fn test_10001_parse_ok() {
        let data = "\
count;word0;word1;word2
33031;0;0;0
31497;0;0;STOP
17270;1;1;1
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordTrigram> = parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {} {}", record.count, record.word0, record.word1, record.word2)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "33031 0 0 0,31497 0 0 STOP,17270 1 1 1");
    }
}
