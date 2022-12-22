use std::path::{Path,PathBuf};
use serde::Deserialize;
use std::fs;

const DEFAULT_CONFIG: &'static str = include_str!("default_config.toml");

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Eq)]
#[serde(tag = "type", content = "content")]
pub enum MinerCPUStrategy {
    #[serde(rename = "min")]
    Min,
    #[serde(rename = "half")]
    Half,
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "cpu")]
    CPU { count: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Eq)]
#[serde(tag = "type", content = "content")]
pub enum MinerFilterMode {
    /// Search only for `new` programs. Don't waste time mining for `existing` programs.
    #[serde(rename = "new")]
    New,

    /// Search for both `new` programs and `existing` programs.
    /// 
    /// The majority of programs found are improvements to `existing` programs.
    /// It's time consuming comparing performance between the `new` program vs the `existing` program,
    /// and picking most optimal program.
    /// 
    /// The minority programs found are `new` programs.
    /// No time is spent on comparing performance, since there is no `existing` program.
    #[serde(rename = "all")]
    All,
}

#[derive(Clone, Debug)]
pub struct Config {
    basedir: PathBuf,
    loda_programs_repository: PathBuf,
    loda_rust_repository: PathBuf,
    loda_rust_executable: PathBuf,
    miner_sync_executable_command_windows: String,
    miner_sync_executable: PathBuf,
    loda_cpp_executable: PathBuf,
    oeis_stripped_file: PathBuf,
    oeis_names_file: PathBuf,
    loda_submitted_by: String,
    miner_metrics_listen_port: u16,
    loda_patterns_repository: PathBuf,
    loda_outlier_programs_repository: PathBuf,
    miner_program_upload_endpoint: String,
    miner_filter_mode: MinerFilterMode,
    miner_cpu_strategy: MinerCPUStrategy,
    arc_repository_data_training: PathBuf,
    loda_arc_challenge_repository: PathBuf,
}

impl Config {
    pub fn default_config() -> String {
        DEFAULT_CONFIG.to_string()
    }

    pub fn load() -> Self {
        load_config_from_home_dir()
    }

    pub fn basedir(&self) -> PathBuf {
        PathBuf::from(&self.basedir)
    }

    pub fn analytics_dir(&self) -> PathBuf {
        let path = self.basedir.join("analytics");
        assert!(path.is_absolute());
        path
    }

