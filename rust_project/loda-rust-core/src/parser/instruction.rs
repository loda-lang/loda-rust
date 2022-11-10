use std::fmt;
use super::{InstructionId, InstructionParameter};

#[derive(Clone, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParameterType;

    #[test]
    fn test_10000_equal_yes() {
        let instance = Instruction { 
            instruction_id: InstructionId::Add, 
            parameter_vec: vec![
                InstructionParameter::new(ParameterType::Direct, 0),
                InstructionParameter::new(ParameterType::Constant, 1)], 
                line_number: 0 
            };        
        assert_eq!(instance, instance);
    }

    #[test]
    fn test_10001_equal_no() {
        let instance0 = Instruction { 
            instruction_id: InstructionId::Add, 
            parameter_vec: vec![
                InstructionParameter::new(ParameterType::Direct, 0),
                InstructionParameter::new(ParameterType::Constant, 1)], 
                line_number: 0 
            };        
        let instance1 = Instruction { 
            instruction_id: InstructionId::Subtract, 
            parameter_vec: vec![
                InstructionParameter::new(ParameterType::Direct, 0),
                InstructionParameter::new(ParameterType::Constant, 1)], 
                line_number: 0 
            };        
        assert_ne!(instance0, instance1);
    }

    #[test]
    fn test_10002_equal_no() {
        let instance0 = Instruction { 
            instruction_id: InstructionId::Add, 
            parameter_vec: vec![
                InstructionParameter::new(ParameterType::Direct, 0),
                InstructionParameter::new(ParameterType::Constant, 1)], 
                line_number: 0 
            };        
        let instance1 = Instruction { 
            instruction_id: InstructionId::Add, 
            parameter_vec: vec![
                InstructionParameter::new(ParameterType::Direct, 0),
                InstructionParameter::new(ParameterType::Constant, 2)], 
                line_number: 0 
            };        
        assert_ne!(instance0, instance1);
    }
}
