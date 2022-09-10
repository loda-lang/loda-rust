use super::{Instruction, ParameterType};

pub fn contain_parameter_type_indirect(instruction_vec: &Vec<Instruction>) -> bool {
    for instruction in instruction_vec {
        for param in &instruction.parameter_vec {
            if param.parameter_type == ParameterType::Indirect {
                return true;
            }
        }
    }
    false
}
