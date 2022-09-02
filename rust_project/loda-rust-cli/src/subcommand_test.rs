//! The `loda-rust test-xyz` subcommands, runs automated tests.
use crate::oeis::OeisId;
use crate::lodacpp::{LodaCpp, LodaCppCheck};
use crate::config::Config;
use crate::postmine::{ParentDirAndChildFile, path_for_oeis_program};
use std::time::Duration;
use std::error::Error;
use std::path::{Path, PathBuf};

pub struct SubcommandTest {}

impl SubcommandTest {
    pub fn test_integration_with_lodacpp() -> Result<(), Box<dyn Error>> {
        println!("run tests here");

        const LODACPP_CHECK_TIME_LIMIT_IN_SECONDS: u64 = 10;
        const OEIS_ID_TO_CHECK: u32 = 21052;

        let config = Config::load();
        let loda_cpp_executable: PathBuf = config.loda_cpp_executable();
        let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();

        let lodacpp = LodaCpp::new(loda_cpp_executable);
        let parent_dir_child_file: ParentDirAndChildFile = path_for_oeis_program(&loda_programs_oeis_dir, OeisId::from(OEIS_ID_TO_CHECK));
        let path: &Path = parent_dir_child_file.child_file();
        println!("path: {:?}", path);

        let time_limit = Duration::from_secs(LODACPP_CHECK_TIME_LIMIT_IN_SECONDS);
        let ok_error = lodacpp.perform_check2(path, time_limit);
        println!("ok_error: {:?}", ok_error);

        Ok(())
    }
}
