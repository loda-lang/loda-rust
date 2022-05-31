use std::path::{Path,PathBuf};
use serde::Deserialize;
use std::fs;

const DEFAULT_CONFIG: &'static str =
r#"# Configuration for LODA Rust

# Absolute path to the "loda-programs" repository dir.
loda_programs_repository = "$HOME/loda/programs"

# Absolute path to the "loda-cpp" repository dir.
loda_cpp_repository = "$HOME/git/loda-cpp"

# Absolute path to the "loda" executable file.
loda_cpp_executable = "$HOME/loda/bin/loda"

# Absolute path to the "loda-rust" repository dir.
loda_rust_repository = "$HOME/git/loda-rust"

# Absolute path to the unzipped OEIS stripped file.
oeis_stripped_file = "$HOME/loda/oeis/stripped"

# Absolute path to the unzipped OEIS names file.
oeis_names_file = "$HOME/loda/oeis/names"

# Who to be credited when discovering new programs.
loda_submitted_by = "John Doe"

# When mining with metrics enabled, this is the port that the metrics can be accessed.
miner_metrics_listen_port = 8090

# What loda programs are similar to each other.
loda_identify_similar_programs_repository = "$HOME/git/loda-identify-similar-programs"

# Patterns that are frequently used in loda programs.
loda_patterns_repository = "$HOME/git/loda-patterns"

# Absolute path to the "loda-outlier-programs" repository dir.
loda_outlier_programs_repository = "$HOME/git/loda-outlier-programs"
"#;


#[derive(Debug)]
pub struct Config {
    basedir: PathBuf,
    loda_programs_repository: PathBuf,
    loda_rust_repository: PathBuf,
    loda_cpp_repository: PathBuf,
    loda_cpp_executable: PathBuf,
    oeis_stripped_file: PathBuf,
    oeis_names_file: PathBuf,
    loda_submitted_by: String,
    miner_metrics_listen_port: u16,
    loda_identify_similar_programs_repository: PathBuf,
    loda_patterns_repository: PathBuf,
    loda_outlier_programs_repository: PathBuf,
}

impl Config {
    pub fn default_config() -> String {
        DEFAULT_CONFIG.to_string()
    }

    pub fn load() -> Self {
        load_config_from_home_dir()
    }

    pub fn analytics_dir(&self) -> PathBuf {
        let name = Path::new("analytics");
        let path = self.basedir.join(name);
        assert!(path.is_dir());
        path
    }

