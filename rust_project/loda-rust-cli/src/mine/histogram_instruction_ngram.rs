use std::error::Error;
use std::io::BufRead;
use serde::Deserialize;
use serde::de::DeserializeOwned;

pub struct HistogramInstructionNgram {
}

impl HistogramInstructionNgram {
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

#[derive(Debug, Deserialize)]
struct RecordBigram {
    count: u32,
    word0: String,
    word1: String,
}

#[derive(Debug, Deserialize)]
struct RecordTrigram {
    count: u32,
    word0: String,
    word1: String,
    word2: String,
}

#[derive(Debug, Deserialize)]
struct RecordSkipgram {
    count: u32,
    word0: String,
    word2: String,
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
