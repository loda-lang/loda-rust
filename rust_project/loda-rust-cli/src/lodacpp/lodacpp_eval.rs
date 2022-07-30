use super::{LodaCpp, LodaCppError, LodaCppEvalOk};
use std::path::Path;
use std::process::{Command, Child, Stdio};
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
        let optional_status_code: Option<i32> = match child.wait_timeout(time_limit).unwrap() {
            Some(status) => {
                println!("exited with status: {:?}", status);
                status.code()
            },
            None => {
                // child hasn't exited yet
                println!("killing");
                child.kill().unwrap();
                child.wait().unwrap().code()
            }
        };

        let output = child
            .wait_with_output()
            .expect("failed to wait on child");

        let output_stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
        let trimmed_output: String = output_stdout.trim_end().to_string();
        // println!("status: {}", output.status);
        // println!("stdout: {:?}", trimmed_output);
        // println!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));

        let status_code: i32 = match optional_status_code {
            Some(code) => {
                println!("Did get status code {:?}", code);
                code
            },
            None => {
                println!("Didn't get a status code");
                panic!();
            }
        };
        if status_code != 0 {
            return Err(Box::new(LodaCppError::new(trimmed_output)));
        }

        LodaCppEvalOk::parse(&output_stdout, term_count)
    }
}
