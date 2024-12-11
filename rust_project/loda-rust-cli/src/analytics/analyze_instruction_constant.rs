use super::{AnalyticsDirectory, BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};
use crate::common::create_csv_file;
use loda_rust_core;
use loda_rust_core::parser::{Instruction, InstructionId, InstructionParameter, ParameterType, ParsedProgram};
use std::path::PathBuf;
use std::error::Error;
use std::collections::HashMap;
use serde::Serialize;

type HistogramKey = (InstructionId,i32);

static DISCARD_EXTREME_VALUES_BEYOND_THIS_LIMIT: i64 = 10000;

/// Creates a histogram about what constants goes with each instruction.
///
/// The most used combo: `add $0,1` (addition by 1) and is used 38175 times.
/// 
/// Almost as popular combo: `sub $0,1` (subtract by 1) and is used 35075 times.
/// 
/// Less popular is the combo: `mul $0,-1` (multiply by -1) and is used 429 times.
/// 
/// The majority of constants are only used a single time.
///
/// Traverses all the programs inside the `loda-programs/oeis` dir.
///
/// There are programs with unwanted huge magic constants.
/// So too extreme values gets ignored.
/// 
/// This outputs a `histogram_instruction_constant.csv` file, with this format:
/// 
/// ```csv
/// count;instruction;constant
/// 532;add;1
/// 531;sub;1
/// 308;mov;1
/// 252;mul;2
/// 167;div;2
/// 137;mov;2
/// 121;add;2
/// 98;pow;2
/// 78;cmp;0
/// 69;bin;2
/// ```
pub struct AnalyzeInstructionConstant {
    analytics_directory: AnalyticsDirectory,
    histogram: HashMap<HistogramKey,u32>,
    number_of_constant_processed_unsuccessful: u32,
    number_of_constant_processed_successful: u32,
}

impl AnalyzeInstructionConstant {
    pub fn new(analytics_directory: AnalyticsDirectory) -> Self {
        Self {
            analytics_directory,
            histogram: HashMap::new(),
            number_of_constant_processed_unsuccessful: 0,
            number_of_constant_processed_successful: 0,
        }
    }

    fn analyze_inner(&mut self, program_id: u32, parsed_program: &ParsedProgram) {
        for instruction in &parsed_program.instruction_vec {
            if instruction.instruction_id == InstructionId::EvalSequence {
                continue;
            }
            if instruction.instruction_id == InstructionId::LoopBegin {
                continue;
            }
            if instruction.instruction_id == InstructionId::LoopEnd {
                continue;
            }
            if instruction.parameter_vec.len() != 2 {
                continue;
            }
            let source_parameter: &InstructionParameter = instruction.parameter_vec.last().unwrap();
            if source_parameter.parameter_type != ParameterType::Constant {
                continue;
            }
            let value: i64 = source_parameter.parameter_value;
            let success: bool = self.analyze_instruction_and_constant(program_id, &instruction, value);
            if success {
                self.number_of_constant_processed_successful += 1;
            } else {
                self.number_of_constant_processed_unsuccessful += 1;
            }
        }
    }

    fn analyze_instruction_and_constant(&mut self, program_id: u32, instruction: &Instruction, raw_value: i64) -> bool {
        if raw_value.abs() > DISCARD_EXTREME_VALUES_BEYOND_THIS_LIMIT {
            debug!("program_id: {:?}, Ignoring too extreme constant: {:?}", program_id, raw_value);
            return false;
        }
        let value: i32 = raw_value as i32;
        if instruction.instruction_id == InstructionId::Add && value == 0 {
            debug!("program_id: {:?}, add by 0, can be eliminated", program_id);
            return false;
        }
        if instruction.instruction_id == InstructionId::Subtract && value == 0 {
            debug!("program_id: {:?}, subtract by 0, can be eliminated", program_id);
            return false;
        }
        if instruction.instruction_id == InstructionId::Multiply && value == 1 {
            debug!("program_id: {:?}, multiply by 1, can be eliminated", program_id);
            return false;
        }
        if instruction.instruction_id == InstructionId::Divide && value == 1 {
            debug!("program_id: {:?}, divide by 1, can be eliminated", program_id);
            return false;
        }
        if instruction.instruction_id == InstructionId::Divide && value == 0 {
            debug!("program_id: {:?}, detected a dangerous divide by 0", program_id);
            return false;
        }
        let key: HistogramKey = (instruction.instruction_id, value);
        let counter = self.histogram.entry(key).or_insert(0);
        *counter += 1;
        true
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeInstructionConstant {
    fn plugin_name(&self) -> &'static str {
        "AnalyzeInstructionConstant"
    }
    
    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> Result<(), Box<dyn Error>> {
        self.analyze_inner(context.program_id, &context.parsed_program);
        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn Error>> {
        let records: Vec<Record> = Record::sorted_records_from_histogram(&self.histogram);

        // Save as a CSV file
        let output_path: PathBuf = self.analytics_directory.histogram_instruction_constant_file();
        create_csv_file(&records, &output_path)
    }

    fn human_readable_summary(&self) -> String {
        let rows: Vec<String> = vec![
            format!("number of constants processed unsuccessful: {:?}", self.number_of_constant_processed_unsuccessful),
            format!("number of constants processed successful: {:?}", self.number_of_constant_processed_successful),
            format!("number of items in histogram: {:?}", self.histogram.len())
        ];
        rows.join("\n")
    }
}

#[derive(Debug, Serialize, PartialEq)]
struct Record {
    count: u32,
    instruction: String,
    constant: i32,
}

impl Record {
    fn sorted_records_from_histogram(histogram: &HashMap<HistogramKey,u32>) -> Vec<Record> {
        // Convert from dictionary to array
        let mut records = Vec::<Record>::new();
        for (histogram_key, histogram_count) in histogram {
            let instruction_name: String = histogram_key.0.to_string();
            let record = Record {
                count: *histogram_count,
                instruction: instruction_name,
                constant: histogram_key.1
            };
            records.push(record);
        }

        // Move the most frequently occuring items to the top
        // Move the lesser used items to the bottom
        records.sort_unstable_by_key(|item| (item.count, item.instruction.clone(), item.constant));
        records.reverse();

        records
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_sorted_records_from_histogram() {
        // Arrange
        let mut histogram: HashMap<HistogramKey,u32> = HashMap::new();
        let key: HistogramKey = (InstructionId::Add, 42);
        histogram.insert(key, 1);
        let key: HistogramKey = (InstructionId::Multiply, 1337);
        histogram.insert(key, 3);
        let key: HistogramKey = (InstructionId::Subtract, 666);
        histogram.insert(key, 2);

        // Act
        let records: Vec<Record> = Record::sorted_records_from_histogram(&histogram);

        // Assert
        let expected: Vec<Record> = vec![
            Record { count: 3, instruction: "mul".to_string(), constant: 1337 },
            Record { count: 2, instruction: "sub".to_string(), constant: 666 },
            Record { count: 1, instruction: "add".to_string(), constant: 42 },
        ];
        assert_eq!(records, expected);
    }
}
