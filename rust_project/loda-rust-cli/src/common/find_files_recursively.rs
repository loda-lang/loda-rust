use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn find_asm_files_recursively(rootdir: &Path) -> Vec<PathBuf> {
    find_files_recursively(rootdir, "asm")
}

pub fn find_csv_files_recursively(rootdir: &Path) -> Vec<PathBuf> {
    find_files_recursively(rootdir, "csv")
}

fn find_files_recursively(rootdir: &Path, file_extension: &str) -> Vec<PathBuf> {

    fn is_hidden(entry: &DirEntry, file_extension_inner: &str) -> bool {
        if entry.file_type().is_file() {
            // If this is a file and it has the wrong extension, then ignore it
            if let Some(extension) = entry.path().extension() {
                if extension != file_extension_inner {
                    // debug!("ignore file {:?}", entry);
                    return true
                }
            }
        }
        // If the name starts with a dot character, then ignore the file/dir
        entry.file_name()
             .to_str()
             .map(|s| s.starts_with("."))
             .unwrap_or(false)
    }
    
    let walker = WalkDir::new(rootdir).into_iter();
    let mut paths: Vec<PathBuf> = vec!();
    for entry in walker.filter_entry(|e| !is_hidden(e, file_extension)) {
        let entry = match entry {
            Ok(value) => value,
            Err(_) => {
                // debug!("Cannot obtain info about path. error: {:?}", error);
                continue;
            }
        };
        if !entry.file_type().is_file() {
            // debug!("Ignoring a path that isn't a file");
            continue;
        }
        let path: PathBuf = entry.into_path();
        paths.push(path);
    }
    // debug!("number of paths: {} in dir: {}", paths.len(), rootdir.display());
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use std::error::Error;

    #[test]
    fn test_10000_empty_dir() -> Result<(), Box<dyn Error>> {
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("find_files_recursively_test_10000_empty_dir");
        fs::create_dir(&basedir)?;
        let paths = find_asm_files_recursively(&basedir);
        assert_eq!(paths.len(), 0);
        Ok(())
    }

    #[test]
    fn test_10001_non_existing_dir() -> Result<(), Box<dyn Error>> {
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("non-existing-dir");
        let paths = find_asm_files_recursively(&basedir);
        assert_eq!(paths.len(), 0);
        Ok(())
    }

    #[test]
    fn test_10002_ignore_dotgit_dir() -> Result<(), Box<dyn Error>> {
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("find_files_recursively_test_10002_ignore_dotgit_dir");
        fs::create_dir(&basedir)?;
        let dir0 = PathBuf::from(&basedir).join(".git");
        fs::create_dir(&dir0)?;
        fs::write(dir0.join("file0.asm"), b"I'm an asm file inside .git, so I am to be ignored")?;
        fs::write(dir0.join("ignore.md"), b"Ignore this file. It doesn't have the asm file extension")?;
        let paths = find_asm_files_recursively(&basedir);
        assert_eq!(paths.len(), 0);
        Ok(())
    }

    #[test]
    fn test_10003_ignore_dir_with_asm_extension() -> Result<(), Box<dyn Error>> {
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("find_files_recursively_test_10003_ignore_dir_with_asm_extension");
        fs::create_dir(&basedir)?;
        let dir0 = PathBuf::from(&basedir).join("ignore-this-dir-but-visit-its-children.asm");
        fs::create_dir(&dir0)?;
        fs::write(dir0.join("file0.asm"), b"I'm an asm file inside a dir named 'ignore-this-dir.asm'")?;
        fs::write(dir0.join("file1.asm"), b"I'm also an asm file inside a dir named 'ignore-this-dir.asm'")?;
        let paths = find_asm_files_recursively(&basedir);
        assert_eq!(paths.len(), 2);
        Ok(())
    }

    #[test]
    fn test_20000_find_files_simple() -> Result<(), Box<dyn Error>> {
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("find_files_recursively_test_20000_find_files_simple");
        fs::create_dir(&basedir)?;
        fs::write(basedir.join("file0.asm"), b"I'm an asm file")?;
        fs::write(basedir.join("readme.md"), b"Ignore this file. It doesn't have the asm file extension")?;
        fs::write(basedir.join("file1.asm"), b"I'm also an asm file")?;
        fs::write(basedir.join(".gitignore"), b"Ignore this file. It doesn't have the asm file extension")?;
        let paths = find_asm_files_recursively(&basedir);
        assert_eq!(paths.len(), 2);
        Ok(())
    }

    #[test]
    fn test_30000_find_files_in_subdirs() -> Result<(), Box<dyn Error>> {
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("find_files_recursively_test_30000_find_files_in_subdirs");
        fs::create_dir(&basedir)?;
        let dir0 = PathBuf::from(&basedir).join("dir0");
        fs::create_dir(&dir0)?;
        let dir1 = PathBuf::from(&basedir).join("dir1");
        fs::create_dir(&dir1)?;
        fs::write(basedir.join("file0.asm"), b"I'm an asm file")?;
        fs::write(basedir.join("ignore.md"), b"Ignore this file. It doesn't have the asm file extension")?;
        fs::write(basedir.join("file2.asm"), b"I'm also an asm file")?;
        fs::write(basedir.join(".gitignore"), b"Ignore this file. It doesn't have the asm file extension")?;
        fs::write(basedir.join("file4.asm"), b"I'm also an asm file")?;
        let paths = find_asm_files_recursively(&basedir);
        assert_eq!(paths.len(), 3);
        Ok(())
    }
}
