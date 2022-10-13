use crate::common::RecordTrigram;
use super::random_indexes_with_distance;
use std::collections::HashMap;
use rand::Rng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SourceValue {
    ProgramStart,
    ProgramStop,
    Constant,
    Direct(i32),
    None,
}

impl SourceValue {
    /// Convert string to an enum.
    /// 
    /// Convert "42" to `SourceValue::Direct(42)`.
    /// 
    /// Convert "START" to `SourceValue::ProgramStart`.
    /// 
    /// Convert "STOP" to `SourceValue::ProgramStop`.
    /// 
    /// Convert "CONST" to `SourceValue::Constant`.
    /// 
    /// Convert "NONE" to `SourceValue::None`.
    /// 
    /// Convert "JUNK" to `Optional::None`.
    fn parse<S>(content: S) -> Option<SourceValue> where S: Into<String> {
        let s: String = content.into();
        match s.as_str() {
            "START" => {
                return Some(SourceValue::ProgramStart);
            },
            "STOP" => {
                return Some(SourceValue::ProgramStop);
            },
            "CONST" => {
                return Some(SourceValue::Constant);
            },
            "NONE" => {
                return Some(SourceValue::None);
            },
            _ => {}
        }
        if s.starts_with('+') {
            return None;
        }
        if s.starts_with('-') {
            return None;
        }
        match s.parse::<i32>() {
            Ok(value) => {
                return Some(SourceValue::Direct(value));
            },
            Err(_) => {
                return None;
            }
        }
    }

    #[allow(dead_code)]
    fn to_string(&self) -> String {
        match self {
            Self::Direct(value) => return format!("{}", value),
            Self::ProgramStart => return "START".to_string(),
            Self::ProgramStop => return "STOP".to_string(),
            Self::Constant => return "CONST".to_string(),
            Self::None => return "NONE".to_string(),
        }
    }
}

type HistogramKey = (SourceValue,SourceValue);
type ValueAndWeight = (SourceValue,u32);
type HistogramValue = Vec<ValueAndWeight>;

pub struct SuggestSource {
    histogram: HashMap<HistogramKey, HistogramValue>
}

impl SuggestSource {
    pub fn new() -> Self {
        Self {
            histogram: HashMap::new()
        }
    }

    const SHUFFLE_COUNT: usize = 50;

