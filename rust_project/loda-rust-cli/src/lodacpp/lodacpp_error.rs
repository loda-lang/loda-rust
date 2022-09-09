use std::fmt;

#[derive(Debug, PartialEq)]
pub enum LodaCppError {
    NonZeroExitCode,
    NoOutput,
    Timeout,
    ParseTerms,
    ParseSteps,
}

impl std::error::Error for LodaCppError {}

impl fmt::Display for LodaCppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NonZeroExitCode => write!(f, "Non-zero exit code"),
            Self::NoOutput => write!(f, "No output from loda-cpp"),
            Self::Timeout => write!(f, "Timeout"),
            Self::ParseTerms => write!(f, "Cannot extract terms from loda-cpp output"),
            Self::ParseSteps => write!(f, "Cannot extract steps from loda-cpp output"),
        }
    }
}
