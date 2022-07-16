use std::path::PathBuf;

pub struct CandidateProgram {
    path: PathBuf,
    output: String,
}

impl CandidateProgram {
    pub fn new(path: PathBuf, output: String) -> Self {
        Self {
            path: path,
            output: output,
        }
    }
}
