use std::rc::Rc;
use std::path::PathBuf;
use super::DependencyManager;
use crate::execute::{ProgramRunner, RegisterValue, RunMode};
use crate::config::Config;

pub enum SubcommandEvaluateMode {
    PrintTerms,
    PrintSteps,
    PrintDebug,
}

pub fn subcommand_evaluate(
    program_id: u64, 
    number_of_terms: u64,
    mode: SubcommandEvaluateMode,
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
    match mode {
        SubcommandEvaluateMode::PrintTerms => {
            program_runner.print_terms(number_of_terms);
        },
        SubcommandEvaluateMode::PrintSteps => {
            program_runner.print_steps(number_of_terms);
        },
        SubcommandEvaluateMode::PrintDebug => {
            program_runner.print_debug(number_of_terms);
        }
    }
}

impl ProgramRunner {
    fn print_terms(&self, count: u64) {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        if count < 1 {
            panic!("Expected number of terms to be 1 or greater.");
        }
        let step_count_limit: u64 = 10000000;
        let mut step_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let result_run = self.run(input, RunMode::Silent, &mut step_count, step_count_limit);
            let output: RegisterValue = match result_run {
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
        debug!("stats: step_count: {}", step_count);
    }

    fn print_steps(&self, count: u64) {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        if count < 1 {
            panic!("Expected number of terms to be 1 or greater.");
        }
        let step_count_limit: u64 = 10000000;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let mut step_count: u64 = 0;
            let result_run = self.run(input, RunMode::Silent, &mut step_count, step_count_limit);
            if let Err(error) = result_run {
                panic!("Failure while computing term {}, error: {:?}", index, error);
            }
            if index == 0 {
                print!("{}", step_count);
                continue;
            }
            print!(",{}", step_count);
        }
        print!("\n");
    }

    fn print_debug(&self, count: u64) {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        if count < 1 {
            panic!("Expected number of terms to be 1 or greater.");
        }
        let step_count_limit: u64 = 10000000;
        let mut step_count: u64 = 0;
        for index in 0..(count as i64) {
            println!("INPUT: a({})", index);
            let input = RegisterValue::from_i64(index);
            let result_run = self.run(input, RunMode::Verbose, &mut step_count, step_count_limit);
            let output: RegisterValue = match result_run {
                Ok(value) => value,
                Err(error) => {
                    panic!("Failure while computing term {}, error: {:?}", index, error);
                }
            };
            println!("OUTPUT: a({}) = {}", index, output.0);
        }
        debug!("stats: step_count: {}", step_count);
    }
}
