use super::LodaCppError;
use std::error::Error;
use regex::Regex;
use lazy_static::lazy_static;
use std::io;
use std::io::BufRead;

lazy_static! {
    // Extract the `term index` from `loda-cpp check` output.
    static ref EXTRACT_TERM_INDEX: Regex = Regex::new(
        "^(\\d+) \\d+$"
    ).unwrap();
}

fn parse_line<S: AsRef<str>>(line: S) -> Option<u32> {
    let line: &str = line.as_ref();
    let re = &EXTRACT_TERM_INDEX;
    let captures = match re.captures(line) {
        Some(value) => value,
        None => {
            return None;
        }
    };
    let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
    let number_of_correct_terms: u32 = match capture1.parse::<u32>() {
        Ok(value) => value,
        Err(_error) => {
            return None;
        }
    };
    Some(number_of_correct_terms)
}

fn extract_number_of_correct_terms(input: &str) -> u32 {
    let mut input_u8: &[u8] = input.as_bytes();
    let reader: &mut dyn io::BufRead = &mut input_u8;
    let mut last_term_index: u32 = 0;
    let mut current_line_number: u32 = 0;
    for line in reader.lines() {
        current_line_number += 1;
        let line: String = match line {
            Ok(value) => value,
            Err(error) => {
                error!("Problem reading line #{:?}. {:?}", current_line_number, error);
                continue;
            }
        };
        if let Some(term_index) = parse_line(&line) {
            last_term_index = term_index;
        }
    }
    last_term_index + 1
}

#[derive(Debug, PartialEq)]
pub enum LodaCppCheckStatus {
    FullMatch,
    PartialMatch,
}

#[derive(Debug)]
pub struct LodaCppCheckResult {
    pub status: LodaCppCheckStatus,
    pub number_of_correct_terms: u32,
}

impl LodaCppCheckResult {
    pub fn parse<S: AsRef<str>>(input_raw: S) -> Result<LodaCppCheckResult, Box<dyn Error>> {
        let input_raw: &str = input_raw.as_ref();
        let input_trimmed: &str = input_raw.trim();
        if input_trimmed.ends_with("ok") {
            let number_of_correct_terms: u32 = extract_number_of_correct_terms(&input_trimmed);
            return Ok(Self {
                status: LodaCppCheckStatus::FullMatch,
                number_of_correct_terms: number_of_correct_terms,
            });
        }
        if input_trimmed.ends_with("error") {
            let number_of_correct_terms: u32 = extract_number_of_correct_terms(&input_trimmed);
            return Ok(Self {
                status: LodaCppCheckStatus::PartialMatch,
                number_of_correct_terms: number_of_correct_terms,
            });
        }
        Err(Box::new(LodaCppError::ParseCheck))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_parse_line_some() {
        assert_eq!(parse_line("0 0"), Some(0));
        assert_eq!(parse_line("42 100"), Some(42));
        assert_eq!(parse_line("10000 1"), Some(10000));
    }

    #[test]
    fn test_10001_parse_line_none() {
        assert_eq!(parse_line("-1 100"), None);
        assert_eq!(parse_line("ok"), None);
        assert_eq!(parse_line("error"), None);
        assert_eq!(parse_line("123 456 -> expected 500"), None);
    }

    #[test]
    fn test_20000_parse_ok() {
        // Arrange
        let content = 
r#"
0 1
1 30
2 21
3 77
4 93
5 87
ok
"#;

        // Act
        let result = LodaCppCheckResult::parse(content).expect("Should be able to parse ok");

        // Assert
        assert_eq!(result.status, LodaCppCheckStatus::FullMatch);
        assert_eq!(result.number_of_correct_terms, 6);
    }    

    #[test]
    fn test_30000_parse_error() {
        // Arrange
        let content = 
r#"
0 2
1 1
2 0
3 1
4 0
5 0
6 1
7 0
8 1
9 0
10 0
11 9 -> expected 5
error
"#;

        // Act
        let result = LodaCppCheckResult::parse(content).expect("Should be able to parse ok");

        // Assert
        assert_eq!(result.status, LodaCppCheckStatus::PartialMatch);
        assert_eq!(result.number_of_correct_terms, 11);
    }    
}
