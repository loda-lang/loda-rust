//! The `loda-rust test-xyz` subcommands, runs automated tests.
use crate::lodacpp::{LodaCpp, LodaCppCheck, LodaCppCheckResult, LodaCppCheckStatus};
use crate::config::Config;
use crate::postmine::{ParentDirAndChildFile, path_for_oeis_program};
use loda_rust_core::oeis::OeisId;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::Context;

pub struct SubcommandTest {}

impl SubcommandTest {
    pub fn test_integration_with_lodacpp() -> Result<(), Box<dyn Error>> {
        Self::test_integration_with_lodacpp_check_ok()?;
        Self::test_integration_with_lodacpp_check_timeout()?;
        println!("test integration with lodacpp: Completed successfully.");
        Ok(())
    }

    /// We want the command to do a full match, so we set a high timeout
    /// and pick a sequence A021052 (Decimal expansion of 1/48.) with a tiny bfile (99 terms),
    /// that can quickly be computed.
    /// This way we can be fairly sure that the command finishes successfully.
    pub fn test_integration_with_lodacpp_check_ok() -> anyhow::Result<()> {
        // Arrange
        let oeis_id = OeisId::from(21052);
        let time_limit = Duration::from_secs(15);
        let config = Config::load();
        let loda_cpp_executable: PathBuf = config.loda_cpp_executable();
        let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
        let output_file: PathBuf = config.basedir().join("test_integration_with_lodacpp_check_ok.txt");
        let lodacpp = LodaCpp::new(loda_cpp_executable);
        let parent_dir_child_file: ParentDirAndChildFile = path_for_oeis_program(&loda_programs_oeis_dir, oeis_id);
        let program_path: &Path = parent_dir_child_file.child_file();
        debug!("program_path: {:?}", program_path);

        // Act
        let ok_error = lodacpp.perform_check_and_save_output(program_path, time_limit, &output_file);

        // Assert
        debug!("ok_error: {:?}", ok_error);
        let check_result: LodaCppCheckResult = ok_error.expect("Should return Ok");
        if check_result.status != LodaCppCheckStatus::FullMatch {
            return Err(anyhow::anyhow!("Problem with {}. Expected 'loda check {:?}' to be a full match, but got {:?}. output_file: {:?}", oeis_id.a_number(), program_path, check_result, output_file));
        }
        fs::remove_file(&output_file)
            .with_context(|| format!("Unable to delete temporary file: {:?}", &output_file))?;
        Ok(())
    }

    /// We want the command to timeout, so we set a low timeout
    /// and pick a sequence A000040 (The primes) with a huge bfile,
    /// that takes a long time to compute.
    /// This way we can be fairly sure that timeout happens.
    pub fn test_integration_with_lodacpp_check_timeout() -> anyhow::Result<()> {
        // Arrange
        let oeis_id = OeisId::from(40);
        let time_limit = Duration::from_secs(1);
        let config = Config::load();
        let loda_cpp_executable: PathBuf = config.loda_cpp_executable();
        let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
        let output_file: PathBuf = config.basedir().join("test_integration_with_lodacpp_check_timeout.txt");
        let lodacpp = LodaCpp::new(loda_cpp_executable);
        let parent_dir_child_file: ParentDirAndChildFile = path_for_oeis_program(&loda_programs_oeis_dir, oeis_id);
        let program_path: &Path = parent_dir_child_file.child_file();
        debug!("program_path: {:?}", program_path);

        // Act
        let ok_error = lodacpp.perform_check_and_save_output(program_path, time_limit, &output_file);

        // Assert
        debug!("ok_error: {:?}", ok_error);
        let check_result: LodaCppCheckResult = ok_error.expect("Should return Ok");
        if check_result.status != LodaCppCheckStatus::Timeout {
            return Err(anyhow::anyhow!("Problem with {}. Expected 'loda check {:?}' to be timeout, but got {:?}. output_file: {:?}", oeis_id.a_number(), program_path, check_result, output_file));
        }
        fs::remove_file(&output_file)
            .with_context(|| format!("Unable to delete temporary file: {:?}", &output_file))?;
        Ok(())
    }
}
