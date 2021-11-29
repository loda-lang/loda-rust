use std::error::Error;
use std::fmt;

// Errors that can be caught while creating the program.
#[derive(Debug)]
pub enum CreateError {
    // The `clr` instruction.
    ClearRangeLengthMustBeNonNegative,
    ClearRangeLengthExceedsLimit,
}

impl fmt::Display for CreateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ClearRangeLengthMustBeNonNegative => 
                write!(f, "clear range length must be non-negative"),
            Self::ClearRangeLengthExceedsLimit => 
                write!(f, "clear range length must be less than or equal to 255"),
        }
    }
}

impl Error for CreateError {}
