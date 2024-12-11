use super::Config;
use std::path::PathBuf;

pub enum ValidateConfigTask {
    OeisMine,
}

pub trait ValidateConfig {
    fn validate_config_for_task(&self, task: ValidateConfigTask) -> anyhow::Result<()>;
}

impl ValidateConfig for Config {
    fn validate_config_for_task(&self, task: ValidateConfigTask) -> anyhow::Result<()> {
        let mut inner = ValidateConfigInner::new(self.clone());
        match task {
            ValidateConfigTask::OeisMine => {
                inner.validate_config_for_task_oeis_mine()?;
            }
        }
        Ok(())
    }
}

struct ValidateConfigInner {
    config: Config,
    messages: Vec<String>,
}

impl ValidateConfigInner {
    fn new(config: Config) -> Self {
        Self {
            config,
            messages: vec!()
        }
    }

    fn validate_config_for_task_oeis_mine(&mut self) -> anyhow::Result<()> {
        self.miner_sync_executable()?;
        self.to_result()
    }

    fn to_result(&self) -> anyhow::Result<()> { 
        if self.messages.is_empty() {
            return Ok(());
        }
        let messages_joined: String = self.messages.join("\n");
        Err(anyhow::anyhow!("There is one or more issues with the ~/.loda-rust/config.toml file:\n{}", messages_joined))
    }

    fn miner_sync_executable(&mut self) -> anyhow::Result<()> {
        let path: PathBuf = self.config.miner_sync_executable();
        if !path.is_absolute() {
            let s = format!("The 'miner_sync_executable' must be an absolute path, but got: {:?}", path);
            self.messages.push(s);
            return Ok(());
        }
        if !path.is_file() {
            let s = format!("The 'miner_sync_executable' must be an absolute path to an executable file or script, but got: {:?}", path);
            self.messages.push(s);
            return Ok(());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::error::Error;
    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;
    use crate::config::config_from_toml_content;

    #[test]
    fn test_10000_miner_sync_executable_error() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_10000_miner_sync_executable_error");
        fs::create_dir(&homedir)?;
        let content = 
        r#"
        miner_sync_executable = "/non-existing-dir/non-existing-executable"
        "#;
        let basedir = PathBuf::from(Path::new("non-existing-basedir"));
        let config: Config = config_from_toml_content(content.to_string(), basedir, homedir);
        let mut instance = ValidateConfigInner::new(config);

        // Act
        instance.miner_sync_executable()?;

        // Assert
        let result = instance.to_result().expect_err("error");
        let error_message = result.to_string();
        assert_eq!(error_message.contains("The 'miner_sync_executable' must be an absolute path to an executable"), true);
        Ok(())
    }

    #[test]
    fn test_10001_miner_sync_executable_ok() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_10001_miner_sync_executable_ok");
        fs::create_dir(&homedir)?;

        // Create a file for the `miner_sync_executable`
        let path_miner_sync_executable = homedir.join("miner_sync_executable");
        let mut file = File::create(&path_miner_sync_executable)?;
        file.write_all("I'm a file".as_bytes())?;

        // Create a config file that points to the `miner_sync_executable` file
        let config_content = format!("miner_sync_executable = \"{}\"", path_miner_sync_executable.to_string_lossy());
        let basedir = PathBuf::from(Path::new("non-existing-basedir"));
        let config: Config = config_from_toml_content(config_content, basedir, homedir);

        let mut instance = ValidateConfigInner::new(config);

        // Act
        instance.miner_sync_executable()?;

        // Assert
        instance.to_result().expect("no error");
        Ok(())
    }
}
