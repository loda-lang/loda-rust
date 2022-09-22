use std::fmt;
use super::create_program::*;
use super::parse_program::*;
use crate::execute::Program;

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

/// This function can parse simple programs, without the `seq` instruction.
/// This function does not resolve dependencies.
/// 
/// For parsing more complicated programs, use the `DependencyManager`.
#[allow(dead_code)]
pub fn simple_parse(input: &str) -> Result<Program, ParseError> {
    let parsed_program: ParsedProgram = ParsedProgram::parse_program(input)?;
    let create_program = CreateProgram::new();
    let program: Program = create_program.create_program(&parsed_program.instruction_vec)?;
    Ok(program)
}
