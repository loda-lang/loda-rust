use super::{LodaCpp, LodaCppError, LodaCppEvalSteps};
use std::error::Error;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub trait LodaCppEvalStepsExecute {
    fn eval_steps(&self, term_count: usize, loda_program_path: &Path, time_limit: Duration) -> Result<LodaCppEvalSteps, Box<dyn Error>>;
}

impl LodaCppEvalStepsExecute for LodaCpp {
    fn eval_steps(&self, term_count: usize, loda_program_path: &Path, time_limit: Duration) -> Result<LodaCppEvalSteps, Box<dyn Error>> {
        assert!(loda_program_path.is_absolute());
        assert!(loda_program_path.is_file());
        
        let mut child: Child = Command::new(self.loda_cpp_executable())
            .arg("eval")
            .arg(loda_program_path)
            .arg("-t")
            .arg(term_count.to_string())
            .arg("-s")
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
                error!("Killing loda-cpp, eval {} steps, exceeded time limit: {:?}, loda_program_path: {:?}", term_count, time_limit, loda_program_path);
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
            error!("Expected exit_code: 0, but got exit_code: {:?}, loda_program_path: {:?}", optional_exit_code, loda_program_path);
            error!("stdout: {:?}", output_stdout);
            error!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));
            return Err(Box::new(LodaCppError::NonZeroExitCode));
        }

        LodaCppEvalSteps::parse(&output_stdout, term_count)
    }
}
