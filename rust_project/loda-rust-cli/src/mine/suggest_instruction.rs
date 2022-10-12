use crate::common::RecordTrigram;
use super::random_indexes_with_distance;
use loda_rust_core::parser::InstructionId;
use std::collections::HashMap;
use rand::Rng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

type HistogramKey = (String,String);
type InstructionAndWeight = (InstructionId,u32);
type HistogramValue = Vec<InstructionAndWeight>;

pub struct SuggestInstruction {
    histogram: HashMap<HistogramKey, HistogramValue>
}

impl SuggestInstruction {
    pub fn new() -> Self {
        Self {
            histogram: HashMap::new()
        }
    }

    const SHUFFLE_COUNT: usize = 20;

    pub fn populate(&mut self, records_original: &Vec<RecordTrigram>) {
        let mut records: Vec<RecordTrigram> = records_original.clone();
        let seed: u64 = 1;
        let mut rng = StdRng::seed_from_u64(seed);
        let indexes: Vec<usize> = random_indexes_with_distance(&mut rng, records.len(), Self::SHUFFLE_COUNT);
        for index in indexes {
            records[index].count = records_original[index].count;
        }

        for record in records {
            let key: HistogramKey = (record.word0.clone(), record.word2.clone());
            let instruction_id: InstructionId = match InstructionId::parse(&record.word1, 0) {
                Ok(instruction_id) => instruction_id,
                Err(_) => {
                    continue;
                }
            };
            if instruction_id == InstructionId::LoopBegin ||
                instruction_id == InstructionId::LoopEnd ||
                instruction_id == InstructionId::Clear ||
                instruction_id == InstructionId::EvalSequence {
                // Don't suggest these instructions
                continue;
            }
            let instruction_and_weight: InstructionAndWeight = (instruction_id, record.count);
            let item = self.histogram.entry(key).or_insert(vec!());
            (*item).push(instruction_and_weight);
        }
    }

    /// If it's the beginning of the program then set `prev_word` to `None`.
    /// 
    /// If it's the end of the program then set `next_word` to `None`.
    #[allow(dead_code)]
    fn candidates(&self, prev_word: Option<InstructionId>, next_word: Option<InstructionId>) -> Option<&HistogramValue> {
        let word0: String = match prev_word {
            Some(instruction_id) => instruction_id.shortname().to_string(),
            None => "START".to_string()
        };
        let word2: String = match next_word {
            Some(instruction_id) => instruction_id.shortname().to_string(),
            None => "STOP".to_string()
        };
        let key: HistogramKey = (word0, word2);
        self.histogram.get(&key)
    }

    #[allow(dead_code)]
    pub fn best_instruction(&self, prev_word: Option<InstructionId>, next_word: Option<InstructionId>) -> Option<InstructionId> {
        let histogram_value: &HistogramValue = match self.candidates(prev_word, next_word) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let instruction_id: InstructionId = match histogram_value.first() {
            Some(value) => value.0,
            None => {
                return None;
            }
        };
        Some(instruction_id)
    }

    #[allow(dead_code)]
    pub fn choose_weighted<R: Rng + ?Sized>(&self, rng: &mut R, prev_word: Option<InstructionId>, next_word: Option<InstructionId>) -> Option<InstructionId> {
        let histogram_value: &HistogramValue = match self.candidates(prev_word, next_word) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let value: InstructionId = histogram_value.choose_weighted(rng, |item| item.1).unwrap().0;
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn mockdata() -> Vec<RecordTrigram> {
        let v = vec![
            RecordTrigram {
                count: 1000,
                word0: "mov".to_string(),
                word1: "div".to_string(),
                word2: "mul".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "START".to_string(),
                word1: "sub".to_string(),
                word2: "add".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "gcd".to_string(),
                word1: "min".to_string(),
                word2: "STOP".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "START".to_string(),
                word1: "max".to_string(),
                word2: "STOP".to_string()
            },
        ];
        v
    }

    fn exercise_choose_weighted(prev_word: Option<InstructionId>, next_word: Option<InstructionId>) -> Option<InstructionId> {
        let mock = mockdata();
        let mut si = SuggestInstruction::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<InstructionId> = si.choose_weighted(
            &mut rng, prev_word, next_word
        );
        actual
    }

    #[test]
    fn test_10000_choose_weighted_surrounded_by_other_words() {
        let actual: Option<InstructionId> = exercise_choose_weighted(
            Some(InstructionId::Move), Some(InstructionId::Multiply)
        );
        assert_eq!(actual, Some(InstructionId::Divide));
    }

    #[test]
    fn test_10001_choose_weighted_start_of_program() {
        let actual: Option<InstructionId> = exercise_choose_weighted(
            None, Some(InstructionId::Add)
        );
        assert_eq!(actual, Some(InstructionId::Subtract));
    }

    #[test]
    fn test_10002_choose_weighted_end_of_program() {
        let actual: Option<InstructionId> = exercise_choose_weighted(
            Some(InstructionId::GCD), None
        );
        assert_eq!(actual, Some(InstructionId::Min));
    }

    #[test]
    fn test_10003_choose_weighted_start_and_end_of_program() {
        let actual: Option<InstructionId> = exercise_choose_weighted(
            None, None
        );
        assert_eq!(actual, Some(InstructionId::Max));
    }

    #[test]
    fn test_10004_choose_weighted_unrecognized_input() {
        let actual: Option<InstructionId> = exercise_choose_weighted(
            Some(InstructionId::DivideIf), Some(InstructionId::DivideIf)
        );
        assert_eq!(actual, None);
    }
}
