use std::path::{Path, PathBuf};

pub struct LodaCpp {
    loda_cpp_executable: PathBuf,
}

impl LodaCpp {
    pub fn new(loda_cpp_executable: PathBuf) -> Self {
        assert!(loda_cpp_executable.is_absolute());
        assert!(loda_cpp_executable.is_file());
        Self {
            loda_cpp_executable: loda_cpp_executable,
        }
    }

    pub fn loda_cpp_executable(&self) -> &Path {
        &self.loda_cpp_executable
    }
}

pub struct LodaCppError {
    stdout: String,
}

impl LodaCppError {
    pub fn new(stdout: String) -> Self {
        Self {
            stdout: stdout
        }
    }    

    #[allow(dead_code)]
    pub fn stdout(&self) -> &String {
        &self.stdout
    }
}
