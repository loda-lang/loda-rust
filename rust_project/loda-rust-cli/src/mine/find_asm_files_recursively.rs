use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn find_asm_files_recursively(rootdir: &Path) -> Vec<PathBuf> {

    fn is_hidden(entry: &DirEntry) -> bool {
        entry.file_name()
             .to_str()
             .map(|s| s.starts_with("."))
             .unwrap_or(false)
    }
    
    let walker = WalkDir::new(rootdir).into_iter();
    let mut paths: Vec<PathBuf> = vec!();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();
        let path: PathBuf = entry.into_path();
        let extension = match path.extension() {
            Some(value) => value,
            None => {
                // debug!("path has no extension. Ignoring: {:}", path.display());
                continue;
            }
        };
        if extension != "asm" {
            // debug!("path is not an asm file. Ignoring: {:}", path.display());
            continue;
        }
        // debug!("path: {:?}", path.display());
        paths.push(path);
    }
    debug!("number of paths: {} in dir: {}", paths.len(), rootdir.display());
    paths
}
