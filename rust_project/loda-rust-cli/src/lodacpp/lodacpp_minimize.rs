use super::LodaCpp;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::Duration;
use anyhow::Context;
use wait_timeout::ChildExt;

pub trait LodaCppMinimize {
    fn minimize(&self, loda_program_path: &Path, time_limit: Duration) -> anyhow::Result<String>;
}

impl LodaCppMinimize for LodaCpp {
    fn minimize(&self, loda_program_path: &Path, time_limit: Duration) -> anyhow::Result<String> {
        if !loda_program_path.is_absolute() {
            return Err(anyhow::anyhow!("minimize program. Expected path to be absolute, but it's not. path: {:?}", loda_program_path));
        }
        if !loda_program_path.is_file() {
            return Err(anyhow::anyhow!("minimize program. Expected path to be file, but it's not. path: {:?}", loda_program_path));
        }

        let mut child: Child = Command::new(self.loda_cpp_executable())
            .arg("minimize")
            .arg(loda_program_path)
            .stdout(Stdio::piped())
            .spawn()
            .with_context(|| format!("minimize program. failed to execute process: loda-cpp. path: {:?}", &loda_program_path))?;

        let optional_exit_status: Option<ExitStatus> = child
            .wait_timeout(time_limit)
            .with_context(|| format!("minimize program. unable to 'wait_timeout' for child process. path: {:?}", &loda_program_path))?;

        let optional_exit_code: Option<i32> = match optional_exit_status {
            Some(exit_status) => {
                debug!("minimize program, exited with status: {:?}", exit_status);
                exit_status.code()
            },
            None => {
                // child hasn't exited yet
                debug!("Killing loda-cpp, minimize program, exceeded time limit: {:?}, loda_program_path: {:?}", time_limit, loda_program_path);
                child.kill()?;
                debug!("wait");
                child.wait()?;
                debug!("killed successfully");
                return Err(anyhow::anyhow!("minimize program exceeded time limit: {:?}, loda_program_path: {:?}", time_limit, loda_program_path));
            }
        };

        let output: Output = child
            .wait_with_output()
            .with_context(|| format!("minimize program. failed to wait on child. path: {:?}", &loda_program_path))?;

        let output_stdout: String = String::from_utf8_lossy(&output.stdout).to_string();

        if optional_exit_code != Some(0) {
            debug!("minimize program: Expected exit_code: 0, but got exit_code: {:?}, loda_program_path: {:?}", optional_exit_code, loda_program_path);
            debug!("stdout: {:?}", output_stdout);
            debug!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!(
                "minimize program: Expected exit_code: 0, but got exit_code: {:?}, loda_program_path: {:?}\nstdout: {:?}\nstderr: {:?}", 
                optional_exit_code, loda_program_path, output_stdout, String::from_utf8_lossy(&output.stderr)
            ));
        }

        let trimmed_output: String = output_stdout.trim().to_string();
        Ok(trimmed_output + "\n")
    }
}
