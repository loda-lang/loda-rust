use std::path::Path;
use std::process::{Command, Output};

pub struct MinerSyncExecute;

impl MinerSyncExecute {
    pub fn execute(executable_path: &Path) -> anyhow::Result<()> {
        if !executable_path.is_absolute() {
            return Err(anyhow::anyhow!("MinerSyncExecute expected absolute path, but got executable_path: {:?}", executable_path));
        }
        if !executable_path.is_file() {
            return Err(anyhow::anyhow!("MinerSyncExecute expected executable file, but got something else. executable_path: {:?}", executable_path));
        }
        debug!("MinerSyncExecute.execute: {:?}", executable_path);
        let output: Output = Command::new(executable_path)
            .output()
            .map_err(|e| anyhow::anyhow!("MinerSyncExecute unable to run process. executable_path: {:?} error: {:?}", executable_path, e))?;

        let output_stdout: String = String::from_utf8_lossy(&output.stdout).trim().to_string();    
        if !output.status.success() {
            return Err(anyhow::anyhow!("MinerSyncExecute with failing error code. executable_path: {:?} output: {:?}", executable_path, output_stdout));
        }
        if !output_stdout.is_empty() {
            println!("MinerSyncExecute output: {:?}", output_stdout);
        }
        Ok(())
    }
}
