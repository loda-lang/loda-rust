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
            ParameterType::Direct => {
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
        let register_index = RegisterIndex(parameter_value as u64);
        Ok(RegisterIndexAndType {
            register_index: register_index,
            register_type: register_type
        })
    }
}

impl fmt::Display for RegisterIndexAndType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.register_type.prefix(), self.register_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_fmt_direct() {
        let instance = RegisterIndexAndType {
            register_index: RegisterIndex(42),
            register_type: RegisterType::Direct,
        };
        assert_eq!(instance.to_string(), "$42");
    }

    #[test]
    fn test_10001_fmt_indirect() {
        let instance = RegisterIndexAndType {
            register_index: RegisterIndex(123),
            register_type: RegisterType::Indirect,
        };
        assert_eq!(instance.to_string(), "$$123");
    }
}
