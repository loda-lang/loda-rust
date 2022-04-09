use std::error::Error;
use serde::Deserialize;
use std::path::Path;
use crate::common::parse_csv_file;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RecordSimilar {
    #[serde(rename = "program id")]
    pub program_id: u32,

    #[serde(rename = "overlap")]
    pub overlap_count: u16,
}

impl RecordSimilar {
    #[allow(dead_code)]
    pub fn parse_csv(path: &Path) -> Result<Vec<RecordSimilar>, Box<dyn Error>> {
        parse_csv_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::parse_csv_data;

    #[test]
    fn test_10000_parse_ok() {
        let data = "\
program id;overlap
20761;27
147601;13
157912;12
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordSimilar> = parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {}", record.program_id, record.overlap_count)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "20761 27,147601 13,157912 12");
    }
}
