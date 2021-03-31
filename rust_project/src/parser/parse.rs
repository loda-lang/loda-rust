use std::fmt;
use super::create_program::*;
use super::parse_program::*;

pub struct ParseResult {
    pub created_program: CreatedProgram,
}

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


pub fn parse(input: &str) -> Result<ParseResult, ParseError> {
    let parsed_program: ParsedProgram = parse_program(input)?;

    let created_program: CreatedProgram = create_program(&parsed_program.instruction_vec)?;

    Ok(ParseResult {
        created_program: created_program,
    })
}

pub fn parse2(parsed_program: &ParsedProgram) -> Result<ParseResult, ParseError> {
    let created_program: CreatedProgram = create_program(&parsed_program.instruction_vec)?;

    Ok(ParseResult {
        created_program: created_program,
    })
}
