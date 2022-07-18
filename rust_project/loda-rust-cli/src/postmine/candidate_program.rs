use crate::postmine::PostMineError;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};

pub struct CandidateProgram {
    path: PathBuf,
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
            path: PathBuf::from(path),
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

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl fmt::Display for CandidateProgram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.id_string)
    }
}
