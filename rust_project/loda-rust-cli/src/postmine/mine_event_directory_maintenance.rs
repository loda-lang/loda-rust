use crate::common::{find_asm_files_recursively, MineEventDirectoryScan};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::Context;

pub struct MineEventDirectoryMaintenance {
    paths_scheduled_for_removal: Vec<PathBuf>,
}

impl MineEventDirectoryMaintenance {
    /// Identifies the oldest programs that are to be deleted.
    /// 
    /// Only considers programs that have already been processed.
    /// 
    /// Ignores pending programs.
    /// 
    /// This function itself is non-destructive. It does not delete files.
    pub fn scan(rootdir: &Path, keep_newest_count: Option<usize>) -> Self {
        // Obtain paths to already processed programs
        let paths_all: Vec<PathBuf> = find_asm_files_recursively(rootdir);
        let instance = MineEventDirectoryScan::scan(&paths_all);
        let paths: Vec<PathBuf> = instance.already_processed_paths();
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
        if let Some(keep_newest_count) = keep_newest_count {
            let item_count: usize = items.len();
            let truncate_count: usize = item_count - keep_newest_count.min(item_count);
            items.truncate(truncate_count);
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
            debug!("MineEventDirectoryMaintenance: ok.");
            return;
        }
        debug!("MineEventDirectoryMaintenance: already processed programs scheduled for removal: {}", count);
    }

    /// This function is destructive and erases the scheduled files from disk.
    pub fn perform_removal_of_scheduled_files(&self) -> anyhow::Result<()> {
        for path in &self.paths_scheduled_for_removal {
            fs::remove_file(path)
                .with_context(|| format!("perform_removal_of_scheduled_files: Unable to remove file: {:?}", path))?;
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
            ("e.keep.asm", unixtime_matrix + 50),
            ("f.reject.asm", unixtime_matrix + 100),
            ("d.reject.asm", unixtime_matrix),
            ("a.keep.asm", unixtime_alien),
            ("c.reject.asm", unixtime_alien + 200),
            ("h.keep.asm", unixtime_matrix + 300),
            ("b.reject.asm", unixtime_alien + 100),
            ("g.reject.asm", unixtime_matrix + 200),
        ];
        let path_offset_vec: Vec<(PathBuf, i64)> = filename_unixtime_array.iter()
            .map(|(filename,offset)| (basedir.join(filename), *offset) ).collect();
        for (path, seconds) in path_offset_vec {
            fs::write(&path, "; empty program with a comment")?;
            let mtime = FileTime::from_unix_time(seconds, 0);
            set_file_mtime(&path, mtime)?;
        }

        // Act
        let instance = MineEventDirectoryMaintenance::scan(&basedir, None);
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
        let expected = "a.keep.asm
b.reject.asm
c.reject.asm
d.reject.asm
e.keep.asm
f.reject.asm
g.reject.asm
h.keep.asm";
        assert_eq!(actual, expected);
        Ok(())
    }
}
