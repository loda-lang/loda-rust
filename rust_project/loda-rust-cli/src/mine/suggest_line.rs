use crate::common::RecordTrigram;
use loda_rust_core::parser::ParsedProgram;
use super::random_indexes_with_distance;
use std::collections::HashMap;
use rand::Rng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LineValue {
    ProgramStart,
    ProgramStop,
    Line(String),
}

impl LineValue {
    /// Convert string to an enum.
    /// 
    /// Convert "mov $0,42" to `LineValue::Line("mov $0,42")`.
    /// 
    /// Convert "START" to `LineValue::ProgramStart`.
    /// 
    /// Convert "STOP" to `LineValue::ProgramStop`.
    /// 
    /// Typechecks that the input is a valid LODA program, 
    /// if it can't be parsed then `Err` is returned.
    fn parse<S>(content: S) -> anyhow::Result<LineValue> where S: Into<String> {
        let s: String = content.into();
        match s.as_str() {
            "START" => {
                return Ok(LineValue::ProgramStart);
            },
            "STOP" => {
                return Ok(LineValue::ProgramStop);
            },
            _ => {}
        }
        let result = ParsedProgram::parse_program(&s);
        let parsed_program: ParsedProgram = match result {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Cannot parse item. error: {:?}", error));
            }
        };
        if parsed_program.instruction_vec.is_empty() {
            return Err(anyhow::anyhow!("Program is empty"));
        }
        if parsed_program.instruction_vec.len() != 1 {
            return Err(anyhow::anyhow!("Expected 1 instruction but got something else"));
        }
        let formatted_program = format!("{}", parsed_program);
        if formatted_program != s {
            return Err(anyhow::anyhow!("The input seems to contain unwanted blank characters"));
        }
        return Ok(LineValue::Line(s));
    }

    #[allow(dead_code)]
    fn to_string(&self) -> String {
        match self {
            Self::ProgramStart => return "START".to_string(),
            Self::ProgramStop => return "STOP".to_string(),
            Self::Line(value) => return value.clone(),
        }
    }
}

type HistogramKey = (LineValue,LineValue);
type ValueAndWeight = (LineValue,u32);
type HistogramValue = Vec<ValueAndWeight>;

#[derive(Clone)]
pub struct SuggestLine {
    histogram: HashMap<HistogramKey, HistogramValue>
}

impl SuggestLine {
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
            let value0: LineValue = match LineValue::parse(&record.word0) {
                Ok(value) => value,
                Err(error) => {
                    debug!("SuggestLine.populate(). ignoring row {}. column 0. error: {:?}", index, error);
                    continue;
                }
            };
            let value1: LineValue = match LineValue::parse(&record.word1) {
                Ok(value) => value,
                Err(error) => {
                    debug!("SuggestLine.populate(). ignoring row {}. column 1. error: {:?}", index, error);
                    continue;
                }
            };
            let value2: LineValue = match LineValue::parse(&record.word2) {
                Ok(value) => value,
                Err(error) => {
                    debug!("SuggestLine.populate(). ignoring row {}. column 2. error: {:?}", index, error);
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
    fn candidates(&self, prev_word: LineValue, next_word: LineValue) -> Option<&HistogramValue> {
        let key: HistogramKey = (prev_word, next_word);
        self.histogram.get(&key)
    }

    #[allow(dead_code)]
    pub fn best_candidate(&self, prev_word: LineValue, next_word: LineValue) -> Option<LineValue> {
        let histogram_value: &HistogramValue = match self.candidates(prev_word, next_word) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let candidate_value: LineValue = match histogram_value.first() {
            Some(value) => value.0.clone(),
            None => {
                return None;
            }
        };
        Some(candidate_value)
    }

    pub fn choose_weighted<R: Rng + ?Sized>(&self, rng: &mut R, prev_word: LineValue, next_word: LineValue) -> Option<LineValue> {
        let histogram_value: &HistogramValue = match self.candidates(prev_word, next_word) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let value: LineValue = histogram_value.choose_weighted(rng, |item| item.1).unwrap().0.clone();
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
        "gcd $0,$1",
        "lpb $0",
        "lpe",
        "add $$0,1",
        "mul $0,-1",
        "junk",
        "; comment",
        " ",
        " add $0,1",
        "add $0,1 ; comment",
    ];

    static OUTPUT: &'static [&'static str] = &[
        "START",
        "STOP",
        "gcd $0,$1",
        "lpb $0",
        "lpe",
        "add $$0,1",
        "mul $0,-1",
        "IGNORE",
        "IGNORE",
        "IGNORE",
        "IGNORE",
        "IGNORE",
    ];

    fn process<S: AsRef<str>>(input: S) -> String {
        let input = input.as_ref();
        let target_value: LineValue = match LineValue::parse(input) {
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
                count: 12581,
                word0: "lpe".to_string(),
                word1: "mov $0,$1".to_string(),
                word2: "STOP".to_string()
            },
            RecordTrigram {
                count: 4258,
                word0: "mov $4,$0".to_string(),
                word1: "max $4,0".to_string(),
                word2: "cmp $4,$0".to_string()
            },
            RecordTrigram {
                count: 1,
                word0: "lpb $0".to_string(),
                word1: "lpb $3,5".to_string(),
                word2: "lpb $3,2".to_string()
            },
            RecordTrigram {
                count: 1471,
                word0: "START".to_string(),
                word1: "add $0,1".to_string(),
                word2: "mov $1,$0".to_string()
            },
        ];
        v
    }

    fn exercise_choose_weighted(prev_word: LineValue, next_word: LineValue) -> Option<LineValue> {
        let mock = mockdata();
        let mut si = SuggestLine::new();
        si.populate(&mock);
        let mut rng = StdRng::seed_from_u64(0);
        let actual: Option<LineValue> = si.choose_weighted(
            &mut rng, 
            prev_word, 
            next_word
        );
        actual
    }

    #[test]
    fn test_20000_choose_weighted_surrounded_by_other_words0() {
        let actual: Option<LineValue> = exercise_choose_weighted(
            LineValue::Line("lpe".to_string()), 
            LineValue::ProgramStop
        );
        assert_eq!(actual, Some(LineValue::Line("mov $0,$1".to_string())));
    }
    
    #[test]
    fn test_20001_choose_weighted_surrounded_by_other_words1() {
        let actual: Option<LineValue> = exercise_choose_weighted(
            LineValue::Line("mov $4,$0".to_string()), 
            LineValue::Line("cmp $4,$0".to_string())
        );
        assert_eq!(actual, Some(LineValue::Line("max $4,0".to_string())));
    }
    
    #[test]
    fn test_20002_choose_weighted_surrounded_by_other_words2() {
        let actual: Option<LineValue> = exercise_choose_weighted(
            LineValue::Line("lpb $0".to_string()), 
            LineValue::Line("lpb $3,2".to_string())
        );
        assert_eq!(actual, Some(LineValue::Line("lpb $3,5".to_string())));
    }
    
    #[test]
    fn test_20003_choose_weighted_surrounded_by_other_words3() {
        let actual: Option<LineValue> = exercise_choose_weighted(
            LineValue::ProgramStart, 
            LineValue::Line("mov $1,$0".to_string())
        );
        assert_eq!(actual, Some(LineValue::Line("add $0,1".to_string())));
    }
    
    #[test]
    fn test_20004_choose_weighted_unrecognized_input() {
        let actual: Option<LineValue> = exercise_choose_weighted(
            LineValue::Line("add $0,0".to_string()), 
            LineValue::Line("add $1,0".to_string())
        );
        assert_eq!(actual, None);
    }
}
