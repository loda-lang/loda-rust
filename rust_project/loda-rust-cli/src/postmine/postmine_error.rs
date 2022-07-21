use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum PostMineError {
    NoPendingProgramsInMineEventDir,
    UnableToExtractFilenameFromPath,
    CannotMutateCandidateProgramWithAlreadyResolvedState,
}

impl fmt::Display for PostMineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoPendingProgramsInMineEventDir =>
                write!(f, "NoPendingProgramsInMineEventDir"),
            Self::UnableToExtractFilenameFromPath =>
                write!(f, "UnableToExtractFilenameFromPath"),
            Self::CannotMutateCandidateProgramWithAlreadyResolvedState =>
                write!(f, "CannotMutateCandidateProgramWithAlreadyResolvedState"),
        }
    }
}

impl Error for PostMineError {}
