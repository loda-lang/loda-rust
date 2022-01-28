use super::RecordTrigram;
use std::collections::HashMap;
use rand::Rng;
use rand::seq::SliceRandom;

type HistogramKey = (String,String);
type ValueAndWeight = (i32,u32);
type HistogramValue = Vec<ValueAndWeight>;

pub struct SuggestTarget {
    histogram: HashMap<HistogramKey, HistogramValue>
}

impl SuggestTarget {
    pub fn new() -> Self {
        Self {
            histogram: HashMap::new()
        }
    }

    // Convert string to an optional integer
    // Convert "42" to Some(42).
    // Convert "NONE" to None.
    // Convert "JUNK" to None.
    fn xparse_register<S>(content: S) -> Option<i32> where S: Into<String> {
        let s: String = content.into();
        match s.parse::<i32>() {
            Ok(value) => {
                return Some(value);
            },
            Err(_) => {
                return None;
            }
        }
    }

    fn parse_register<S>(content: S) -> i32 where S: Into<String> {
        let s: String = content.into();
        match s.parse::<i32>() {
            Ok(value) => {
                return value;
            },
            Err(_) => {
                return -1;
            }
        }
    }

    pub fn populate(&mut self, records: &Vec<RecordTrigram>) {
        for record in records {
            let value1: i32 = Self::parse_register(&record.word1);
            let key: HistogramKey = (record.word0.clone(), record.word2.clone());
            let value_and_weight: ValueAndWeight = (value1, record.count);
            let item = self.histogram.entry(key).or_insert(vec!());
            (*item).push(value_and_weight);
        }
    }

    // If it's the beginning of the program then set prev_word to None.
    // If it's the end of the program then set next_word to None.
    #[allow(dead_code)]
    fn candidates(&self, prev_word: Option<i32>, next_word: Option<i32>) -> Option<&HistogramValue> {
        let word0: String = match prev_word {
            Some(value) => value.to_string(),
            None => "START".to_string()
        };
        let word2: String = match next_word {
            Some(value) => value.to_string(),
            None => "STOP".to_string()
        };
        let key: HistogramKey = (word0, word2);
        self.histogram.get(&key)
    }

    #[allow(dead_code)]
    pub fn best_candidate(&self, prev_word: Option<i32>, next_word: Option<i32>) -> Option<i32> {
        let histogram_value: &HistogramValue = match self.candidates(prev_word, next_word) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let candidate_value: i32 = match histogram_value.first() {
            Some(value) => value.0,
            None => {
                return None;
            }
        };
        Some(candidate_value)
    }

    #[allow(dead_code)]
    pub fn choose_weighted<R: Rng + ?Sized>(&self, rng: &mut R, prev_word: Option<i32>, next_word: Option<i32>) -> Option<i32> {
        let histogram_value: &HistogramValue = match self.candidates(prev_word, next_word) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let value: i32 = histogram_value.choose_weighted(rng, |item| item.1).unwrap().0;
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
                word0: "0".to_string(),
                word1: "0".to_string(),
                word2: "0".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "1".to_string(),
                word1: "1".to_string(),
                word2: "1".to_string()
            },
        ];
        v
    }

    #[test]
    fn test_10000_choose_weighted_instruction_surrounded_by_other_words0() {
        let mock = mockdata();
        let mut si = SuggestTarget::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<i32> = si.choose_weighted(&mut rng, Some(0), Some(0));
        assert_eq!(actual, Some(0));
    }

    #[test]
    fn test_10001_choose_weighted_instruction_surrounded_by_other_words1() {
        let mock = mockdata();
        let mut si = SuggestTarget::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<i32> = si.choose_weighted(&mut rng, Some(1), Some(1));
        assert_eq!(actual, Some(1));
    }

    /*
    #[test]
    fn test_10001_choose_weighted_start_of_program() {
        let mock = mockdata();
        let mut si = SuggestInstruction::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: InstructionId = si.choose_weighted(&mut rng, None, Some(InstructionId::Add)).unwrap();
        assert_eq!(actual, InstructionId::Subtract);
    }

    #[test]
    fn test_10002_choose_weighted_end_of_program() {
        let mock = mockdata();
        let mut si = SuggestInstruction::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: InstructionId = si.choose_weighted(&mut rng, Some(InstructionId::GCD), None).unwrap();
        assert_eq!(actual, InstructionId::Min);
    }

    #[test]
    fn test_10003_choose_weighted_start_and_end_of_program() {
        let mock = mockdata();
        let mut si = SuggestInstruction::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: InstructionId = si.choose_weighted(&mut rng, None, None).unwrap();
        assert_eq!(actual, InstructionId::Max);
    }

    #[test]
    fn test_10004_choose_weighted_unrecognized_input() {
        let mock = mockdata();
        let mut si = SuggestInstruction::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<InstructionId> = si.choose_weighted(
            &mut rng, Some(InstructionId::DivideIf), Some(InstructionId::DivideIf)
        );
        assert_eq!(actual, None);
    } */
}
