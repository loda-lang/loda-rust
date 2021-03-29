use std::path::{Path,PathBuf};
use std::fs;
use std::fs::File;
use std::io::prelude::*;

pub fn subcommand_install() {
    // Obtain $HOME environment variable
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

    // Create basedir if needed.
    if !basedir.exists() {
        match fs::create_dir(&basedir) {
            Ok(_) => {},
            Err(err) => {
                panic!("Unable to create directory in homedir: {:?}, error: {:?}", basedir, err);
            }
        }
    }
    if !basedir.is_dir() {
        panic!("Cannot install. Expected a directory, but got something else. {:?}, Possible solution, remove the thing that uses the same name.", basedir);
    }

    let mut number_of_issues = 0;
    // Create readme.txt if needed.
    if let Err(error) = create_readme_in_basedir(&basedir) {
        error!("Unable to create 'readme.txt' file, error: {:?}", error);
        number_of_issues += 1;
    }

    // Create config.toml if needed.
    if let Err(error) = create_config_in_basedir(&basedir) {
        error!("Unable to create 'config.toml' file, error: {:?}", error);
        number_of_issues += 1;
    }

    // Create `cache/` dir if needed.
    if let Err(error) = create_cache_dir_in_basedir(&basedir) {
        error!("Unable to create 'cache' dir, error: {:?}", error);
        number_of_issues += 1;
    }

    // Create `mine-event/` dir if needed.
    if let Err(error) = create_mine_event_dir_in_basedir(&basedir) {
        error!("Unable to create 'mine-event' dir, error: {:?}", error);
        number_of_issues += 1;
    }

    if number_of_issues > 0 {
        error!("Installation problems occurred. Try delete the '$HOME/.loda-lab' and try again.");
        return;
    }
    
    println!("install success");
}

fn create_readme_in_basedir(basedir: &Path) -> std::io::Result<()> {
    let path: PathBuf = basedir.join(Path::new("readme.txt"));
    if path.is_file() {
        return Ok(());
    }

    let content = 
r#"The directory `.loda-lab` holds the config+data for LODA Lab.

https://github.com/neoneye/loda-lab
"#;

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn create_config_in_basedir(basedir: &Path) -> std::io::Result<()> {
    let path: PathBuf = basedir.join(Path::new("config.toml"));
    if path.is_file() {
        return Ok(());
    }

    let content = 
r#"# Configuration for LODA Lab

# Absolute path to the dir that contains all the LODA programs.
loda_program_rootdir = "/Users/JOHNDOE/git/loda/programs/oeis"

# Absolute path to the unzipped OEIS stripped file.
oeis_stripped_file = "/Users/JOHNDOE/.loda/oeis/stripped"
"#;

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn create_cache_dir_in_basedir(basedir: &Path) -> std::io::Result<()> {
    let path: PathBuf = basedir.join(Path::new("cache"));
    if path.is_dir() {
        return Ok(());
    }
    fs::create_dir(path)
}

fn create_mine_event_dir_in_basedir(basedir: &Path) -> std::io::Result<()> {
    let path: PathBuf = basedir.join(Path::new("mine-event"));
    if path.is_dir() {
        return Ok(());
    }
    fs::create_dir(path)
}
