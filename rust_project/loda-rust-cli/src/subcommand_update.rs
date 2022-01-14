use loda_rust_core;
use loda_rust_core::config::Config;
use crate::mine::{create_cache_files, load_program_ids_csv_file};
use crate::mine::validate_programs;
use crate::mine::{DontMine, HistogramInstructionConstantAnalyzer};
use std::path::PathBuf;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Instant;

fn obtain_dontmine_program_ids() -> HashSet<u32> {
    let config = Config::load();
    let path = config.cache_dir_dont_mine_file();
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

fn populate_bloomfilter() {
    let start_time = Instant::now();
    println!("populate_bloomfilter begin");
    let config = Config::load();
    let oeis_stripped_file: PathBuf = config.oeis_stripped_file();
    let cache_dir: PathBuf = config.cache_dir();
    let program_ids_to_ignore: HashSet<u32> = obtain_dontmine_program_ids();
    create_cache_files(&oeis_stripped_file, &cache_dir, &program_ids_to_ignore);

    println!("populate_bloomfilter end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}

pub fn subcommand_update() {
    let start_time = Instant::now();
    println!("update begin");
    DontMine::run();
    HistogramInstructionConstantAnalyzer::run();
    let _ = validate_programs();
    populate_bloomfilter();
    println!("update end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
