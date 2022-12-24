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
        self.analytics_directory.join("last_analytics_timestamp.txt")
    }

    pub fn analytics_log_file(&self) -> PathBuf {
        self.analytics_directory.join("analytics_log.txt")
    }

    pub fn dont_mine_file(&self) -> PathBuf {
        self.analytics_directory.join("dont_mine.csv")
    }

    pub fn programs_valid_file(&self) -> PathBuf {
        self.analytics_directory.join("programs_valid.csv")
    }

    pub fn programs_invalid_file(&self) -> PathBuf {
        self.analytics_directory.join("programs_invalid.csv")
    }

    pub fn programs_invalid_verbose_file(&self) -> PathBuf {
        self.analytics_directory.join("programs_invalid_verbose.csv")
    }

    pub fn complexity_all_file(&self) -> PathBuf {
        self.analytics_directory.join("complexity_all.csv")
    }

    pub fn complexity_dont_optimize_file(&self) -> PathBuf {
        self.analytics_directory.join("complexity_dont_optimize.csv")
    }

    pub fn indirect_memory_access_file(&self) -> PathBuf {
        self.analytics_directory.join("indirect_memory_access.csv")
    }

    pub fn dependencies_file(&self) -> PathBuf {
        self.analytics_directory.join("dependencies.csv")
    }

    pub fn program_rank_file(&self) -> PathBuf {
        self.analytics_directory.join("program_rank.csv")
    }

    pub fn program_popularity_file(&self) -> PathBuf {
        self.analytics_directory.join("program_popularity.csv")
    }

    pub fn histogram_instruction_constant_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_instruction_constant.csv")
    }

    pub fn histogram_instruction_unigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_instruction_unigram.csv")
    }

    pub fn histogram_instruction_bigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_instruction_bigram.csv")
    }

    pub fn histogram_instruction_trigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_instruction_trigram.csv")
    }

    pub fn histogram_instruction_skipgram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_instruction_skipgram.csv")
    }

    pub fn histogram_target_unigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_target_unigram.csv")
    }

    pub fn histogram_target_bigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_target_bigram.csv")
    }

    pub fn histogram_target_trigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_target_trigram.csv")
    }

    pub fn histogram_target_skipgram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_target_skipgram.csv")
    }

    pub fn histogram_source_unigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_source_unigram.csv")
    }

    pub fn histogram_source_bigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_source_bigram.csv")
    }

    pub fn histogram_source_trigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_source_trigram.csv")
    }

    pub fn histogram_source_skipgram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_source_skipgram.csv")
    }

    pub fn histogram_line_unigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_line_unigram.csv")
    }

    pub fn histogram_line_bigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_line_bigram.csv")
    }

    pub fn histogram_line_trigram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_line_trigram.csv")
    }

    pub fn histogram_line_skipgram_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_line_skipgram.csv")
    }

    pub fn histogram_oeis_stripped_file(&self) -> PathBuf {
        self.analytics_directory.join("histogram_oeis_stripped.csv")
    }

    pub fn program_modified_file(&self) -> PathBuf {
        self.analytics_directory.join("program_modified.csv")
    }
}
