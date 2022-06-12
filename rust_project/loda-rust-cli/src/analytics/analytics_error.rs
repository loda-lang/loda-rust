use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum AnalyticsError {
    BatchProgramAnalyzer(String),
}

impl fmt::Display for AnalyticsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BatchProgramAnalyzer(message) =>
                write!(f, "BatchProgramAnalyzer. message: {}", message),
        }
    }
}

impl Error for AnalyticsError {}
