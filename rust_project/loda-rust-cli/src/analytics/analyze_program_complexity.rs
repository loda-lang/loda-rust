use super::{AnalyticsDirectory, BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};
use crate::common::{create_csv_file, save_program_ids_csv_file};
use loda_rust_core;
use loda_rust_core::parser::{InstructionId, ParsedProgram};
use std::path::PathBuf;
use std::error::Error;
use std::collections::HashMap;
use serde::Serialize;

const IGNORE_ANY_PROGRAM_SHORTER_THAN: usize = 8;
const IGNORE_PROGRAM_WITHOUT_LOOPS_SHORTER_THAN: usize = 13;
const IGNORE_PROGRAM_WITHOUT_NESTED_SEQ_SHORTER_THAN: usize = 9;
const CONSIDER_ANY_PROGRAM_LONGER_THAN: usize = 60;
const ONE_SEQ_AND_NUMBER_OF_LINES_OF_OTHER_STUFF: usize = 10;

enum ProgramComplexityClassification {
    SimpleAndShort,
    SimpleWithoutLoops,
    MediumWithLoops,
    ComplexOneSeqAndOtherStuff,
    ComplexTwoOrMoreSeq,
    ComplexNestedSeq,
    ComplexAndLong,
    ComplexOtherReasons,
}

impl ProgramComplexityClassification {
    fn is_optimizable(&self) -> bool {
        match self {
            ProgramComplexityClassification::SimpleAndShort => false,
            ProgramComplexityClassification::SimpleWithoutLoops => false,
            ProgramComplexityClassification::MediumWithLoops => false,
            ProgramComplexityClassification::ComplexOneSeqAndOtherStuff => true,
            ProgramComplexityClassification::ComplexTwoOrMoreSeq => true,
            ProgramComplexityClassification::ComplexNestedSeq => true,
            ProgramComplexityClassification::ComplexAndLong => true,
            ProgramComplexityClassification::ComplexOtherReasons => false,
        }
    }

    fn optimizable_string(&self) -> String {
        if self.is_optimizable() {
            return "1".to_string();
        } else {
            return "0".to_string();
        }
    }

    fn comment(&self) -> String {
        match self {
            ProgramComplexityClassification::SimpleAndShort => "very short program, low chance it can be optimized further".to_string(),
            ProgramComplexityClassification::SimpleWithoutLoops => "short program without loops, low chance it can be optimized further".to_string(),
            ProgramComplexityClassification::MediumWithLoops => "simple loops without eval seq, medium chance it can be optimized".to_string(),
            ProgramComplexityClassification::ComplexOneSeqAndOtherStuff => "one seq and other stuff, high chance it can be optimized".to_string(),
            ProgramComplexityClassification::ComplexTwoOrMoreSeq => "two or more eval seq, high chance it can be optimized".to_string(),
            ProgramComplexityClassification::ComplexNestedSeq => "eval seq inside loop, high chance it can be optimized".to_string(),
            ProgramComplexityClassification::ComplexAndLong => "long program, high chance that it can be optimized".to_string(),
            ProgramComplexityClassification::ComplexOtherReasons => "complex for other reasons, high chance it can be optimized".to_string(),
        }
    }
}

pub struct AnalyzeProgramComplexity {
    analytics_directory: AnalyticsDirectory,
    classifications: HashMap<u32, ProgramComplexityClassification>,
}

impl AnalyzeProgramComplexity {
    pub fn new(analytics_directory: AnalyticsDirectory) -> Self {
        Self {
            analytics_directory,
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
        if parsed_program.has_one_seq_and_other_stuff() {
            return ProgramComplexityClassification::ComplexOneSeqAndOtherStuff;
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

    fn save_all(&self) -> Result<(), Box<dyn Error>> {
        // Convert from dictionary to array
        let mut records = Vec::<RecordProgram>::new();
        for (key, value) in &self.classifications {
            let record = RecordProgram {
                program_id: *key,
                optimizable: value.optimizable_string(),
                comment: value.comment()
            };
            records.push(record);
        }

        // Move the lowest program ids to the top
        // Move the highest program ids to the bottom
        records.sort_unstable_by_key(|item| (item.program_id));

        // Save as a CSV file
        let output_path: PathBuf = self.analytics_directory.complexity_all_file();
        create_csv_file(&records, &output_path)
    }

    /// Extract program ids of those programs that has little/no chance of being optimized
    fn extract_dont_optimize_program_ids(&self) -> Vec<u32> {
        let mut program_ids = Vec::<u32>::new();
        for (key, value) in &self.classifications {
            if !value.is_optimizable() {
                program_ids.push(*key);
            }
        }
        program_ids.sort();
        program_ids
    }

    fn save_dont_optimize(&self) -> Result<(), Box<dyn Error>> {
        let program_ids = self.extract_dont_optimize_program_ids();
        let output_path: PathBuf = self.analytics_directory.complexity_dont_optimize_file();
        save_program_ids_csv_file(&program_ids, &output_path)
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeProgramComplexity {
    fn plugin_name(&self) -> &'static str {
        "AnalyzeProgramComplexity"
    }
    
    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> Result<(), Box<dyn Error>> {
        let classification = Self::classify(&context.parsed_program);
        self.classifications.insert(context.program_id, classification);
        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn Error>> {
        self.save_all()?;
        self.save_dont_optimize()?;
        Ok(())
    }

    fn human_readable_summary(&self) -> String {
        let program_ids = self.extract_dont_optimize_program_ids();
        let dont_optimize_count = program_ids.len();
        let total_count = self.classifications.len();
        let mut optimize_count: usize = 0;
        if total_count > dont_optimize_count {
            optimize_count = total_count - dont_optimize_count;
        }
        let ratio = ((optimize_count * 100) as f32) / (total_count.max(1) as f32);
        format!("optimize: {}, dontoptimize: {}, optimize/total: {:.1}%", optimize_count, dont_optimize_count, ratio)
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

trait HasOneSeqAndOtherStuff {
    fn has_one_seq_and_other_stuff(&self) -> bool;
}

impl HasOneSeqAndOtherStuff for ParsedProgram {
    fn has_one_seq_and_other_stuff(&self) -> bool {
        let mut count_seq: usize = 0;
        let mut count_other: usize = 0;
        for instruction in &self.instruction_vec {
            if instruction.instruction_id == InstructionId::EvalSequence {
                count_seq += 1;
            } else {
                count_other += 1;
            }
        }
        (count_seq >= 1) && (count_other >= ONE_SEQ_AND_NUMBER_OF_LINES_OF_OTHER_STUFF)
    }
}

#[derive(Serialize)]
struct RecordProgram {
    #[serde(rename = "program id")]
    program_id: u32,
    #[serde(rename = "is optimizable")]
    optimizable: String,
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

    fn has_one_seq_and_other_stuff(input0: &str) -> String {
        let result = ParsedProgram::parse_program(input0);
        let parsed_program: ParsedProgram = match result {
            Ok(value) => value,
            Err(error) => {
                return format!("BOOM: {:?}", error);
            }
        };
        parsed_program.has_one_seq_and_other_stuff().to_string()
    }

    #[test]
    fn test_40000_has_one_seq_and_other_stuff() {
        assert_eq!(has_one_seq_and_other_stuff("add $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1"), "false");
        assert_eq!(has_one_seq_and_other_stuff("seq $0,40\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1"), "false");
        assert_eq!(has_one_seq_and_other_stuff("seq $0,40\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1\nadd $0,1"), "true");
    }
}
