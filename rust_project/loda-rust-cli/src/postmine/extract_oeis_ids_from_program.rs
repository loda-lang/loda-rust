use loda_rust_core::control::{DependencyManager, DependencyManagerError, DependencyManagerFileSystemMode};
use loda_rust_core::parser::{ParsedProgram, ParseProgramError, ParseParametersError};
use loda_rust_core::parser::{create_program, CreatedProgram, CreateProgramError};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, Program, ProgramId, ProgramRunner, ProgramRunnerManager, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::node_binomial::NodeBinomialLimit;
use loda_rust_core::execute::node_power::NodePowerLimit;
use crate::common::oeis_id_from_path;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fmt;
use std::fs;

pub fn extract_oeis_ids_from_program(program_path: &Path) {
/*    
    if !program_path.is_file() {
        debug!("Missing program {:?}.", program_path);
        return Err(Box::new(ValidateSingleProgramError::MissingFile));
    }
    let program_contents: String = match fs::read_to_string(&program_path) {
        Ok(value) => value,
        Err(io_error) => {
            debug!("Something went wrong reading the file. {:?}", io_error);
            return Err(Box::new(ValidateSingleProgramError::Load));
        }
    };

    let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&program_contents) {
        Ok(value) => value,
        Err(error) => {
            return Err(DependencyManagerError::ParseProgram(error));
        }
    };

    let direct_dependencies: Vec<u64> = parsed_program.direct_dependencies();

    // let oeis_id: OeisId = oeis_id_from_path(program_path);
*/
}

// def extract_oeis_ids_from_program_file(path)
//     unless File.exist?(path)
//         raise "file does not exist: #{path}"
//     end
//     path =~ /\bA0*(\d+)[.]asm$/
//     program_id = $1.to_i
//     if program_id == 0
//         raise "Unable to process file at path: #{path}"
//     end
//     content = IO.read(path)
//     sequence_instructions = content.scan(/^\s*seq .*,\s*(\d+)$/).flatten
//     sequence_program_ids = sequence_instructions.map { |seq_program_id| seq_program_id.to_i }
//     # puts "program: #{program_id} depends on: #{sequence_program_ids}"
//     return [program_id] + sequence_program_ids
// end

// def extract_oeis_ids_from_program_files(paths)
//     program_ids = []
//     paths.each do |path|
//         program_ids += extract_oeis_ids_from_program_file(path)
//     end
//     program_ids
// end
