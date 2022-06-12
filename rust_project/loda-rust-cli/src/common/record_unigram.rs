use std::error::Error;
use serde::{Deserialize, Serialize};
use std::path::Path;
use super::parse_csv_file;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct RecordUnigram {
    pub count: u32,
    pub word: String,
}

impl RecordUnigram {
    #[allow(dead_code)]
    pub fn parse_csv(path: &Path) -> Result<Vec<RecordUnigram>, Box<dyn Error>> {
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
count;word
28868;add
9898;mov
871;gcd
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordUnigram> = parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {}", record.count, record.word)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "28868 add,9898 mov,871 gcd");
    }
}
