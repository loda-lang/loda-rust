use std::fmt;
use super::{EXTRACT_ROW_RE,Instruction,InstructionParameter,ParameterType,ParseParametersError,parse_parameters,remove_comment,extract_offset,ExtractOffsetError};
use super::{InstructionId,ParseInstructionIdError,ParseInstructionId};

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedProgram {
    pub optional_offset: Option<i32>,
    pub instruction_vec: Vec<Instruction>,
}

impl ParsedProgram {
    pub fn new() -> Self {
        Self {
            optional_offset: None,
            instruction_vec: vec!()
        }
    }

    pub fn without_offset(self) -> Self {
        Self {
            optional_offset: None,
            instruction_vec: self.instruction_vec
        }
    }
}

impl fmt::Display for ParsedProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rows: Vec<String> = vec!();
        if let Some(offset) = &self.optional_offset {
            rows.push(format!("#offset {}", offset));
        }
        for instruction in &self.instruction_vec {
            rows.push(instruction.to_string());
        }
        let rows_joined: String = rows.join("\n");
        write!(f, "{}", rows_joined)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseProgramError {
    SyntaxError(usize),
    ParseInstructionId(ParseInstructionIdError),
    ParseParameters(ParseParametersError),
    ExtractOffset(ExtractOffsetError),
}

impl fmt::Display for ParseProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::SyntaxError(line_number) => write!(f, "Syntax error in line {}", line_number),
            Self::ParseInstructionId(ref err) => write!(f, "ParseInstructionId error: {}", err),
            Self::ParseParameters(ref err) => write!(f, "ParseParameters error: {}", err),
            Self::ExtractOffset(ref err) => write!(f, "ExtractOffset error: {}", err),
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

impl From<ExtractOffsetError> for ParseProgramError {
    fn from(err: ExtractOffsetError) -> ParseProgramError {
        ParseProgramError::ExtractOffset(err)
    }
}

