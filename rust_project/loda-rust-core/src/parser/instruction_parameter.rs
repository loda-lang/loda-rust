use std::fmt;
use super::ParameterType;

#[derive(Clone, Debug)]
pub struct InstructionParameter {
    pub parameter_type: ParameterType,
    pub parameter_value: i64,
}

impl InstructionParameter {
    pub fn new(parameter_type: ParameterType, parameter_value: i64) -> Self {
        Self {
            parameter_type: parameter_type,
            parameter_value: parameter_value
        }
    }
}

impl fmt::Display for InstructionParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.parameter_type.prefix(), self.parameter_value)
    }
}
