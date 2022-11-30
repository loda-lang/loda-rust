use super::InstructionId;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ParseInstructionIdError {
    UnrecognizedInstructionId(usize),
}

impl fmt::Display for ParseInstructionIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::UnrecognizedInstructionId(line_number) => 
                write!(f, "Unrecognized instruction in line {}", line_number),
        }
    }
}

pub trait ParseInstructionId {
    fn parse(input: &str, line_number: usize) -> Result<InstructionId,ParseInstructionIdError>;
}

impl ParseInstructionId for InstructionId {
    fn parse(input: &str, line_number: usize) -> Result<InstructionId,ParseInstructionIdError> {
        match input {
            "add" => Ok(Self::Add),
            "bin" => Ok(Self::Binomial),
            "clr" => Ok(Self::Clear),
            "cmp" => Ok(Self::Compare),
            "dif" => Ok(Self::DivideIf),
            "div" => Ok(Self::Divide),
            "gcd" => Ok(Self::GCD),
            "lpb" => Ok(Self::LoopBegin),
            "lpe" => Ok(Self::LoopEnd),
            "max" => Ok(Self::Max),
            "min" => Ok(Self::Min),
            "mod" => Ok(Self::Modulo),
            "mov" => Ok(Self::Move),
            "mul" => Ok(Self::Multiply),
            "pow" => Ok(Self::Power),
            "seq" => Ok(Self::EvalSequence),
            "sub" => Ok(Self::Subtract),
            "trn" => Ok(Self::Truncate),
            _     => Err(ParseInstructionIdError::UnrecognizedInstructionId(line_number)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_10000_parse_ok() {
        {
            let instruction_id: InstructionId = InstructionId::parse("add", 1).unwrap();
            assert_eq!(instruction_id, InstructionId::Add);
        }
        {
            let instruction_id: InstructionId = InstructionId::parse("seq", 1).unwrap();
            assert_eq!(instruction_id, InstructionId::EvalSequence);
        }
    }

    #[test]
    fn test_10001_parse_error_with_line_number() {
        let err: ParseInstructionIdError = InstructionId::parse("nonexisting", 666).unwrap_err();
        let line_number: usize;
        match err {
            ParseInstructionIdError::UnrecognizedInstructionId(the_line_number) => {
                line_number = the_line_number;
            }
        }
        assert_eq!(line_number, 666);
    }

    #[test]
    fn test_10002_parse_error() {
        // Instructions must be lowercase. 
        InstructionId::parse("Add", 1).expect_err("should fail");
        InstructionId::parse("ADD", 1).expect_err("should fail");

        // No weird prefix/suffix allowed 
        InstructionId::parse("add_", 1).expect_err("should fail");
        InstructionId::parse("_add", 1).expect_err("should fail");
        InstructionId::parse("addd", 1).expect_err("should fail");
    }
}
