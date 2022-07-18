use crate::postmine::{PathUtil, PostMineError};
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};

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
}

impl fmt::Display for CandidateProgram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.id_string)
    }
}
