use regex::Regex;
use lazy_static::lazy_static;
use std::path::Path;

lazy_static! {
    // Extract the sequence number "123456" from a string like this "dir1/dir2/A123456.asm".
    static ref EXTRACT_SEQUENCE_NUMBER: Regex = Regex::new(
        "\\bA(\\d+)[.]asm$"
    ).unwrap();
}

pub fn program_id_from_path(path: &Path) -> Option<u32> {
    let re = &EXTRACT_SEQUENCE_NUMBER;
    let s = path.to_string_lossy();
    let captures = match re.captures(&s) {
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
        let path = Path::new(input);
        match program_id_from_path(&path) {
            Some(program_id) => return format!("{:?}", program_id),
            None => return "NONE".to_string()
        }
    }

    #[test]
    fn test_10000_parse() {
        assert_eq!(parse("dir/dir/A0.asm"), "0");
        assert_eq!(parse("dir/dir/A1.asm"), "1");
        assert_eq!(parse("dir/dir/A000040.asm"), "40");
        assert_eq!(parse("dir/dir/A123456.asm"), "123456");
        assert_eq!(parse("# comment"), "NONE");
        assert_eq!(parse("Ajunk"), "NONE");
        assert_eq!(parse("A junk"), "NONE");
        assert_eq!(parse("junk/_A000040.asm"), "NONE");
        assert_eq!(parse("junk/B000040.asm"), "NONE");
        assert_eq!(parse("junk/X000040.asm"), "NONE");
    }
}
