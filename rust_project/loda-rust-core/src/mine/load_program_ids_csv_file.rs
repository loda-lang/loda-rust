use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use serde::Deserialize;

pub fn load_program_ids_csv_file(path: &Path) -> Result<Vec<u32>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    process_csv_data(&mut reader)
}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "program id")]
    program_id: u32,
}

fn process_csv_data(reader: &mut dyn BufRead) -> Result<Vec<u32>, Box<dyn Error>> {
    let mut rows = Vec::<u32>::new();
    let mut csv_reader = csv::Reader::from_reader(reader);
    for result in csv_reader.deserialize() {
        let record: Record = result?;
        rows.push(record.program_id);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_10000_process_csv_data() {
        let data = "\
program id
10

45
1234
";
        let mut input: &[u8] = data.as_bytes();
        let program_ids: Vec<u32> = process_csv_data(&mut input).unwrap();
        assert_eq!(program_ids, vec![10, 45, 1234]);
    }
}
