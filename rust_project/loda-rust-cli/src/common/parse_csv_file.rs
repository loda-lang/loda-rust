use std::error::Error;
use std::io::BufReader;
use std::path::Path;
use std::fs::File;
use serde::de::DeserializeOwned;
use super::parse_csv_data;

pub fn parse_csv_file<D: DeserializeOwned>(path: &Path) -> Result<Vec<D>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let records: Vec<D> = parse_csv_data::<D>(&mut reader)?;
    Ok(records)
}