    pub fn analytics_dir_dont_mine_file(&self) -> PathBuf {
        let path = self.analytics_dir().join("dont_mine.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
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

    pub fn mine_event_dir(&self) -> PathBuf {
        let name = Path::new("mine-event");
        let path = self.basedir.join(name);
        assert!(path.is_dir());
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

    pub fn loda_cpp_repository(&self) -> PathBuf {
        let path = &self.loda_cpp_repository;
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn loda_cpp_executable(&self) -> PathBuf {
        let path = &self.loda_cpp_executable;
        assert!(path.is_absolute());
        assert!(path.is_file());
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

    pub fn loda_identify_similar_programs_repository(&self) -> PathBuf {
        let path = &self.loda_identify_similar_programs_repository;
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn loda_identify_similar_programs_repository_oeis(&self) -> PathBuf {
        let name = Path::new("oeis");
        let path = self.loda_identify_similar_programs_repository().join(name);
        assert!(path.is_dir());
        path
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
}

#[derive(Debug, Deserialize)]
struct ConfigFallback {
    loda_programs_repository: String,
    oeis_stripped_file: String,
    loda_rust_repository: String,
    loda_cpp_repository: String,
    loda_cpp_executable: String,
    oeis_names_file: String,
    loda_submitted_by: String,
    miner_metrics_listen_port: u16,
    loda_identify_similar_programs_repository: String,
    loda_patterns_repository: String,
    loda_outlier_programs_repository: String,
}

#[derive(Debug, Deserialize)]
struct ConfigCustom {
    loda_programs_repository: Option<String>,
    oeis_stripped_file: Option<String>,
    loda_rust_repository: Option<String>,
    loda_cpp_repository: Option<String>,
    loda_cpp_executable: Option<String>,
    oeis_names_file: Option<String>,
    loda_submitted_by: Option<String>,
    miner_metrics_listen_port: Option<u16>,
    loda_identify_similar_programs_repository: Option<String>,
    loda_patterns_repository: Option<String>,
    loda_outlier_programs_repository: Option<String>,
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
    let loda_cpp_repository: String = custom.loda_cpp_repository.unwrap_or(fallback.loda_cpp_repository);
    let loda_cpp_executable: String = custom.loda_cpp_executable.unwrap_or(fallback.loda_cpp_executable);
    let loda_submitted_by: String = custom.loda_submitted_by.unwrap_or(fallback.loda_submitted_by);
    let miner_metrics_listen_port: u16 = custom.miner_metrics_listen_port.unwrap_or(fallback.miner_metrics_listen_port);
    let loda_identify_similar_programs_repository: String = custom.loda_identify_similar_programs_repository.unwrap_or(fallback.loda_identify_similar_programs_repository);
    let loda_patterns_repository: String = custom.loda_patterns_repository.unwrap_or(fallback.loda_patterns_repository);
    let loda_outlier_programs_repository: String = custom.loda_outlier_programs_repository.unwrap_or(fallback.loda_outlier_programs_repository);
    Config {
        basedir: basedir,
        loda_programs_repository: simpleenv.resolve_path(&loda_programs_repository),
        oeis_stripped_file: simpleenv.resolve_path(&oeis_stripped_file),
        oeis_names_file: simpleenv.resolve_path(&oeis_names_file),
        loda_rust_repository: simpleenv.resolve_path(&loda_rust_repository),
        loda_cpp_repository: simpleenv.resolve_path(&loda_cpp_repository),
        loda_cpp_executable: simpleenv.resolve_path(&loda_cpp_executable),
        loda_submitted_by: loda_submitted_by.clone(),
        miner_metrics_listen_port: miner_metrics_listen_port,
        loda_identify_similar_programs_repository: simpleenv.resolve_path(&loda_identify_similar_programs_repository),
        loda_patterns_repository: simpleenv.resolve_path(&loda_patterns_repository),
        loda_outlier_programs_repository: simpleenv.resolve_path(&loda_outlier_programs_repository),
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
        let tempdir = tempfile::tempdir().unwrap();
        let homedir = PathBuf::from(&tempdir.path()).join("test_30000_fallback_config");
        fs::create_dir(&homedir)?;

        let basedir = PathBuf::from(Path::new("non-existing-basedir"));
        let config: Config = config_from_toml_content(Config::default_config(), basedir, homedir);

        assert_eq!(config.basedir.to_str().unwrap(), "non-existing-basedir");
        assert_has_suffix(&config.loda_programs_repository, "/loda/programs")?;
        assert_has_suffix(&config.oeis_stripped_file, "/loda/oeis/stripped")?;
        assert_has_suffix(&config.oeis_names_file, "/loda/oeis/names")?;
        assert_has_suffix(&config.loda_rust_repository, "/git/loda-rust")?;
        assert_has_suffix(&config.loda_cpp_repository, "/git/loda-cpp")?;
        assert_has_suffix(&config.loda_cpp_executable, "/loda/bin/loda")?;
        assert_eq!(config.loda_submitted_by, "John Doe");
        assert_eq!(config.miner_metrics_listen_port, 8090);
        assert_has_suffix(&config.loda_identify_similar_programs_repository, "/git/loda-identify-similar-programs")?;
        assert_has_suffix(&config.loda_patterns_repository, "/git/loda-patterns")?;
        assert_has_suffix(&config.loda_outlier_programs_repository, "/git/loda-outlier-programs")?;

        Ok(())
    }
}
