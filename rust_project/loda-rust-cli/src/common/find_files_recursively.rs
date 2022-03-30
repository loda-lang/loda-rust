use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn find_asm_files_recursively(rootdir: &Path) -> Vec<PathBuf> {
    find_files_recursively(rootdir, "asm")
}

pub fn find_csv_files_recursively(rootdir: &Path) -> Vec<PathBuf> {
    find_files_recursively(rootdir, "csv")
}

fn find_files_recursively(rootdir: &Path, find_extension: &str) -> Vec<PathBuf> {

    fn is_hidden(entry: &DirEntry, find_extension_inner: &str) -> bool {
        if entry.file_type().is_file() {
            // If this is a file and it has the wrong extension, then ignore it
            if let Some(extension) = entry.path().extension() {
                if extension != find_extension_inner {
                    // debug!("ignore file {:?}", entry);
                    return true
                }
            }
        } else {
            // If this is directory with same extension, then ignore traversing this directory
            if let Some(extension) = entry.path().extension() {
                if extension == find_extension_inner {
                    // debug!("ignore dir {:?}", entry);
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
    for entry in walker.filter_entry(|e| !is_hidden(e, find_extension)) {
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
    use std::fs::File;
    use std::io::prelude::*;

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
        let mut file0 = File::create(dir0.join("file0.asm"))?;
        file0.write_all(b"I'm an asm file inside .git, so I am to be ignored")?;
        let mut file1 = File::create(dir0.join("ignore.md"))?;
        file1.write_all(b"Ignore this file. It doesn't have the asm file extension")?;
        let paths = find_asm_files_recursively(&basedir);
        assert_eq!(paths.len(), 0);
        Ok(())
    }

    #[test]
    fn test_10003_ignore_dir_with_asm_extension() -> Result<(), Box<dyn Error>> {
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("find_files_recursively_test_10003_ignore_dir_with_asm_extension");
        fs::create_dir(&basedir)?;
        let dir0 = PathBuf::from(&basedir).join("ignore-this-dir.asm");
        fs::create_dir(&dir0)?;
        let mut file0 = File::create(dir0.join("file0.asm"))?;
        file0.write_all(b"I'm an asm file inside a dir named 'ignore-this-dir.asm', so I am to be ignored")?;
        let paths = find_asm_files_recursively(&basedir);
        assert_eq!(paths.len(), 0);
        Ok(())
    }

    #[test]
    fn test_20000_find_files_simple() -> Result<(), Box<dyn Error>> {
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("find_files_recursively_test_20000_find_files_simple");
        fs::create_dir(&basedir)?;
        let mut file0 = File::create(basedir.join("file0.asm"))?;
        file0.write_all(b"I'm an asm file")?;
        let mut file1 = File::create(basedir.join("readme.md"))?;
        file1.write_all(b"Ignore this file. It doesn't have the asm file extension")?;
        let mut file2 = File::create(basedir.join("file1.asm"))?;
        file2.write_all(b"I'm also an asm file")?;
        let mut file3 = File::create(basedir.join(".gitignore"))?;
        file3.write_all(b"Ignore this file. It doesn't have the asm file extension")?;
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
        let mut file0 = File::create(dir0.join("file0.asm"))?;
        file0.write_all(b"I'm an asm file")?;
        let mut file1 = File::create(dir0.join("ignore.md"))?;
        file1.write_all(b"Ignore this file. It doesn't have the asm file extension")?;
        let mut file2 = File::create(dir1.join("file2.asm"))?;
        file2.write_all(b"I'm also an asm file")?;
        let mut file3 = File::create(dir1.join(".gitignore"))?;
        file3.write_all(b"Ignore this file. It doesn't have the asm file extension")?;
        let mut file4 = File::create(dir1.join("file4.asm"))?;
        file4.write_all(b"I'm also an asm file")?;    
        let paths = find_asm_files_recursively(&basedir);
        assert_eq!(paths.len(), 3);
        Ok(())
    }
}
