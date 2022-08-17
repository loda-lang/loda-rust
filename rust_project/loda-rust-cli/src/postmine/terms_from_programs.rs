use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Context;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Extract the terms row from a loda .asm program
    // The `(?m)` enables multiline matching.
    static ref EXTRACT_TERMS_FROM_LODA_PROGRAM: Regex = Regex::new(
        "(?m)^; (-?\\d+(?:,-?\\d+)+)$"
    ).unwrap();
}

/// Extract the first `terms coment` from a program
/// 
/// Returns None if there is no `terms comment`.
pub fn terms_from_program(program_path: &Path) -> anyhow::Result<Option<String>> {
    let program_contents: String = fs::read_to_string(program_path)
        .with_context(|| format!("Read program from {:?}", program_path))?;
    let re = &EXTRACT_TERMS_FROM_LODA_PROGRAM;
    let captures = match re.captures(&program_contents) {
        Some(value) => value,
        None => {
            return Ok(None);
        }
    };
    let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
    let terms_string: String = capture1.to_string();
    Ok(Some(terms_string))
}

pub fn terms_from_programs(paths: &Vec<PathBuf>) -> anyhow::Result<()> {
    for path in paths {
        terms_from_program(&path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_10000_terms_from_program_ok() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_terms_from_program_ok");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("input.asm");
        let input_content = 
r#"; A000321: H_n(-1/2), where H_n(x) is Hermite polynomial of degree n.
; Submitted by GolfSierra
; 1,-1,-1,5,1,-41,31,461,-895,-6481,22591,107029,-604031,-1964665,17669471

mov $2,1
lpb $0
  sub $0,1
  add $1,$2
  mul $1,2
  sub $2,$1
  add $1,$2
  mul $1,$0
lpe
mov $0,$2
"#;

        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;

        // Act
        let optional_terms: Option<String> = terms_from_program(&input_path)?;
        
        // Assert
        let actual: String = optional_terms.expect("Expected a string with the terms");
        assert_eq!(actual, "1,-1,-1,5,1,-41,31,461,-895,-6481,22591,107029,-604031,-1964665,17669471");
        Ok(())
    }

    #[test]
    fn test_10001_terms_from_program_ok() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10001_terms_from_program_ok");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("input.asm");
        let input_content = 
r#"; I'm an tiny program
; without a "terms comment"
mul $0,2
; placeholder footer
"#;

        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;

        // Act
        let optional_terms: Option<String> = terms_from_program(&input_path)?;
        
        // Assert
        assert!(optional_terms.is_none());
        Ok(())
    }

    #[test]
    fn test_10002_terms_from_program_ok() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10002_terms_from_program_ok");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("input.asm");
        let input_content = 
r#"; Program with multiple terms comments
; 1,1,1,1,1,1,1
; 2,2,2,2,2,2,2
; 3,3,3,3,3,3,3
; Always picks the first terms comment
"#;

        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;

        // Act
        let optional_terms: Option<String> = terms_from_program(&input_path)?;
        
        // Assert
        let actual: String = optional_terms.expect("Expected a string with the terms");
        assert_eq!(actual, "1,1,1,1,1,1,1");
        Ok(())
    }

    #[test]
    fn test_20000_terms_from_program_error_no_such_file() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_20000_terms_from_program_error_no_such_file");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("non-existing");

        // Act
        let result = terms_from_program(&input_path);
        
        // Assert
        assert!(result.is_err());
        Ok(())
    }
}
