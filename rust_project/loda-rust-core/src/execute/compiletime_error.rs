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
