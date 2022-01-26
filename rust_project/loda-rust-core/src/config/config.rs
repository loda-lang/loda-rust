use std::path::{Path,PathBuf};
use serde::Deserialize;
use std::fs;

const DEFAULT_CONFIG: &'static str =
r#"# Configuration for LODA Rust

# Absolute path to the "loda-programs" repository dir.
loda_programs_repository = "/Users/JOHNDOE/loda/programs"

# Absolute path to the "loda-cpp" repository dir.
loda_cpp_repository = "/Users/JOHNDOE/git/loda-cpp"

# Absolute path to the "loda" executable file.
loda_cpp_executable = "/Users/JOHNDOE/loda/bin/loda"

# Absolute path to the "loda-rust" repository dir.
loda_rust_repository = "/Users/JOHNDOE/git/loda-rust"

# Absolute path to the unzipped OEIS stripped file.
oeis_stripped_file = "/Users/JOHNDOE/loda/oeis/stripped"

# Absolute path to the unzipped OEIS names file.
oeis_names_file = "/Users/JOHNDOE/loda/oeis/names"

# Absolute path to the dir that holds the accumulated mismatches.
loda_rust_mismatches = "/Users/JOHNDOE/git/loda-rust/resources/programs/mismatch"

# Who to be credited when discovering new programs.
loda_submitted_by = "John Doe"
"#;


#[derive(Debug)]
pub struct Config {
    basedir: PathBuf,
    loda_programs_repository: String,
    loda_rust_repository: String,
    loda_cpp_repository: String,
    loda_cpp_executable: String,
    oeis_stripped_file: String,
    oeis_names_file: String,
    loda_rust_mismatches: String,
    loda_submitted_by: String,
}

impl Config {
    pub fn default_config() -> String {
        DEFAULT_CONFIG.to_string()
    }

    pub fn load() -> Self {
        load_config_from_home_dir()
    }

    pub fn cache_dir(&self) -> PathBuf {
        let name = Path::new("cache");
        let path = self.basedir.join(name);
        assert!(path.is_dir());
        path
    }

    pub fn cache_dir_dont_mine_file(&self) -> PathBuf {
        let path = self.cache_dir().join("dont_mine.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn cache_dir_histogram_instruction_constant_file(&self) -> PathBuf {
        let path = self.cache_dir().join("histogram_instruction_constant.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn cache_dir_histogram_instruction_unigram_file(&self) -> PathBuf {
        let path = self.cache_dir().join("histogram_instruction_unigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn cache_dir_histogram_instruction_bigram_file(&self) -> PathBuf {
        let path = self.cache_dir().join("histogram_instruction_bigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn cache_dir_histogram_instruction_trigram_file(&self) -> PathBuf {
        let path = self.cache_dir().join("histogram_instruction_trigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn cache_dir_histogram_instruction_skipgram_file(&self) -> PathBuf {
        let path = self.cache_dir().join("histogram_instruction_skipgram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn cache_dir_histogram_target_unigram_file(&self) -> PathBuf {
        let path = self.cache_dir().join("histogram_target_unigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn cache_dir_histogram_target_bigram_file(&self) -> PathBuf {
        let path = self.cache_dir().join("histogram_target_bigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn cache_dir_histogram_target_trigram_file(&self) -> PathBuf {
        let path = self.cache_dir().join("histogram_target_trigram.csv");
        assert!(path.is_absolute());
        PathBuf::from(path)
    }

    pub fn cache_dir_histogram_target_skipgram_file(&self) -> PathBuf {
        let path = self.cache_dir().join("histogram_target_skipgram.csv");
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
        let path = Path::new(&self.loda_programs_repository);
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
        let path = Path::new(&self.oeis_stripped_file);
        assert!(path.is_absolute());
        assert!(path.is_file());
        PathBuf::from(path)
    }

    pub fn oeis_names_file(&self) -> PathBuf {
        let path = Path::new(&self.oeis_names_file);
        assert!(path.is_absolute());
        assert!(path.is_file());
        PathBuf::from(path)
    }

    pub fn loda_rust_repository(&self) -> PathBuf {
        let path = Path::new(&self.loda_rust_repository);
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn loda_cpp_repository(&self) -> PathBuf {
        let path = Path::new(&self.loda_cpp_repository);
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn loda_cpp_executable(&self) -> PathBuf {
        let path = Path::new(&self.loda_cpp_executable);
        assert!(path.is_absolute());
        assert!(path.is_file());
        PathBuf::from(path)
    }

    pub fn loda_rust_mismatches(&self) -> PathBuf {
        let path = Path::new(&self.loda_rust_mismatches);
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn loda_submitted_by(&self) -> String {
        self.loda_submitted_by.clone()
    }
}

#[derive(Debug, Deserialize)]
struct ConfigInner {
    loda_programs_repository: String,
    oeis_stripped_file: String,
    loda_rust_repository: String,
    loda_cpp_repository: String,
    loda_cpp_executable: String,
    oeis_names_file: String,
    loda_rust_mismatches: String,
    loda_submitted_by: String,
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
    config_from_toml_content(toml_content, basedir)
}

fn config_from_toml_content(toml_content: String, basedir: PathBuf) -> Config {
    let inner: ConfigInner = toml::from_str(&toml_content).unwrap();
    Config {
        basedir: basedir,
        loda_programs_repository: inner.loda_programs_repository.clone(),
        oeis_stripped_file: inner.oeis_stripped_file.clone(),
        oeis_names_file: inner.oeis_names_file.clone(),
        loda_rust_repository: inner.loda_rust_repository.clone(),
        loda_cpp_repository: inner.loda_cpp_repository.clone(),
        loda_cpp_executable: inner.loda_cpp_executable.clone(),
        loda_rust_mismatches: inner.loda_rust_mismatches.clone(),
        loda_submitted_by: inner.loda_submitted_by.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000() {
        let basedir = PathBuf::from(Path::new("non-existing-basedir"));
        let config: Config = config_from_toml_content(Config::default_config(), basedir);
        assert_eq!(config.basedir.to_str().unwrap(), "non-existing-basedir");
        assert_eq!(config.loda_programs_repository, "/Users/JOHNDOE/loda/programs");
        assert_eq!(config.oeis_stripped_file, "/Users/JOHNDOE/loda/oeis/stripped");
        assert_eq!(config.oeis_names_file, "/Users/JOHNDOE/loda/oeis/names");
        assert_eq!(config.loda_rust_repository, "/Users/JOHNDOE/git/loda-rust");
        assert_eq!(config.loda_cpp_repository, "/Users/JOHNDOE/git/loda-cpp");
        assert_eq!(config.loda_cpp_executable, "/Users/JOHNDOE/loda/bin/loda");
        assert_eq!(config.loda_rust_mismatches, "/Users/JOHNDOE/git/loda-rust/resources/programs/mismatch");
        assert_eq!(config.loda_submitted_by, "John Doe");
    }
}
