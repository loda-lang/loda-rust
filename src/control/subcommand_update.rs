use crate::config::Config;
use crate::mine::check_fixed_length_sequence::create_cache_file;
use crate::mine::dont_mine::load_dontmine_file;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

fn obtain_dontmine_program_ids(loda_lab_repository: &Path) -> HashSet<u32> {
    let relative_path = Path::new("resources/dont_mine.csv");
    let path = loda_lab_repository.join(relative_path);

    let hashset: HashSet<u32> = match load_dontmine_file(&path) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to loading the dontmine file. path: {:?} error: {:?}", path, error);
        }
    };
    println!("loaded dontmine file. number of records: {}", hashset.len());
    hashset
}

pub fn subcommand_update() {
    let config = Config::load();
    let oeis_stripped_file: PathBuf = config.oeis_stripped_file();
    let cache_dir: PathBuf = config.cache_dir();
    let loda_lab_repository: PathBuf = config.loda_lab_repository();

    println!("update begin");
    
    let program_ids_to_ignore: HashSet<u32> = obtain_dontmine_program_ids(&loda_lab_repository);

    {
        let destination_file = cache_dir.join(Path::new("fixed_length_sequence_10terms.json"));
        create_cache_file(&oeis_stripped_file, &destination_file, 10, &program_ids_to_ignore);
    }
    {
        let destination_file = cache_dir.join(Path::new("fixed_length_sequence_20terms.json"));
        create_cache_file(&oeis_stripped_file, &destination_file, 20, &program_ids_to_ignore);
    }

    println!("update end");
}