    pub fn analytics_dir_dependencies_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("dependencies.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_program_rank_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("program_rank.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_program_popularity_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("program_popularity.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_instruction_constant_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_instruction_constant.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_instruction_unigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_instruction_unigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_instruction_bigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_instruction_bigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_instruction_trigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_instruction_trigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_instruction_skipgram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_instruction_skipgram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_target_unigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_target_unigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_target_bigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_target_bigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_target_trigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_target_trigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_target_skipgram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_target_skipgram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_source_unigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_source_unigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_source_bigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_source_bigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_source_trigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_source_trigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_source_skipgram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_source_skipgram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_line_unigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_line_unigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_line_bigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_line_bigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_line_trigram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_line_trigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_line_skipgram_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_line_skipgram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_histogram_oeis_stripped_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("histogram_oeis_stripped.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_indirect_memory_access_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("indirect_memory_access.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_complexity_all_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("complexity_all.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_complexity_dont_optimize_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("complexity_dont_optimize.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn analytics_dir_programs_valid_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("programs_valid.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn mine_event_dir(&self) -> PathBuf {
        let name = Path::new("mine-event");
        let path = self.basedir.join(name);
        assert!(path.is_absolute());
        path
    }

    pub fn postmine_dir(&self) -> PathBuf {
        let name = Path::new("postmine");
        let path = self.basedir.join(name);
        assert!(path.is_absolute());
        path
    }

    pub fn loda_programs_repository(&self) -> PathBuf {
        let path = &self.loda_programs_repository;
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn loda_programs_oeis_dir(&self) -> PathBuf {
        let path = self.loda_programs_repository().join("oeis");
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn loda_programs_oeis_deny_file(&self) -> PathBuf {
        let path = self.loda_programs_oeis_dir().join("deny.txt");
        assert!(path.is_absolute());
        assert!(path.is_file());
        PathBuf::from(path)
    }

    pub fn oeis_stripped_file(&self) -> PathBuf {
        let path = &self.oeis_stripped_file;
        assert!(path.is_absolute());
        assert!(path.is_file());
        PathBuf::from(path)
    }

    pub fn oeis_names_file(&self) -> PathBuf {
        let path = &self.oeis_names_file;
        assert!(path.is_absolute());
        assert!(path.is_file());
        PathBuf::from(path)
    }

    pub fn loda_rust_repository(&self) -> PathBuf {
        let path = &self.loda_rust_repository;
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    #[allow(dead_code)]
    pub fn loda_rust_executable(&self) -> PathBuf {
        let path = &self.loda_rust_executable;
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn miner_sync_executable_command_windows(&self) -> String {
        self.miner_sync_executable_command_windows.clone()
    }

    pub fn miner_sync_executable(&self) -> PathBuf {
        let path = &self.miner_sync_executable;
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn loda_cpp_executable(&self) -> PathBuf {
        let path = &self.loda_cpp_executable;
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn loda_submitted_by(&self) -> String {
        self.loda_submitted_by.clone()
    }

    pub fn miner_metrics_listen_port(&self) -> u16 {
        let port: u16 = self.miner_metrics_listen_port;
        assert!(port >= 80);
        assert!(port <= 32767);
        return port;
    }

    pub fn similar_programs(&self) -> PathBuf {
        let path = self.basedir.join("similar-programs");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn loda_patterns_repository(&self) -> PathBuf {
        let path = &self.loda_patterns_repository;
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn loda_patterns_repository_simple_constant(&self) -> PathBuf {
        let name = Path::new("simple_constant");
        let path = self.loda_patterns_repository().join(name);
        assert!(path.is_dir());
        path
    }

    pub fn loda_outlier_programs_repository(&self) -> PathBuf {
        let path = &self.loda_outlier_programs_repository;
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn loda_outlier_programs_repository_oeis_divergent(&self) -> PathBuf {
        let name = Path::new("oeis_divergent");
        let path = self.loda_outlier_programs_repository().join(name);
        assert!(path.is_dir());
        path
    }

    pub fn miner_program_upload_endpoint(&self) -> &String {
        &self.miner_program_upload_endpoint
    }

    /// How the time should be spent.
    /// - Mine only for `new` programs.
    /// - Mine for `new` programs and improvements to `existing` programs.
    pub fn miner_filter_mode(&self) -> MinerFilterMode {
        self.miner_filter_mode
    }

    pub fn miner_cpu_strategy(&self) -> MinerCPUStrategy {
        self.miner_cpu_strategy
    }

    #[allow(dead_code)]
    pub fn arc_repository_data_training(&self) -> PathBuf {
        let path = &self.arc_repository_data_training;
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    #[allow(dead_code)]
    pub fn loda_arc_challenge_repository(&self) -> PathBuf {
        let path = &self.loda_arc_challenge_repository;
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn loda_arc_challenge_repository_programs(&self) -> PathBuf {
        let name = Path::new("programs");
        let path = self.loda_arc_challenge_repository().join(name);
        assert!(path.is_dir());
        path
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFallback {
    loda_programs_repository: String,
    oeis_stripped_file: String,
    loda_rust_repository: String,
    loda_rust_executable: String,
    miner_sync_executable_command_windows: String,
    miner_sync_executable: String,
    loda_cpp_executable: String,
    oeis_names_file: String,
    loda_submitted_by: String,
    miner_metrics_listen_port: u16,
    loda_patterns_repository: String,
    loda_outlier_programs_repository: String,
    miner_program_upload_endpoint: String,
    miner_filter_mode: MinerFilterMode,
    miner_cpu_strategy: MinerCPUStrategy,
    arc_repository_data_training: String,
    loda_arc_challenge_repository: String,
}

#[derive(Debug, Deserialize)]
struct ConfigCustom {
    loda_programs_repository: Option<String>,
    oeis_stripped_file: Option<String>,
    loda_rust_repository: Option<String>,
    loda_rust_executable: Option<String>,
    miner_sync_executable_command_windows: Option<String>,
    miner_sync_executable: Option<String>,
    loda_cpp_executable: Option<String>,
    oeis_names_file: Option<String>,
    loda_submitted_by: Option<String>,
    miner_metrics_listen_port: Option<u16>,
    loda_patterns_repository: Option<String>,
    loda_outlier_programs_repository: Option<String>,
    miner_program_upload_endpoint: Option<String>,
    miner_filter_mode: Option<MinerFilterMode>,
    miner_cpu_strategy: Option<MinerCPUStrategy>,
    arc_repository_data_training: Option<String>,
    loda_arc_challenge_repository: Option<String>,
}

fn load_config_from_home_dir() -> Config {
    #![allow(warnings)]
    let homedir: PathBuf = match std::env::home_dir() {
        Some(value) => value,
        None => {
            panic!("Unable to get home_dir!");
        }
    };
    assert!(homedir.is_dir());
    assert!(homedir.is_absolute());

    let basedir_name = Path::new(".loda-rust");
    let basedir: PathBuf = homedir.join(basedir_name);
    if !basedir.is_dir() {
        panic!("Expected a '$HOME/.loda-rust' directory, but got something else. {:?}, Possible solution, remove the thing that uses the same name.", basedir);
    }
    let path_to_config: PathBuf = basedir.join(Path::new("config.toml"));
    if !path_to_config.is_file() {
        panic!("Cannot locate the file '$HOME/.loda-rust/config.toml'");
    }

    let toml_content: String = fs::read_to_string(path_to_config).unwrap();
    config_from_toml_content(toml_content, basedir, homedir)
}

struct SimpleEnvironment {
    homedir: PathBuf,
}

impl SimpleEnvironment {
    fn new(homedir: PathBuf) -> Self {
        assert!(homedir.is_absolute());
        assert!(homedir.is_dir());
        Self {
            homedir: homedir
        }
    }

    fn resolve_path(&self, path_raw: &String) -> PathBuf {
        let path_relativeto_home: String = path_raw.replacen("$HOME/", "", 1);
        let is_relativeto_home = path_relativeto_home.len() != path_raw.len();
        if is_relativeto_home {
            let relative_path = Path::new(&path_relativeto_home);
            return self.homedir.join(relative_path);
        }
        let absolute_path = Path::new(&path_raw);
        assert!(absolute_path.is_absolute());
        PathBuf::from(absolute_path)
    }
}

fn config_from_toml_content(toml_content: String, basedir: PathBuf, homedir: PathBuf) -> Config {
    assert!(homedir.is_absolute());
    let simpleenv = SimpleEnvironment::new(homedir);
    let fallback: ConfigFallback = toml::from_str(&DEFAULT_CONFIG).unwrap();
    let custom: ConfigCustom = toml::from_str(&toml_content).unwrap();

    let loda_programs_repository: String = custom.loda_programs_repository.unwrap_or(fallback.loda_programs_repository);
    let oeis_stripped_file: String = custom.oeis_stripped_file.unwrap_or(fallback.oeis_stripped_file);
    let loda_rust_repository: String = custom.loda_rust_repository.unwrap_or(fallback.loda_rust_repository);
    let oeis_names_file: String = custom.oeis_names_file.unwrap_or(fallback.oeis_names_file);
    let loda_rust_executable: String = custom.loda_rust_executable.unwrap_or(fallback.loda_rust_executable);
    let miner_sync_executable_command_windows: String = custom.miner_sync_executable_command_windows.unwrap_or(fallback.miner_sync_executable_command_windows);
    let miner_sync_executable: String = custom.miner_sync_executable.unwrap_or(fallback.miner_sync_executable);
    let loda_cpp_executable: String = custom.loda_cpp_executable.unwrap_or(fallback.loda_cpp_executable);
    let loda_submitted_by: String = custom.loda_submitted_by.unwrap_or(fallback.loda_submitted_by);
    let miner_metrics_listen_port: u16 = custom.miner_metrics_listen_port.unwrap_or(fallback.miner_metrics_listen_port);
    let loda_patterns_repository: String = custom.loda_patterns_repository.unwrap_or(fallback.loda_patterns_repository);
    let loda_outlier_programs_repository: String = custom.loda_outlier_programs_repository.unwrap_or(fallback.loda_outlier_programs_repository);
    let miner_program_upload_endpoint: String = custom.miner_program_upload_endpoint.unwrap_or(fallback.miner_program_upload_endpoint);
    let miner_filter_mode: MinerFilterMode = custom.miner_filter_mode.unwrap_or(fallback.miner_filter_mode);
    let miner_cpu_strategy: MinerCPUStrategy = custom.miner_cpu_strategy.unwrap_or(fallback.miner_cpu_strategy);
    let arc_repository_data_training: String = custom.arc_repository_data_training.unwrap_or(fallback.arc_repository_data_training);
    let loda_arc_challenge_repository: String = custom.loda_arc_challenge_repository.unwrap_or(fallback.loda_arc_challenge_repository);
    Config {
        basedir: basedir,
        loda_programs_repository: simpleenv.resolve_path(&loda_programs_repository),
        oeis_stripped_file: simpleenv.resolve_path(&oeis_stripped_file),
        oeis_names_file: simpleenv.resolve_path(&oeis_names_file),
        loda_rust_repository: simpleenv.resolve_path(&loda_rust_repository),
        loda_rust_executable: simpleenv.resolve_path(&loda_rust_executable),
        miner_sync_executable_command_windows: miner_sync_executable_command_windows,
        miner_sync_executable: simpleenv.resolve_path(&miner_sync_executable),
        loda_cpp_executable: simpleenv.resolve_path(&loda_cpp_executable),
        loda_submitted_by: loda_submitted_by,
        miner_metrics_listen_port: miner_metrics_listen_port,
        loda_patterns_repository: simpleenv.resolve_path(&loda_patterns_repository),
        loda_outlier_programs_repository: simpleenv.resolve_path(&loda_outlier_programs_repository),
        miner_program_upload_endpoint: miner_program_upload_endpoint,
        miner_filter_mode: miner_filter_mode,
        miner_cpu_strategy: miner_cpu_strategy,
        arc_repository_data_training: simpleenv.resolve_path(&arc_repository_data_training),
        loda_arc_challenge_repository: simpleenv.resolve_path(&loda_arc_challenge_repository),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use std::error::Error;
    use std::fmt;
    
    #[test]
    fn test_10000_expand_homedir() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_10000_expand_homedir");
        fs::create_dir(&homedir)?;
        let subdir = homedir.join("subdir");
        let simpleenv = SimpleEnvironment::new(homedir);

        // Act
        let resolved_path: PathBuf = simpleenv.resolve_path(&"$HOME/subdir".to_string());

        // Assert
        let subdir_string: String = subdir.to_str().unwrap().to_string();
        let resolved_string: String = resolved_path.to_str().unwrap().to_string();
        assert_eq!(subdir_string, resolved_string);
        assert_eq!(resolved_string.contains("$HOME"), false);
        assert_eq!(resolved_string.ends_with("/subdir"), true);
        Ok(())
    }

    #[test]
    fn test_10001_absolute_path() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_10001_absolute_path");
        fs::create_dir(&homedir)?;
        let subdir = homedir.join("subdir");
        let simpleenv = SimpleEnvironment::new(homedir);
        let subdir_string: String = subdir.to_str().unwrap().to_string();

        // Act
        let resolved_path: PathBuf = simpleenv.resolve_path(&subdir_string);

        // Assert
        let resolved_string: String = resolved_path.to_str().unwrap().to_string();
        assert_eq!(subdir_string, resolved_string);
        assert_eq!(resolved_string.contains("$HOME"), false);
        assert_eq!(resolved_string.ends_with("/subdir"), true);
        Ok(())
    }

    #[derive(Clone, Debug)]
    enum CheckSuffixError {
        WrongSuffix(String)
    }

    impl fmt::Display for CheckSuffixError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                CheckSuffixError::WrongSuffix(message) => {
                    return write!(f, "CheckSuffixError: {}", message)
                }
            }
        }
    }

    impl Error for CheckSuffixError {}

    fn assert_has_suffix(path: &Path, expected_suffix: &str) -> Result<(), CheckSuffixError> {
        let path_string: String = path.to_str().expect("Expected Some, but got None").to_string();
        if !path_string.ends_with(expected_suffix) {
            let message = format!("Expected suffix {:?}, but got {:?}", expected_suffix, path_string);
            return Err(CheckSuffixError::WrongSuffix(message));
        }
        Ok(())
    }

    #[test]
    fn test_20000_assert_has_suffix() -> Result<(), Box<dyn Error>> {
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_20000_assert_has_suffix");
        fs::create_dir(&homedir)?;
        let result_ok = assert_has_suffix(&homedir, "/test_20000_assert_has_suffix");
        assert!(result_ok.is_ok());
        let result_err = assert_has_suffix(&homedir, "/no-such-thing");
        assert!(result_err.is_err());
        Ok(())
    }

    #[test]
    fn test_30000_fallback_config() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_30000_fallback_config");
        fs::create_dir(&homedir)?;
        let basedir = PathBuf::from(Path::new("non-existing-basedir"));

        // Act
        let config: Config = config_from_toml_content(Config::default_config(), basedir, homedir);

        // Assert
        assert_eq!(config.basedir.to_str().unwrap(), "non-existing-basedir");
        assert_has_suffix(&config.loda_programs_repository, "/loda/programs")?;
        assert_has_suffix(&config.oeis_stripped_file, "/loda/oeis/stripped")?;
        assert_has_suffix(&config.oeis_names_file, "/loda/oeis/names")?;
        assert_has_suffix(&config.loda_rust_repository, "/git/loda-rust")?;
        assert_has_suffix(&config.loda_rust_executable, "/git/loda-rust/rust_project/target/release/loda-rust")?;
        assert_eq!(config.miner_sync_executable_command_windows, "ruby");
        assert_has_suffix(&config.miner_sync_executable, "/git/loda-rust/script/miner_sync_simple.rb")?;
        assert_has_suffix(&config.loda_cpp_executable, "/loda/bin/loda")?;
        assert_eq!(config.loda_submitted_by, "John Doe");
        assert_eq!(config.miner_program_upload_endpoint, "http://api.loda-lang.org/miner/v1/programs");
        assert_eq!(config.miner_metrics_listen_port, 8090);
        assert_has_suffix(&config.loda_patterns_repository, "/git/loda-patterns")?;
        assert_has_suffix(&config.loda_outlier_programs_repository, "/git/loda-outlier-programs")?;
        assert_has_suffix(&config.arc_repository_data_training, "/git/ARC/data/training")?;
        assert_has_suffix(&config.loda_arc_challenge_repository, "/git/loda-arc-challenge")?;
        assert_eq!(config.miner_filter_mode, MinerFilterMode::New);
        assert_eq!(config.miner_cpu_strategy, MinerCPUStrategy::Max);
        Ok(())
    }

    #[test]
    fn test_40000_override_loda_submitted_by() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_40000_override_loda_submitted_by");
        fs::create_dir(&homedir)?;
        let content = 
        r#"
        loda_submitted_by = "Leonardo di ser Piero da Vinci"
        "#;
        let basedir = PathBuf::from(Path::new("non-existing-basedir"));

        // Act
        let config: Config = config_from_toml_content(content.to_string(), basedir, homedir);

        // Assert
        assert_eq!(config.loda_submitted_by, "Leonardo di ser Piero da Vinci");
        Ok(())
    }

    #[test]
    fn test_40001_override_loda_programs_repository() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_40001_override_loda_programs_repository");
        fs::create_dir(&homedir)?;
        let repodir = homedir.join("the-loda-programs-repo");
        fs::create_dir(&repodir)?;
        let content = 
        r#"
        loda_programs_repository = "$HOME/the-loda-programs-repo"
        "#;
        let basedir = PathBuf::from(Path::new("non-existing-basedir"));

        // Act
        let config: Config = config_from_toml_content(content.to_string(), basedir, homedir);

        // Assert
        assert_has_suffix(&config.loda_programs_repository, "/the-loda-programs-repo")?;
        assert!(config.loda_programs_repository.is_absolute());
        assert!(config.loda_programs_repository.is_dir());
        Ok(())
    }

    #[test]
    fn test_40002_override_miner_filter_mode() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_40002_override_miner_filter_mode");
        fs::create_dir(&homedir)?;
        let content = 
        r#"
        [miner_filter_mode]
        type = "all"
        "#;
        let basedir = PathBuf::from(Path::new("non-existing-basedir"));

        // Act
        let config: Config = config_from_toml_content(content.to_string(), basedir, homedir);

        // Assert
        assert_eq!(config.miner_filter_mode, MinerFilterMode::All);
        Ok(())
    }

    #[test]
    fn test_40003_override_miner_cpu_strategy() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_40003_override_miner_cpu_strategy");
        fs::create_dir(&homedir)?;
        let content = 
        r#"
        [miner_cpu_strategy]
        type = "cpu"
        [miner_cpu_strategy.content]
        count = 8
        "#;
        let basedir = PathBuf::from(Path::new("non-existing-basedir"));

        // Act
        let config: Config = config_from_toml_content(content.to_string(), basedir, homedir);

        // Assert
        assert_eq!(config.miner_cpu_strategy, MinerCPUStrategy::CPU {count: 8});
        Ok(())
    }
}
