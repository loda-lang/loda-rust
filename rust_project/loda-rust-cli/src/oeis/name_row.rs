use super::OeisId;
use std::fmt;
use regex::Regex;
use lazy_static::lazy_static;

pub struct NameRow {
    oeis_id: OeisId,
    name: String,
}

impl NameRow {
    pub fn parse(line: &String) -> Option<Self> {
        parse_name_row(line)
    }

    pub fn new(oeis_id: OeisId, name: String) -> Self {
        Self {
            oeis_id: oeis_id,
            name: name,
        }
    }

    pub fn oeis_id(&self) -> OeisId {
        self.oeis_id
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

impl fmt::Display for NameRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.oeis_id.a_number(), self.name)
    }
}

lazy_static! {
    /// Extract sequence number and sequence name.
    /// 
    /// With an input like this `A123456 sequence name`.
    /// then sequence number is `123456`.
    /// and sequence name is `sequence name`.
    static ref EXTRACT_SEQUENCE_NUMBER_AND_NAME: Regex = Regex::new(
        "^A(\\d+) (.+)$"
    ).unwrap();
}

fn parse_name_row(line: &String) -> Option<NameRow> {
    if !line.starts_with("A") {
        return None;            
    }

    let re = &EXTRACT_SEQUENCE_NUMBER_AND_NAME;
    let captures = match re.captures(&line) {
        Some(value) => value,
        None => {
            debug!("Unable to extract sequence number and name");
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

    let capture2: &str = captures.get(2).map_or("", |m| m.as_str());
    let name: String = capture2.to_string();
    Some(NameRow::new(oeis_id, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> String {
        match NameRow::parse(&input.to_string()) {
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
        assert_eq!(parse("A000001 Number of groups of order n."), "A000001 Number of groups of order n.");
        assert_eq!(parse("A000007 The characteristic function of {0}: a(n) = 0^n."), "A000007 The characteristic function of {0}: a(n) = 0^n.");
        assert_eq!(parse("A999999 x"), "A999999 x");
        assert_eq!(parse("A999999 "), "NONE");
        assert_eq!(parse("A999999"), "NONE");
    }
}
