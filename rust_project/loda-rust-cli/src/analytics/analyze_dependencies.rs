use crate::common::create_csv_file;
use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{InstructionId, InstructionParameter, ParameterType, ParsedProgram};
use std::path::PathBuf;
use std::error::Error;
use serde::Serialize;
use std::convert::TryFrom;
use super::{BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};

pub struct AnalyzeDependencies {
    config: Config,
    dependencies: Vec<RecordDependency>,
}

impl AnalyzeDependencies {
    pub fn new() -> Self {
        Self {
            config: Config::load(),
            dependencies: vec!(),
        }
    }

    fn append_dependencies(&mut self, caller_program_id: u32, callee_program_ids: Vec<u32>) {
        for callee_program_id in callee_program_ids {
            let record = RecordDependency {
                caller_program_id: caller_program_id,
                callee_program_id: callee_program_id
            };
            self.dependencies.push(record);
        }
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeDependencies {
    fn plugin_name(&self) -> &'static str {
        "AnalyzeDependencies"
    }

    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> Result<(), Box<dyn Error>> {
        let callee_program_ids: Vec<u32> = context.parsed_program.extract_program_ids();
        self.append_dependencies(context.program_id, callee_program_ids);
        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn Error>> {
        let mut records: Vec<RecordDependency> = self.dependencies.clone();
        records.sort_unstable_by_key(|item| (item.caller_program_id, item.callee_program_id));

        // Save as a CSV file
        let output_path: PathBuf = self.config.analytics_dir_dependencies_file();
        create_csv_file(&records, &output_path)
    }

    fn human_readable_summary(&self) -> String {
        format!("number of dependencies: {}", self.dependencies.len())
    }
}


#[derive(Clone, Serialize)]
struct RecordDependency {
    #[serde(rename = "caller program id")]
    caller_program_id: u32,
    #[serde(rename = "callee program id")]
    callee_program_id: u32,
}

trait ExtractProgramIds {
    fn extract_program_ids(&self) -> Vec<u32>;
}

impl ExtractProgramIds for ParsedProgram {
    fn extract_program_ids(&self) -> Vec<u32> {
        let mut program_ids = Vec::<u32>::new();
        for instruction in &self.instruction_vec {
            if instruction.instruction_id != InstructionId::EvalSequence {
                continue;
            }
            if instruction.parameter_vec.len() != 2 {
                continue;
            }
            let parameter1: &InstructionParameter = &instruction.parameter_vec[1];
            if parameter1.parameter_type != ParameterType::Constant {
                continue;
            }
            let parameter_value_raw: i64 = parameter1.parameter_value;
            let program_id: u32 = match u32::try_from(parameter_value_raw).ok() {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            program_ids.push(program_id);
        }
        program_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_program_ids(input0: &str) -> String {
        let result = ParsedProgram::parse_program(input0);
        let parsed_program: ParsedProgram = match result {
            Ok(value) => value,
            Err(error) => {
                return format!("BOOM: {:?}", error);
            }
        };
        let program_ids: Vec<u32> = parsed_program.extract_program_ids();
        if program_ids.is_empty() {
            return "EMPTY".to_string();
        }
        let program_id_strings: Vec<String> = 
            program_ids.iter().map(|program_id| program_id.to_string()).collect();
        program_id_strings.join(",")
    }

    #[test]
    fn test_10000_extract_program_ids() {
        assert_eq!(extract_program_ids("seq $0,40\nseq $0,40"), "40,40");
        assert_eq!(extract_program_ids("seq $0,40\nlpb $0\nseq $0,45\nlpe"), "40,45");
        assert_eq!(extract_program_ids(""), "EMPTY");
        assert_eq!(extract_program_ids("; comment\nmul $0,1\n\n; comment"), "EMPTY");
        assert_eq!(extract_program_ids("seq"), "EMPTY");
        assert_eq!(extract_program_ids("seq $0"), "EMPTY");
        assert_eq!(extract_program_ids("seq $0,$0"), "EMPTY");
        assert_eq!(extract_program_ids("seq $0,40,$1"), "EMPTY");
        assert_eq!(extract_program_ids("seq $0,-40"), "EMPTY");
        assert_eq!(extract_program_ids("seq $0,0"), "0");
    }
}
