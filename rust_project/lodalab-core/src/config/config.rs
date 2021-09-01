use std::path::{Path,PathBuf};
use serde::Deserialize;
use std::fs;

const DEFAULT_CONFIG: &'static str =
r#"# Configuration for LODA Rust

# Absolute path to the LODA Cpp repository dir.
loda_cpp_repository = "/Users/JOHNDOE/git/loda-cpp"

# Absolute path to the LODA Rust repository dir.
loda_rust_repository = "/Users/JOHNDOE/git/loda-rust"

# Absolute path to the dir that contains all the LODA programs repository's "oeis" dir.
loda_program_rootdir = "/Users/JOHNDOE/git/loda-programs/oeis"

# Absolute path to the unzipped OEIS stripped file.
oeis_stripped_file = "/Users/JOHNDOE/.loda/oeis/stripped"
"#;


#[derive(Debug)]
pub struct Config {
    basedir: PathBuf,
    loda_program_rootdir: String,
    oeis_stripped_file: String,
    loda_rust_repository: String,
    loda_cpp_repository: String,
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

    pub fn mine_event_dir(&self) -> PathBuf {
        let name = Path::new("mine-event");
        let path = self.basedir.join(name);
        assert!(path.is_dir());
        path
    }

    pub fn loda_program_rootdir(&self) -> PathBuf {
        let path = Path::new(&self.loda_program_rootdir);
        assert!(path.is_absolute());
        assert!(path.is_dir());
        PathBuf::from(path)
    }

    pub fn oeis_stripped_file(&self) -> PathBuf {
        let path = Path::new(&self.oeis_stripped_file);
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
}

#[derive(Debug, Deserialize)]
struct ConfigInner {
    loda_program_rootdir: String,
    oeis_stripped_file: String,
    loda_rust_repository: String,
    loda_cpp_repository: String,
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
        loda_program_rootdir: inner.loda_program_rootdir.clone(),
        oeis_stripped_file: inner.oeis_stripped_file.clone(),
        loda_rust_repository: inner.loda_rust_repository.clone(),
        loda_cpp_repository: inner.loda_cpp_repository.clone(),
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
        assert_eq!(config.loda_program_rootdir, "/Users/JOHNDOE/git/loda-programs/oeis");
        assert_eq!(config.oeis_stripped_file, "/Users/JOHNDOE/.loda/oeis/stripped");
        assert_eq!(config.loda_rust_repository, "/Users/JOHNDOE/git/loda-rust");
        assert_eq!(config.loda_cpp_repository, "/Users/JOHNDOE/git/loda-cpp");
    }
}
