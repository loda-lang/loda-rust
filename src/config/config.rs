use std::path::{Path,PathBuf};
use serde::Deserialize;
use toml::de::Error;
use std::fs;

#[derive(Debug)]
pub struct Config {
    loda_program_rootdir: String,
    oeis_stripped_file: String,
}

impl Config {
    pub fn load() -> Self {
        let config_inner = ConfigInner::load_from_home_dir();
        config_inner.to_config()
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
}

#[derive(Debug, Deserialize)]
struct ConfigInner {
    loda_program_rootdir: String,
    oeis_stripped_file: String,
}

impl ConfigInner {
    fn load_from_home_dir() -> Self {
        let homedir: PathBuf = match std::env::home_dir() {
            Some(value) => value,
            None => {
                panic!("Unable to get home_dir!");
            }
        };
        assert!(homedir.is_dir());
        assert!(homedir.is_absolute());
    
        let basedir_name = Path::new(".loda-lab");
        let basedir: PathBuf = homedir.join(basedir_name);
        if !basedir.is_dir() {
            panic!("Expected a '$HOME/.loda-lab' directory, but got something else. {:?}, Possible solution, remove the thing that uses the same name.", basedir);
        }
        let path_to_config: PathBuf = basedir.join(Path::new("config.toml"));
        if !path_to_config.is_file() {
            panic!("Cannot locate the file '$HOME/.loda-lab/config.toml'");
        }
    
        let toml_content: String = fs::read_to_string(path_to_config).unwrap();
        let config: ConfigInner = toml::from_str(&toml_content).unwrap();

        config
    }

    fn to_config(&self) -> Config {
        Config {
            loda_program_rootdir: self.loda_program_rootdir.clone(),
            oeis_stripped_file: self.oeis_stripped_file.clone(),
        }
    }
}
