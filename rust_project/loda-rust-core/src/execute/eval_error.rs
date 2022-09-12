use std::error::Error;
use std::fmt;

pub struct ValidateCallError {}

/// Errors that can arise while evaluating the program.
#[derive(Debug)]
pub enum EvalError {
    /// During mining it makes little sense if the values are too extreme to 
    /// possible lead to a result. Here the CheckValue settings controls the limit.
    /// When not-mining there are no limit to the register value.
    InputOutOfRange,
    OutputOutOfRange,

    /// Programs are usually well behaved for 0 and greater values.
    /// However for negative values the behavior is undefined.
    EvalSequenceWithNegativeParameter,

    /// When a mathematical function is evaluated outside of its domain of definition.
    DivisionByZero,

    /// Binomial with N >= 34 and the result value can no longer fit into a 32bit integer.
    /// Binomial with N >= 67 and the result value can no longer fit into a 64bit integer.
    /// During mining, it can be a time waster computing binomial with huge values.
    BinomialDomainError,

    /// When a mathematical function is evaluated outside of its domain of definition.
    PowerZeroDivision,
    PowerExponentTooHigh,
    /// During mining, it can be a time waster computing power with huge values.
    PowerExceededLimit,

    /// Range length is beyond the ProgramState max length
    LoopRangeLengthExceededLimit,

    /// Stuck in a loop that takes way too long time to compute
    LoopCountExceededLimit,

    /// Using way too many cpu cycles
    StepCountExceededLimit,

    // When attempting to use a constant as the first parameter for an instruction
    CannotGetAddressOfConstant,
    CannotSetValueOfConstant,
    CannotConvertParameterValueToBigInt,
    CannotConvertBigIntToRegisterIndex,
    UnsupportedInstruction,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InputOutOfRange => 
                write!(f, "Input is out of range"),
            Self::OutputOutOfRange => 
                write!(f, "Output is out of range"),
            Self::EvalSequenceWithNegativeParameter => 
                write!(f, "Eval sequence with a negative parameter"),
            Self::DivisionByZero => 
                write!(f, "Division by zero"),
            Self::BinomialDomainError => 
                write!(f, "Binomial domain error"),
            Self::PowerZeroDivision => 
                write!(f, "Power zero division"),
            Self::PowerExponentTooHigh => 
                write!(f, "Power exponent too high"),
            Self::PowerExceededLimit => 
                write!(f, "Power exceeded limit"),
            Self::LoopRangeLengthExceededLimit => 
                write!(f, "Loop range length exceeded limit"),
            Self::LoopCountExceededLimit => 
                write!(f, "Loop count exceeded limit, stuck in a loop that takes way too long time to compute"),
            Self::StepCountExceededLimit => 
                write!(f, "Step count exceeded limit, using way too many cpu cycles"),
            Self::CannotGetAddressOfConstant => 
                write!(f, "Cannot get address of a constant"),
            Self::CannotSetValueOfConstant => 
                write!(f, "Cannot set value of a constant"),
            Self::CannotConvertParameterValueToBigInt => 
                write!(f, "Cannot convert parameter value to BigInt"),
            Self::CannotConvertBigIntToRegisterIndex => 
                write!(f, "Cannot convert BigInt to register index"),
            Self::UnsupportedInstruction => 
                write!(f, "Unsupported instruction"),
        }
    }
}

impl Error for EvalError {}
