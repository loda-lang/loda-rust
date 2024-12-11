use super::Word;
use loda_rust_core::parser::ParsedProgram;

pub trait WordsFromProgram {
    fn as_words(&self) -> Vec<Word>;
}

impl WordsFromProgram for ParsedProgram {
    fn as_words(&self) -> Vec<Word> {
        let mut words: Vec<Word> = self.instruction_ids().iter().map(|instruction_id| {
            Word::Instruction(*instruction_id)
        }).collect();
        words.insert(0, Word::ProgramStart);
        words.push(Word::ProgramStop);
        words
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loda_rust_core::parser::InstructionId;

    fn process(input: &str) -> Vec<Word> {
        let result = ParsedProgram::parse_program(input);
        let parsed_program: ParsedProgram = match result {
            Ok(value) => value,
            Err(_) => {
                return vec!();
            }
        };
        parsed_program.as_words()
    }

    #[test]
    fn test_10000_as_words() {
        assert_eq!(process(""), vec![Word::ProgramStart, Word::ProgramStop]);
        assert_eq!(process("; comment"), vec![Word::ProgramStart, Word::ProgramStop]);
        assert_eq!(process("mul $0,2"), vec![Word::ProgramStart, Word::Instruction(InstructionId::Multiply), Word::ProgramStop]);
        assert_eq!(process("mul $0,2\nadd $0,1"), vec![Word::ProgramStart, Word::Instruction(InstructionId::Multiply), Word::Instruction(InstructionId::Add), Word::ProgramStop]);
    }
}
