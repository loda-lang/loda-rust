use crate::parser::validate_loops::ValidateLoopError;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CreateInstructionErrorType {
    ExpectZeroParameters,
    ExpectOneOrTwoParameters,
    ExpectTwoParameters,
    ParameterMustBeRegister,
    ParameterMustBeConstant,
    ConstantMustBeNonNegative,
    LoopWithConstantRangeIsTooHigh,
    RegisterIndexMustBeNonNegative,
    RegisterIndexTooHigh,
    NodeCreateError,
}

#[derive(Debug, PartialEq)]
pub struct CreateInstructionError {
    line_number: usize,
    error_type: CreateInstructionErrorType,
}

impl CreateInstructionError {
    pub fn new(line_number: usize, error_type: CreateInstructionErrorType) -> Self {
        Self {
            line_number: line_number,
            error_type: error_type
        }
    }
}

impl fmt::Display for CreateInstructionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} in line {}", self.error_type, self.line_number)
    }
}

#[derive(Debug, PartialEq)]
pub enum CreateProgramError {
    ValidateLoops(ValidateLoopError),
    CreateInstruction(CreateInstructionError),
    CannotPopEmptyStack,
}

impl fmt::Display for CreateProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ValidateLoops(ref err) => 
                write!(f, "ValidateLoops error: {}", err),
            Self::CreateInstruction(ref err) => 
                write!(f, "CreateInstruction error: {}", err),
            Self::CannotPopEmptyStack => 
                write!(f, "Mismatch between loop begins and loop ends. stack is empty."),
        }
    }
}

impl From<ValidateLoopError> for CreateProgramError {
    fn from(err: ValidateLoopError) -> CreateProgramError {
        CreateProgramError::ValidateLoops(err)
    }
}

impl From<CreateInstructionError> for CreateProgramError {
    fn from(err: CreateInstructionError) -> CreateProgramError {
        CreateProgramError::CreateInstruction(err)
    }
}
