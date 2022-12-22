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

    pub fn last_analytics_timestamp_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("last_analytics_timestamp.txt");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

}
