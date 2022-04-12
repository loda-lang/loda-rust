use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{Instruction, InstructionId, InstructionParameter, ParsedProgram};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;
use std::collections::HashSet;
use csv::WriterBuilder;
use serde::Serialize;
use super::{BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};

const IGNORE_ANY_PROGRAM_SHORTER_THAN: usize = 5;
const IGNORE_PROGRAM_WITHOUT_LOOPS_SHORTER_THAN: usize = 10;

enum ProgramComplexityClassification {
    Simple,
    Complex,
}

pub struct AnalyzeProgramComplexity {
    simple_programs: HashSet<u32>,
    complex_programs: HashSet<u32>,
}

impl AnalyzeProgramComplexity {
    pub fn new() -> Self {
        Self {
            simple_programs: HashSet::new(),
            complex_programs: HashSet::new(),
        }
    }

    fn classify(parsed_program: &ParsedProgram) -> ProgramComplexityClassification {
        if parsed_program.instruction_vec.len() < IGNORE_ANY_PROGRAM_SHORTER_THAN {
            return ProgramComplexityClassification::Simple;
        }
        let has_loops: bool = parsed_program.has_one_or_more_loops();
        if !has_loops {
            if parsed_program.instruction_vec.len() < IGNORE_PROGRAM_WITHOUT_LOOPS_SHORTER_THAN {
                return ProgramComplexityClassification::Simple;
            }
        }
        let has_seq_inside_loops: bool = parsed_program.has_one_or_more_seq_inside_loops();
        if has_seq_inside_loops {
            return ProgramComplexityClassification::Complex;
        }

        return ProgramComplexityClassification::Complex;
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeProgramComplexity {
    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> bool {
        match Self::classify(&context.parsed_program) {
            ProgramComplexityClassification::Complex => {
                self.complex_programs.insert(context.program_id);
            },
            ProgramComplexityClassification::Simple => {
                self.simple_programs.insert(context.program_id);
            },
        }
        true
    }

    fn save(&self) {
    }
}

trait HasOneOrMoreLoops {
    fn has_one_or_more_loops(&self) -> bool;
}

impl HasOneOrMoreLoops for ParsedProgram {
    fn has_one_or_more_loops(&self) -> bool {
        for instruction in &self.instruction_vec {
            if instruction.instruction_id == InstructionId::LoopBegin {
                return true;
            }
            if instruction.instruction_id == InstructionId::LoopEnd {
                return true;
            }
        }
        false
    }
}

trait HasOneOrMoreSeqInsideLoops {
    fn has_one_or_more_seq_inside_loops(&self) -> bool;
}

impl HasOneOrMoreSeqInsideLoops for ParsedProgram {
    fn has_one_or_more_seq_inside_loops(&self) -> bool {
        let mut depth: i32 = 0;
        for instruction in &self.instruction_vec {
            match instruction.instruction_id {
                InstructionId::LoopBegin => {
                    depth += 1;
                },
                InstructionId::LoopEnd => {
                    depth -= 1;
                },
                InstructionId::EvalSequence => {
                    if depth > 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_one_or_more_loops(input0: &str) -> String {
        let result = ParsedProgram::parse_program(input0);
        let parsed_program: ParsedProgram = match result {
            Ok(value) => value,
            Err(error) => {
                return format!("BOOM: {:?}", error);
            }
        };
        parsed_program.has_one_or_more_loops().to_string()
    }

    #[test]
    fn test_10000_has_one_or_more_loops() {
        assert_eq!(has_one_or_more_loops(""), "false");
        assert_eq!(has_one_or_more_loops("; comment\nmul $0,1\n\n; comment"), "false");
        assert_eq!(has_one_or_more_loops("mul $0,7\nlpb $0\ndiv $0,3\nadd $0,10\nlpe"), "true");
        assert_eq!(has_one_or_more_loops("lpb $0\nlpb $1\nlpe\nlpe"), "true");
        assert_eq!(has_one_or_more_loops("; junk\nlpe\n; junk"), "true");
    }

    fn has_one_or_more_seq_inside_loops(input0: &str) -> String {
        let result = ParsedProgram::parse_program(input0);
        let parsed_program: ParsedProgram = match result {
            Ok(value) => value,
            Err(error) => {
                return format!("BOOM: {:?}", error);
            }
        };
        parsed_program.has_one_or_more_seq_inside_loops().to_string()
    }

    #[test]
    fn test_20000_has_one_or_more_seq_inside_loops() {
        assert_eq!(has_one_or_more_seq_inside_loops(""), "false");
        assert_eq!(has_one_or_more_seq_inside_loops("; comment\nmul $0,1\n\n; comment"), "false");
        assert_eq!(has_one_or_more_seq_inside_loops("lpb $0\nsub $0,1\nlpe"), "false");
        assert_eq!(has_one_or_more_seq_inside_loops("lpb $0\nseq $0,40\nlpe"), "true");
        assert_eq!(has_one_or_more_seq_inside_loops("lpb $0\nmov $1,$0\nlpb $1\nsub $1,1\nlpe\nsub $0,1\nlpe"), "false");
        assert_eq!(has_one_or_more_seq_inside_loops("lpb $0\nmov $1,$0\nlpb $1\nseq $1,40\nlpe\nsub $0,1\nlpe"), "true");
    }
}
