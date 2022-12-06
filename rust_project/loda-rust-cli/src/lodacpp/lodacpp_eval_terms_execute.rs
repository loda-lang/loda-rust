use super::{LodaCpp, LodaCppError, LodaCppEvalTerms};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::Duration;
use anyhow::Context;
use wait_timeout::ChildExt;

pub trait LodaCppEvalTermsExecute {
    fn eval_terms(&self, term_count: usize, loda_program_path: &Path, time_limit: Duration) -> anyhow::Result<LodaCppEvalTerms>;
}

impl LodaCppEvalTermsExecute for LodaCpp {
    fn eval_terms(&self, term_count: usize, loda_program_path: &Path, time_limit: Duration) -> anyhow::Result<LodaCppEvalTerms> {
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
                debug!("Killing loda-cpp, eval {} terms, exceeded time limit: {:?}, loda_program_path: {:?}", term_count, time_limit, loda_program_path);
                child.kill()?;
                debug!("wait");
                child.wait()?;
                debug!("killed successfully");
                let error = Err(LodaCppError::Timeout);
                return error.with_context(|| {
                    format!(
                        "Killed loda-cpp, eval {} terms, exceeded time limit: {:?}, loda_program_path: {:?}", 
                        term_count, time_limit, loda_program_path
                    )
                });
            }
        };

        let output: Output = child
            .wait_with_output()
            .expect("failed to wait on child");

        let output_stdout: String = String::from_utf8_lossy(&output.stdout).to_string();

        if optional_exit_code != Some(0) {
            debug!("Expected exit_code: 0, but got exit_code: {:?}, loda_program_path: {:?}", optional_exit_code, loda_program_path);
            debug!("stdout: {:?}", output_stdout);
            debug!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));
            let error = Err(LodaCppError::NonZeroExitCode);
            return error.with_context(|| {
                format!(
                    "Expected exit_code: 0, but got exit_code: {:?}, loda_program_path: {:?}\nstdout: {}\nstderr: {}", 
                    optional_exit_code, loda_program_path, output_stdout, String::from_utf8_lossy(&output.stderr)
                )
            });
        }

        let result_parse = LodaCppEvalTerms::parse(&output_stdout, term_count);
        match result_parse {
            Ok(value) => {
                return Ok(value);
            },
            Err(error) => {
                return Err(anyhow::anyhow!("LodaCppEvalTerms::parse returned error: {:?}", error));
            }
        }
    }
}
