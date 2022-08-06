use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum PostMineError {
    NoPendingProgramsInMineEventDir,
    UnableToExtractFilenameFromPath,
    CannotMutateCandidateProgramWithAlreadyResolvedState,
    CannotConstructUniqueFilenameForMismatch,
}

impl fmt::Display for PostMineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoPendingProgramsInMineEventDir =>
                write!(f, "No pending programs in the 'mine-event' dir"),
            Self::UnableToExtractFilenameFromPath =>
                write!(f, "Unable to extract filename from path"),
            Self::CannotMutateCandidateProgramWithAlreadyResolvedState =>
                write!(f, "Cannot mutate candidate program with already resolved state"),
            Self::CannotConstructUniqueFilenameForMismatch =>
                write!(f, "Cannot construct a unique filename for a mismatch program"),
        }
    }
}

impl Error for PostMineError {}
