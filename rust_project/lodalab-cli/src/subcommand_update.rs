use lodalab_core;
use lodalab_core::config::Config;
use lodalab_core::mine::{create_cache_file, load_program_ids_csv_file};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::iter::FromIterator;

fn obtain_dontmine_program_ids(loda_rust_repository: &Path) -> HashSet<u32> {
    let relative_path = Path::new("resources/dont_mine.csv");
    let path = loda_rust_repository.join(relative_path);

    let program_ids: Vec<u32> = match load_program_ids_csv_file(&path) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load the dontmine file. path: {:?} error: {:?}", path, error);
        }
    };
    let hashset: HashSet<u32> = HashSet::from_iter(program_ids.iter().cloned());
    println!("loaded dontmine file. number of records: {}", hashset.len());
    hashset
}

pub fn subcommand_update() {
    let config = Config::load();
    let oeis_stripped_file: PathBuf = config.oeis_stripped_file();
    let cache_dir: PathBuf = config.cache_dir();
    let loda_rust_repository: PathBuf = config.loda_rust_repository();

    println!("update begin");
    
    let program_ids_to_ignore: HashSet<u32> = obtain_dontmine_program_ids(&loda_rust_repository);

    {
        let destination_file = cache_dir.join(Path::new("fixed_length_sequence_10terms.json"));
        create_cache_file(&oeis_stripped_file, &destination_file, 10, &program_ids_to_ignore);
    }
    {
        let destination_file = cache_dir.join(Path::new("fixed_length_sequence_20terms.json"));
        create_cache_file(&oeis_stripped_file, &destination_file, 20, &program_ids_to_ignore);
    }
    {
        let destination_file = cache_dir.join(Path::new("fixed_length_sequence_30terms.json"));
        create_cache_file(&oeis_stripped_file, &destination_file, 30, &program_ids_to_ignore);
    }
    {
        let destination_file = cache_dir.join(Path::new("fixed_length_sequence_40terms.json"));
        create_cache_file(&oeis_stripped_file, &destination_file, 40, &program_ids_to_ignore);
    }

    println!("update end");
}
