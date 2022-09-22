use loda_rust_core::control::{DependencyManager, DependencyManagerError, DependencyManagerFileSystemMode};
use loda_rust_core::parser::{ParseProgramError, ParseParametersError};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::node_power::NodePowerLimit;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Context;

const NUMBER_OF_TERMS_TO_VALIDATE: u64 = 1;

pub struct ValidateSingleProgram {
    loda_programs_oeis_dir: PathBuf,
}

impl ValidateSingleProgram {
    pub fn new(loda_programs_oeis_dir: PathBuf) -> Self {
        Self {
            loda_programs_oeis_dir: loda_programs_oeis_dir,
        }
    }

    pub fn run(&self, program_path: &Path) -> anyhow::Result<()> {
        // Load the program
        if !program_path.is_file() {
            anyhow::bail!("Missing program: {:?}", program_path);
        }
        let program_contents: String = fs::read_to_string(&program_path)
            .with_context(|| format!("The program cannot be loaded: {:?}", program_path))?;

        // Parse the program
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
                // Determine if this program contains double-dollar parameters,
                // since LODA-RUST does not yet support the double-dollar parameter type.
                // Example: `mov $$0,$2`
                if error.uses_indirect_memory_access() {
                    anyhow::bail!("The program uses indirect memory adressing, which loda-rust does not yet support: {:?} error: {:?}", program_path, error);
                }

                // Detect programs that have cyclic dependencies.
                if error.is_cyclic_dependency() {
                    anyhow::bail!("The program has a cyclic dependency: {:?} error: {:?}", program_path, error);
                }

                // Cannot parse program for other reasons such as
                // Unknown instructions, invalid instruction parameters
                // Unbalanced loop begin/end.
                anyhow::bail!("The program cannot be loaded: {:?} error: {:?}", program_path, error);
            }
        };

        // Eval 1 term with the program
        let mut cache = ProgramCache::new();
        match program_runner.compute_terms(NUMBER_OF_TERMS_TO_VALIDATE, &mut cache) {
            Ok(_) => {},
            Err(error) => {
                anyhow::bail!("The program cannot be run: {:?} error: {:?}", program_path, error);
            }
        }

        // The program seems ok
        Ok(())
    }
}

trait UsesIndirectMemoryAccess {
    /// Determines if it's an error related to `indirect memory access`.
    /// As of July 2022, LODA-RUST does not yet support LODA-CPP's `$$` parameter type.
    fn uses_indirect_memory_access(&self) -> bool;
}

impl UsesIndirectMemoryAccess for DependencyManagerError {
    fn uses_indirect_memory_access(&self) -> bool {
        if let DependencyManagerError::ParseProgram(error) = self {
            return error.uses_indirect_memory_access();
        }
        false
    }
}

impl UsesIndirectMemoryAccess for ParseProgramError {
    fn uses_indirect_memory_access(&self) -> bool {
        if let ParseProgramError::ParseParameters(error) = self {
            return error.uses_indirect_memory_access();
        }
        false
    }
}

impl UsesIndirectMemoryAccess for ParseParametersError {
    fn uses_indirect_memory_access(&self) -> bool {
        if let ParseParametersError::UnrecognizedParameterType(_raw_input_line) = self {
            return true;
        }
        false
    }
}

trait IsCyclicDependency {
    /// Determines if it's an error related to `cyclic dependency`.
    fn is_cyclic_dependency(&self) -> bool;
}

impl IsCyclicDependency for DependencyManagerError {
    fn is_cyclic_dependency(&self) -> bool {
        if let DependencyManagerError::CyclicDependency(_program_id) = self {
            return true;
        }
        false
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
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_10000_valid_ok_direct() -> anyhow::Result<()> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_valid_ok_direct");
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
        validate_single_program.run(&input_path).expect("Is not supposed to fail");

        // Assert
        Ok(())
    }

    #[test]
    fn test_10001_valid_ok_indirect() -> anyhow::Result<()> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10001_valid_ok_indirect");
        fs::create_dir(&basedir)?;
        let validate_single_program = ValidateSingleProgram::new(basedir.clone());
        let input_path: PathBuf = basedir.join("19840101-054915-1251916462.asm");

        let input_content = 
r#"
lpb $0
  mov $$0,$2 ; indirect memory access
  mov $2,1
  sub $0,$2
lpe
mov $0,$10
add $0,1
mod $0,2
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;
        input_file.sync_all()?;

        // Act
        validate_single_program.run(&input_path).expect("Is not supposed to fail");

        // Assert
        Ok(())
    }

    #[test]
    fn test_20000_missing_file() -> anyhow::Result<()> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_20000_missing_file");
        fs::create_dir(&basedir)?;
        let validate_single_program = ValidateSingleProgram::new(basedir.clone());
        let input_path: PathBuf = basedir.join("non-existing.asm");

        // Act
        let error = validate_single_program.run(&input_path).expect_err("Is supposed to fail");

        // Assert
        assert!(error.to_string().starts_with("Missing program"));
        Ok(())
    }

    #[test]
    fn test_30000_cannot_run() -> anyhow::Result<()> {
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
        let error = validate_single_program.run(&input_path).expect_err("Is supposed to fail");

        // Assert
        assert!(error.to_string().starts_with("The program cannot be run"));
        Ok(())
    }

    #[test]
    fn test_40000_cannot_load() -> anyhow::Result<()> {
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
        let error = validate_single_program.run(&input_path).expect_err("Is supposed to fail");

        // Assert
        assert!(error.to_string().starts_with("The program cannot be loaded"));
        Ok(())
    }

    #[test]
    fn test_50000_cyclic_dependency() -> anyhow::Result<()> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_60000_cyclic_dependency");
        fs::create_dir(&basedir)?;
        let dir000 = basedir.join("000");
        fs::create_dir(&dir000)?;
        let validate_single_program = ValidateSingleProgram::new(basedir.clone());
        let input_path: PathBuf = dir000.join("A000045.asm");

        let input_content = 
r#"
seq $0,45 ; This program depends on itself
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;
        input_file.sync_all()?;

        // Act
        let error = validate_single_program.run(&input_path).expect_err("Is supposed to fail");

        // Assert
        assert!(error.to_string().starts_with("The program has a cyclic dependency"));
        Ok(())
    }
}
