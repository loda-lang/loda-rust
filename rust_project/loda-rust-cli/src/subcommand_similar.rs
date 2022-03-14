use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs;
use crate::mine::find_asm_files_recursively;
use crate::mine::RecordBigram;
use loda_rust_core::parser::ParsedProgram;
use loda_rust_core::config::Config;


pub fn subcommand_similar() {
    let start_time = Instant::now();
    println!("similar begin");

    let config = Config::load();

    let instruction_bigram_csv: PathBuf = config.cache_dir_histogram_instruction_bigram_file();
    let instruction_vec: Vec<RecordBigram> = RecordBigram::parse_csv(&instruction_bigram_csv).expect("Unable to load instruction bigram csv");
    println!("number of rows in bigram.csv: {}", instruction_vec.len());

    let dir_containing_programs: PathBuf = config.loda_programs_oeis_dir();
    let paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_programs);
    let number_of_paths = paths.len();
    if number_of_paths <= 0 {
        error!("Expected 1 or more programs, but there are no programs to analyze");
        return;
    }
    println!("will process {} programs", number_of_paths);

    
    let mut sum = 0;
    for path in paths {
        let parsed_program: ParsedProgram = match load_program(&path) {
            Some(value) => value,
            None => {
                continue;
            }
        };
        sum += parsed_program.instruction_vec.len();
    }
    println!("number of rows total: {}", sum);

    println!("similar end, elapsed: {:?} ms", start_time.elapsed().as_millis());
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
