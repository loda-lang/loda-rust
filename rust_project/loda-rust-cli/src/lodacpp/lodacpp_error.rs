use std::fmt;

#[derive(Debug)]
pub enum LodaCppError {
    NonZeroExitCode,
    NoOutput,
    ParseTerms,
    Timeout,
}

impl std::error::Error for LodaCppError {}

impl fmt::Display for LodaCppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NonZeroExitCode => write!(f, "Non-zero exit code"),
            Self::NoOutput => write!(f, "No output from loda-cpp"),
            Self::ParseTerms => write!(f, "Cannot extract terms from from loda-cpp output"),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}
