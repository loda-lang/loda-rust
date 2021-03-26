use super::Settings;
use crate::mine::check_fixed_length_sequence::create_cache_file;
use crate::mine::dont_mine::load_dontmine_file;
use std::path::Path;
use std::collections::HashSet;

fn obtain_dontmine_program_ids() -> HashSet<u32> {
    let path = Path::new("script/data/dont_mine.csv");
    let hashset: HashSet<u32> = match load_dontmine_file(&path) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to loading the dontmine file. path: {:?} error: {:?}", path, error);
        }
    };
    println!("loaded dontmine file. number of records: {}", hashset.len());
    hashset
}

pub fn subcommand_update(_settings: &Settings) {
    println!("update begin");
    
    let program_ids_to_ignore: HashSet<u32> = obtain_dontmine_program_ids();

    let oeis_stripped_file = Path::new("/Users/neoneye/.loda/oeis/stripped");
    let destination_file = Path::new("cache/fixed_length_sequence_10terms.json");
    create_cache_file(oeis_stripped_file, destination_file, 10, &program_ids_to_ignore);

    println!("update end");
}
