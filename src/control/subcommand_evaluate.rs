use std::rc::Rc;
use std::path::PathBuf;
use super::DependencyManager;
use crate::execute::{ProgramRunner, RegisterValue, RunMode};
use crate::config::Config;

pub fn subcommand_evaluate(
    program_id: u64, 
    number_of_terms: u64,
    show_instructions: bool,
) {
    let config = Config::load();
    let loda_program_rootdir: PathBuf = config.loda_program_rootdir();

    let mut dm = DependencyManager::new(
        loda_program_rootdir,
    );
    dm.load(program_id);
    let program_runner: Rc::<ProgramRunner> = match dm.program_run_manager.get(program_id) {
        Some(value) => value,
        None => {
            panic!("Failed to load program");
        }
    };
    if show_instructions {
        program_runner.print_instructions_during_evaluate(number_of_terms);
    } else {
        program_runner.print_terms_as_they_are_computed(number_of_terms);
    }
}

impl ProgramRunner {
    fn print_terms_as_they_are_computed(&self, count: u64) {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        if count < 1 {
            panic!("Expected number of terms to be 1 or greater.");
        }
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = match self.run(input, RunMode::Silent) {
                Ok(value) => value,
                Err(error) => {
                    panic!("Failure while computing term {}, error: {:?}", index, error);
                }
            };
            if index == 0 {
                print!("{}", output.0);
                continue;
            }
            print!(",{}", output.0);
        }
        print!("\n");
    }

    fn print_instructions_during_evaluate(&self, count: u64) {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        if count < 1 {
            panic!("Expected number of terms to be 1 or greater.");
        }
        for index in 0..(count as i64) {
            println!("INPUT: a({})", index);
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = match self.run(input, RunMode::Verbose) {
                Ok(value) => value,
                Err(error) => {
                    panic!("Failure while computing term {}, error: {:?}", index, error);
                }
            };
            println!("OUTPUT: a({}) = {}", index, output.0);
        }
    }
}
