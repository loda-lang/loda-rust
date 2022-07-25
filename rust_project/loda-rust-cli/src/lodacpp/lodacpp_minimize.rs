use std::path::Path;
use std::process::Command;
use super::{LodaCpp, LodaCppError};

pub trait LodaCppMinimize {
    fn minimize(&self, loda_program_path: &Path) -> Result<String, LodaCppError>;
}

impl LodaCppMinimize for LodaCpp {
    fn minimize(&self, loda_program_path: &Path) -> Result<String, LodaCppError> {
        assert!(loda_program_path.is_absolute());
        assert!(loda_program_path.is_file());
        
        let output = Command::new(self.loda_cpp_executable())
            .arg("minimize")
            .arg(loda_program_path)
            .output()
            .expect("failed to execute process: loda-cpp");

        let output_stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
        let trimmed_output: String = output_stdout.trim_end().to_string();
        // println!("status: {}", output.status);
        // println!("stdout: {:?}", trimmed_output);
        // println!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            return Err(LodaCppError::new(trimmed_output));
        }

        Ok(trimmed_output + "\n")
    }
}
