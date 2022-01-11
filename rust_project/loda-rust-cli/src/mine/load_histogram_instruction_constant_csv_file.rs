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

struct MostPopularConstant {
    instruction_and_valueweightvector: HashMap<InstructionId, ValueAndWeightVector>
}

impl MostPopularConstant {
    fn new(records: &Vec<Record>) -> MostPopularConstant {
        let hashset: HashSet<InstructionId> = Record::unique_instruction_ids(records);
        let unique_instruction_ids: Vec<InstructionId> = Vec::from_iter(hashset);

        let instruction_ids: &[InstructionId] = &[
            InstructionId::Add,
            InstructionId::Divide,
        ];
        let mut result: HashMap<InstructionId, ValueAndWeightVector> = HashMap::new();
        for instruction_id in instruction_ids {
            let value_and_weight_vec: ValueAndWeightVector = 
                Self::extract_value_and_weight_vec(records, &instruction_id);
            result.insert(*instruction_id, value_and_weight_vec);
        }
        Self {
            instruction_and_valueweightvector: result
        }
    }

    fn extract_value_and_weight_vec(records: &Vec<Record>, instruction_id: &InstructionId) -> ValueAndWeightVector {
        let mut value_and_weight_vec: ValueAndWeightVector = vec!();
        let needle: &str = instruction_id.shortname();
        for record in records {
            if record.instruction != needle {
                continue;
            }
            let value = (record.constant, record.count);
            value_and_weight_vec.push(value);
        }
        value_and_weight_vec
    }

    fn choose_weighted<R: Rng + ?Sized>(&self, rng: &mut R, instruction_id: InstructionId) -> Option<i32> {
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

fn process_csv_data(reader: &mut dyn BufRead) -> Result<Vec<Record>, Box<dyn Error>> {
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

impl Record {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_10000_process_csv_data() {
        let data = "\
count;instruction;constant
36545;add;1
33648;sub;1
17147;mul;-2
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<Record> = process_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {} {}", record.count, record.instruction, record.constant)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "36545 add 1,33648 sub 1,17147 mul -2");
    }
}
