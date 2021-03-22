use super::Settings;
use crate::mine::check_fixed_length_sequence::CheckFixedLengthSequence;
use std::path::Path;

pub fn subcommand_mine(_settings: &Settings) {
    println!("step1");
    let cache_file = Path::new("cache/fixed_length_sequence_5terms.json");
    let checker: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&cache_file);
    println!("step2");

    // TODO: mining
}
