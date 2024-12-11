use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find_postmine_directories(rootdir: &Path) -> Vec<PathBuf> {
    let suffix: &str = "-postmine";
    let walker = WalkDir::new(rootdir)
        .min_depth(1)
        .max_depth(1)
        .into_iter();
    let mut paths: Vec<PathBuf> = vec!();
    for entry in walker {
        let entry = match entry {
            Ok(value) => value,
            Err(_) => {
                // debug!("Cannot obtain info about path. error: {:?}", error);
                continue;
            }
        };

        // We are only interested in dirs
        if !entry.file_type().is_dir() {
            // debug!("Ignoring a path that isn't a dir. {:?}", entry);
            continue;
        }

        // We are not interested in `*.xyz` names 
        if entry.path().extension() != None {
            // debug!("ignore path that has an extension {:?}", entry);
            continue;
        }

        // If this is a dir and it has the wrong extension, then ignore it
        let has_correct_suffix: bool = entry.file_name()
            .to_str()
            .map(|s| s.ends_with(suffix))
            .unwrap_or(false);
        if !has_correct_suffix {
            // debug!("ignore path with wrong suffix {:?}", entry);
            continue;
        }

        // If the file_name starts with a dot character, then ignore the path
        let has_dot_prefix: bool = entry.file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false);
        if has_dot_prefix {
            // debug!("ignore path with dot prefix {:?}", entry);
            continue;
        }
        
        // debug!("found {}", entry.path().display());
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
        let basedir = PathBuf::from(&tempdir.path()).join("find_postmine_directories_test_10000_empty_dir");
        fs::create_dir(&basedir)?;
        let paths = find_postmine_directories(&basedir);
        assert_eq!(paths.len(), 0);
        Ok(())
    }

    #[test]
    fn test_10001_non_existing_dir() -> Result<(), Box<dyn Error>> {
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("non-existing-dir");
        let paths = find_postmine_directories(&basedir);
        assert_eq!(paths.len(), 0);
        Ok(())
    }

    #[test]
    fn test_10002_success() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("find_postmine_directories_test_10002_success");
        fs::create_dir(&basedir)?;
        fs::write(basedir.join("ignore-file0.txt"), b"ignore this file")?;
        fs::write(basedir.join("A000040.asm"), b"ignore this file")?;
        fs::write(basedir.join("ignore-file1.txt"), b"ignore this file")?;
        fs::write(basedir.join("ignore-postmine.txt"), b"ignore this file")?;
        let dir0 = PathBuf::from(&basedir).join("a-postmine");
        fs::create_dir(&dir0)?;
        fs::write(dir0.join("logfile.txt"), b"ignore this file")?;
        let dir1 = PathBuf::from(&basedir).join("666-postmine.ignore");
        fs::create_dir(&dir1)?;
        let dir2 = PathBuf::from(&basedir).join(".ignore-postmine");
        fs::create_dir(&dir2)?;
        let dir3 = PathBuf::from(&basedir).join("b-postmine");
        fs::create_dir(&dir3)?;
        let dir4 = PathBuf::from(&basedir).join("c-POSTMINE");
        fs::create_dir(&dir4)?;
        let dir5 = PathBuf::from(&basedir).join("c-postmine-ignore");
        fs::create_dir(&dir5)?;

        // Act
        let paths = find_postmine_directories(&basedir);

        // Assert
        let mut name_vec: Vec<String> = paths.iter()
            .map(|path| {
                match path.file_name() {
                    Some(value) => value.to_string_lossy().to_string(),
                    None => "None".to_string()
                }
            }).collect();
        name_vec.sort();
        let actual: String = name_vec.join("\n");
        let expected = "a-postmine
b-postmine";
        assert_eq!(actual, expected);

        Ok(())
    }
}
