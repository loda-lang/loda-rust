use std::error::Error;
use serde::Deserialize;
use std::path::Path;
use super::parse_csv_file;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RecordSkipgram {
    pub count: u32,
    pub word0: String,
    pub word2: String,
}

impl RecordSkipgram {
    #[allow(dead_code)]
    pub fn parse_csv(path: &Path) -> Result<Vec<RecordSkipgram>, Box<dyn Error>> {
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

    #[test]
    fn test_10001_parse_ok() {
        let data = "\
count;word0;word2
56263;0;0
37328;1;0
31837;0;STOP
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordSkipgram> = parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {}", record.count, record.word0, record.word2)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "56263 0 0,37328 1 0,31837 0 STOP");
    }
}
