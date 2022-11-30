use loda_rust_core::parser::{InstructionId, ParseInstructionId};
use std::fmt;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Word {
    ProgramStart,
    ProgramStop,
    Instruction(InstructionId)
}

impl Word {
    pub fn parse(raw: &str) -> Option<Word> {
        match raw {
            "START" => {
                return Some(Word::ProgramStart);
            },
            "STOP" => {
                return Some(Word::ProgramStop);
            },
            _ => {}
        }
        match InstructionId::parse(raw, 1) {
            Ok(instruction_id) => {
                return Some(Word::Instruction(instruction_id));
            },
            Err(_) => {
                return None;
            }
        }
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ProgramStart => write!(f, "START"),
            Self::ProgramStop => write!(f, "STOP"),
            Self::Instruction(instruction) => write!(f, "{}", instruction)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> String {
        match Word::parse(input) {
            Some(word) => word.to_string(),
            None => "NONE".to_string()
        }
    }

    #[test]
    fn test_10000_parse_success() {
        assert_eq!(parse("START"), "START");
        assert_eq!(parse("STOP"), "STOP");
        assert_eq!(parse("mov"), "mov");
        assert_eq!(parse("add"), "add");
    }

    #[test]
    fn test_10001_parse_unrecognized() {
        assert_eq!(parse(""), "NONE");
        assert_eq!(parse("unrecognized"), "NONE");
        assert_eq!(parse("junk"), "NONE");
        assert_eq!(parse("start"), "NONE");
        assert_eq!(parse("stop"), "NONE");
        assert_eq!(parse("MOV"), "NONE");
        assert_eq!(parse("ADD"), "NONE");
    }
}
