use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum PostMineError {
    NoPendingProgramsInMineEventDir,
}

impl fmt::Display for PostMineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoPendingProgramsInMineEventDir =>
                write!(f, "NoPendingProgramsInMineEventDir"),
        }
    }
}

impl Error for PostMineError {}
