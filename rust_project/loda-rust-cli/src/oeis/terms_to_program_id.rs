use loda_rust_core;
use loda_rust_core::util::{BigIntVec, BigIntVecToString};
use super::{OeisId, ProcessStrippedFile, StrippedRow};
use num_bigint::BigInt;
use std::io;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;

pub type TermsToProgramIdSet = HashMap::<String, HashSet<u32>>;

pub fn load_terms_to_program_id_set(
    oeis_stripped_file: &Path,
    minimum_number_of_required_terms: usize,
    term_count: usize,
    padding_value: &BigInt, 
) -> Result<TermsToProgramIdSet, Box<dyn Error>> {
    let file = File::open(oeis_stripped_file)?;
    let mut reader = BufReader::new(file);
    build_terms_to_program_id_set(
        &mut reader, 
        minimum_number_of_required_terms, 
        term_count,
        padding_value
    )
}

fn build_terms_to_program_id_set(
    oeis_stripped_file_reader: &mut dyn io::BufRead,
    minimum_number_of_required_terms: usize,
    term_count: usize,
    padding_value: &BigInt, 
) -> Result<TermsToProgramIdSet, Box<dyn Error>> {
    let mut terms_to_program_id = TermsToProgramIdSet::new();

    let callback = |row: &StrippedRow, _| {
        let bigint_vec_ref: &BigIntVec = row.bigint_vec_ref();
        let key: String = bigint_vec_ref.to_compact_comma_string();
        let entry = terms_to_program_id.entry(key).or_insert_with(|| HashSet::new());
        entry.insert(row.oeis_id().raw());
    };
    let mut processor = ProcessStrippedFile::new();
    let oeis_ids_to_ignore = HashSet::<OeisId>::new();
    processor.execute(
        oeis_stripped_file_reader,
        minimum_number_of_required_terms,
        term_count,
        &oeis_ids_to_ignore,
        padding_value, 
        true,
        callback
    );
    debug!("number of items in terms_to_program_id: {}", terms_to_program_id.len());
    Ok(terms_to_program_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Zero;

    const INPUT_STRIPPED_SEQUENCE_MOCKDATA: &str = r#"
# OEIS Sequence Data (http://oeis.org/stripped.gz)
# Last Modified: January 32 01:01 UTC 1984
# Use of this content is governed by the
# OEIS End-User License: http://oeis.org/LICENSE
A000040 ,2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97,101,103,107,109,113,127,131,137,139,149,151,157,163,167,173,179,181,191,193,197,199,211,223,227,229,233,239,241,251,257,263,269,271,
A000045 ,0,1,1,2,3,5,8,13,21,34,55,89,144,233,377,610,987,1597,2584,4181,6765,10946,17711,28657,46368,75025,121393,196418,317811,514229,832040,1346269,2178309,3524578,5702887,9227465,14930352,24157817,39088169,63245986,102334155,
A112088 ,2,3,5,7,11,16,24,36,54,81,122,183,274,411,617,925,1388,2082,3123,4684,7026,10539,15809,23713,35570,53355,80032,120048,180072,270108,405162,607743,911615,1367422,2051133,3076700,4615050,6922575,10383862,
A117093 ,2,3,5,7,11,13,16,17,18,19,23,28,29,30,31,37,38,39,40,41,43,47,53,58,59,61,67,71,72,73,78,79,81,82,83,88,89,95,96,97,98,99,100,
"#;

    fn lookup(dict: &TermsToProgramIdSet, key: &str) -> String {
        let program_id_set: HashSet<u32> = match dict.get(key) {
            Some(value) => value.clone(),
            None => {
                return "no value for key".to_string();
            }
        };
        let mut program_id_vec: Vec<u32> = program_id_set.into_iter().collect();
        program_id_vec.sort();
        let program_id_strings: Vec<String> = program_id_vec.iter().map(|program_id| program_id.to_string()).collect();
        program_id_strings.join(",")
    }

    #[test]
    fn test_10000_build_terms_to_program_id_set() -> Result<(), Box<dyn Error>> {
        // Arrange
        let mut input: &[u8] = INPUT_STRIPPED_SEQUENCE_MOCKDATA.as_bytes();
        let padding_value = BigInt::zero();
        // Act
        let dict = build_terms_to_program_id_set(
            &mut input, 
            0, 
            5,
            &padding_value
        )?;
        // Assert
        assert_eq!(dict.len(), 2);
        assert_eq!(lookup(&dict, "2,3,5,7,11"), "40,112088,117093");
        assert_eq!(lookup(&dict, "0,1,1,2,3"), "45");
        assert_eq!(lookup(&dict, "non-existing"), "no value for key");
        Ok(())
    }
}
