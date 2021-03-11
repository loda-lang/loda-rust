use std::path::{Path,PathBuf};
use std::env;

pub struct Settings {
    pub loda_program_rootdir: PathBuf,
}

impl Settings {
    pub fn new() -> Self {
        // for (key, value) in env::vars() {
        //     println!("{}: {}", key, value);
        // }

        let loda_program_rootdir_raw: String = env::var("LODA_PROGRAM_ROOTDIR")
            .expect("LODA_PROGRAM_ROOTDIR is not set in .env file");

        let loda_program_rootdir = Path::new(&loda_program_rootdir_raw);
        assert!(loda_program_rootdir.is_absolute());
        assert!(loda_program_rootdir.is_dir());

        Self {
            loda_program_rootdir: PathBuf::from(loda_program_rootdir),
        }
    }
}
