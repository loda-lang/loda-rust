use std::fmt;
use super::extract_row_re::EXTRACT_ROW_RE;
use super::instruction_id::{InstructionId,ParseInstructionIdError};
use super::instruction::{Instruction,InstructionParameter};
use super::parameter_type::ParameterType;
use super::parse_parameters::*;
use super::remove_comment::remove_comment;

pub struct ParsedProgram {
    pub instruction_vec: Vec<Instruction>,
}

impl fmt::Display for ParsedProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instructions: Vec<String> = self.instruction_vec.iter().map(|instruction| {
            instruction.to_string()
        }).collect();
        let instructions_joined: String = instructions.join("\n");
        write!(f, "{}", instructions_joined)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseProgramError {
    SyntaxError(usize),
    ParseInstructionId(ParseInstructionIdError),
    ParseParameters(ParseParametersError),
}

impl fmt::Display for ParseProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::SyntaxError(line_number) => write!(f, "Syntax error in line {}", line_number),
            Self::ParseInstructionId(ref err) => write!(f, "ParseInstructionId error: {}", err),
            Self::ParseParameters(ref err) => write!(f, "ParseParameters error: {}", err),
        }
    }
}

impl From<ParseInstructionIdError> for ParseProgramError {
    fn from(err: ParseInstructionIdError) -> ParseProgramError {
        ParseProgramError::ParseInstructionId(err)
    }
}

impl From<ParseParametersError> for ParseProgramError {
    fn from(err: ParseParametersError) -> ParseProgramError {
        ParseProgramError::ParseParameters(err)
    }
}

impl ParsedProgram {
    // The direct dependencies that this program depends on.
    // This doesn't include the indirect dependencies.
    #[allow(dead_code)]
    pub fn direct_dependencies(&self) -> Vec<u64> {
        let mut program_ids: Vec<u64> = vec!();
        for instruction in &self.instruction_vec {
            if instruction.instruction_id != InstructionId::EvalSequence {
                continue;
            }
            if instruction.parameter_vec.len() != 2 {
                continue;
            }
            let param: &InstructionParameter = &(instruction.parameter_vec[1]);
            if param.parameter_type != ParameterType::Constant {
                continue;
            }
            let program_id: i64 = param.parameter_value;
            if program_id < 0 {
                continue;
            }
            program_ids.push(program_id as u64);
        }
        return program_ids;
    }

    #[allow(dead_code)]
    pub fn instruction_ids(&self) -> Vec<InstructionId> {
        self.instruction_vec.iter().map(|instruction| {
            instruction.instruction_id
        }).collect()
    }

    pub fn parse_program(raw_input: &str) -> Result<ParsedProgram, ParseProgramError> {
        let re = &EXTRACT_ROW_RE;
        let mut instruction_vec: Vec<Instruction> = vec!();
        for (index, raw_input_line) in raw_input.split("\n").enumerate() {
            let line_number: usize = index + 1;
    
            let line0 = remove_comment(raw_input_line);
            let line1: &str = line0.trim_end();
            if line1.is_empty() {
                // skip lines without code
                // if it's a line with just a comment, then skip the line.
                // if it's a line with just blank spaces, then skip the line.
                continue;
            }
    
            let captures = match re.captures(line1) {
                Some(value) => value,
                None => {
                    return Err(ParseProgramError::SyntaxError(line_number));
                }
            };
            let instruction_raw: &str = captures.get(1).map_or("", |m| m.as_str());
            let parameter_string: &str = captures.get(2).map_or("", |m| m.as_str());
    
            let instruction_id: InstructionId = 
                InstructionId::parse(instruction_raw, line_number)?;
    
            let parameter_vec: Vec<InstructionParameter> = 
                parse_parameters(parameter_string, line_number)?;
    
            let instruction = Instruction {
                instruction_id: instruction_id,
                parameter_vec: parameter_vec,
                line_number: line_number,
            };
            instruction_vec.push(instruction);
        }
    
        let parsed_program = Self {
            instruction_vec: instruction_vec
        };
        Ok(parsed_program)
    }    
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(input: &str) -> String {
        let result = ParsedProgram::parse_program(input);
        let parsed_program: ParsedProgram = match result {
            Ok(value) => value,
            Err(error) => {
                return format!("{:?}", error);
            }
        };
        parsed_program.to_string()
    }

    #[test]
    fn test_10000_empty() {
        assert_eq!(process(""), "");
        assert_eq!(process("\n  \n\t  \t"), "");
        assert_eq!(process(" ; comment 1\n;; comment 2"), "");
    }

    #[test]
    fn test_10001_simple() {
        assert_eq!(process("lpb $0\nlpe"), "lpb $0\nlpe");
        assert_eq!(process("lpb $0\n\n\nlpe"), "lpb $0\nlpe");
        assert_eq!(process("lpb $0\n \nlpe"), "lpb $0\nlpe");
        assert_eq!(process("mov $1,2\nmov $3,$4"), "mov $1,2\nmov $3,$4");
        assert_eq!(process("  mov  $1, -2\n\tmov $3\t, $44"), "mov $1,-2\nmov $3,$44");
        assert_eq!(process("\tmov $1,2 ; comment"), "mov $1,2");
    }

    #[test]
    fn test_10002_junk() {
        assert_eq!(process("mov0"), "SyntaxError(1)");
        assert_eq!(process("mov$0"), "SyntaxError(1)");
        assert_eq!(process("boom $1"), "ParseInstructionId(UnrecognizedInstructionId(1))");
        assert_eq!(process("mov $x"), "ParseParameters(UnrecognizedParameter(1))");
    }

    #[test]
    fn test_10003_direct_dependencies() {
        {
            let parsed_program: ParsedProgram = ParsedProgram::parse_program(
                "seq $1,40 ; fibonacci\nseq $2,40; fib again!\nseq $3,10\nseq $4,45").unwrap();
            assert_eq!(parsed_program.direct_dependencies(), vec!(40,40,10,45));
        }
        {
            let parsed_program: ParsedProgram = ParsedProgram::parse_program(
                "mov $1,$0\nadd $1,$1").unwrap();
            assert_eq!(parsed_program.direct_dependencies().is_empty(), true);
        }
        {
            let parsed_program: ParsedProgram = ParsedProgram::parse_program(
                ";negative parameter is ignored\nseq $1,-1000\nseq $1,-100").unwrap();
            assert_eq!(parsed_program.direct_dependencies().is_empty(), true);
        }
    }

    #[test]
    fn test_10004_instruction_ids() {
        let parsed_program: ParsedProgram = ParsedProgram::parse_program(
            "mov $1,$0\nlpb $0\ndiv $1,2\nsub $0,$1\nlpe").unwrap();
        let expected = vec![
            InstructionId::Move,
            InstructionId::LoopBegin,
            InstructionId::Divide,
            InstructionId::Subtract,
            InstructionId::LoopEnd
        ];
        assert_eq!(parsed_program.instruction_ids(), expected);
    }
}
