use crate::common::RecordTrigram;
use loda_rust_core::parser::extract_parameter_re::EXTRACT_PARAMETER_RE;
use loda_rust_core::parser::ParameterType;
use super::random_indexes_with_distance;
use std::str::FromStr;
use std::collections::HashMap;
use rand::Rng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TargetValue {
    ProgramStart,
    ProgramStop,
    Direct(i32),
    Indirect(i32),
    None,
}

impl TargetValue {
    /// Convert string to an enum.
    /// 
    /// Convert "42" to `TargetValue::Direct(42)`.
    /// 
    /// Convert "START" to `TargetValue::ProgramStart`.
    /// 
    /// Convert "STOP" to `TargetValue::ProgramStop`.
    /// 
    /// Convert "NONE" to `TargetValue::None`.
    /// 
    /// Convert "JUNK" to `Optional::None`.
    fn parse<S>(content: S) -> anyhow::Result<TargetValue> where S: Into<String> {
        let s: String = content.into();
        match s.as_str() {
            "START" => {
                return Ok(TargetValue::ProgramStart);
            },
            "STOP" => {
                return Ok(TargetValue::ProgramStop);
            },
            "NONE" => {
                return Ok(TargetValue::None);
            },
            _ => {}
        }
        let re = &EXTRACT_PARAMETER_RE;
        let captures = match re.captures(&s) {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Unrecognized parameter, doesn't satisfy regex pattern."));
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let capture2: &str = captures.get(2).map_or("", |m| m.as_str());

        let parameter_type: ParameterType = match ParameterType::from_str(capture1) {
            Ok(value) => value,
            _ => {
                return Err(anyhow::anyhow!("Unrecognized parameter type"));
            }
        };
        let parameter_value: i32 = match i32::from_str(capture2) {
            Ok(value) => value,
            _ => {
                return Err(anyhow::anyhow!("Parameter value cannot be parsed as i32"));
            }
        };

        // Reject redundant leading zeroes, such as `0001`.
        // Reject redundant minus prefix `-0`.
        if parameter_value.to_string() != capture2 {
            return Err(anyhow::anyhow!("Strict incorrect parameter value"));
        }

        // Allow negative constants.
        // Reject negative addresses, such as `$-123` and `$$-4`.
        let check_negative: bool = match parameter_type {
            ParameterType::Direct | ParameterType::Indirect => true,
            ParameterType::Constant => false
        };
        if check_negative && parameter_value < 0 {
            return Err(anyhow::anyhow!("Negative value not allowed for this parameter type"));
        }

        match parameter_type {
            ParameterType::Constant => {
                return Err(anyhow::anyhow!("Encountered a ParameterType::Contant for target, which no instructions supports"));
            },
            ParameterType::Direct => {
                return Ok(TargetValue::Direct(parameter_value));
            },
            ParameterType::Indirect => {
                return Ok(TargetValue::Indirect(parameter_value));
            },
        }
    }

    #[allow(dead_code)]
    fn to_string(&self) -> String {
        match self {
            Self::ProgramStart => return "START".to_string(),
            Self::ProgramStop => return "STOP".to_string(),
            Self::Direct(value) => return format!("${}", value),
            Self::Indirect(value) => return format!("$${}", value),
            Self::None => return "NONE".to_string(),
        }
    }
}

type HistogramKey = (TargetValue,TargetValue);
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

    const SHUFFLE_COUNT: usize = 0;

