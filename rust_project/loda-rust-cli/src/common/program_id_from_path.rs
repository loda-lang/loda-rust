use regex::Regex;
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

lazy_static! {
    /// Extract the sequence number "123456" from a string like this "dir1/dir2/A123456.asm".
    static ref EXTRACT_SEQUENCE_NUMBER: Regex = Regex::new(
        "\\bA(\\d+)(?:\\D.*)?$"
    ).unwrap();
}

pub fn program_id_from_path(path: &Path) -> Option<u32> {
    let re = &EXTRACT_SEQUENCE_NUMBER;
    let file_stem: &OsStr = match path.file_stem() {
        Some(value) => value,
        None => {
            debug!("Unable to extract file_stem");
            return None;
        }
    };
    let s = file_stem.to_string_lossy();
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

pub fn program_ids_from_paths(paths: Vec<PathBuf>) -> Vec<u32> {
    let mut program_ids: Vec<u32> = vec!();
    for path in paths {
        let program_id: u32 = match program_id_from_path(&path) {
            Some(program_id) => program_id,
            None => {
                warn!("Unable to extract program_id from {:?}", path);
                continue;
            }
        };
        program_ids.push(program_id);
    }
    program_ids.sort();
    program_ids
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
    fn test_10000_program_id_from_path() {
        assert_eq!(parse("dir/dir/A0.csv"), "0");
        assert_eq!(parse("dir/dir/A1.asm"), "1");
        assert_eq!(parse("dir/dir/A000040.csv"), "40");
        assert_eq!(parse("dir/dir/A123456.asm"), "123456");
        assert_eq!(parse("A172330_90_0.csv"), "172330");
        assert_eq!(parse("dir/dir/A323130_63_4.asm"), "323130");
        assert_eq!(parse("A172330ignore.csv"), "172330");
        assert_eq!(parse("A100 A200 A300.asm"), "100");
        assert_eq!(parse("# comment"), "NONE");
        assert_eq!(parse("Ajunk"), "NONE");
        assert_eq!(parse("A junk"), "NONE");
        assert_eq!(parse("junk/_A000040.asm"), "NONE");
        assert_eq!(parse("junk/B000040.csv"), "NONE");
        assert_eq!(parse("junk/X000040.asm"), "NONE");
    }

    #[test]
    fn test_10001_program_ids_from_paths() {
        let input: Vec<PathBuf> = vec![
            PathBuf::from("dir/A123456.asm"),
            PathBuf::from("dir/A000045.csv"),
            PathBuf::from("dir/A112088_test.asm"),
        ];
        let program_ids: Vec<u32> = program_ids_from_paths(input);
        assert_eq!(program_ids, vec![45, 112088, 123456]);
    }
}
