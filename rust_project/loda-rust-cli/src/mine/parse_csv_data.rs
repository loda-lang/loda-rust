use std::error::Error;
use std::io::BufRead;
use serde::de::DeserializeOwned;

#[allow(dead_code)]
pub fn parse_csv_data<D: DeserializeOwned>(reader: &mut dyn BufRead) 
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    
    #[derive(Debug, Deserialize)]
    struct Record {
        count: u32,
        comment: String,
    }
    
    #[test]
    fn test_10000_parse_ok() {
        let data = "\
count;comment
0;a
1;b
999;c
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<Record> = parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {}", record.count, record.comment)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "0 a,1 b,999 c");
    }
    
    #[test]
    fn test_20000_parse_error() {
        let data = "\
count;comment
bad29843;Expected unsigned, but got junk
";
        let mut input: &[u8] = data.as_bytes();
        let result = parse_csv_data::<Record>(&mut input);
        assert_eq!(result.is_err(), true);
    }
    
    #[test]
    fn test_20001_parse_error() {
        let data = "\
count;comment
-29843;Expected unsigned, but got a negative value
";
        let mut input: &[u8] = data.as_bytes();
        let result = parse_csv_data::<Record>(&mut input);
        assert_eq!(result.is_err(), true);
    }
    
    #[test]
    fn test_20002_parse_error() {
        let data = "\
count;comment
;Expected unsigned, but got an empty string
";
        let mut input: &[u8] = data.as_bytes();
        let result = parse_csv_data::<Record>(&mut input);
        assert_eq!(result.is_err(), true);
    }
}
