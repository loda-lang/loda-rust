use loda_rust_core::parser::ParsedProgram;
use crate::oeis::{OeisId, OeisIdHashSet};
use std::path::{Path, PathBuf};
use std::fs;
use std::convert::TryFrom;
use anyhow::Context;

pub fn oeis_ids_from_program(program_path: &Path) -> anyhow::Result<OeisIdHashSet> {
    // Load asm file
    let program_contents: String = fs::read_to_string(&program_path)
        .with_context(|| format!("Read program from {:?}", &program_path))?;

    // Convert file content to a program
    let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&program_contents) {
        Ok(value) => value,
        Err(error) => {
            return Err(anyhow::anyhow!("Parse program from {:?} error: {:?}", &program_path, error));
        }
    };

    // Loop over the `seq` instructions and gather their oeis ids
    let mut oeis_ids = OeisIdHashSet::new();
    let direct_dependencies: Vec<u64> = parsed_program.direct_dependencies();
    for direct_dependency in direct_dependencies {
        match u32::try_from(direct_dependency) {
            Ok(oeis_id_raw) => {
                oeis_ids.insert(OeisId::from(oeis_id_raw));
            },
            Err(error) => {
                return Err(anyhow::anyhow!("Value is outside than what OeisId can represent. program_path: {:?} error: {:?}", &program_path, error));
            }
        }
    }
    Ok(oeis_ids)
}

pub fn oeis_ids_from_programs(paths: &Vec<PathBuf>) -> anyhow::Result<OeisIdHashSet> {
    let mut oeis_ids = OeisIdHashSet::new();
    for path in paths {
        let new_oeis_ids: OeisIdHashSet = oeis_ids_from_program(&path)
            .with_context(|| format!("Extract oeis ids from program: {:?}", path))?;
        oeis_ids.extend(&new_oeis_ids);
    }
    Ok(oeis_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;
    
    #[test]
    fn test_10000_oeis_ids_from_program_ok() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_oeis_ids_from_program_ok");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("A026233.asm");

        let input_content = 
r#"
; A026233: a(n) = j if n is the j-th prime, else a(n) = k if n is the k-th nonprime.
mov $1,$0
seq $0,66246 ; 0 unless n is a composite number A002808(k) when a(n) = k.
add $0,1
seq $1,159081 ; Let d be the largest element of A008578 which divides n, then a(n) is the position of d in A008578.
sub $1,1
max $1,$0
mov $0,$1
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;

        // Act
        let oeis_ids: OeisIdHashSet = oeis_ids_from_program(&input_path)?;

        // Assert
        let mut expected = OeisIdHashSet::new();
        expected.insert(OeisId::from(66246));
        expected.insert(OeisId::from(159081));
        assert_eq!(oeis_ids, expected);
        Ok(())
    }

    #[test]
    fn test_10001_oeis_ids_from_program_missing_file() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10001_oeis_ids_from_program_missing_file");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("non-existing.asm");

        // Act
        let result = oeis_ids_from_program(&input_path);

        // Assert
        let error = result.expect_err("should fail");
        let error_message: String = error.to_string();
        assert_eq!(error_message.contains("Read program"), true);
        Ok(())
    }

    #[test]
    fn test_10002_oeis_ids_from_program_parse_error() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10002_oeis_ids_from_program_parse_error");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("A000040.asm");

        let input_content = 
r#"
boom $0,3 ; non-existing instruction
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;

        // Act
        let result = oeis_ids_from_program(&input_path);

        // Assert
        let error = result.expect_err("should fail");
        let error_message: String = error.to_string();
        assert_eq!(error_message.contains("Parse program"), true);
        Ok(())
    }
}
