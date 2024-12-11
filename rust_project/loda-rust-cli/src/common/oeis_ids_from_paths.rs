use loda_rust_core::oeis::{OeisId, OeisIdHashSet};
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

pub fn oeis_ids_from_paths(paths: &Vec<PathBuf>) -> OeisIdHashSet {
    let mut oeis_ids = OeisIdHashSet::new();
    for path in paths {
        let oeis_id: OeisId = match oeis_id_from_path(path) {
            Some(oeis_id) => oeis_id,
            None => {
                warn!("Unable to extract oeis_id from {:?}", path);
                continue;
            }
        };
        oeis_ids.insert(oeis_id);
    }
    oeis_ids
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

    #[test]
    fn test_20000_oeis_ids_from_paths() {
        // Arrange
        let input: Vec<PathBuf> = vec![
            PathBuf::from("dir/A123456.asm"),
            PathBuf::from("dir/A000045.csv"),
            PathBuf::from("dir/A112088_test.asm"),
        ];

        // Act
        let actual: OeisIdHashSet = oeis_ids_from_paths(&input);

        // Assert
        let mut expected = OeisIdHashSet::new();
        expected.insert(OeisId::from(45));
        expected.insert(OeisId::from(123456));
        expected.insert(OeisId::from(112088));
        assert_eq!(actual, expected);
    }
}
