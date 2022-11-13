use super::find_postmine_directories;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::Context;

pub struct PostmineDirectoryMaintenance {
    paths_scheduled_for_removal: Vec<PathBuf>,
}

impl PostmineDirectoryMaintenance {
    /// Identifies the oldest `postmine` directories that are to be deleted.
    /// 
    /// This function itself is non-destructive. It does not delete data from disk.
    pub fn scan(rootdir: &Path, keep_newest_count: Option<usize>) -> Self {
        // Obtain paths to `~/.loda-rust/postmine/12345-postmine` directories
        let paths: Vec<PathBuf> = find_postmine_directories(rootdir);
        // Obtain unixtime with second precision
        let mut items = Vec::<(PathBuf,u64)>::with_capacity(paths.len());
        for path in paths {
            let seconds: u64 = match Self::get_modified_time(&path) {
                Ok(value) => value,
                Err(error) => {
                    error!("cannot get modified date for path: {:?} error: {:?}", path, error);
                    continue;
                }
            };
            items.push((path.clone(), seconds));
        }
        // Arrange oldest first and newest last
        items.sort_unstable_by_key(|(_path, seconds)| *seconds);
        // Take the oldest items, and keep X newest items
        if let Some(count) = keep_newest_count {
            items.truncate(items.len() - count);
        }
        // Extract only the path, and get rid of the unixtime
        let paths_scheduled_for_removal: Vec<PathBuf> = items.iter()
            .map(|(path,_seconds)| path.clone() ).collect();
        Self { paths_scheduled_for_removal }
    }

    fn get_modified_time(path: &Path) -> anyhow::Result<u64> {
        let metadata: Metadata = fs::metadata(path)?;
        let mtime: SystemTime = metadata.modified()?;
        let since_the_epoch: Duration = mtime.duration_since(UNIX_EPOCH)?;
        let seconds: u64 = since_the_epoch.as_secs();
        Ok(seconds)
    }

    pub fn print_summary(&self) {
        let count: usize = self.paths_scheduled_for_removal.len();
        if count == 0 {
            debug!("PostmineDirectoryMaintenance: ok.");
            return;
        }
        debug!("PostmineDirectoryMaintenance: postmine directories scheduled for removal: {}", count);
    }

    /// This function is destructive and erases the scheduled dirs+dircontent from disk.
    pub fn perform_removal_of_scheduled_dirs(&self) -> anyhow::Result<()> {
        for path in &self.paths_scheduled_for_removal {
            fs::remove_dir_all(path)
                .with_context(|| format!("perform_removal_of_scheduled_dirs: Unable to remove directory: {:?}", path))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use filetime::{set_file_mtime, FileTime};

    #[test]
    fn test_10000_scan() -> anyhow::Result<()> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_scan");
        fs::create_dir(&basedir)?;
        let unixtime_alien: i64 = 308534461; // release date of "alien"
        let unixtime_matrix: i64 = 922309173; // release date of "the matrix"
        let filename_unixtime_array: [(&str, i64); 8] = [
            ("e-postmine", unixtime_matrix + 50),
            ("f-postmine", unixtime_matrix + 100),
            ("d-postmine", unixtime_matrix),
            ("a-postmine", unixtime_alien),
            ("c-postmine", unixtime_alien + 200),
            ("h-postmine", unixtime_matrix + 300),
            ("b-postmine", unixtime_alien + 100),
            ("g-postmine", unixtime_matrix + 200),
        ];
        let path_offset_vec: Vec<(PathBuf, i64)> = filename_unixtime_array.iter()
            .map(|(filename,offset)| (basedir.join(filename), *offset) ).collect();
        for (path, seconds) in path_offset_vec {
            fs::create_dir(&path)?;
            let logfile = path.join("iteration1_log.txt");
            fs::write(&logfile, "I'm a logfile")?;
            let mtime = FileTime::from_unix_time(seconds, 0);
            set_file_mtime(&path, mtime)?;
        }

        // Act
        let instance = PostmineDirectoryMaintenance::scan(&basedir, None);
        instance.print_summary();

        // Assert
        let name_vec: Vec<String> = instance.paths_scheduled_for_removal.iter()
            .map(|path| {
                match path.file_name() {
                    Some(value) => value.to_string_lossy().to_string(),
                    None => "None".to_string()
                }
            }).collect();
        let actual: String = name_vec.join("\n");
        let expected = "a-postmine
b-postmine
c-postmine
d-postmine
e-postmine
f-postmine
g-postmine
h-postmine";
        assert_eq!(actual, expected);
        Ok(())
    }
}
