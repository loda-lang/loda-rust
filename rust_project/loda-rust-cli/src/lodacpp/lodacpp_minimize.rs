use super::{LodaCpp, LodaCppError};
use std::error::Error;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub trait LodaCppMinimize {
    fn minimize(&self, loda_program_path: &Path, time_limit: Duration) -> Result<String, Box<dyn Error>>;
}

impl LodaCppMinimize for LodaCpp {
    fn minimize(&self, loda_program_path: &Path, time_limit: Duration) -> Result<String, Box<dyn Error>> {
        assert!(loda_program_path.is_absolute());
        assert!(loda_program_path.is_file());

        let mut child: Child = Command::new(self.loda_cpp_executable())
            .arg("minimize")
            .arg(loda_program_path)
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
                error!("Killing loda-cpp, minimize program, exceeded time limit: {:?}, loda_program_path: {:?}", time_limit, loda_program_path);
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

        let trimmed_output: String = output_stdout.trim().to_string();
        Ok(trimmed_output + "\n")
    }
}
