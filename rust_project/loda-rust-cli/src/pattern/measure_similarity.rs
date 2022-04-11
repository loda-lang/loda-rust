use loda_rust_core::parser::{Instruction, InstructionParameter, ParameterType};

pub enum ProgramSimilarity {
    NotSimilar,
    SimilarWithDifferentConstants(usize),
}

impl ProgramSimilarity {
    pub fn measure_similarity(instruction_vec0: &Vec<Instruction>, instruction_vec1: &Vec<Instruction>) -> ProgramSimilarity {
        // Reject if the number of instructions differs
        if instruction_vec0.len() != instruction_vec1.len() {
            return ProgramSimilarity::NotSimilar;
        }

        // Reject if the instructions differs
        for index in 0..instruction_vec0.len() {
            if instruction_vec0[index].instruction_id != instruction_vec1[index].instruction_id {
                return ProgramSimilarity::NotSimilar;
            }
        }

        let mut number_of_differencies: usize = 0;
        for index in 0..instruction_vec0.len() {
            let instruction0: &Instruction = &instruction_vec0[index];
            let instruction1: &Instruction = &instruction_vec1[index];
            let parameters0: &Vec<InstructionParameter> = &instruction0.parameter_vec;
            let parameters1: &Vec<InstructionParameter> = &instruction1.parameter_vec;

            // Reject if the number of parameters differs
            if parameters0.len() != parameters1.len() {
                return ProgramSimilarity::NotSimilar;
            }

            for parameter_index in 0..parameters0.len() {
                let parameter0: &InstructionParameter = &parameters0[parameter_index];
                let parameter1: &InstructionParameter = &parameters1[parameter_index];

                // Reject if the parameter type differs
                if parameter0.parameter_type != parameter1.parameter_type {
                    return ProgramSimilarity::NotSimilar;
                }

                let is_same_value = parameter0.parameter_value == parameter1.parameter_value;

                match parameter0.parameter_type {
                    ParameterType::Constant => {
                        if !is_same_value {
                            number_of_differencies += 1;
                        }
                    },
                    ParameterType::Register => {
                        if !is_same_value {
                            return ProgramSimilarity::NotSimilar;
                        }
                    },
                }
            }
        }
        ProgramSimilarity::SimilarWithDifferentConstants(number_of_differencies)
    }
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
        let similarity = ProgramSimilarity::measure_similarity(&parsed_program0.instruction_vec, &parsed_program1.instruction_vec);
        match similarity {
            ProgramSimilarity::NotSimilar => { 
                return "NotSimilar".to_string();
            },
            ProgramSimilarity::SimilarWithDifferentConstants(count) => {
                return format!("similar{}", count);
            }
        }
    }

    #[test]
    fn test_10000_similar0() {
        assert_eq!(process("", ""), "similar0");
        assert_eq!(process("\n  \n\t  \t", "  \t\n ; "), "similar0");
        assert_eq!(process(" ; comment 1\n;; comment 2", ""), "similar0");
        assert_eq!(process("mul $0,1", "; comment\nmul $0,1\n\n; comment"), "similar0");
        assert_eq!(process("  mul  $0 , $1", "mul $0,$1"), "similar0");
    }

    #[test]
    fn test_10001_similar1() {
        assert_eq!(process("mul $0,1", "mul $0,2"), "similar1");
        assert_eq!(process("mul $0,1\nadd $0,10", "mul $0,2\nadd $0,10"), "similar1");
        assert_eq!(process("add $0,10\nmul $0,1", "add $0,10\nmul $0,2"), "similar1");
        assert_eq!(process("add $0,$0\nmul $0,1", "add $0,$0\nmul $0,2"), "similar1");
    }

    #[test]
    fn test_10002_similar2() {
        assert_eq!(process("mul $0,7\ndiv $0,3", "mul $0,3\ndiv $0,7"), "similar2");
        assert_eq!(process("add $0,10\nmul $0,7\ndiv $0,3", "add $0,10\nmul $0,3\ndiv $0,7"), "similar2");
        assert_eq!(process("mul $0,7\nadd $0,10\ndiv $0,3", "mul $0,3\nadd $0,10\ndiv $0,7"), "similar2");
        assert_eq!(process("mul $0,7\ndiv $0,3\nadd $0,10", "mul $0,3\ndiv $0,7\nadd $0,10"), "similar2");
    }

    #[test]
    fn test_20001_notsimilar1() {
        assert_eq!(process("", "add $0,2"), "NotSimilar");
        assert_eq!(process("mul $0,2", "add $0,2"), "NotSimilar");
        assert_eq!(process("mul $0,2\nadd $0,1", "add $0,1\nmul $0,2"), "NotSimilar");
    }
}
