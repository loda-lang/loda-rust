use crate::oeis::OeisId;
use regex::Regex;
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

lazy_static! {
    /// Extract the sequence number "123456" from a path like this "dir1/dir2/A123456.asm".
    static ref EXTRACT_SEQUENCE_NUMBER: Regex = Regex::new(
        "\\bA(\\d+)(?:\\D.*)?$"
    ).unwrap();
}

pub fn oeis_id_from_path(path: &Path) -> Option<OeisId> {
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
    return Some(OeisId::from(sequence_number));
}

pub fn program_id_from_path(path: &Path) -> Option<u32> {
    match oeis_id_from_path(path) {
        Some(oeis_id) => {
            return Some(oeis_id.raw());
        }
        None => {
            return None;
        }
    }
}

pub fn oeis_ids_from_paths(paths: Vec<PathBuf>) -> Vec<OeisId> {
    let mut oeis_ids: Vec<OeisId> = vec!();
    for path in paths {
        let oeis_id: OeisId = match oeis_id_from_path(&path) {
            Some(oeis_id) => oeis_id,
            None => {
                warn!("Unable to extract oeis_id from {:?}", path);
                continue;
            }
        };
        oeis_ids.push(oeis_id);
    }
    oeis_ids.sort();
    oeis_ids
}

#[allow(dead_code)]
pub fn program_ids_from_paths(paths: Vec<PathBuf>) -> Vec<u32> {
    let oeis_ids: Vec<OeisId> = oeis_ids_from_paths(paths);
    oeis_ids.iter().map(|oeis_id| oeis_id.raw()).collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    fn parse_oeis_id(input: &str) -> String {
        let path = Path::new(input);
        match oeis_id_from_path(&path) {
            Some(oeis_id) => return oeis_id.a_number(),
            None => return "NONE".to_string()
        }
    }

    #[test]
    fn test_10000_oeis_id_from_path() {
        assert_eq!(parse_oeis_id("dir/dir/A0.csv"), "A000000");
        assert_eq!(parse_oeis_id("dir/dir/A1.asm"), "A000001");
        assert_eq!(parse_oeis_id("dir/dir/A000040.csv"), "A000040");
        assert_eq!(parse_oeis_id("dir/dir/A123456.asm"), "A123456");
        assert_eq!(parse_oeis_id("A172330_90_0.csv"), "A172330");
        assert_eq!(parse_oeis_id("dir/dir/A323130_63_4.asm"), "A323130");
        assert_eq!(parse_oeis_id("A172330ignore.csv"), "A172330");
        assert_eq!(parse_oeis_id("A100 A200 A300.asm"), "A000100");
        assert_eq!(parse_oeis_id("# comment"), "NONE");
        assert_eq!(parse_oeis_id("Ajunk"), "NONE");
        assert_eq!(parse_oeis_id("A junk"), "NONE");
        assert_eq!(parse_oeis_id("junk/_A000040.asm"), "NONE");
        assert_eq!(parse_oeis_id("junk/B000040.csv"), "NONE");
        assert_eq!(parse_oeis_id("junk/X000040.asm"), "NONE");
    }

    fn parse_program_id(input: &str) -> String {
        let path = Path::new(input);
        match program_id_from_path(&path) {
            Some(program_id) => return format!("{:?}", program_id),
            None => return "NONE".to_string()
        }
    }

    #[test]
    fn test_10001_program_id_from_path() {
        assert_eq!(parse_program_id("dir/dir/A0.csv"), "0");
        assert_eq!(parse_program_id("dir/dir/A1.asm"), "1");
        assert_eq!(parse_program_id("dir/dir/A000040.csv"), "40");
        assert_eq!(parse_program_id("dir/dir/A123456.asm"), "123456");
        assert_eq!(parse_program_id("A172330_90_0.csv"), "172330");
        assert_eq!(parse_program_id("dir/dir/A323130_63_4.asm"), "323130");
        assert_eq!(parse_program_id("A172330ignore.csv"), "172330");
        assert_eq!(parse_program_id("A100 A200 A300.asm"), "100");
        assert_eq!(parse_program_id("# comment"), "NONE");
        assert_eq!(parse_program_id("Ajunk"), "NONE");
        assert_eq!(parse_program_id("A junk"), "NONE");
        assert_eq!(parse_program_id("junk/_A000040.asm"), "NONE");
        assert_eq!(parse_program_id("junk/B000040.csv"), "NONE");
        assert_eq!(parse_program_id("junk/X000040.asm"), "NONE");
    }

    #[test]
    fn test_20000_oeis_ids_from_paths() {
        let input: Vec<PathBuf> = vec![
            PathBuf::from("dir/A123456.asm"),
            PathBuf::from("dir/A000045.csv"),
            PathBuf::from("dir/A112088_test.asm"),
        ];
        let oeis_ids: Vec<OeisId> = oeis_ids_from_paths(input);
        assert_eq!(oeis_ids, vec![OeisId::from(45), OeisId::from(112088), OeisId::from(123456)]);
    }

    #[test]
    fn test_20001_program_ids_from_paths() {
        let input: Vec<PathBuf> = vec![
            PathBuf::from("dir/A123456.asm"),
            PathBuf::from("dir/A000045.csv"),
            PathBuf::from("dir/A112088_test.asm"),
        ];
        let program_ids: Vec<u32> = program_ids_from_paths(input);
        assert_eq!(program_ids, vec![45, 112088, 123456]);
    }
}
