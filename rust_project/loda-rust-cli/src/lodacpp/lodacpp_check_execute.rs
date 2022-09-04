use super::{LodaCpp, LodaCppError, LodaCppCheckResult};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;
use subprocess::{Popen, PopenConfig, Redirection};
use std::ffi::OsStr;

pub trait LodaCppCheck {
    fn perform_check(&self, loda_program_path: &Path, time_limit: Duration) -> Result<LodaCppCheckResult, Box<dyn Error>>;
    fn perform_check_and_save_output(&self, loda_program_path: &Path, time_limit: Duration, save_output_to_path: &Path) -> Result<LodaCppCheckResult, Box<dyn Error>>;

    fn perform_check_and_save_output2(&self, loda_program_path: &Path, time_limit: Duration, save_output_to_path: &Path) -> anyhow::Result<LodaCppCheckResult>;
}

impl LodaCppCheck for LodaCpp {
    fn perform_check(&self, loda_program_path: &Path, time_limit: Duration) -> Result<LodaCppCheckResult, Box<dyn Error>> {
        lodacpp_perform_check_impl_command(&self, loda_program_path, time_limit, None)
    }

    fn perform_check_and_save_output(&self, loda_program_path: &Path, time_limit: Duration, save_output_to_path: &Path) -> Result<LodaCppCheckResult, Box<dyn Error>> {
        lodacpp_perform_check_impl_command(&self, loda_program_path, time_limit, Some(save_output_to_path))
    }

    fn perform_check_and_save_output2(&self, loda_program_path: &Path, time_limit: Duration, save_output_to_path: &Path) -> anyhow::Result<LodaCppCheckResult> {
        lodacpp_perform_check_impl_subprocess(&self, loda_program_path, time_limit, save_output_to_path)
    }
}

fn lodacpp_perform_check_impl_command(
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
        debug!("did save output");
    }

    if optional_exit_code != Some(0) {
        error!("Expected exit_code: 0, but got exit_code: {:?}", optional_exit_code);
        error!("stdout: {:?}", output_stdout);
        error!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));
        return Err(Box::new(LodaCppError::NonZeroExitCode));
    }

    LodaCppCheckResult::parse(&output_stdout, false)
}

fn lodacpp_perform_check_impl_subprocess(
    loda_cpp: &LodaCpp, 
    loda_program_path: &Path, 
    time_limit: Duration,
    save_output_to_path: &Path
) -> anyhow::Result<LodaCppCheckResult> {
    debug!("will perform check of {:?}, time_limit: {:?}", loda_program_path, time_limit);
    assert!(loda_program_path.is_absolute());
    assert!(loda_program_path.is_file());
    
    let argv = [
        loda_cpp.loda_cpp_executable().as_os_str(),
        OsStr::new("check"),
        loda_program_path.as_os_str(),
        OsStr::new("-b"),
    ];
        
    assert!(save_output_to_path.is_absolute());
    let output_file = std::fs::File::create(save_output_to_path)?;

    let mut child = Popen::create(
        &argv,
        PopenConfig {
            stdout: Redirection::File(output_file),
            ..Default::default()
        },
    )?;

    let mut did_timeout = false;
    if let Some(exit_status) = child.wait_timeout(time_limit)? {
        debug!("the child process has exited before reaching the time limit. exit-code: {:?}", exit_status);
    } else {
        debug!("timeout, kill child process");
        child.kill()?;
        did_timeout = true;
    }

    let contents: String = fs::read_to_string(&save_output_to_path)?;
    let check_result = LodaCppCheckResult::parse(&contents, did_timeout)
        .map_err(|e| anyhow::anyhow!("Unable to parse the stdout from loda check. path: {:?} error: {:?}", save_output_to_path, e))?;
    Ok(check_result)
}
