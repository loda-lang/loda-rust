use crate::common::{find_asm_files_recursively, program_id_from_asm_path};
use loda_rust_core::config::Config;
use loda_rust_core::parser::ParsedProgram;
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::collections::HashSet;

const PROGRAM_LENGTH_MINIMUM: usize = 1;
const PROGRAM_LENGTH_MAXIMUM: usize = 80;

pub fn subcommand_pattern() {
    let start_time = Instant::now();

    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let similarity_repository_rootdir: PathBuf = config.similarity_repository_oeis();

    let mut paths: Vec<PathBuf> = find_asm_files_recursively(&loda_programs_oeis_dir);
    paths.sort();
    let number_of_paths = paths.len();
    if number_of_paths <= 0 {
        error!("Expected 1 or more programs, but there are no programs to analyze");
        return;
    }
    println!("will process {} programs", number_of_paths);

    // Parse all programs.
    // Ignoring too short/long programs.
    let mut program_meta_vec = Vec::<ProgramMeta>::new();
    for path in paths {
        let program_meta = match analyze_program(&path) {
            Some(value) => value,
            None => {
                continue;
            }
        };
        program_meta_vec.push(program_meta);
    }
    println!("number of program_meta items: {}", program_meta_vec.len());

    // Obtain the number of lines of all programs.
    let mut program_length_set = HashSet::<u16>::new();
    for program_meta in program_meta_vec {
        program_length_set.insert(program_meta.line_count);
    }
    let mut program_length_vec: Vec<u16> = program_length_set.into_iter().collect();
    program_length_vec.sort();
    println!("line_count's: {:?}", program_length_vec);

    

    println!("pattern, elapsed: {:?} ms", start_time.elapsed().as_millis());
}

fn analyze_program(
    path: &Path, 
) -> Option<ProgramMeta> {
    let program_id: u32 = match program_id_from_asm_path(path) {
        Some(value) => value,
        None => {
            return None;
        }
    };
    let parsed_program: ParsedProgram = match load_program(path) {
        Some(value) => value,
        None => {
            return None;
        }
    };
    let line_count_raw: usize = parsed_program.instruction_vec.len();
    if line_count_raw < PROGRAM_LENGTH_MINIMUM {
        return None;
    }
    if line_count_raw > PROGRAM_LENGTH_MAXIMUM {
        error!("analyze_program. Skipping a program that is too long. path: {:?}", path);
        return None;
    }
    Some(ProgramMeta::new(program_id, line_count_raw as u16,  parsed_program))
}

fn load_program(path: &Path) -> Option<ParsedProgram> {
    let contents: String = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(error) => {
            error!("load program, error: {:?} path: {:?}", error, path);
            return None;
        }
    };
    let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&contents) {
        Ok(value) => value,
        Err(error) => {
            error!("load program, error: {:?} path: {:?}", error, path);
            return None;
        }
    };
    Some(parsed_program)
}

struct ProgramMeta {
    program_id: u32,
    line_count: u16,
    parsed_program: ParsedProgram,
}

impl ProgramMeta {
    fn new(program_id: u32, line_count: u16, parsed_program: ParsedProgram) -> Self {
        Self {
            program_id: program_id,
            line_count: line_count,
            parsed_program: parsed_program,
        }
    }
}
