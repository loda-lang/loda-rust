use anyhow::Context;
use loda_rust_core::util::BigIntVec;
use loda_rust_core::oeis::{OeisId, OeisIdHashSet};
use crate::postmine::{PathUtil, PostMineError};
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    PendingProcessing,
    Keep,
    Reject,
}

pub struct CandidateProgram {
    state: State,
    path_original: PathBuf,
    path_keep: PathBuf,
    path_reject: PathBuf,
    filename_original: String,
    lodacpp_terms: BigIntVec,
    possible_ids: OeisIdHashSet,
    keep_ids: OeisIdHashSet,
    minimized_program: String,
}

impl CandidateProgram {
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        assert!(path.is_absolute());
        assert!(path.is_file());

        let filename_osstr: &OsStr = path.file_name().ok_or(PostMineError::UnableToExtractFilenameFromPath)?;
        let filename_original: String = filename_osstr.to_string_lossy().to_string();

        let instance = Self {
            state: State::PendingProcessing,
            path_original: PathBuf::from(path),
            path_keep: PathUtil::path_keep(path),
            path_reject: PathUtil::path_reject(path),
            filename_original: filename_original,
            lodacpp_terms: vec!(),
            possible_ids: HashSet::new(),
            keep_ids: HashSet::new(),
            minimized_program: String::new(),
        };
        Ok(instance)
    }

    #[allow(dead_code)]
    pub fn filename_original(&self) -> &String {
        &self.filename_original
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn update_lodacpp_terms(&mut self, terms: BigIntVec) {
        self.lodacpp_terms = terms;
    }

    pub fn lodacpp_terms(&self) -> &BigIntVec {
        &self.lodacpp_terms
    }

    pub fn possible_id_insert(&mut self, id: OeisId) {
        self.possible_ids.insert(id);
    }

    pub fn is_possible_ids_empty(&self) -> bool {
        self.possible_ids.is_empty()
    }

    pub fn possible_ids(&self) -> &OeisIdHashSet {
        &self.possible_ids
    }

    pub fn possible_id_vec(&self) -> Vec<OeisId> {
        let mut ids: Vec<OeisId> = self.possible_ids.clone().into_iter().collect();
        ids.sort();
        ids
    }

    pub fn keep_id_insert(&mut self, id: OeisId) {
        self.keep_ids.insert(id);
    }

    pub fn is_keep_ids_empty(&self) -> bool {
        self.keep_ids.is_empty()
    }

    pub fn keep_id_vec(&self) -> Vec<OeisId> {
        let mut ids: Vec<OeisId> = self.keep_ids.clone().into_iter().collect();
        ids.sort();
        ids
    }

    pub fn keep_program_ids_as_string(&self) -> String {
        let ids: Vec<OeisId> = self.keep_id_vec();
        let strings: Vec<String> = ids.iter().map(|id| id.a_number()).collect();
        strings.join(",")
    }

    pub fn assign_minimized_program(&mut self, minimized_program: String) {
        self.minimized_program = minimized_program;
    }

    pub fn minimized_program(&self) -> &String {
        &self.minimized_program
    }

    pub fn path_original(&self) -> &Path {
        &self.path_original
    }

    #[allow(dead_code)]
    pub fn path_reject(&self) -> &Path {
        &self.path_reject
    }

    #[allow(dead_code)]
    pub fn path_keep(&self) -> &Path {
        &self.path_keep
    }

    /// Prefix error messages with `;`, so that when they can be inserted into a LODA program
    /// and the LODA program is still a valid program.
    fn prefix_with_semicolon<I: AsRef<str>>(input: I) -> String {
        // let v: Vec<&str> = input.as_ref().split("\n").collect();
        // v.join("\n; ")
        input.as_ref().replace("\n", "\n; ")
    }

    pub fn perform_reject<I: AsRef<str>>(&mut self, reason_reject: I) -> anyhow::Result<()> {
        if self.state != State::PendingProcessing {
            return Err(anyhow::anyhow!("perform_reject of candidate program with already resolved state"));
        }
        fs::rename(&self.path_original, &self.path_reject)
            .with_context(|| format!("perform_reject: Unable to rename file from: {:?} to: {:?}", &self.path_original, &self.path_reject))?;
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path_reject)
            .with_context(|| format!("perform_reject: Unable to open in append-mode. path: {:?}", &self.path_reject))?;
        let reason: String = Self::prefix_with_semicolon(reason_reject.as_ref());
        writeln!(file, "\n; reject-reason: {}", reason)
            .with_context(|| format!("perform_reject: Unable to append to rejection-reason to file: {:?}", &self.path_reject))?;
        self.state = State::Reject;
        Ok(())
    }

    pub fn perform_keep<I: AsRef<str>>(&mut self, reason_keep: I) -> anyhow::Result<()> {
        if self.state != State::PendingProcessing {
            return Err(anyhow::anyhow!("perform_keep of candidate program with already resolved state"));
        }
        fs::rename(&self.path_original, &self.path_keep)
            .with_context(|| format!("perform_keep: Unable to rename file from: {:?} to: {:?}", &self.path_original, &self.path_keep))?;
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path_keep)
            .with_context(|| format!("perform_keep: Unable to open in append-mode. path: {:?}", &self.path_keep))?;
        let reason: String = Self::prefix_with_semicolon(reason_keep.as_ref());
        writeln!(file, "\n; keep-reason: {}", reason)
            .with_context(|| format!("perform_keep: Unable to append to keep-reason to file: {:?}", &self.path_keep))?;
        self.state = State::Keep;
        Ok(())
    }

    pub fn perform_keep_or_reject_based_result(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_keep_ids_empty() {
            let oeis_ids: Vec<OeisId> = self.possible_id_vec();
            if oeis_ids.is_empty() {
                self.perform_reject("Doesn't correspond to any known OEIS sequence")
                    .context("perform_keep_or_reject_based_result doesn't correspond to any known OEIS sequence")?;
                return Ok(());
            }
            let message = format!("Worse than the existing programs: {:?}", oeis_ids);
            self.perform_reject(message)
                .context("perform_keep_or_reject_based_result worse than the existing program")?;
            return Ok(());
        }
        let keep_program_ids: String = self.keep_program_ids_as_string();
        let keep_reason: String = format!("Corresponds to: {}", keep_program_ids);
        self.perform_keep(keep_reason)
            .context("perform_keep_or_reject_based_result")?;
        return Ok(())
    }
}

