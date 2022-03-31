use crate::common::{find_asm_files_recursively, find_csv_files_recursively, program_id_from_path};
use loda_rust_core::config::Config;
use loda_rust_core::parser::ParsedProgram;
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;

const PROGRAM_LENGTH_MINIMUM: usize = 1;
const PROGRAM_LENGTH_MAXIMUM: usize = 80;

pub fn subcommand_pattern() {
    let start_time = Instant::now();

    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let similarity_repository_oeis_dir: PathBuf = config.similarity_repository_oeis();

    let mut similarity_csv_paths: Vec<PathBuf> = find_csv_files_recursively(&similarity_repository_oeis_dir);
    similarity_csv_paths.sort();
    let number_of_similarity_csv_paths = similarity_csv_paths.len();
    if number_of_similarity_csv_paths <= 0 {
        error!("Expected 1 or more similarity csv files, but there are none to analyze");
        return;
    }
    println!("number of similarity csv files: {}", number_of_similarity_csv_paths);

    let mut program_asm_paths: Vec<PathBuf> = find_asm_files_recursively(&loda_programs_oeis_dir);
    program_asm_paths.sort();
    let number_of_program_asm_paths = program_asm_paths.len();
    if number_of_program_asm_paths <= 0 {
        error!("Expected 1 or more program asm files, but there are none to analyze");
        return;
    }
    println!("number of program asm files: {}", number_of_program_asm_paths);

    // Parse all programs.
    // Ignoring too short/long programs.
    let mut program_meta_vec = Vec::<ProgramMeta>::new();
    for path in program_asm_paths {
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
    for program_meta in &program_meta_vec {
        program_length_set.insert(program_meta.line_count);
    }
    let mut program_length_vec: Vec<u16> = program_length_set.into_iter().collect();
    program_length_vec.sort();
    println!("line_count's: {:?}", program_length_vec);

    traverse_by_program_length(&program_length_vec, &program_meta_vec);

    println!("pattern, elapsed: {:?} ms", start_time.elapsed().as_millis());
}

fn traverse_by_program_length(program_length_vec: &Vec<u16>, program_meta_vec: &Vec<ProgramMeta>) {
    for program_length in program_length_vec {
        let programs_with_approx_same_length: Vec<&ProgramMeta> = 
            program_meta_vec.iter()
            .filter(|&pm| pm.line_count == *program_length)
            .collect();
        process_programs_with_approx_same_length(*program_length, &programs_with_approx_same_length);
    }
}

fn process_programs_with_approx_same_length(program_length: u16, program_meta_vec: &Vec<&ProgramMeta>) {
    println!("program_length: {:?}  number of programs: {:?}", program_length, program_meta_vec.len());
}

fn analyze_program(
    path: &Path, 
) -> Option<ProgramMeta> {
    let program_id: u32 = match program_id_from_path(path) {
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
