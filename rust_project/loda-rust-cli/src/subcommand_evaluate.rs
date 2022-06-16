use loda_rust_core;
use std::time::Instant;
use std::rc::Rc;
use std::path::PathBuf;
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::node_binomial::NodeBinomialLimit;
use loda_rust_core::execute::node_power::NodePowerLimit;
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
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();

    let mut dm = DependencyManager::new(
        DependencyManagerFileSystemMode::System,
        loda_programs_oeis_dir,
    );
    let program_runner: Rc::<ProgramRunner> = match dm.load(program_id) {
        Ok(value) => value,
        Err(error) => {
            panic!("Failed to load program: {:?}", error);
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

trait PrintTermsStepsDebug {
    fn print_terms(&self, count: u64);
    fn print_steps(&self, count: u64);
    fn print_debug(&self, count: u64);
}

impl PrintTermsStepsDebug for ProgramRunner {
    fn print_terms(&self, count: u64) {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        if count < 1 {
            panic!("Expected number of terms to be 1 or greater.");
        }
        let mut cache = ProgramCache::new();
        let step_count_limit: u64 = 1000000000;
        let mut step_count: u64 = 0;
        let start_time = Instant::now();
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let result_run = self.run(
                &input, 
                RunMode::Silent, 
                &mut step_count, 
                step_count_limit,
                NodeRegisterLimit::Unlimited,
                NodeBinomialLimit::Unlimited,
                NodeLoopLimit::Unlimited,
                NodePowerLimit::Unlimited,
                &mut cache
            );
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
        debug!("steps: {}", step_count);
        debug!("cache: {}", cache.hit_miss_info());
        debug!("elapsed: {:?} ms", start_time.elapsed().as_millis());
    }

    fn print_steps(&self, count: u64) {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        if count < 1 {
            panic!("Expected number of terms to be 1 or greater.");
        }
        let mut cache = ProgramCache::new();
        let step_count_limit: u64 = 1000000000;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let mut step_count: u64 = 0;
            let result_run = self.run(
                &input, 
                RunMode::Silent, 
                &mut step_count, 
                step_count_limit,
                NodeRegisterLimit::Unlimited,
                NodeBinomialLimit::Unlimited,
                NodeLoopLimit::Unlimited,
                NodePowerLimit::Unlimited,
                &mut cache,
            );
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
        let mut cache = ProgramCache::new();
        let step_count_limit: u64 = 1000000000;
        let mut step_count: u64 = 0;
        for index in 0..(count as i64) {
            println!("INPUT: a({})", index);
            let input = RegisterValue::from_i64(index);
            let result_run = self.run(
                &input, 
                RunMode::Verbose, 
                &mut step_count, 
                step_count_limit,
                NodeRegisterLimit::Unlimited,
                NodeBinomialLimit::Unlimited,
                NodeLoopLimit::Unlimited,
                NodePowerLimit::Unlimited,
                &mut cache,
            );
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
