use std::fmt;
use super::parse_program::*;
use crate::execute::compiletime_error::*;

#[derive(Debug)]
pub enum ParseError {
    ParseProgram(ParseProgramError),
    CreateProgram(CreateProgramError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ParseProgram(ref err) => 
                write!(f, "Unable to parse program. {}", err),
            Self::CreateProgram(ref err) => 
                write!(f, "Unable to create program: {}", err),
        }
    }
}

impl From<ParseProgramError> for ParseError {
    fn from(err: ParseProgramError) -> ParseError {
        ParseError::ParseProgram(err)
    }
}

impl From<CreateProgramError> for ParseError {
    fn from(err: CreateProgramError) -> ParseError {
        ParseError::CreateProgram(err)
    }
}
