use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{InstructionId, ParsedProgram};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;
use csv::WriterBuilder;
use serde::Serialize;
use super::{BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};

const IGNORE_ANY_PROGRAM_SHORTER_THAN: usize = 3;
const IGNORE_PROGRAM_WITHOUT_LOOPS_SHORTER_THAN: usize = 10;
const IGNORE_PROGRAM_WITHOUT_NESTED_SEQ_SHORTER_THAN: usize = 10;
const CONSIDER_ANY_PROGRAM_LONGER_THAN: usize = 60;

enum ProgramComplexityClassification {
    SimpleAndShort,
    SimpleWithoutLoops,
    MediumWithLoops,
    ComplexTwoOrMoreSeq,
    ComplexNestedSeq,
    ComplexAndLong,
    ComplexOtherReasons,
}

impl ProgramComplexityClassification {
    fn classification(&self) -> String {
        match self {
            ProgramComplexityClassification::SimpleAndShort => "0".to_string(),
            ProgramComplexityClassification::SimpleWithoutLoops => "0".to_string(),
            ProgramComplexityClassification::MediumWithLoops => "0".to_string(),
            ProgramComplexityClassification::ComplexTwoOrMoreSeq => "1".to_string(),
            ProgramComplexityClassification::ComplexNestedSeq => "1".to_string(),
            ProgramComplexityClassification::ComplexAndLong => "1".to_string(),
            ProgramComplexityClassification::ComplexOtherReasons => "1".to_string(),
        }
    }

    fn comment(&self) -> String {
        match self {
            ProgramComplexityClassification::SimpleAndShort => "very short program, low chance it can be optimized further".to_string(),
            ProgramComplexityClassification::SimpleWithoutLoops => "short program without loops, low chance it can be optimized further".to_string(),
            ProgramComplexityClassification::MediumWithLoops => "simple loops without eval seq, medium chance it can be optimized".to_string(),
            ProgramComplexityClassification::ComplexTwoOrMoreSeq => "two or more eval seq, high chance it can be optimized".to_string(),
            ProgramComplexityClassification::ComplexNestedSeq => "eval seq inside loop, high chance it can be optimized".to_string(),
            ProgramComplexityClassification::ComplexAndLong => "long program, high chance that it can be optimized".to_string(),
            ProgramComplexityClassification::ComplexOtherReasons => "complex for other reasons, high chance it can be optimized".to_string(),
        }
    }
}

pub struct AnalyzeProgramComplexity {
    config: Config,
    classifications: HashMap<u32, ProgramComplexityClassification>,
}

impl AnalyzeProgramComplexity {
    pub fn new() -> Self {
        Self {
            config: Config::load(),
            classifications: HashMap::new(),
        }
    }

    fn classify(parsed_program: &ParsedProgram) -> ProgramComplexityClassification {
        let number_of_instructions: usize = parsed_program.instruction_vec.len();
        if number_of_instructions > CONSIDER_ANY_PROGRAM_LONGER_THAN {
            return ProgramComplexityClassification::ComplexAndLong;
        }
        if parsed_program.has_two_or_more_seq() {
            return ProgramComplexityClassification::ComplexTwoOrMoreSeq;            
        }
        if number_of_instructions < IGNORE_ANY_PROGRAM_SHORTER_THAN {
            return ProgramComplexityClassification::SimpleAndShort;
        }
        let has_loops: bool = parsed_program.has_one_or_more_loops();
        if !has_loops {
            if number_of_instructions < IGNORE_PROGRAM_WITHOUT_LOOPS_SHORTER_THAN {
                return ProgramComplexityClassification::SimpleWithoutLoops;
            }
        }
        let has_seq_inside_loops: bool = parsed_program.has_one_or_more_seq_inside_loops();
        if has_seq_inside_loops {
            return ProgramComplexityClassification::ComplexNestedSeq;
        } else {
            if number_of_instructions < IGNORE_PROGRAM_WITHOUT_NESTED_SEQ_SHORTER_THAN {
                return ProgramComplexityClassification::MediumWithLoops;
            }
        }
        return ProgramComplexityClassification::ComplexOtherReasons;
    }

    fn save_inner(&self) {
        // Convert from dictionary to array
        let mut records = Vec::<RecordProgram>::new();
        for (key, value) in &self.classifications {
            let record = RecordProgram {
                program_id: *key,
                classification: value.classification(),
                comment: value.comment()
            };
            records.push(record);
        }

        // Move the lowest program ids to the top
        // Move the highest program ids to the bottom
        records.sort_unstable_by_key(|item| (item.program_id));

        // Save as a CSV file
        let output_path: PathBuf = self.config.cache_dir_complexity_file();
        match Self::create_csv_file(&records, &output_path) {
            Ok(_) => {
                println!("saved complexity.csv");
            },
            Err(error) => {
                println!("cannot save complexity.csv error: {:?}", error);
            }
        }
    }

    fn create_csv_file<S: Serialize>(records: &Vec<S>, output_path: &Path) -> Result<(), Box<dyn Error>> {
        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_path(output_path)?;
        for record in records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeProgramComplexity {
    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> bool {
        let classification = Self::classify(&context.parsed_program);
        self.classifications.insert(context.program_id, classification);
        true
    }

    fn save(&self) {
        self.save_inner();
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

trait HasTwoOrMoreSeq {
    fn has_two_or_more_seq(&self) -> bool;
}

impl HasTwoOrMoreSeq for ParsedProgram {
    fn has_two_or_more_seq(&self) -> bool {
        let mut count: usize = 0;
        for instruction in &self.instruction_vec {
            if instruction.instruction_id == InstructionId::EvalSequence {
                count += 1;
            }
        }
        count >= 2
    }
}

#[derive(Serialize)]
struct RecordProgram {
    #[serde(rename = "program id")]
    program_id: u32,
    classification: String,
    comment: String,
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

    fn has_two_or_more_seq(input0: &str) -> String {
        let result = ParsedProgram::parse_program(input0);
        let parsed_program: ParsedProgram = match result {
            Ok(value) => value,
            Err(error) => {
                return format!("BOOM: {:?}", error);
            }
        };
        parsed_program.has_two_or_more_seq().to_string()
    }

    #[test]
    fn test_30000_has_two_or_more_seq() {
        assert_eq!(has_two_or_more_seq(""), "false");
        assert_eq!(has_two_or_more_seq("; comment\nmul $0,1\n\n; comment"), "false");
        assert_eq!(has_two_or_more_seq("lpb $0\nsub $0,1\nlpe"), "false");
        assert_eq!(has_two_or_more_seq("lpb $0\nseq $0,40\nlpe"), "false");
        assert_eq!(has_two_or_more_seq("lpb $0\nmov $1,$0\nlpb $1\nsub $1,1\nlpe\nsub $0,1\nlpe"), "false");
        assert_eq!(has_two_or_more_seq("lpb $0\nmov $1,$0\nlpb $1\nseq $1,40\nlpe\nsub $0,1\nlpe"), "false");
        assert_eq!(has_two_or_more_seq("seq $0,40"), "false");
        assert_eq!(has_two_or_more_seq("seq $0,40\nseq $0,40"), "true");
        assert_eq!(has_two_or_more_seq("seq $0,40\nmul $0,100\nseq $0,40"), "true");
    }
}
