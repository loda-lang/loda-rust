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

pub struct ValidateSingleProgram {
    loda_programs_oeis_dir: PathBuf,
}

impl ValidateSingleProgram {
    pub fn new(loda_programs_oeis_dir: PathBuf) -> Self {
        Self {
            loda_programs_oeis_dir: loda_programs_oeis_dir,
        }
    }

    pub fn run(&self, program_path: &Path) -> Result<(), Box<dyn Error>> {
        if !program_path.is_file() {
            error!("Missing program {:?}.", program_path);
            return Err(Box::new(ValidateSingleProgramError::MissingFile));
        }

        let program_contents: String = match fs::read_to_string(&program_path) {
            Ok(value) => value,
            Err(io_error) => {
                error!("Something went wrong reading the file. {:?}", io_error);
                return Err(Box::new(ValidateSingleProgramError::Load));
            }
        };

        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            self.loda_programs_oeis_dir.clone(),
        );

        let result_parse = dm.parse(
            ProgramId::ProgramWithoutId, 
            &program_contents
        );
        let program_runner: ProgramRunner = match result_parse {
            Ok(value) => value,
            Err(error) => {
                error!("Cannot parse program {:?}: {:?}", program_path, error);
                return Err(Box::new(ValidateSingleProgramError::Load));
            }
        };

        let mut cache = ProgramCache::new();
        match program_runner.compute_terms(NUMBER_OF_TERMS_TO_VALIDATE, &mut cache) {
            Ok(_) => {},
            Err(error) => {
                error!("Cannot run program {:?}: {:?}", program_path, error);
                return Err(Box::new(ValidateSingleProgramError::Run));
            }
        }
        println!("The existing program {:?} seems ok", program_path);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;

    fn format_result(result: Result<(), Box<dyn Error>>) -> String {
        let error = match result {
            Ok(_) => {
                return "OK".to_string();
            },
            Err(error) => error
        };
        if let Some(vsp_error) = error.downcast_ref::<ValidateSingleProgramError>() {
            match vsp_error {
                ValidateSingleProgramError::MissingFile => {
                    return "ERROR-MISSING-FILE".to_string();
                },
                ValidateSingleProgramError::Load => {
                    return "ERROR-LOAD".to_string();
                },
                ValidateSingleProgramError::Run => {
                    return "ERROR-RUN".to_string();
                }
            }
        }
        format!("UNKNOWN-ERROR: {:?}", error)
    }

    #[test]
    fn test_10000_valid_ok() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_valid_ok");
        fs::create_dir(&basedir)?;
        let validate_single_program = ValidateSingleProgram::new(basedir.clone());
        let input_path: PathBuf = basedir.join("19840101-054915-1251916462.asm");

        let input_content = 
r#"
mul $0,2 ; multiply by 2 is fine
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;
        input_file.sync_all()?;

        // Act
        let result = validate_single_program.run(&input_path);

        // Assert
        assert_eq!(format_result(result), "OK");
        Ok(())
    }

    #[test]
    fn test_20000_missing_file() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_20000_missing_file");
        fs::create_dir(&basedir)?;
        let validate_single_program = ValidateSingleProgram::new(basedir.clone());
        let input_path: PathBuf = basedir.join("non-existing.asm");

        // Act
        let result = validate_single_program.run(&input_path);

        // Assert
        assert_eq!(format_result(result), "ERROR-MISSING-FILE");
        Ok(())
    }

    #[test]
    fn test_30000_cannot_run() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_30000_cannot_run");
        fs::create_dir(&basedir)?;
        let validate_single_program = ValidateSingleProgram::new(basedir.clone());
        let input_path: PathBuf = basedir.join("19840101-054915-1251916462.asm");

        let input_content = 
r#"
div $0,0 ; division by zero
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;
        input_file.sync_all()?;

        // Act
        let result = validate_single_program.run(&input_path);

        // Assert
        assert_eq!(format_result(result), "ERROR-RUN");
        Ok(())
    }

    #[test]
    fn test_40000_cannot_load() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_40000_cannot_load");
        fs::create_dir(&basedir)?;
        let validate_single_program = ValidateSingleProgram::new(basedir.clone());
        let input_path: PathBuf = basedir.join("19840101-054915-1251916462.asm");

        let input_content = 
r#"
boom $0,0 ; no instruction named "boom"
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;
        input_file.sync_all()?;

        // Act
        let result = validate_single_program.run(&input_path);

        // Assert
        assert_eq!(format_result(result), "ERROR-LOAD");
        Ok(())
    }
}
