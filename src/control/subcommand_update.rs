use super::Settings;
use crate::mine::check_fixed_length_sequence::create_cache_file;
use std::path::Path;

pub fn subcommand_update(_settings: &Settings) {
    println!("updating cache");
    let oeis_stripped_file = Path::new("/Users/neoneye/.loda/oeis/stripped");
    let destination_file = Path::new("cache/fixed_length_sequence_5terms.json");
    create_cache_file(oeis_stripped_file, destination_file, 5);
}
