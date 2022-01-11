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

type ValueAndWeight = (i32,u32);
type ValueAndWeightVector = Vec<ValueAndWeight>;

// Instructions that takes a constant value.
//
// The most used combo: `add $0,1` (addition)
// Almost as popular combo: `sub $0,1` (subtract by 1)
//
// Usecase:
// During mining, when mutating an `add` instruction.
//
// It's a time waster making poor choices of constants.
// The miner originally picked random integers, but it was excruciating slow.
//
// Time can be saved this way. 
// Before mining: analyze all programs and build a histogram.
// During mining: make weighted choices from the histogram.
pub struct HistogramInstructionConstant {
    instruction_and_valueweightvector: HashMap<InstructionId, ValueAndWeightVector>
}

impl HistogramInstructionConstant {
    pub fn load_csv_file(path: &Path) -> Result<HistogramInstructionConstant, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let records: Vec<Record> = Record::parse_csv_data(&mut reader)?;
        let instruction_and_valueweightvector: HashMap<InstructionId, ValueAndWeightVector> = 
            Record::instruction_and_valueweightvector(&records);
        let result = Self {
            instruction_and_valueweightvector: instruction_and_valueweightvector
        };
        Ok(result)
    }

    pub fn choose_weighted<R: Rng + ?Sized>(&self, rng: &mut R, instruction_id: InstructionId) -> Option<i32> {
        let value_and_weight_vec: &ValueAndWeightVector = 
        match self.instruction_and_valueweightvector.get(&instruction_id) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let value: i32 = value_and_weight_vec.choose_weighted(rng, |item| item.1).unwrap().0;
        Some(value)
    }
}

#[derive(Debug, Deserialize)]
struct Record {
    count: u32,
    instruction: String,
    constant: i32,
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

    fn unique_instruction_ids(records: &Vec<Record>) -> HashSet<InstructionId> {
        let mut instruction_ids = Vec::<InstructionId>::new();
        for record in records {
            match InstructionId::parse(&record.instruction, 0) {
                Ok(instruction_id) => {
                    instruction_ids.push(instruction_id);
                },
                Err(_) => {}
            }
        }
        HashSet::from_iter(instruction_ids.iter().cloned())
    }

    fn value_and_weight_vec(records: &Vec<Record>, instruction_id: InstructionId) -> ValueAndWeightVector {
        let mut value_and_weight_vec: ValueAndWeightVector = vec!();
        let needle: &str = instruction_id.shortname();
        let mut already_known = HashSet::<i32>::new();
        for record in records {
            if record.instruction != needle {
                continue;
            }
            if already_known.contains(&record.constant) {
                // Keep only the first value. Ignore following duplicates.
                continue;
            }
            let value = (record.constant, record.count);
            value_and_weight_vec.push(value);
            already_known.insert(record.constant);
        }
        value_and_weight_vec
    }

    fn instruction_and_valueweightvector(records: &Vec<Record>) -> HashMap<InstructionId, ValueAndWeightVector> {
        let instruction_ids: HashSet<InstructionId> = Record::unique_instruction_ids(&records);
        let mut result: HashMap<InstructionId, ValueAndWeightVector> = HashMap::new();
        for instruction_id in instruction_ids {
            let value_and_weight_vec: ValueAndWeightVector = 
                Record::value_and_weight_vec(&records, instruction_id);
            if value_and_weight_vec.is_empty() {
                continue;
            }
            result.insert(instruction_id, value_and_weight_vec);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_10000_parse_csv_data() {
        let data = "\
count;instruction;constant
36545;add;1
33648;sub;1
17147;mul;-2
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<Record> = Record::parse_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {}", record.count, record.instruction, record.constant)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "36545 add 1,33648 sub 1,17147 mul -2");
    }
    
    #[test]
    fn test_10001_unique_instruction_ids() {
        let data = "\
count;instruction;constant
36545;add;1
9232;add;2
666;unknown;23
555;sub;1
171;mul;-2
92;add;3
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<Record> = Record::parse_csv_data(&mut input).unwrap();
        let actual: HashSet<InstructionId> = Record::unique_instruction_ids(&records);
        let v = vec![InstructionId::Add, InstructionId::Subtract, InstructionId::Multiply];
        let expected: HashSet<InstructionId> = HashSet::from_iter(v);
        assert_eq!(actual, expected);
    }
    
    #[test]
    fn test_10002_value_and_weight_vec_typical_data() {
        let data = "\
count;instruction;constant
36545;add;1
92;add;2
9232;add;3
100;add;4
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<Record> = Record::parse_csv_data(&mut input).unwrap();
        let actual: ValueAndWeightVector = Record::value_and_weight_vec(&records, InstructionId::Add);
        let expected: ValueAndWeightVector = vec![(1,36545),(2,92),(3,9232),(4,100)];
        assert_eq!(actual, expected);
    }
    
    #[test]
    fn test_10003_value_and_weight_vec_without_duplicates() {
        let data = "\
count;instruction;constant
1000;add;1
1000;add;2
1000;add;1
1000;add;3
1001;add;1
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<Record> = Record::parse_csv_data(&mut input).unwrap();
        let actual: ValueAndWeightVector = Record::value_and_weight_vec(&records, InstructionId::Add);
        let expected: ValueAndWeightVector = vec![(1,1000),(2,1000),(3,1000)];
        assert_eq!(actual, expected);
    }
    
    #[test]
    fn test_10004_instruction_and_valueweightvector() {
        let data = "\
count;instruction;constant
1001;add;1
1002;add;2
999;div;2
998;mul;-1
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<Record> = Record::parse_csv_data(&mut input).unwrap();
        let actual: HashMap<InstructionId, ValueAndWeightVector> = Record::instruction_and_valueweightvector(&records);
        assert_eq!(actual.len(), 3);
    }
}
