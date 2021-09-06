use loda_rust_core::util::BigIntVec;
use crate::oeis::stripped_sequence::*;
use serde::{Serialize, Deserialize};
use bloomfilter::*;
use std::io;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::prelude::*;
use std::collections::HashSet;

pub struct CheckFixedLengthSequence {
    bloom: Bloom::<BigIntVec>,
    term_count: usize,
}

impl CheckFixedLengthSequence {
    pub fn new(bloom: Bloom::<BigIntVec>, term_count: usize) -> Self {
        Self {
            bloom: bloom,
            term_count: term_count,
        }
    }

    // Returns `false` if the integer sequence is unknown.
    // The caller doesn't have to do any more checks.
    //
    // Returns `true` if the integer sequence is known or unknown.
    // The caller will have to do more time consuming checks in order to determine 
    // if the integer sequences is known or unknown.
    pub fn check(&self, bigint_vec_ref: &BigIntVec) -> bool {
        self.bloom.check(bigint_vec_ref)
    }

    fn to_representation(&self) -> CheckFixedLengthSequenceInternalRepresentation {
        CheckFixedLengthSequenceInternalRepresentation {
            term_count: self.term_count,
            bloom_bitmap: self.bloom.bitmap(),
            bloom_bitmap_bits: self.bloom.number_of_bits(),
            bloom_k_num: self.bloom.number_of_hash_functions(),
            bloom_sip_keys: self.bloom.sip_keys()
        }
    }

    pub fn save(&self, path: &Path) {
        let file = match File::create(&path) {
            Ok(value) => value,
            Err(error) => {
                panic!("Unable to create file at path: {:?}, error: {:?}", path, error);
            }
        };
        let representation = self.to_representation();
        match ::serde_json::to_writer(&file, &representation) {
            Ok(_value) => {},
            Err(error) => {
                panic!("Unable to save representation to path: {:?}, error: {:?}", path, error);
            }
        };
    }

    pub fn load(path: &Path) -> Self {
        let mut data = String::new();
        let mut file = File::open(path)
            .expect("Unable to open file");
        file.read_to_string(&mut data).expect("Unable to read string");
        let representation: CheckFixedLengthSequenceInternalRepresentation = serde_json::from_str(&data).unwrap();
        representation.create_instance()
    }
}

// I cannot compile the dependency "bloomfilter" with "serde" feature enabled.
// This is my kludgy workaround that can serialize/deserialize 
// all the fields of the bloomfilter.
#[derive(Serialize, Deserialize)]
struct CheckFixedLengthSequenceInternalRepresentation {
    term_count: usize,
    bloom_bitmap: Vec<u8>,
    bloom_bitmap_bits: u64,
    bloom_k_num: u32,
    bloom_sip_keys: [(u64, u64); 2],
}

impl CheckFixedLengthSequenceInternalRepresentation {
    fn create_instance(&self) -> CheckFixedLengthSequence {
        let bloom = Bloom::<BigIntVec>::from_existing(
            &self.bloom_bitmap,
            self.bloom_bitmap_bits,
            self.bloom_k_num,
            self.bloom_sip_keys
        );
        CheckFixedLengthSequence {
            bloom: bloom,
            term_count: self.term_count,
        }
    }
}

struct SequenceProcessor {
    counter: usize,
}

impl SequenceProcessor {
    fn new() -> Self {
        Self {
            counter: 0
        }
    }
}

pub fn create_cache_file(oeis_stripped_file: &Path, destination_file: &Path, term_count: usize, program_ids_to_ignore: &HashSet<u32>) {
    assert!(oeis_stripped_file.is_absolute());
    assert!(oeis_stripped_file.is_file());

    let file = File::open(oeis_stripped_file).unwrap();
    let mut reader = BufReader::new(file);

    let mut processor = SequenceProcessor::new();
    let x = &mut processor;

    let items_count: usize = 400000;
    let false_positive_rate: f64 = 0.01;
    let mut bloom = Bloom::<BigIntVec>::new_for_fp_rate(items_count, false_positive_rate);
    let y = &mut bloom;

    let process_callback = |stripped_sequence: &StrippedSequence| {
        // debug!("call {:?}", stripped_sequence.sequence_number);
        (*x).counter += 1;

        let vec: &BigIntVec = stripped_sequence.bigint_vec_ref();
        (*y).set(vec);
    };
    create_inner(&mut reader, term_count, program_ids_to_ignore, true, process_callback);
    debug!("counter: {:?}", processor.counter);

    let instance = CheckFixedLengthSequence::new(bloom, term_count);

    println!("saving cache file: {:?}", destination_file);
    instance.save(destination_file);
}