impl fmt::Display for CandidateProgram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.filename_original)
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
    fn test_10000_filename_original() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_filename_original");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("19840101-054915-1251916462.asm");

        let input_content = 
r#"; A123456
mul $0,2
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;
        input_file.sync_all()?;

        // Act
        let candidate_program: CandidateProgram = CandidateProgram::new(&input_path)?;

        // Assert
        assert_eq!(candidate_program.filename_original(), "19840101-054915-1251916462.asm");
        Ok(())
    }

    #[test]
    fn test_20000_prefix_with_semicolon() {
        assert_eq!(CandidateProgram::prefix_with_semicolon("x"), "x");
        assert_eq!(CandidateProgram::prefix_with_semicolon("x\ny"), "x\n; y");
        assert_eq!(CandidateProgram::prefix_with_semicolon("x\ny\nz"), "x\n; y\n; z");
    }

    #[test]
    fn test_30000_perform_reject() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_20000_perform_reject");
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
        let mut candidate_program: CandidateProgram = CandidateProgram::new(&input_path)?;

        // Act
        candidate_program.perform_reject("REJECT-REASON1\nREJECT-REASON2\nREJECT-REASON3")?;

        // Assert
        assert_eq!(candidate_program.path_original().is_file(), false);
        assert_eq!(candidate_program.path_reject().is_file(), true);
        let output_content: String = fs::read_to_string(candidate_program.path_reject())?;
        assert_eq!(output_content.ends_with("\n; reject-reason: REJECT-REASON1\n; REJECT-REASON2\n; REJECT-REASON3\n"), true);
        Ok(())
    }

    #[test]
    fn test_30001_perform_keep() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_20001_perform_keep");
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
        let mut candidate_program: CandidateProgram = CandidateProgram::new(&input_path)?;

        // Act
        candidate_program.perform_keep("KEEP-REASON1\nKEEP-REASON2\nKEEP-REASON3")?;

        // Assert
        assert_eq!(candidate_program.path_original().is_file(), false);
        assert_eq!(candidate_program.path_keep().is_file(), true);
        let output_content: String = fs::read_to_string(candidate_program.path_keep())?;
        assert_eq!(output_content.ends_with("\n; keep-reason: KEEP-REASON1\n; KEEP-REASON2\n; KEEP-REASON3\n"), true);
        Ok(())
    }
}
