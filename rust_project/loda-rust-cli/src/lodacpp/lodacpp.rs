use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LodaCpp {
    loda_cpp_executable: PathBuf,
}

impl LodaCpp {
    pub fn new(loda_cpp_executable: PathBuf) -> Self {
        assert!(loda_cpp_executable.is_absolute());
        assert!(loda_cpp_executable.is_file());
        Self {
            loda_cpp_executable: loda_cpp_executable,
        }
    }

    pub fn loda_cpp_executable(&self) -> &Path {
        &self.loda_cpp_executable
    }
}

#[derive(Debug)]
pub enum LodaCppError {
    NonZeroExitCode,
    NoOutput,
    Parse,
    Timeout,
}

impl std::error::Error for LodaCppError {}

impl fmt::Display for LodaCppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NonZeroExitCode => write!(f, "Non-zero exit code"),
            Self::NoOutput => write!(f, "No output from loda-cpp"),
            Self::Parse => write!(f, "Cannot parse the output from loda-cpp"),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}
