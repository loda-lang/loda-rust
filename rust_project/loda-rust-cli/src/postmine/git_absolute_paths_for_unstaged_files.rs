use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use anyhow::Context;

// git: obtain modified-files and new-file
// https://stackoverflow.com/a/26891150/78336
#[allow(dead_code)]
pub fn git_absolute_paths_for_unstaged_files(dir_inside_repo: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let original_path: PathBuf = env::current_dir()
        .context("get current dir")?;
    env::set_current_dir(&dir_inside_repo)
        .with_context(|| format!("set current dir {:?}", dir_inside_repo))?;

    let output_result = Command::new("git")
        .arg("ls-files")
        .arg("--exclude-standard")
        .arg("--modified")
        .arg("--others")
        .output();
    
    env::set_current_dir(&original_path)
        .with_context(|| format!("set current dir to original dir {:?}", original_path))?;
    let actual_path: PathBuf = env::current_dir()
        .context("get current dir3")?;
    if original_path != actual_path {
        return Err(anyhow::anyhow!("Unable to restore current directory. Expected: {:?}, Actual: {:?}", original_path, actual_path));
    }

    let output: Output = output_result    
        .with_context(|| format!("git ls-files inside dir {:?}", dir_inside_repo))?;

    let output_stdout: String = String::from_utf8_lossy(&output.stdout).to_string();    
    // debug!("output: {:?}", output_stdout);

    let path_strings = output_stdout.trim().split("\n");
    // debug!("path_strings: {:?}", path_strings);

    let mut paths = Vec::<PathBuf>::new();
    for path_string in path_strings {
        let absolute_path: PathBuf = dir_inside_repo.join(path_string);
        paths.push(absolute_path);
    }
    Ok(paths)
}
