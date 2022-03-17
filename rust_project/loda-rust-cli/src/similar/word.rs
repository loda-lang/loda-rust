use loda_rust_core::parser::InstructionId;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Word {
    Start,
    Stop,
    Instruction(InstructionId)
}

impl Word {
    pub fn parse(raw: &str) -> Option<Word> {
        match raw {
            "START" => {
                return Some(Word::Start);
            },
            "STOP" => {
                return Some(Word::Stop);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> String {
        let word: Word = match Word::parse(input) {
            Some(value) => value,
            None => return "NONE".to_string()
        };
        match word {
            Word::Start => return "Start".to_string(),
            Word::Stop => return "Stop".to_string(),
            Word::Instruction(instruction) => {
                return instruction.shortname().to_string();
            }
        }
    }

    #[test]
    fn test_10000_parse_success() {
        assert_eq!(parse("START"), "Start");
        assert_eq!(parse("STOP"), "Stop");
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
