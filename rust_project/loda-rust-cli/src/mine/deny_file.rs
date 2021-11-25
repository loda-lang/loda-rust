use regex::Regex;
use lazy_static::lazy_static;
use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

lazy_static! {
    // Extract the sequence number "000017" from a string like this "A000017: Erroneous version of A032522.".
    static ref EXTRACT_SEQUENCE_NUMBER: Regex = Regex::new(
        "^A(\\d+):"
    ).unwrap();
}

fn parse_line(line: &String) -> Option<u32> {
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

pub fn load_program_ids_from_deny_file(path: &Path) -> Result<Vec<u32>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    load_program_ids_from_deny_file_inner(
        &mut reader, 
    )
}

fn load_program_ids_from_deny_file_inner(
    reader: &mut dyn io::BufRead
) -> Result<Vec<u32>, Box<dyn Error>> {
    let mut program_ids: Vec<u32> = vec!();
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
        let program_id: u32 = match parse_line(&line) {
            Some(value) => value,
            None => {
                error!("Unable to parse line #{:?}. Skipping.", current_line_number);
                continue;
            }
        };
        program_ids.push(program_id);
    }
    Ok(program_ids)
}


#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> String {
        match parse_line(&input.to_string()) {
            Some(program_id) => return program_id.to_string(),
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
    fn test_10001_parse_multiple_lines() {
        let mut input: &[u8] = INPUT_DATA.as_bytes();
        let result: Vec<u32> = load_program_ids_from_deny_file_inner(&mut input).unwrap();
        assert_eq!(vec![17, 154, 381, 480, 572], result);
    }
}
