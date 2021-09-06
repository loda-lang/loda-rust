use num_bigint::BigInt;
use std::fmt;
use regex::Regex;
use lazy_static::lazy_static;
use loda_rust_core::util::BigIntVec;

pub struct StrippedSequence {
    pub sequence_number: u32,
    bigint_vec: BigIntVec,
}

impl StrippedSequence {
    pub fn new(sequence_number: u32, bigint_vec: BigIntVec) -> Self {
        Self {
            sequence_number: sequence_number,
            bigint_vec: bigint_vec,
        }
    }

    pub fn bigint_vec_ref(&self) -> &BigIntVec {
        &self.bigint_vec
    }

    pub fn len(&self) -> usize {
        self.bigint_vec.len()
    }
}

impl fmt::Display for StrippedSequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let strings: Vec<String> = self.bigint_vec.iter().map(|bigint| {
            bigint.to_string()
        }).collect();
        let strings_joined: String = strings.join(",");
        let spacer: &str = match strings_joined.is_empty() {
            true => "",
            false => " "
        };
        write!(f, "{}{}{}", self.sequence_number, spacer, strings_joined)
    }
}

lazy_static! {
    // Extract the sequence number "123456" from a string like this "A123456 ".
    static ref EXTRACT_SEQUENCE_NUMBER: Regex = Regex::new(
        "^A(\\d+)"
    ).unwrap();
}

pub fn parse_stripped_sequence_line(line: &String, max_term_count: Option<usize>) -> Option<StrippedSequence> {
    if !line.starts_with("A") {
        return None;            
    }
    let mut iter = line.split(",");

    // Process the first column
    // The first column is like this "A123456 "
    // This code extracts the sequence number, 123456
    let sequence_number_raw: &str = match iter.next() {
        Some(value) => value,
        None => {
            debug!("Unable to pattern match row");
            return None;
        }
    };
    let re = &EXTRACT_SEQUENCE_NUMBER;
    let captures = match re.captures(&sequence_number_raw) {
        Some(value) => value,
        None => {
            debug!("Unable to extract sequence number");
            return None;
        }
    };
    let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
    let sequence_number_string: String = capture1.to_string();
    let sequence_number: u32 = match sequence_number_string.parse() {
        Ok(value) => value,
        _ => {
            debug!("Unable to parse sequence number as u32");
            return None;
        }
    };

    // Process the following columns
    let max_term_count_inner: usize = match max_term_count {
        Some(value) => value,
        None => usize::MAX
    };
    let mut bigint_vec: BigIntVec = vec!();
    for _ in 0..max_term_count_inner {
        let string = match iter.next() {
            Some(value) => value,
            None => {
                // end of sequence
                break;
            }
        };
        if string.is_empty() {
            // end of sequence
            break;
        }
        let bytes: &[u8] = string.as_bytes();
        let bigint: BigInt = match BigInt::parse_bytes(bytes, 10) {
            Some(value) => value,
            None => {
                error!("Unable to parse a number as BigInt. '{}'", string);
                return None;
            }
        };
        bigint_vec.push(bigint);
    }

    let seq = StrippedSequence::new(sequence_number, bigint_vec);
    return Some(seq);
}


#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> String {
        match parse_stripped_sequence_line(&input.to_string(), None) {
            Some(stripped_sequence) => return stripped_sequence.to_string(),
            None => return "NONE".to_string()
        }
    }

    fn parse_with_limit(input: &str, max_term_count: usize) -> String {
        match parse_stripped_sequence_line(&input.to_string(), Some(max_term_count)) {
            Some(stripped_sequence) => return stripped_sequence.to_string(),
            None => return "NONE".to_string()
        }
    }

    #[test]
    fn test_10000_parse() {
        assert_eq!(parse(""), "NONE");
        assert_eq!(parse("# comment"), "NONE");
        assert_eq!(parse("Ajunk"), "NONE");
        assert_eq!(parse("A junk"), "NONE");
        assert_eq!(parse("A000040 ,2,3,5,7,11,13,17,19,23,"), "40 2,3,5,7,11,13,17,19,23");
        assert_eq!(parse_with_limit("A000040 ,2,3,5,7,11,13,17,19,23,", 0), "40");
        assert_eq!(parse_with_limit("A000040 ,2,3,5,7,11,13,17,19,23,", 2), "40 2,3");
        assert_eq!(parse_with_limit("A000040 ,2,3,5,", 8), "40 2,3,5");
    }

    const INPUT_STRIPPED_SEQUENCE_DATA: &str = r#"
# OEIS Sequence Data (http://oeis.org/stripped.gz)
# Last Modified: January 32 01:01 UTC 1984
# Use of this content is governed by the
# OEIS End-User License: http://oeis.org/LICENSE
A000010 ,1,1,2,2,4,2,6,4,6,4,10,4,12,6,8,8,16,6,18,8,12,10,22,
A000040 ,2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,
"#;

    #[test]
    fn test_10001_parse_file() {
        let s = INPUT_STRIPPED_SEQUENCE_DATA.to_string();
        let mut line_count_sequences: usize = 0;
        let mut line_count_junk: usize = 0;
        for line in s.lines() {
            match parse_stripped_sequence_line(&line.to_string(), Some(5)) {
                Some(_) => { 
                    line_count_sequences += 1;
                },
                None => {
                    line_count_junk += 1;
                }
            }
        }
        assert_eq!(line_count_sequences, 2);
        assert_eq!(line_count_junk, 5);
    }
}
