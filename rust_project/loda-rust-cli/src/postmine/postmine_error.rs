use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum PostMineError {
    UnableToExtractFilenameFromPath,
}

impl fmt::Display for PostMineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnableToExtractFilenameFromPath =>
                write!(f, "Unable to extract filename from path"),
        }
    }
}

impl Error for PostMineError {}
