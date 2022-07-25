use loda_rust_core::util::BigIntVec;
use crate::postmine::{PathUtil, PostMineError};
use crate::oeis::OeisId;
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
    id_string: String,
    lodacpp_terms: BigIntVec,
    possible_ids: HashSet::<OeisId>,
    keep_program_ids: HashSet::<u32>,
}

impl CandidateProgram {
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        assert!(path.is_absolute());
        assert!(path.is_file());

        let id_osstr: &OsStr = path.file_name().ok_or(PostMineError::UnableToExtractFilenameFromPath)?;
        let id_string: String = id_osstr.to_string_lossy().to_string();

        let instance = Self {
            state: State::PendingProcessing,
            path_original: PathBuf::from(path),
            path_keep: PathUtil::path_keep(path),
            path_reject: PathUtil::path_reject(path),
            id_string: id_string,
            lodacpp_terms: vec!(),
            possible_ids: HashSet::new(),
            keep_program_ids: HashSet::new(),
        };
        Ok(instance)
    }

    pub fn id_string(&self) -> &String {
        &self.id_string
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

    pub fn possible_id_insert(&mut self, oeis_id: OeisId) {
        self.possible_ids.insert(oeis_id);
    }

    pub fn is_possible_ids_empty(&self) -> bool {
        self.possible_ids.is_empty()
    }

    pub fn possible_ids(&self) -> &HashSet<OeisId> {
        &self.possible_ids
    }

    pub fn possible_id_vec(&self) -> Vec<OeisId> {
        let mut program_ids_sorted: Vec<OeisId> = self.possible_ids.clone().into_iter().collect();
        program_ids_sorted.sort();
        program_ids_sorted
    }

    pub fn keep_program_ids_insert(&mut self, program_id: u32) {
        self.keep_program_ids.insert(program_id);
    }

    pub fn is_keep_program_ids_empty(&self) -> bool {
        self.keep_program_ids.is_empty()
    }

    pub fn keep_program_id_vec(&self) -> Vec<u32> {
        let mut program_ids_sorted: Vec<u32> = self.keep_program_ids.clone().into_iter().collect();
        program_ids_sorted.sort();
        program_ids_sorted
    }

    pub fn keep_program_ids_as_string(&self) -> String {
        let program_ids: Vec<u32> = self.keep_program_id_vec();
        let strings: Vec<String> = program_ids.iter().map(|program_id| format!("A{:0>6}", program_id)).collect();
        strings.join(",")
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

    pub fn perform_reject<I: AsRef<str>>(&mut self, reason_reject: I) -> Result<(), Box<dyn Error>> {
        if self.state != State::PendingProcessing {
            return Err(Box::new(PostMineError::CannotMutateCandidateProgramWithAlreadyResolvedState));
        }
        fs::rename(&self.path_original, &self.path_reject)?;
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path_reject)?;
        writeln!(file, "\n; reject-reason: {}", reason_reject.as_ref())?;
        self.state = State::Reject;
        Ok(())
    }

    pub fn perform_keep<I: AsRef<str>>(&mut self, reason_keep: I) -> Result<(), Box<dyn Error>> {
        if self.state != State::PendingProcessing {
            return Err(Box::new(PostMineError::CannotMutateCandidateProgramWithAlreadyResolvedState));
        }
        fs::rename(&self.path_original, &self.path_keep)?;
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path_keep)?;
        writeln!(file, "\n; keep-reason: {}", reason_keep.as_ref())?;
        self.state = State::Keep;
        Ok(())
    }

    pub fn perform_keep_or_reject_based_result(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_keep_program_ids_empty() {
            self.perform_reject("Doesn't correspond to any known OEIS sequence")?;
            return Ok(());
        } else {
            let keep_program_ids: String = self.keep_program_ids_as_string();
            let keep_reason: String = format!("Corresponds to: {}", keep_program_ids);
            self.perform_keep(keep_reason)?;
            return Ok(())
        }
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
        let mut candidate_program: CandidateProgram = CandidateProgram::new(&input_path)?;

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
        let mut candidate_program: CandidateProgram = CandidateProgram::new(&input_path)?;

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
