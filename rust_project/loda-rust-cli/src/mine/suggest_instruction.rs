use super::histogram_instruction_ngram::{RecordTrigram, TrigramVec};
use loda_rust_core::parser::InstructionId;
use std::collections::HashMap;
use rand::Rng;
use rand::seq::SliceRandom;
use std::error::Error;

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

    pub fn populate(&mut self, ngram: &dyn TrigramVec) -> Result<(), Box<dyn Error>> {
        let records: Vec<RecordTrigram> = ngram.trigram_vec()?;
        for record in records {
            let key: HistogramKey = (record.word0, record.word2);
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
        Ok(())
    }

    // If it's the beginning of the program then set prev_instruction to None.
    // If it's the end of the program then set next_instruction to None.
    #[allow(dead_code)]
    fn candidate_instructions(&self, prev_instruction: Option<InstructionId>, next_instruction: Option<InstructionId>) -> Option<&HistogramValue> {
        let word0: String = match prev_instruction {
            Some(instruction_id) => instruction_id.shortname().to_string(),
            None => "START".to_string()
        };
        let word2: String = match next_instruction {
            Some(instruction_id) => instruction_id.shortname().to_string(),
            None => "STOP".to_string()
        };
        let key: HistogramKey = (word0, word2);
        self.histogram.get(&key)
    }

    #[allow(dead_code)]
    pub fn best_instruction(&self, prev_instruction: Option<InstructionId>, next_instruction: Option<InstructionId>) -> Option<InstructionId> {
        let histogram_value: &HistogramValue = match self.candidate_instructions(prev_instruction, next_instruction) {
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
    pub fn choose_weighted<R: Rng + ?Sized>(&self, rng: &mut R, prev_instruction: Option<InstructionId>, next_instruction: Option<InstructionId>) -> Option<InstructionId> {
        let histogram_value: &HistogramValue = match self.candidate_instructions(prev_instruction, next_instruction) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let instruction_id: InstructionId = histogram_value.choose_weighted(rng, |item| item.1).unwrap().0;
        Some(instruction_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    struct MockTrigramVec {}

    impl TrigramVec for MockTrigramVec {
        fn trigram_vec(&self) -> Result<Vec<RecordTrigram>, Box<dyn Error>> {
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
            Ok(v)
        }
    }

    #[test]
    fn test_10000_choose_weighted_instruction_surrounded_by_other_instructions() {
        let mock = MockTrigramVec {};
        let mut si = SuggestInstruction::new();
        si.populate(&mock).expect("should not fail");
        let mut rng = StdRng::seed_from_u64(0);
        let actual: InstructionId = si.choose_weighted(&mut rng, Some(InstructionId::Move), Some(InstructionId::Multiply)).unwrap();
        assert_eq!(actual, InstructionId::Divide);
    }

    #[test]
    fn test_10001_choose_weighted_start_of_program() {
        let mock = MockTrigramVec {};
        let mut si = SuggestInstruction::new();
        si.populate(&mock).expect("should not fail");
        let mut rng = StdRng::seed_from_u64(0);
        let actual: InstructionId = si.choose_weighted(&mut rng, None, Some(InstructionId::Add)).unwrap();
        assert_eq!(actual, InstructionId::Subtract);
    }

    #[test]
    fn test_10002_choose_weighted_end_of_program() {
        let mock = MockTrigramVec {};
        let mut si = SuggestInstruction::new();
        si.populate(&mock).expect("should not fail");
        let mut rng = StdRng::seed_from_u64(0);
        let actual: InstructionId = si.choose_weighted(&mut rng, Some(InstructionId::GCD), None).unwrap();
        assert_eq!(actual, InstructionId::Min);
    }

    #[test]
    fn test_10003_choose_weighted_start_and_end_of_program() {
        let mock = MockTrigramVec {};
        let mut si = SuggestInstruction::new();
        si.populate(&mock).expect("should not fail");
        let mut rng = StdRng::seed_from_u64(0);
        let actual: InstructionId = si.choose_weighted(&mut rng, None, None).unwrap();
        assert_eq!(actual, InstructionId::Max);
    }

    #[test]
    fn test_10004_choose_weighted_unrecognized_input() {
        let mock = MockTrigramVec {};
        let mut si = SuggestInstruction::new();
        si.populate(&mock).expect("should not fail");
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<InstructionId> = si.choose_weighted(
            &mut rng, Some(InstructionId::DivideIf), Some(InstructionId::DivideIf)
        );
        assert_eq!(actual, None);
    }
}
