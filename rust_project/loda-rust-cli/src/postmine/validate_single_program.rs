use crate::config::Config;
use crate::oeis::OeisId;
use loda_rust_core::control::{DependencyManager, DependencyManagerFileSystemMode};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::node_binomial::NodeBinomialLimit;
use loda_rust_core::execute::node_power::NodePowerLimit;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fmt;
use std::fs;

const NUMBER_OF_TERMS_TO_VALIDATE: u64 = 1;

#[derive(Debug)]
pub enum ValidateSingleProgramError {
    MissingFile,
    Load,
    Run,
}

impl std::error::Error for ValidateSingleProgramError {}

impl fmt::Display for ValidateSingleProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingFile => write!(f, "Missing program"),
            Self::Load => write!(f, "The program cannot be loaded."),
            Self::Run => write!(f, "The program cannot be run."),
        }
    }
}

pub struct ValidateSingleProgram {}

impl ValidateSingleProgram {
    pub fn run(program_id: OeisId, program_path: &Path) -> Result<(), Box<dyn Error>> {
        if !program_path.is_file() {
            error!("Missing program {}.", program_id);
            return Err(Box::new(ValidateSingleProgramError::MissingFile));
        }

        let program_contents: String = match fs::read_to_string(&program_path) {
            Ok(value) => value,
            Err(io_error) => {
                error!("Something went wrong reading the file. {:?}", io_error);
                return Err(Box::new(ValidateSingleProgramError::Load));
            }
        };

        let config = Config::load();
        let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();

        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            loda_programs_oeis_dir,
        );

        let result_parse = dm.parse(
            ProgramId::ProgramWithoutId, 
            &program_contents
        );
        let program_runner: ProgramRunner = match result_parse {
            Ok(value) => value,
            Err(error) => {
                error!("Cannot parse program {}: {:?}", program_id, error);
                return Err(Box::new(ValidateSingleProgramError::Load));
            }
        };

        let mut cache = ProgramCache::new();
        match program_runner.compute_terms(NUMBER_OF_TERMS_TO_VALIDATE, &mut cache) {
            Ok(_) => {},
            Err(error) => {
                error!("Cannot run program {:?}: {:?}", program_id, error);
                return Err(Box::new(ValidateSingleProgramError::Run));
            }
        }
        println!("The existing program {} seems ok", program_id);
        Ok(())
    }
}

trait ComputeTerms {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) -> Result<(), Box<dyn Error>>;
}

impl ComputeTerms for ProgramRunner {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) -> Result<(), Box<dyn Error>> {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        if count < 1 {
            panic!("Expected number of terms to be 1 or greater.");
        }
        let step_count_limit: u64 = 1000000000;
        let mut step_count: u64 = 0;
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
                cache
            );
            let _ = match result_run {
                Ok(value) => value,
                Err(error) => {
                    debug!("Failure while computing term {}, error: {:?}", index, error);
                    return Err(Box::new(error));
                }
            };
        }
        return Ok(());
    }
}
