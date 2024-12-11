use super::{LodaCpp, LodaCppCheckResult};
use std::fs;
use std::path::Path;
use std::time::Duration;
use subprocess::{Popen, PopenConfig, Redirection};
use std::ffi::OsStr;

pub trait LodaCppCheck {
    fn perform_check_and_save_output(&self, loda_program_path: &Path, time_limit: Duration, save_output_to_path: &Path) -> anyhow::Result<LodaCppCheckResult>;
}

impl LodaCppCheck for LodaCpp {
    fn perform_check_and_save_output(&self, loda_program_path: &Path, time_limit: Duration, save_output_to_path: &Path) -> anyhow::Result<LodaCppCheckResult> {
        lodacpp_perform_check_impl(&self, loda_program_path, time_limit, save_output_to_path)
    }
}

fn lodacpp_perform_check_impl(
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
