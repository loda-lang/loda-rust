use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{Instruction, InstructionId, InstructionParameter, ParameterType, ParsedProgram};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;
use std::fs;
use csv::WriterBuilder;
use serde::Serialize;
use std::time::Instant;
use super::find_asm_files_recursively;
use super::program_id_from_path;

type HistogramKey = (InstructionId,i32);

static DISCARD_EXTREME_VALUES_BEYOND_THIS_LIMIT: i64 = 10000;

// Creates a histogram about what constants goes with each instruction.
//
// The most used combo: `add $0,1` (addition by 1) and is used 38175 times.
// Almost as popular combo: `sub $0,1` (subtract by 1) and is used 35075 times.
// Less popular is the combo: `mul $0,-1` (multiply by -1) and is used 429 times.
// The majority of constants are only used a single time.
//
// Traverses all the programs inside the "loda-programs/oeis" dir.
//
// There are programs with unwanted huge magic constants.
// So too extreme values gets ignored.
// 
// This script outputs a `histogram_instruction_constant.csv` file, with this format:
// 
//     count;instruction;constant
//     532;add;1
//     531;sub;1
//     308;mov;1
//     252;mul;2
//     167;div;2
//     137;mov;2
//     121;add;2
//     98;pow;2
//     78;cmp;0
//     69;bin;2
// 
pub struct HistogramInstructionConstantAnalyzer {
    config: Config,
    histogram: HashMap<HistogramKey,u32>,
    number_of_program_files_that_could_not_be_loaded: u32,
    number_of_constant_processed_unsuccessful: u32,
    number_of_constant_processed_successful: u32,
}

impl HistogramInstructionConstantAnalyzer {
    pub fn run() {
        let mut instance = Self {
            config: Config::load(),
            histogram: HashMap::new(),
            number_of_program_files_that_could_not_be_loaded: 0,
            number_of_constant_processed_unsuccessful: 0,
            number_of_constant_processed_successful: 0,
        };
        instance.analyze_all_program_files();
        instance.save();
    }

    fn analyze_all_program_files(&mut self) {
        let dir_containing_programs: PathBuf = self.config.loda_programs_oeis_dir();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_programs);
        let number_of_paths = paths.len();
        if number_of_paths <= 0 {
            error!("Expected 1 or more programs, but there are no programs to analyze");
            return;
        }
        let max_index: usize = number_of_paths - 1;
        println!("number of programs to be analyzed: {:?}", paths.len());
        let mut progress_time = Instant::now();
        for (index, path) in paths.iter().enumerate() {
            let elapsed: u128 = progress_time.elapsed().as_millis();
            let is_last: bool = index == max_index;
            if elapsed >= 1000 || is_last {
                let percent: usize = (index * 100) / max_index;
                println!("progress: {}%  {} of {}", percent, index + 1, number_of_paths);
                progress_time = Instant::now();
            }
            self.analyze_program_file(&path);
        }
        println!("number of program files that could not be loaded: {:?}", self.number_of_program_files_that_could_not_be_loaded);
        println!("number of constants processed unsuccessful: {:?}", self.number_of_constant_processed_unsuccessful);
        println!("number of constants processed successful: {:?}", self.number_of_constant_processed_successful);
        println!("number of items in histogram: {:?}", self.histogram.len());
    }

    fn analyze_program_file(&mut self, path_to_program: &PathBuf) {
        let program_id: u32 = match program_id_from_path(&path_to_program) {
            Some(program_id) => program_id,
            None => {
                debug!("Unable to extract program_id from {:?}", path_to_program);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return;
            }
        };
        let contents: String = match fs::read_to_string(&path_to_program) {
            Ok(value) => value,
            Err(error) => {
                debug!("loading program_id: {:?}, something went wrong reading the file: {:?}", program_id, error);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return;
            }
        };
        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&contents) {
            Ok(value) => value,
            Err(error) => {
                debug!("loading program_id: {:?}, something went wrong parsing the program: {:?}", program_id, error);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return;
            }
        };
        for instruction in parsed_program.instruction_vec {
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

    fn save(&self) {
        // Convert from dictionary to array
        let mut records = Vec::<Record>::new();
        for (histogram_key, histogram_count) in &self.histogram {
            let instruction_name: String = histogram_key.0.shortname().to_string();
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

        // Save as a CSV file
        let output_path: PathBuf = self.config.cache_dir_histogram_instruction_constant_file();
        match Self::create_csv_file(&records, &output_path) {
            Ok(_) => {
                println!("save ok");
            },
            Err(error) => {
                println!("save error: {:?}", error);
            }
        }
    }
    
    fn create_csv_file(records: &Vec<Record>, output_path: &Path) -> Result<(), Box<dyn Error>> {
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

#[derive(Serialize)]
struct Record {
    count: u32,
    instruction: String,
    constant: i32,
}