    pub fn populate(&mut self, records_original: &Vec<RecordTrigram>) {
        // Make some noise in the histogram to prevent getting stuck in a local minimum.
        let mut records: Vec<RecordTrigram> = records_original.clone();
        let seed: u64 = 1;
        let mut rng = StdRng::seed_from_u64(seed);
        let indexes: Vec<usize> = random_indexes_with_distance(&mut rng, records.len(), Self::SHUFFLE_COUNT);
        for index in indexes {
            records[index].count = records_original[index].count;
        }

        for (index, record) in records.iter().enumerate() {
            let value0: TargetValue = match TargetValue::parse(&record.word0) {
                Ok(value) => value,
                Err(error) => {
                    debug!("SuggestTarget.populate(). ignoring row {}. column 0. error: {:?}", index, error);
                    continue;
                }
            };
            let value1: TargetValue = match TargetValue::parse(&record.word1) {
                Ok(value) => value,
                Err(error) => {
                    debug!("SuggestTarget.populate(). ignoring row {}. column 1. error: {:?}", index, error);
                    continue;
                }
            };
            let value2: TargetValue = match TargetValue::parse(&record.word2) {
                Ok(value) => value,
                Err(error) => {
                    debug!("SuggestTarget.populate(). ignoring row {}. column 2. error: {:?}", index, error);
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
    /// If it's a `lpe` (loop end) instruction that has no parameter, then use `None`.
    fn candidates(&self, prev_word: TargetValue, next_word: TargetValue) -> Option<&HistogramValue> {
        let key: HistogramKey = (prev_word, next_word);
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

    static INPUT: &'static [&'static str] = &[
        "START",
        "STOP",
        "NONE",
        "42",
        "$42",
        "$$42",
        "0",
        "$0",
        "$$0",
        "-1",
        "+42",
        "boom",
        "",
        " 0",
        " 0 ",
        "-0",
        "$-4",
        "$$-4",
    ];

    static OUTPUT: &'static [&'static str] = &[
        "START",
        "STOP",
        "NONE",
        "IGNORE",
        "$42",
        "$$42",
        "IGNORE",
        "$0",
        "$$0",
        "IGNORE",
        "IGNORE",
        "IGNORE",
        "IGNORE",
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
        let target_value: TargetValue = match TargetValue::parse(input) {
            Ok(value) => value,
            Err(_) => {
                return "IGNORE".to_string();
            }
        };
        target_value.to_string()
    }

    #[test]
    fn test_10000_target_value_parse() {
        for (index, input) in INPUT.iter().enumerate() {
            assert_eq!(process(input), OUTPUT[index]);
        }
    }

    fn mockdata() -> Vec<RecordTrigram> {
        let v = vec![
            RecordTrigram {
                count: 1000,
                word0: "$0".to_string(),
                word1: "$0".to_string(),
                word2: "$0".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "$1".to_string(),
                word1: "$1".to_string(),
                word2: "$1".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "START".to_string(),
                word1: "$20".to_string(),
                word2: "$2".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "$3".to_string(),
                word1: "$30".to_string(),
                word2: "STOP".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "START".to_string(),
                word1: "$40".to_string(),
                word2: "STOP".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "NONE".to_string(),
                word1: "$6".to_string(),
                word2: "NONE".to_string()
            },
            RecordTrigram {
                count: 1000,
                word0: "$$4".to_string(),
                word1: "$9".to_string(),
                word2: "$$3".to_string()
            },
        ];
        v
    }

    fn exercise_choose_weighted(prev_word: TargetValue, next_word: TargetValue) -> Option<TargetValue> {
        let mock = mockdata();
        let mut si = SuggestTarget::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<TargetValue> = si.choose_weighted(
            &mut rng, 
            prev_word, 
            next_word
        );
        actual
    }

    #[test]
    fn test_20000_choose_weighted_surrounded_by_other_words0() {
        let actual: Option<TargetValue> = exercise_choose_weighted(
            TargetValue::Direct(0), 
            TargetValue::Direct(0)
        );
        assert_eq!(actual, Some(TargetValue::Direct(0)));
    }

    #[test]
    fn test_20001_choose_weighted_surrounded_by_other_words1() {
        let actual: Option<TargetValue> = exercise_choose_weighted(
            TargetValue::Direct(1),
            TargetValue::Direct(1)
        );
        assert_eq!(actual, Some(TargetValue::Direct(1)));
    }

    #[test]
    fn test_20002_choose_weighted_start_of_program() {
        let actual: Option<TargetValue> = exercise_choose_weighted(
            TargetValue::ProgramStart,
            TargetValue::Direct(2)
        );
        assert_eq!(actual, Some(TargetValue::Direct(20)));
    }

    #[test]
    fn test_20003_choose_weighted_end_of_program() {
        let actual: Option<TargetValue> = exercise_choose_weighted(
            TargetValue::Direct(3),
            TargetValue::ProgramStop
        );
        assert_eq!(actual, Some(TargetValue::Direct(30)));
    }

    #[test]
    fn test_20004_choose_weighted_start_and_end_of_program() {
        let actual: Option<TargetValue> = exercise_choose_weighted(
            TargetValue::ProgramStart,
            TargetValue::ProgramStop
        );
        assert_eq!(actual, Some(TargetValue::Direct(40)));
    }

    #[test]
    fn test_20005_choose_weighted_surrounded_by_none() {
        let actual: Option<TargetValue> = exercise_choose_weighted(
            TargetValue::None,
            TargetValue::None
        );
        assert_eq!(actual, Some(TargetValue::Direct(6)));
    }

    #[test]
    fn test_20006_choose_weighted_surrounded_by_indirect() {
        let actual: Option<TargetValue> = exercise_choose_weighted(
            TargetValue::Indirect(4),
            TargetValue::Indirect(3)
        );
        assert_eq!(actual, Some(TargetValue::Direct(9)));
    }

    #[test]
    fn test_20007_choose_weighted_unrecognized_input() {
        let actual: Option<TargetValue> = exercise_choose_weighted(
            TargetValue::Direct(666),
            TargetValue::Direct(666)
        );
        assert_eq!(actual, None);
    }
}