fn create_inner<F>(
    reader: &mut dyn io::BufRead, 
    term_count: usize, 
    program_ids_to_ignore: &HashSet<u32>, 
    print_progress: bool, 
    mut callback: F
)
    where F: FnMut(&StrippedSequence)
{
    assert!(term_count >= 1);
    assert!(term_count <= 100);

    let mut count: usize = 0;
    let mut count_callback: usize = 0;
    let mut count_junk: usize = 0;
    let mut count_tooshort: usize = 0;
    let mut count_ignore: usize = 0;
    for line in reader.lines() {
        count += 1;
        if print_progress && ((count % 10000) == 0) {
            println!("progress: {}", count);
        }
        let line: String = line.unwrap();
        let stripped_sequence: StrippedSequence = match parse_stripped_sequence_line(&line, Some(term_count)) {
            Some(value) => value,
            None => {
                count_junk += 1;
                continue;
            }
        };
        if program_ids_to_ignore.contains(&stripped_sequence.sequence_number) {
            count_ignore += 1;
            continue;
        }
        if stripped_sequence.len() != term_count {
            count_tooshort += 1;
            continue;
        }
        callback(&stripped_sequence);
        count_callback += 1;
    }
    debug!("count_sequences: {}", count_callback);
    debug!("count_ignore: {}", count_ignore);
    debug!("count_tooshort: {}", count_tooshort);
    debug!("count_junk: {}", count_junk);
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use rand::prelude::*;
    use std::path::PathBuf;
    use num_bigint::ToBigInt;
    
    #[test]
    fn test_10000_bloomfilter_basic() {
        let mut rng = thread_rng();
        let mut bloom = Bloom::new(10, 80);
        let mut key = vec![0u8, 16];
        rng.fill_bytes(&mut key);
        assert!(bloom.check(&key) == false);
        bloom.set(&key);
        assert!(bloom.check(&key) == true);
    }
    
    #[test]
    fn test_10001_bloomfilter_false_positive_rate() {
        let items_count: usize = 400000;
        let false_positive_rate: f64 = 0.01;
        let bloom = Bloom::<()>::new_for_fp_rate(items_count, false_positive_rate);
        assert_eq!(bloom.number_of_bits(), 3834024);
        assert_eq!(bloom.number_of_hash_functions(), 7);
    }
    
    #[test]
    fn test_10002_bloomfilter_set_check_with_hash_of_string() {
        let mut bloom = Bloom::<String>::new_for_fp_rate(100, 0.1);
        let key = "hello".to_string();
        assert_eq!(bloom.check(&key), false);
        bloom.set(&key);
        assert_eq!(bloom.check(&key), true);
    }

    const INPUT_STRIPPED_SEQUENCE_MOCKDATA: &str = r#"
# OEIS Sequence Data (http://oeis.org/stripped.gz)
# Last Modified: January 32 01:01 UTC 1984
# Use of this content is governed by the
# OEIS End-User License: http://oeis.org/LICENSE
A000040 ,2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,
A000045 ,0,1,1,2,3,5,8,13,21,34,55,89,144,233,377,610,987,1597,
"#;

    impl CheckFixedLengthSequence {
        fn new_mock() -> CheckFixedLengthSequence {
            let mut input: &[u8] = INPUT_STRIPPED_SEQUENCE_MOCKDATA.as_bytes();
            let hashset = HashSet::<u32>::new();
            create_inner(&mut input, 5, &hashset, false)
        }

        fn check_i64(&self, ary: &Vec<i64>) -> bool {
            let ary2: BigIntVec = ary.iter().map(|value| {
                value.to_bigint().unwrap()
            }).collect();
            self.check(&ary2)
        }
    }

    #[test]
    fn test_20000_populate_with_oeis_mockdata() {
        let checker = CheckFixedLengthSequence::new_mock();
        {
            assert_eq!(checker.check_i64(&vec!(2,3,5,7,11)), true);
            assert_eq!(checker.check_i64(&vec!(0,1,1,2,3)), true);
        }

        let sequence_array: Vec<Vec<i64>> = vec!(
            vec!(1,2,3,4,5),
            vec!(0,0,0,0,0),
            vec!(0,1,0,1,0),
            vec!(0,1,10,11,100),
            vec!(0,2,4,8,10),
        );
        let mut count: usize = 0;
        for seq in sequence_array {
            if checker.check_i64(&seq) {
                count += 1;
            }
        }
        assert_eq!(count, 0);
    }

    #[test]
    fn test_20001_save_load() {
        let filename = "test_10004_save_load.json";
        let tempdir = tempfile::tempdir().unwrap();
        let mut path = PathBuf::from(&tempdir.path());
        path.push(filename);

        {
            let checker_original = CheckFixedLengthSequence::new_mock();
            checker_original.save(&path);
        }

        let checker: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path);
        {
            assert_eq!(checker.check_i64(&vec!(2,3,5,7,11)), true);
            assert_eq!(checker.check_i64(&vec!(0,1,1,2,3)), true);
        }

        let sequence_array: Vec<Vec<i64>> = vec!(
            vec!(1,2,3,4,5),
            vec!(0,0,0,0,0),
            vec!(0,1,0,1,0),
            vec!(0,1,10,11,100),
            vec!(0,2,4,8,10),
        );
        let mut count: usize = 0;
        for seq in sequence_array {
            if checker.check_i64(&seq) {
                count += 1;
            }
        }
        assert_eq!(count, 0);
    }
}
