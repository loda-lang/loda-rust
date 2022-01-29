use super::RecordTrigram;
use std::collections::HashMap;
use rand::Rng;
use rand::seq::SliceRandom;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetValue {
    ProgramStart,
    ProgramStop,
    Value(i32),
    None,
}

impl TargetValue {
    // Convert string to an integer
    // Convert "42" to TargetValue::Value(42).
    // Convert "START" to TargetValue::ProgramStart.
    // Convert "STOP" to TargetValue::ProgramStop.
    // Convert "NONE" to TargetValue::None.
    // Convert "JUNK" to Optional::None.
    fn parse<S>(content: S) -> Option<TargetValue> where S: Into<String> {
        let s: String = content.into();
        match s.as_str() {
            "START" => {
                return Some(TargetValue::ProgramStart);
            },
            "STOP" => {
                return Some(TargetValue::ProgramStop);
            },
            "NONE" => {
                return Some(TargetValue::None);
            },
            _ => {}
        }
        match s.parse::<i32>() {
            Ok(value) => {
                return Some(TargetValue::Value(value));
            },
            Err(_) => {
                return None;
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Value(value) => return format!("{}", value),
            Self::ProgramStart => return "START".to_string(),
            Self::ProgramStop => return "STOP".to_string(),
            Self::None => return "NONE".to_string(),
        }
    }
}

type HistogramKey = (String,String);
type ValueAndWeight = (TargetValue,u32);
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

    pub fn populate(&mut self, records: &Vec<RecordTrigram>) {
        for record in records {
            let value1: TargetValue = match TargetValue::parse(&record.word1) {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let key: HistogramKey = (record.word0.clone(), record.word2.clone());
            let value_and_weight: ValueAndWeight = (value1, record.count);
            let item = self.histogram.entry(key).or_insert(vec!());
            (*item).push(value_and_weight);
        }
    }

    // If it's the beginning of the program then set prev_word to ProgramStart.
    // If it's the end of the program then set next_word to ProgramStop.
    // If it's a `lpe` (loop end) instruction that has no parameter, then use None.
    #[allow(dead_code)]
    fn candidates(&self, prev_word: TargetValue, next_word: TargetValue) -> Option<&HistogramValue> {
        let word0: String = prev_word.to_string();
        let word2: String = next_word.to_string();
        let key: HistogramKey = (word0, word2);
        self.histogram.get(&key)
    }

    #[allow(dead_code)]
    pub fn best_candidate(&self, prev_word: TargetValue, next_word: TargetValue) -> Option<TargetValue> {
        let histogram_value: &HistogramValue = match self.candidates(prev_word, next_word) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let candidate_value: TargetValue = match histogram_value.first() {
            Some(value) => value.0.clone(),
            None => {
                return None;
            }
        };
        Some(candidate_value)
    }

    #[allow(dead_code)]
    pub fn choose_weighted<R: Rng + ?Sized>(&self, rng: &mut R, prev_word: TargetValue, next_word: TargetValue) -> Option<TargetValue> {
        let histogram_value: &HistogramValue = match self.candidates(prev_word, next_word) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let value: TargetValue = histogram_value.choose_weighted(rng, |item| item.1).unwrap().0.clone();
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
            RecordTrigram {
                count: 1000,
                word0: "START".to_string(),
                word1: "20".to_string(),
                word2: "2".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "3".to_string(),
                word1: "30".to_string(),
                word2: "STOP".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "START".to_string(),
                word1: "40".to_string(),
                word2: "STOP".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "NONE".to_string(),
                word1: "6".to_string(),
                word2: "NONE".to_string()
            },
        ];
        v
    }

    #[test]
    fn test_10000_choose_weighted_surrounded_by_other_words0() {
        let mock = mockdata();
        let mut si = SuggestTarget::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<TargetValue> = si.choose_weighted(
            &mut rng, 
            TargetValue::Value(0), 
            TargetValue::Value(0)
        );
        assert_eq!(actual, Some(TargetValue::Value(0)));
    }

    #[test]
    fn test_10001_choose_weighted_surrounded_by_other_words1() {
        let mock = mockdata();
        let mut si = SuggestTarget::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<TargetValue> = si.choose_weighted(
            &mut rng, 
            TargetValue::Value(1),
            TargetValue::Value(1)
        );
        assert_eq!(actual, Some(TargetValue::Value(1)));
    }

    #[test]
    fn test_10002_choose_weighted_start_of_program() {
        let mock = mockdata();
        let mut si = SuggestTarget::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<TargetValue> = si.choose_weighted(
            &mut rng, 
            TargetValue::ProgramStart,
            TargetValue::Value(2)
        );
        assert_eq!(actual, Some(TargetValue::Value(20)));
    }

    #[test]
    fn test_10003_choose_weighted_end_of_program() {
        let mock = mockdata();
        let mut si = SuggestTarget::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<TargetValue> = si.choose_weighted(
            &mut rng, 
            TargetValue::Value(3),
            TargetValue::ProgramStop
        );
        assert_eq!(actual, Some(TargetValue::Value(30)));
    }

    #[test]
    fn test_10004_choose_weighted_start_and_end_of_program() {
        let mock = mockdata();
        let mut si = SuggestTarget::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<TargetValue> = si.choose_weighted(
            &mut rng, 
            TargetValue::ProgramStart,
            TargetValue::ProgramStop
        );
        assert_eq!(actual, Some(TargetValue::Value(40)));
    }

    #[test]
    fn test_10005_choose_weighted_surrounded_by_none() {
        let mock = mockdata();
        let mut si = SuggestTarget::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<TargetValue> = si.choose_weighted(
            &mut rng, 
            TargetValue::None,
            TargetValue::None
        );
        assert_eq!(actual, Some(TargetValue::Value(6)));
    }

    #[test]
    fn test_10006_choose_weighted_unrecognized_input() {
        let mock = mockdata();
        let mut si = SuggestTarget::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<TargetValue> = si.choose_weighted(
            &mut rng, 
            TargetValue::Value(666),
            TargetValue::Value(666)
        );
        assert_eq!(actual, None);
    }
}
