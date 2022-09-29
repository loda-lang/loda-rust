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
