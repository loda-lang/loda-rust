use super::{LodaCpp, LodaCppError, LodaCppEvalOk};
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::error::Error;
use std::time::Duration;
use wait_timeout::ChildExt;

pub trait LodaCppEvalWithPath {
    fn eval_with_path(&self, term_count: usize, loda_program_path: &Path) -> Result<LodaCppEvalOk, Box<dyn Error>>;
}

impl LodaCppEvalWithPath for LodaCpp {
    fn eval_with_path(&self, term_count: usize, loda_program_path: &Path) -> Result<LodaCppEvalOk, Box<dyn Error>> {
        assert!(loda_program_path.is_absolute());
        assert!(loda_program_path.is_file());
        
        let mut child: Child = Command::new(self.loda_cpp_executable())
            .arg("eval")
            .arg(loda_program_path)
            .arg("-t")
            .arg(term_count.to_string())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute process: loda-cpp");

        let time_limit = Duration::from_secs(1);
        // let time_limit = Duration::from_millis(1);
        let optional_exit_code: Option<i32> = match child.wait_timeout(time_limit).unwrap() {
            Some(exit_status) => {
                debug!("exited with status: {:?}", exit_status);
                exit_status.code()
            },
            None => {
                // child hasn't exited yet
                debug!("kill");
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

        if optional_exit_code != Some(0) {
            error!("Expected exit_code: 0, but got exit_code: {:?}", optional_exit_code);
            error!("stdout: {:?}", output_stdout);
            error!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));
            return Err(Box::new(LodaCppError::NonZeroExitCode));
        }

        LodaCppEvalOk::parse(&output_stdout, term_count)
    }
}
