use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
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

    pub fn dont_mine_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("dont_mine.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn programs_valid_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("programs_valid.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn programs_invalid_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("programs_invalid.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn programs_invalid_verbose_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("programs_invalid_verbose.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn complexity_all_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("complexity_all.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn complexity_dont_optimize_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("complexity_dont_optimize.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn indirect_memory_access_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("indirect_memory_access.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn dependencies_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("dependencies.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn program_rank_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("program_rank.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn program_popularity_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("program_popularity.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn histogram_instruction_constant_file(&self) -> PathBuf {
        let path = self.analytics_directory.join("histogram_instruction_constant.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }
}
