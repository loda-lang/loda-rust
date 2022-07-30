use std::path::{Path, PathBuf};

#[derive(Debug)]
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
