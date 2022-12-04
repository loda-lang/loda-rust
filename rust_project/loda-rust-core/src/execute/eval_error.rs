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

    /// I have seen a huge backtrace in `BigInt` multiplication code.
    /// So now I have added a limit to how big numbers are to be multiplied.
    MultipliplyExceededLimit,

    AddSubtractExceededLimit,

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

    /// Clear instruction with a too high value
    ClearRangeLengthExceedsLimit,

    /// Using way too many cpu cycles
    StepCountExceededLimit,

    // When attempting to use a constant as the first parameter for an instruction
    CannotGetAddressOfConstant,
    CannotSetValueOfConstant,
    CannotConvertParameterValueToBigInt,
    CannotConvertBigIntToRegisterIndex,
    CannotConvertBigIntToAddress,
    CannotConvertI64ToAddress,
    AddressIsOutsideMaxCapacity,
    AddressWithNegativeValue,
    UnsupportedInstruction,

    /// Unofficial function
    UnofficialFunctionOutputVectorHasIncorrectLength,
    UnofficialFunctionCannotSetOutputValue,
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
            Self::MultipliplyExceededLimit => 
                write!(f, "Multiply is outside limit"),
            Self::AddSubtractExceededLimit => 
                write!(f, "Add/Subtract is outside limit"),
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
            Self::ClearRangeLengthExceedsLimit => 
                write!(f, "Instruction 'clr' range length exceeded limit"),
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
            Self::CannotConvertBigIntToAddress => 
                write!(f, "Cannot convert BigInt to address"),
            Self::CannotConvertI64ToAddress => 
                write!(f, "Cannot convert i64 to address"),
            Self::AddressIsOutsideMaxCapacity => 
                write!(f, "Memory address is outside the number of registers"),
            Self::AddressWithNegativeValue => 
                write!(f, "Memory address with negative value"),
            Self::UnsupportedInstruction => 
                write!(f, "Unsupported instruction"),
            Self::UnofficialFunctionOutputVectorHasIncorrectLength => 
                write!(f, "Unofficial function output vector has incorrect length"),
            Self::UnofficialFunctionCannotSetOutputValue => 
                write!(f, "Unofficial function cannot set output value"),
        }
    }
}

impl Error for EvalError {}