impl ParsedProgram {
    /// Returns `Ok` if the program can be parsed.
    /// 
    /// Returns `Err` if there is a problem during parsing.
    pub fn parse_program(raw_input: &str) -> Result<ParsedProgram, ParseProgramError> {
        let re = &EXTRACT_ROW_RE;
        let mut optional_offset: Option<i32> = None;
        let mut instruction_vec: Vec<Instruction> = vec!();
        for (index, raw_input_line) in raw_input.split("\n").enumerate() {
            let line_number: usize = index + 1;

            // Extract the `#offset` if present.
            let (line0, extracted_offset) = extract_offset(raw_input_line, line_number)?;
            if let Some(value) = extracted_offset {
                if optional_offset.is_some() {
                    // The `#offset` line must occur only once.
                    return Err(ParseProgramError::SyntaxError(line_number));
                }
                if instruction_vec.is_empty() == false {
                    // The `#offset` line must occur before the instructions.
                    return Err(ParseProgramError::SyntaxError(line_number));
                }
                optional_offset = Some(value);
                continue;
            }
    
            // Discard comments and trim the line.
            let line1 = remove_comment(&line0);
            let line2: &str = line1.trim_end();
            if line2.is_empty() {
                // skip lines without code
                // if it's a line with just a comment, then skip the line.
                // if it's a line with just blank spaces, then skip the line.
                continue;
            }
    
            // Parse the instruction
            let captures = match re.captures(line2) {
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
            optional_offset: optional_offset,
            instruction_vec: instruction_vec
        };
        Ok(parsed_program)
    }    

    /// The direct dependencies that this program depends on.
    /// 
    /// This doesn't include the indirect dependencies.
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

    /// Determines if a program uses `ParameterType::Indirect`
    pub fn contain_parameter_type_indirect(&self) -> bool {
        for instruction in &self.instruction_vec {
            for parameter in &instruction.parameter_vec {
                if parameter.parameter_type == ParameterType::Indirect {
                    return true
                }
            }
        }
        false
    }

    pub fn assign_zero_line_numbers(&mut self) {
        for instruction in self.instruction_vec.iter_mut() {
            instruction.line_number = 0;
        }        
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
        assert_eq!(process("add $$1,2\nsub $2,$$1"), "add $$1,2\nsub $2,$$1");
        assert_eq!(process("#offset 0"), "#offset 0");
        assert_eq!(process("#offset 42\n; comment"), "#offset 42");
        assert_eq!(process("#offset  42\nadd  $0 , 1"), "#offset 42\nadd $0,1");

        // UnofficialFunction
        assert_eq!(process("\n\nf11 $1,22 ; comment"), "f11 $1,22");
        assert_eq!(process("f11 $1,22\n f99 $8,8\n"), "f11 $1,22\nf99 $8,8");
    }

    #[test]
    fn test_10002_junk() {
        assert_eq!(process("Add"), "SyntaxError(1)");
        assert_eq!(process("ADD"), "SyntaxError(1)");
        assert_eq!(process("addd"), "ParseInstructionId(UnrecognizedInstructionId(1))");
        assert_eq!(process("mov0"), "ParseInstructionId(UnrecognizedInstructionId(1))");
        assert_eq!(process("mov$0"), "SyntaxError(1)");
        assert_eq!(process("boom $1"), "ParseInstructionId(UnrecognizedInstructionId(1))");
        assert_eq!(process("mov $x"), "ParseParameters(UnrecognizedParameter(1))");
        assert_eq!(process("mov $$$3,4"), "ParseParameters(UnrecognizedParameterType(1))");
        assert_eq!(process("mov $-3,0"), "ParseParameters(NegativeValueNotAllowedForThisParameterType(1))");
        assert_eq!(process("mov $1,$-3"), "ParseParameters(NegativeValueNotAllowedForThisParameterType(1))");
        assert_eq!(process("mov $0,00"), "ParseParameters(StrictIncorrectParameterValue(1))");
        assert_eq!(process("mov $00,-0"), "ParseParameters(StrictIncorrectParameterValue(1))");
        assert_eq!(process("#offset -0"), "ExtractOffset(InvalidSyntax(1))");
        assert_eq!(process("#offset 007"), "ExtractOffset(InvalidSyntax(1))");
        assert_eq!(process("#offset 123 ; comment"), "ExtractOffset(InvalidSyntax(1))");
        assert_eq!(process(" #offset 123"), "ExtractOffset(InvalidSyntax(1))");
        assert_eq!(process("#offset 123 "), "ExtractOffset(InvalidSyntax(1))");
        assert_eq!(process("add $0,1\n#offset 1"), "SyntaxError(2)");
        assert_eq!(process("#offset 42\n#offset 42"), "SyntaxError(2)");
    }
    
    #[test]
    fn test_10003_junk_that_parses_ok() {
        // The parser has no validation to reject these junk instructions.
        // Validation takes place in a later stage.
        assert_eq!(process("mov 3,1"), "mov 3,1");
        assert_eq!(process("mov 3,$1"), "mov 3,$1");
        assert_eq!(process("mov 3,$$1"), "mov 3,$$1");
        assert_eq!(process("mov 0,0"), "mov 0,0");
        assert_eq!(process("mov -3,1"), "mov -3,1");
        assert_eq!(process("seq $3,-100"), "seq $3,-100");
        assert_eq!(process("seq $3,$$3"), "seq $3,$$3");
        assert_eq!(process("div $33,0"), "div $33,0");
    }

    #[test]
    fn test_10004_direct_dependencies() {
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
    fn test_10005_instruction_ids() {
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

    #[test]
    fn test_10006_contain_parameter_type_indirect() {
        {
            let parsed_program: ParsedProgram = ParsedProgram::parse_program(
                "seq $1,40 ; fibonacci\nseq $2,40; fib again!\nseq $3,10\nseq $4,45").unwrap();
            assert_eq!(parsed_program.contain_parameter_type_indirect(), false);
        }
        {
            let parsed_program: ParsedProgram = ParsedProgram::parse_program("mov $$1,5").unwrap();
            assert_eq!(parsed_program.contain_parameter_type_indirect(), true);
        }
        {
            let parsed_program: ParsedProgram = ParsedProgram::parse_program("mov $1,$$1").unwrap();
            assert_eq!(parsed_program.contain_parameter_type_indirect(), true);
        }
    }

    #[test]
    fn test_10007_assign_zero_line_numbers() {
        // Arrange
        let mut parsed_program: ParsedProgram = ParsedProgram::parse_program(
            ";\n\nseq $1,40 ; ignore me\nseq $2,40; and ignore\nseq $3,10\nseq $4,45").unwrap();
        let mut sum0 = 0;
        for instruction in &parsed_program.instruction_vec {
            sum0 += instruction.line_number;
        }
        assert_ne!(sum0, 0);

        // Act
        parsed_program.assign_zero_line_numbers();

        // Assert
        let mut sum1 = 0;
        for instruction in &parsed_program.instruction_vec {
            sum1 += instruction.line_number;
        }
        assert_eq!(sum1, 0);
    }

    #[test]
    fn test_10008_equal_yes() {
        let mut parsed_program0: ParsedProgram = ParsedProgram::parse_program(
            "; junk\n\nseq $1,40 ; fibonacci\nseq $2,40; fib again!\nseq $3,10\nseq $4,45").unwrap();
        let mut parsed_program1: ParsedProgram = ParsedProgram::parse_program(
            "seq $1,40 ; ignore me\nseq $2,40; and ignore\nseq $3,10\nseq $4,45").unwrap();
        assert_ne!(parsed_program0, parsed_program1);
        parsed_program0.assign_zero_line_numbers();
        parsed_program1.assign_zero_line_numbers();
        assert_eq!(parsed_program0, parsed_program1);
    }

    #[test]
    fn test_10009_equal_no() {
        let mut parsed_program0: ParsedProgram = ParsedProgram::parse_program(
            "seq $0,40\nseq $0,45").unwrap();
        let mut parsed_program1: ParsedProgram = ParsedProgram::parse_program(
            "seq $0,45\nseq $0,40").unwrap();
        assert_ne!(parsed_program0, parsed_program1);
        parsed_program0.assign_zero_line_numbers();
        parsed_program1.assign_zero_line_numbers();
        assert_ne!(parsed_program0, parsed_program1);
    }
}
