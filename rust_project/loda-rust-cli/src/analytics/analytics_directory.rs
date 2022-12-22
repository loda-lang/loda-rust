use std::fs;
use std::path::{Path,PathBuf};

pub struct AnalyticsDirectory {
    analytics_directory: PathBuf,
}

impl AnalyticsDirectory {
    pub fn new(analytics_directory: PathBuf) -> anyhow::Result<Self> {
        if !analytics_directory.is_absolute() {
            return Err(anyhow::anyhow!("The analytics_directory must be an absolute path"));
        }
        let instance = Self {
            analytics_directory,
        };
        Ok(instance)
    }

    /// Ensure that the `analytics` dir exist, before writing files to the dir.
    pub fn create_if_needed(&self) -> anyhow::Result<()> {
        if !self.analytics_directory.is_dir() {
            fs::create_dir(&self.analytics_directory)?;
        }
        assert!(self.analytics_directory.is_dir());
        Ok(())
    }

    pub fn last_analytics_timestamp_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("last_analytics_timestamp.txt");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_log_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("analytics_log.txt");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }
}
