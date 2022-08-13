use super::OeisId;
use num_bigint::BigInt;
use std::fmt;
use regex::Regex;
use lazy_static::lazy_static;
use loda_rust_core::util::BigIntVec;

/// As of July 2022, the OEIS 'stripped' file contains ~360k sequences.
///
/// Histogram of sequence lengths.
///
/// | Lengths  |        |      |      |      |      |      |      |      |      |      |
/// |----------|--------|------|------|------|------|------|------|------|------|------|
/// |  0 ..  9 |      0 |    9 |  115 |  632 | 1760 | 2750 | 3381 | 3504 | 4324 | 4560 |
/// | 10 .. 19 |   5401 | 4819 | 5086 | 5198 | 5861 | 7186 | 8170 | 6833 | 7117 | 7082 |
/// | 20 .. 29 |   8781 | 7666 | 6642 | 6126 | 5944 | 5492 | 5388 | 4945 | 5754 | 4826 |
/// | 30 .. 39 |   4966 | 3978 | 3765 | 3751 | 3841 | 3497 | 3962 | 3316 | 2985 | 3328 |
/// | 40+      | 172129 |      |      |      |      |      |      |      |      |      |
///
/// The 50/50 split is around 38 terms.
/// 
/// Half of the sequences are shorter than or equal to 38 terms.
/// 
/// The other half of the sequences are longer than 38 terms.
pub struct StrippedRow {
    oeis_id: OeisId,
    terms: BigIntVec,
}

impl StrippedRow {
    pub fn parse(line: &String, max_term_count: Option<usize>) -> Option<Self> {
        parse_stripped_row(line, max_term_count)
    }

    pub fn new(oeis_id: OeisId, terms: BigIntVec) -> Self {
        Self {
            oeis_id: oeis_id,
            terms: terms,
        }
    }

    pub fn oeis_id(&self) -> OeisId {
        self.oeis_id
    }

    pub fn terms(&self) -> &BigIntVec {
        &self.terms
    }

    pub fn len(&self) -> usize {
        self.terms.len()
    }

    pub fn grow_to_length(&mut self, length: usize, padding_value: &BigInt) {
        let original_length: usize = self.terms.len();
        if original_length >= length {
            return;
        }
        let count: usize = length - original_length;
        for _ in 0..count {
            self.terms.push(padding_value.clone());
        }
    }
}

impl fmt::Display for StrippedRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let strings: Vec<String> = self.terms.iter().map(|bigint| {
            bigint.to_string()
        }).collect();
        let strings_joined: String = strings.join(",");
        let spacer: &str = match strings_joined.is_empty() {
            true => "",
            false => " "
        };
        write!(f, "{}{}{}", self.oeis_id().a_number(), spacer, strings_joined)
    }
}

lazy_static! {
    /// Extract the sequence number "123456" from a string like this "A123456 ".
    static ref EXTRACT_SEQUENCE_NUMBER: Regex = Regex::new(
        "^A(\\d+)"
    ).unwrap();
}

fn parse_stripped_row(line: &String, max_term_count: Option<usize>) -> Option<StrippedRow> {
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
    let oeis_id: OeisId = OeisId::from(sequence_number);

    // Process the following columns
    let max_term_count_inner: usize = match max_term_count {
        Some(value) => value,
        None => usize::MAX
    };
    let mut terms: BigIntVec = vec!();
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
        let term: BigInt = match BigInt::parse_bytes(bytes, 10) {
            Some(value) => value,
            None => {
                error!("Unable to parse a number as BigInt. '{}'", string);
                return None;
            }
        };
        terms.push(term);
    }

    Some(StrippedRow::new(oeis_id, terms))
}


#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Zero;

    fn parse(input: &str) -> String {
        match StrippedRow::parse(&input.to_string(), None) {
            Some(value) => return value.to_string(),
            None => return "NONE".to_string()
        }
    }

    fn parse_with_limit(input: &str, max_term_count: usize) -> String {
        match StrippedRow::parse(&input.to_string(), Some(max_term_count)) {
            Some(value) => return value.to_string(),
            None => return "NONE".to_string()
        }
    }

    #[test]
    fn test_10000_parse() {
        assert_eq!(parse(""), "NONE");
        assert_eq!(parse("# comment"), "NONE");
        assert_eq!(parse("Ajunk"), "NONE");
        assert_eq!(parse("A junk"), "NONE");
        assert_eq!(parse("A000040 ,2,3,5,7,11,13,17,19,23,"), "A000040 2,3,5,7,11,13,17,19,23");
        assert_eq!(parse_with_limit("A000040 ,2,3,5,7,11,13,17,19,23,", 0), "A000040");
        assert_eq!(parse_with_limit("A000040 ,2,3,5,7,11,13,17,19,23,", 2), "A000040 2,3");
        assert_eq!(parse_with_limit("A000040 ,2,3,5,", 8), "A000040 2,3,5");
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
            match StrippedRow::parse(&line.to_string(), Some(5)) {
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

    #[test]
    fn test_10002_grow_to_length() {
        // Arrange
        let input = "A000040 ,2,3,5,7,11,13,17,19,23,";
        let mut row: StrippedRow = StrippedRow::parse(&input.to_string(), None).unwrap();
        assert_eq!(row.len(), 9);
        let padding_value = BigInt::zero();

        // Act
        row.grow_to_length(20, &padding_value);

        // Assert
        assert_eq!(row.to_string(), "A000040 2,3,5,7,11,13,17,19,23,0,0,0,0,0,0,0,0,0,0,0");
    }
}
