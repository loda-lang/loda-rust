use std::fmt;
use super::instruction_id::InstructionId;
use super::parameter_type::ParameterType;

#[derive(Debug)]
pub struct InstructionParameter {
    pub parameter_type: ParameterType,
    pub parameter_value: i64,
}

impl fmt::Display for InstructionParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix: &str = match self.parameter_type {
            ParameterType::Constant => "",
            ParameterType::Register => "$",
        };
        write!(f, "{}{}", prefix, self.parameter_value)
    }
}

#[derive(Debug)]
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