    pub fn populate(&mut self, records_original: &Vec<RecordTrigram>) {
        let mut records: Vec<RecordTrigram> = records_original.clone();
        let seed: u64 = 1;
        let mut rng = StdRng::seed_from_u64(seed);
        let indexes: Vec<usize> = random_indexes_with_distance(&mut rng, records.len(), Self::SHUFFLE_COUNT);
        for index in indexes {
            records[index].count = records_original[index].count;
        }

        for record in records {
            let value0: SourceValue = match SourceValue::parse(&record.word0) {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let value1: SourceValue = match SourceValue::parse(&record.word1) {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let value2: SourceValue = match SourceValue::parse(&record.word2) {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let key: HistogramKey = (value0, value2);
            let value_and_weight: ValueAndWeight = (value1, record.count);
            let item = self.histogram.entry(key).or_insert(vec!());
            (*item).push(value_and_weight);
        }
    }

    /// If it's the beginning of the program then set `prev_word` to `ProgramStart`.
    /// 
    /// If it's the end of the program then set `next_word` to `ProgramStop`.
    /// 
    /// If it's a `lpb` (loop begin) instruction can have a 2nd parameter, then `None` may be used.
    /// 
    /// If it's a `lpe` (loop end) instruction that has no parameters, then use `None`.
    #[allow(dead_code)]
    fn candidates(&self, prev_word: SourceValue, next_word: SourceValue) -> Option<&HistogramValue> {
        let key: HistogramKey = (prev_word, next_word);
        self.histogram.get(&key)
    }

    #[allow(dead_code)]
    pub fn best_candidate(&self, prev_word: SourceValue, next_word: SourceValue) -> Option<SourceValue> {
        let histogram_value: &HistogramValue = match self.candidates(prev_word, next_word) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let candidate_value: SourceValue = match histogram_value.first() {
            Some(value) => value.0.clone(),
            None => {
                return None;
            }
        };
        Some(candidate_value)
    }

    #[allow(dead_code)]
    pub fn choose_weighted<R: Rng + ?Sized>(&self, rng: &mut R, prev_word: SourceValue, next_word: SourceValue) -> Option<SourceValue> {
        let histogram_value: &HistogramValue = match self.candidates(prev_word, next_word) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let value: SourceValue = histogram_value.choose_weighted(rng, |item| item.1).unwrap().0.clone();
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    static INPUT: &'static [&'static str] = &[
        "START",
        "STOP",
        "CONST",
        "NONE",
        "42",
        "0",
        "+42",
        "-1",
        "$1",
        "boom",
        "",
        " 0",
        " 0 ",
    ];

    static OUTPUT: &'static [&'static str] = &[
        "START",
        "STOP",
        "CONST",
        "NONE",
        "42",
        "0",
        "IGNORE",
        "IGNORE",
        "IGNORE",
        "IGNORE",
        "IGNORE",
        "IGNORE",
        "IGNORE",
    ];

    fn process<S: AsRef<str>>(input: S) -> String {
        let input = input.as_ref();
        let source_value: SourceValue = match SourceValue::parse(input) {
            Some(value) => value,
            None => {
                return "IGNORE".to_string();
            }
        };
        source_value.to_string()
    }

    #[test]
    fn test_10000_source_value_parse() {
        for (index, input) in INPUT.iter().enumerate() {
            assert_eq!(process(input), OUTPUT[index]);
        }
    }

    fn mockdata() -> Vec<RecordTrigram> {
        let v = vec![
            RecordTrigram {
                count: 1000,
                word0: "0".to_string(),
                word1: "CONST".to_string(),
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
            RecordTrigram {
                count: 1000,
                word0: "CONST".to_string(),
                word1: "CONST".to_string(),
                word2: "CONST".to_string()
            },
        ];
        v
    }

    fn exercise_choose_weighted(prev_word: SourceValue, next_word: SourceValue) -> Option<SourceValue> {
        let mock = mockdata();
        let mut si = SuggestSource::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<SourceValue> = si.choose_weighted(
            &mut rng, 
            prev_word, 
            next_word
        );
        actual
    }

    #[test]
    fn test_20000_choose_weighted_surrounded_by_other_words0() {
        let actual: Option<SourceValue> = exercise_choose_weighted(
            SourceValue::Direct(0), 
            SourceValue::Direct(0)
        );
        assert_eq!(actual, Some(SourceValue::Constant));
    }

    #[test]
    fn test_20001_choose_weighted_surrounded_by_other_words1() {
        let actual: Option<SourceValue> = exercise_choose_weighted(
            SourceValue::Direct(1),
            SourceValue::Direct(1)
        );
        assert_eq!(actual, Some(SourceValue::Direct(1)));
    }

    #[test]
    fn test_20002_choose_weighted_start_of_program() {
        let actual: Option<SourceValue> = exercise_choose_weighted(
            SourceValue::ProgramStart,
            SourceValue::Direct(2)
        );
        assert_eq!(actual, Some(SourceValue::Direct(20)));
    }

    #[test]
    fn test_20003_choose_weighted_end_of_program() {
        let actual: Option<SourceValue> = exercise_choose_weighted(
            SourceValue::Direct(3),
            SourceValue::ProgramStop
        );
        assert_eq!(actual, Some(SourceValue::Direct(30)));
    }

    #[test]
    fn test_20004_choose_weighted_start_and_end_of_program() {
        let actual: Option<SourceValue> = exercise_choose_weighted(
            SourceValue::ProgramStart,
            SourceValue::ProgramStop
        );
        assert_eq!(actual, Some(SourceValue::Direct(40)));
    }

    #[test]
    fn test_20005_choose_weighted_surrounded_by_none() {
        let actual: Option<SourceValue> = exercise_choose_weighted(
            SourceValue::None,
            SourceValue::None
        );
        assert_eq!(actual, Some(SourceValue::Direct(6)));
    }

    #[test]
    fn test_20006_choose_weighted_surrounded_by_const() {
        let actual: Option<SourceValue> = exercise_choose_weighted(
            SourceValue::Constant,
            SourceValue::Constant
        );
        assert_eq!(actual, Some(SourceValue::Constant));
    }

    #[test]
    fn test_20007_choose_weighted_unrecognized_input() {
        let actual: Option<SourceValue> = exercise_choose_weighted(
            SourceValue::Direct(666),
            SourceValue::Direct(666)
        );
        assert_eq!(actual, None);
    }
}
