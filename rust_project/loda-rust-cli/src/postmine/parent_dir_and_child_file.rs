use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// A path that contains both a parent dir and a child file.
/// 
/// The `loda-programs` repo is organized in a hierarchy of dirs and files.
/// So that each dir contains max 1000 files, that can be browsed without 
/// freezing the computer.
///
/// Examples of what this looks like:
/// 
/// - "/absolute/path/to/loda-programs/oeis/027/A027008.asm"
/// - "/absolute/path/to/loda-outlier-programs/oeis_divergent/041/A041009_30_0.asm"
/// 
/// The `027/` is the parent dir, and the `A027008.asm` is the child file.
/// 
/// When converting an OeisId to path:
/// the first 3 digits of the A-number is used for the parent_dir path.
/// the entire 6 digits of the A-number is used for the child_file path.
/// 
/// The `loda-outlier-programs` repo uses a similar hierarchy of dirs and files.
#[derive(Debug)]
pub struct ParentDirAndChildFile {
    parent_dir: PathBuf,
    child_file: PathBuf
}

impl ParentDirAndChildFile {
    pub fn new(parent_dir: PathBuf, child_file: PathBuf) -> Self {
        Self {
            parent_dir: parent_dir,
            child_file: child_file
        }
    }

    /// Create the `parent_dir` if needed.
    /// 
    /// Call this function before invoking `fs::copy()` with the `child_file` path.
    /// since the `fs::copy()` does not create intermediary paths.
    pub fn create_parent_dir(&self) -> Result<(), Box<dyn Error>> {
        if !self.parent_dir.is_dir() {
            let result = fs::create_dir(&self.parent_dir);
            match result {
                Ok(_) => {},
                Err(error) => {
                    error!("Unable to create parent_dir: {:?}, child_file: {:?}, error: {:?}", self.parent_dir, self.child_file, error);
                    return Err(Box::new(error));
                }
            }
        }
        Ok(())
    }

    pub fn child_file(&self) -> &Path {
        &self.child_file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_10000_create_parent_dir_ok() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_create_parent_dir_ok");
        fs::create_dir(&basedir)?;
        let dir_path: PathBuf = basedir.join("123");
        let file_path: PathBuf = basedir.join("A123456.asm");
        let dir_and_file = ParentDirAndChildFile::new(dir_path.clone(), file_path);
        assert_eq!(dir_path.is_dir(), false);

        // Act
        dir_and_file.create_parent_dir()?;

        // Assert
        assert_eq!(dir_path.is_dir(), true);
        Ok(())
    }

    #[test]
    fn test_10001_create_parent_dir_error() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10001_create_parent_dir_error");
        fs::create_dir(&basedir)?;
        let dir_path: PathBuf = basedir.join("123");
        let file_path: PathBuf = basedir.join("A123456.asm");
        let dir_and_file = ParentDirAndChildFile::new(dir_path.clone(), file_path);

        let mut problem_file = File::create(&dir_path)?;
        writeln!(&mut problem_file, "I'm a file that prevents the dir from being created")?;
        problem_file.sync_all()?;

        assert_eq!(dir_path.is_dir(), false);
        assert_eq!(dir_path.is_file(), true);

        // Act
        let result = dir_and_file.create_parent_dir();

        // Assert
        result.expect_err("should fail with AlreadyExists, File exists, or similar");
        assert_eq!(dir_path.is_dir(), false);
        assert_eq!(dir_path.is_file(), true);
        Ok(())
    }
}
