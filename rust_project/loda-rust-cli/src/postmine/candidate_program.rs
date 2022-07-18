use crate::postmine::{PathUtil, PostMineError};
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;

pub struct CandidateProgram {
    path_original: PathBuf,
    path_keep: PathBuf,
    path_reject: PathBuf,
    id_string: String,
    terms40: Option<String>,
}

impl CandidateProgram {
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        assert!(path.is_absolute());
        assert!(path.is_file());

        let id_osstr: &OsStr = path.file_name().ok_or(PostMineError::UnableToExtractFilenameFromPath)?;
        let id_string: String = id_osstr.to_string_lossy().to_string();

        let instance = Self {
            path_original: PathBuf::from(path),
            path_keep: PathUtil::path_keep(path),
            path_reject: PathUtil::path_reject(path),
            id_string: id_string,
            terms40: None,
        };
        Ok(instance)
    }

    pub fn id_string(&self) -> &String {
        &self.id_string
    }

    pub fn update_terms40(&mut self, terms40: String) {
        self.terms40 = Some(terms40);
    }

    pub fn path_original(&self) -> &Path {
        &self.path_original
    }

    pub fn path_reject(&self) -> &Path {
        &self.path_reject
    }

    pub fn path_keep(&self) -> &Path {
        &self.path_keep
    }

    pub fn perform_reject<I: AsRef<str>>(&self, reason_reject: I) -> Result<(), Box<dyn Error>> {
        fs::rename(&self.path_original, &self.path_reject)?;
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path_reject)?;
        writeln!(file, "\n; reject-reason: {}", reason_reject.as_ref())?;
        Ok(())
    }

    pub fn perform_keep<I: AsRef<str>>(&self, reason_keep: I) -> Result<(), Box<dyn Error>> {
        fs::rename(&self.path_original, &self.path_keep)?;
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path_keep)?;
        writeln!(file, "\n; keep-reason: {}", reason_keep.as_ref())?;
        Ok(())
    }
}

impl fmt::Display for CandidateProgram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.id_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use std::error::Error;
    use std::fs::File;

    #[test]
    fn test_10000_perform_reject() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_perform_reject");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("19840101-054915-1251916462.asm");

        let input_content = 
r#"; A039207
mov $1,1
add $1,$0
div $1,9
mul $1,2
add $0,$1
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;
        input_file.sync_all()?;
        let candidate_program: CandidateProgram = CandidateProgram::new(&input_path)?;

        // Act
        candidate_program.perform_reject("REJECT-REASON")?;

        // Assert
        assert_eq!(candidate_program.path_original().is_file(), false);
        assert_eq!(candidate_program.path_reject().is_file(), true);
        let output_content: String = fs::read_to_string(candidate_program.path_reject())?;
        assert_eq!(output_content.ends_with("\n; reject-reason: REJECT-REASON\n"), true);
        Ok(())
    }

    #[test]
    fn test_10001_perform_keep() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10001_perform_keep");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("19840101-054915-1251916462.asm");

        let input_content = 
r#"; A039207
mov $1,1
add $1,$0
div $1,9
mul $1,2
add $0,$1
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;
        input_file.sync_all()?;
        let candidate_program: CandidateProgram = CandidateProgram::new(&input_path)?;

        // Act
        candidate_program.perform_keep("KEEP-REASON")?;

        // Assert
        assert_eq!(candidate_program.path_original().is_file(), false);
        assert_eq!(candidate_program.path_keep().is_file(), true);
        let output_content: String = fs::read_to_string(candidate_program.path_keep())?;
        assert_eq!(output_content.ends_with("\n; keep-reason: KEEP-REASON\n"), true);
        Ok(())
    }
}
