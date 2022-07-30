use std::fmt;

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
