use std::path::Path;
use std::process::Command;
use num_bigint::BigInt;
use loda_rust_core::util::BigIntVec;
use super::{LodaCpp, LodaCppError};

pub trait LodaCppEvalWithPath {
    fn eval_with_path(&self, term_count: usize, loda_program_path: &Path) -> Result<LodaCppEvalOk, LodaCppError>;
}

impl LodaCppEvalWithPath for LodaCpp {
    fn eval_with_path(&self, term_count: usize, loda_program_path: &Path) -> Result<LodaCppEvalOk, LodaCppError> {
        assert!(loda_program_path.is_absolute());
        assert!(loda_program_path.is_file());
        
        let output = Command::new(self.loda_cpp_executable())
            .arg("eval")
            .arg(loda_program_path)
            .arg("-t")
            .arg(term_count.to_string())
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

        let term_strings = trimmed_output.split(",");
        let mut terms_bigintvec = BigIntVec::with_capacity(term_count);
        for term_string in term_strings {
            let bytes: &[u8] = term_string.as_bytes();
            let bigint: BigInt = match BigInt::parse_bytes(bytes, 10) {
                Some(value) => value,
                None => {
                    error!("Unable to parse a number as BigInt. '{}'", term_string);
                    return Err(LodaCppError::new(trimmed_output));
                }
            };
            terms_bigintvec.push(bigint);
        };

        Ok(LodaCppEvalOk::new(trimmed_output, terms_bigintvec))
    }
}

pub struct LodaCppEvalOk {
    stdout: String,
    terms: BigIntVec,
}

impl LodaCppEvalOk {
    fn new(stdout: String, terms: BigIntVec) -> Self {
        Self {
            stdout: stdout,
            terms: terms,
        }
    }

    #[allow(dead_code)]
    pub fn stdout(&self) -> &String {
        &self.stdout
    }

    #[allow(dead_code)]
    pub fn terms(&self) -> &BigIntVec {
        &self.terms
    }
}
