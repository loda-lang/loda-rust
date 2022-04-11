use loda_rust_core::parser::{Instruction, InstructionParameter, ParameterType};

pub fn instruction_diff_between_constants(instruction0: &Instruction, instruction1: &Instruction) -> Option<(i64, i64)> {
    let parameters0: &Vec<InstructionParameter> = &instruction0.parameter_vec;
    let parameters1: &Vec<InstructionParameter> = &instruction1.parameter_vec;

    // Reject if the number of parameters differs
    if parameters0.len() != parameters1.len() {
        return None;
    }

    for parameter_index in 0..parameters0.len() {
        let parameter0: &InstructionParameter = &parameters0[parameter_index];
        let parameter1: &InstructionParameter = &parameters1[parameter_index];

        // Reject if the parameter type differs
        if parameter0.parameter_type != parameter1.parameter_type {
            return None;
        }
        let is_same_value = parameter0.parameter_value == parameter1.parameter_value;
        match parameter0.parameter_type {
            ParameterType::Constant => {
                if !is_same_value {
                    return Some((parameter0.parameter_value, parameter1.parameter_value));
                }
            },
            ParameterType::Register => {
                if !is_same_value {
                    return None;
                }
            },
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use loda_rust_core::parser::ParsedProgram;

    fn process(input0: &str, input1: &str) -> String {
        let result0 = ParsedProgram::parse_program(input0);
        let parsed_program0: ParsedProgram = match result0 {
            Ok(value) => value,
            Err(error) => {
                return format!("BOOM-INPUT0: {:?}", error);
            }
        };
        let result1 = ParsedProgram::parse_program(input1);
        let parsed_program1: ParsedProgram = match result1 {
            Ok(value) => value,
            Err(error) => {
                return format!("BOOM-INPUT1: {:?}", error);
            }
        };
        let instruction0: &Instruction = parsed_program0.instruction_vec.first().unwrap();
        let instruction1: &Instruction = parsed_program1.instruction_vec.first().unwrap();
        let result = instruction_diff_between_constants(instruction0, instruction1);
        match result {
            Some((value0, value1)) => {
                return format!("{} != {}", value0, value1);
            },
            None => {
                return "None".to_string();
            }
        }
    }

    #[test]
    fn test_10000_none() {
        assert_eq!(process("mul $1,2", "mul $1,2"), "None");
        assert_eq!(process("mul $1,$2", "mul $1,$2"), "None");
        assert_eq!(process("mul $0,3", "add $0,3"), "None");
    }

    #[test]
    fn test_10001_different_constants() {
        assert_eq!(process("mul $1,2", "mul $1,3"), "2 != 3");
        assert_eq!(process("add $1,1", "add $1,10"), "1 != 10");
        assert_eq!(process("mul $0,2", "add $0,1"), "2 != 1");
    }
}
