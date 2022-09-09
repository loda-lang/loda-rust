use crate::parser::{Instruction, InstructionParameter, ParameterType, CreateInstructionError, CreateInstructionErrorType};
use super::{RegisterIndex, RegisterType};
use std::fmt;

/// Access to a register either direct or indirect.
#[derive(Clone, Debug)]
pub struct RegisterIndexAndType {
    pub register_index: RegisterIndex,
    pub register_type: RegisterType,
}

impl RegisterIndexAndType {
    pub fn from_parameter(instruction: &Instruction, parameter: &InstructionParameter) 
        -> Result<RegisterIndexAndType, CreateInstructionError> 
    {
        let register_type: RegisterType;
        match parameter.parameter_type {
            ParameterType::Constant => {
                let err = CreateInstructionError::new(
                    instruction.line_number,
                    CreateInstructionErrorType::ParameterMustBeRegister,
                );
                return Err(err);
            },
            ParameterType::Register => {
                register_type = RegisterType::Direct;
            },
            ParameterType::Indirect => {
                register_type = RegisterType::Indirect;
            }
        }
        let parameter_value: i64 = parameter.parameter_value;
        if parameter_value < 0 {
            let err = CreateInstructionError::new(
                instruction.line_number,
                CreateInstructionErrorType::RegisterIndexMustBeNonNegative,
            );
            return Err(err);
        }
        if parameter_value > 255 {
            let err = CreateInstructionError::new(
                instruction.line_number,
                CreateInstructionErrorType::RegisterIndexTooHigh,
            );
            return Err(err);
        }
        let register_index_value: u8 = parameter_value as u8;
        let register_index = RegisterIndex(register_index_value);
        Ok(RegisterIndexAndType {
            register_index: register_index,
            register_type: register_type
        })
    }
}

impl fmt::Display for RegisterIndexAndType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.register_type.register_prefix(), self.register_index)
    }
}
