use std::fmt;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Extract the sequence number "000017" from a string like this "A000017: Erroneous version of A032522.".
    static ref EXTRACT_SEQUENCE_NUMBER: Regex = Regex::new(
        "^A(\\d+):"
    ).unwrap();
}

pub fn parse_deny_line(line: &String) -> Option<u32> {
    if !line.starts_with("A") {
        return None;            
    }
    let re = &EXTRACT_SEQUENCE_NUMBER;
    let captures = match re.captures(&line) {
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
    return Some(sequence_number);
}


#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> String {
        match parse_deny_line(&input.to_string()) {
            Some(stripped_sequence) => return stripped_sequence.to_string(),
            None => return "NONE".to_string()
        }
    }

    #[test]
    fn test_10000_parse() {
        assert_eq!(parse(""), "NONE");
        assert_eq!(parse("\n"), "NONE");
        assert_eq!(parse("# comment"), "NONE");
        assert_eq!(parse("Ajunk"), "NONE");
        assert_eq!(parse("A junk"), "NONE");
        assert_eq!(parse("A000017: Erroneous version of A032522."), "17");
        assert_eq!(parse("A000154: Erroneous version of A003713."), "154");
    }

    const INPUT_DATA: &str = r#"
A000017: Erroneous version of A032522.
A000154: Erroneous version of A003713.
Ajunk 123 ignore this line
Ignore this line as well
A000381: Essentially the same as A001611.
A000480: a(n) = floor(cos(n)).
A000572: A Beatty sequence: [ n(e+1) ].
"#;

    #[test]
    fn test_10001_parse_file() {
        let s = INPUT_DATA.to_string();
        let mut line_count_sequences: usize = 0;
        let mut line_count_junk: usize = 0;
        for line in s.lines() {
            match parse_deny_line(&line.to_string()) {
                Some(_) => { 
                    line_count_sequences += 1;
                },
                None => {
                    line_count_junk += 1;
                }
            }
        }
        assert_eq!(line_count_sequences, 5);
        assert_eq!(line_count_junk, 3);
    }
}
