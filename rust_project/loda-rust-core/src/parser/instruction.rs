use std::fmt;
use super::{InstructionId, ParameterType};

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

#[derive(Clone, Debug)]
pub struct Instruction {
    pub instruction_id: InstructionId,
    pub parameter_vec: Vec<InstructionParameter>,
    pub line_number: usize,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parameters: Vec<String> = self.parameter_vec.iter().map(|parameter| {
            parameter.to_string()
        }).collect();
        let parameters_joined: String = parameters.join(",");
        let spacer: &str = match parameters.is_empty() {
            true => "",
            false => " "
        };
        write!(f, "{}{}{}", self.instruction_id.shortname(), spacer, parameters_joined)
    }
}
