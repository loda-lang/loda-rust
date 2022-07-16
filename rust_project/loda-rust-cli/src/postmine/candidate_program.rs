use std::path::PathBuf;

pub struct CandidateProgram {
    path: PathBuf,
    terms40: String,
}

impl CandidateProgram {
    pub fn new(path: PathBuf, terms40: String) -> Self {
        Self {
            path: path,
            terms40: terms40,
        }
    }
}
