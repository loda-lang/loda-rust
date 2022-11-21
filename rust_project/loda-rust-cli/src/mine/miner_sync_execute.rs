use std::path::Path;
use std::process::{Command, Output};

#[derive(Clone, Copy, Debug)]
pub enum MinerSyncExecuteStatus {
    /// The "loda-programs" is already uptodate.
    /// 
    /// In this case the `~/.loda-rust/analytics` directory is still uptodate.
    NoChange,

    /// The local "loda-programs" directory has been updated 
    /// with the newest data from the official "loda-programs" repository.
    /// 
    /// In this case the `~/.loda-rust/analytics` directory is outdated and needs to be regenerated.
    Changed,
}

pub struct MinerSyncExecute;

impl MinerSyncExecute {
    pub fn execute(executable_path: &Path) -> anyhow::Result<MinerSyncExecuteStatus> {
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
        let strings = output_stdout.trim().split("\n");
        let last_line: &str = match strings.last() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("MinerSyncExecute no output. Expected one or more lines of text output. executable_path: {:?}", executable_path));
            }
        };
        match last_line {
            "status: nochange" => {
                return Ok(MinerSyncExecuteStatus::NoChange);
            },
            "status: changed" => {
                return Ok(MinerSyncExecuteStatus::Changed);
            },
            _ => {
                return Err(anyhow::anyhow!("MinerSyncExecute Output last line is invalid. executable_path: {:?} output: {:?} last_line: {:?}", executable_path, output_stdout, last_line));
            }
        }
    }
}
