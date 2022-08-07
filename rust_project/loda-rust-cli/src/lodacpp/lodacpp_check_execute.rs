use super::{LodaCpp, LodaCppError, LodaCppCheckResult};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub trait LodaCppCheck {
    fn perform_check(&self, loda_program_path: &Path, time_limit: Duration) -> Result<LodaCppCheckResult, Box<dyn Error>>;
    fn perform_check_and_save_output(&self, loda_program_path: &Path, time_limit: Duration, save_output_to_path: &Path) -> Result<LodaCppCheckResult, Box<dyn Error>>;
}

impl LodaCppCheck for LodaCpp {
    fn perform_check(&self, loda_program_path: &Path, time_limit: Duration) -> Result<LodaCppCheckResult, Box<dyn Error>> {
        lodacpp_perform_check_impl(&self, loda_program_path, time_limit, None)
    }

    fn perform_check_and_save_output(&self, loda_program_path: &Path, time_limit: Duration, save_output_to_path: &Path) -> Result<LodaCppCheckResult, Box<dyn Error>> {
        lodacpp_perform_check_impl(&self, loda_program_path, time_limit, Some(save_output_to_path))
    }
}

fn lodacpp_perform_check_impl(
    loda_cpp: &LodaCpp, 
    loda_program_path: &Path, 
    time_limit: Duration,
    optional_save_output_to_path: Option<&Path>
) -> Result<LodaCppCheckResult, Box<dyn Error>> {
    debug!("will perform check of {:?}, time_limit: {:?}", loda_program_path, time_limit);

    assert!(loda_program_path.is_absolute());
    assert!(loda_program_path.is_file());

    let mut child: Child = Command::new(loda_cpp.loda_cpp_executable())
        .arg("check")
        .arg(loda_program_path)
        .arg("-b")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute process: loda-cpp");

    let optional_exit_status: Option<ExitStatus> = child
        .wait_timeout(time_limit)
        .expect("unable to 'wait_timeout' for child process");

    let optional_exit_code: Option<i32> = match optional_exit_status {
        Some(exit_status) => {
            debug!("exited with status: {:?}", exit_status);
            exit_status.code()
        },
        None => {
            // child hasn't exited yet
            debug!("Killing 'loda-cpp check', exceeded {:?} time limit, loda_program_path: {:?}", time_limit, loda_program_path);
            child.kill()?;
            debug!("wait");
            child.wait()?;
            debug!("killed successfully");
            return Err(Box::new(LodaCppError::Timeout));
        }
    };

    let output: Output = child
        .wait_with_output()
        .expect("failed to wait on child");

    let output_stdout: String = String::from_utf8_lossy(&output.stdout).to_string();

    if let Some(save_output_to_path) = optional_save_output_to_path {
        debug!("Saving 'loda-cpp check' output to file: {:?}", save_output_to_path);
        let mut file = File::create(save_output_to_path)?;
        file.write_all(output_stdout.as_bytes())?;
    }

    if optional_exit_code != Some(0) {
        error!("Expected exit_code: 0, but got exit_code: {:?}", optional_exit_code);
        error!("stdout: {:?}", output_stdout);
        error!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));
        return Err(Box::new(LodaCppError::NonZeroExitCode));
    }

    LodaCppCheckResult::parse(&output_stdout)
}
