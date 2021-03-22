use crate::oeis::stripped_sequence::*;
use serde::{Serialize, Deserialize};
use bloomfilter::*;
use std::io;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use num_bigint::ToBigInt;
use std::io::prelude::*;

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

    pub fn save(&self, filename: &String) {
        let file = match File::create(&filename) {
            Ok(value) => value,
            Err(error) => {
                panic!("Unable to create file at path: {}, error: {:?}", filename, error);
            }
        };
        let representation = self.to_representation();
        match ::serde_json::to_writer(&file, &representation) {
            Ok(_value) => {},
            Err(error) => {
                panic!("Unable to save representation to path: {}, error: {:?}", filename, error);
            }
        };
    }

    pub fn load(filename: &String) -> Self {
        let mut data = String::new();
        let mut file = File::open(filename)
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

fn create_bloom_from_file() -> CheckFixedLengthSequence {
    let oeis_stripped_file = Path::new("/Users/neoneye/.loda/oeis/stripped");
    assert!(oeis_stripped_file.is_absolute());
    assert!(oeis_stripped_file.is_file());

    let file = File::open(oeis_stripped_file).unwrap();
    let mut reader = BufReader::new(file);
    create_bloom_inner(&mut reader)
}

fn create_bloom_inner(reader: &mut dyn io::BufRead) -> CheckFixedLengthSequence {
    let items_count: usize = 400000;
    let false_positive_rate: f64 = 0.01;
    let mut bloom = Bloom::<BigIntVec>::new_for_fp_rate(items_count, false_positive_rate);

    let term_count: usize = 5;
    let mut line_count_sequences: usize = 0;
    let mut line_count_junk: usize = 0;
    for line in reader.lines() {
        let line: String = line.unwrap();
        match parse_stripped_sequence_line(&line, Some(term_count)) {
            Some(line) => { 
                let vec: &BigIntVec = line.bigint_vec_ref();
                if vec.len() != term_count {
                    line_count_junk += 1;
                } else {
                    bloom.set(vec);
                    line_count_sequences += 1;
                }
            },
            None => {
                line_count_junk += 1;
            }
        }
        if line_count_sequences > 100 {
            break;
        }
    }
    debug!("line_count_sequences: {}", line_count_sequences);
    debug!("line_count_junk: {}", line_count_junk);

    CheckFixedLengthSequence::new(bloom, term_count)
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use rand::prelude::*;
    
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

    #[test]
    fn test_10003_populate_with_oeis_mockdata() {
        let mut input: &[u8] = INPUT_STRIPPED_SEQUENCE_MOCKDATA.as_bytes();
        let checker: CheckFixedLengthSequence = create_bloom_inner(&mut input);
        {
            let ary: Vec<i64> = vec!(2,3,5,7,11);
            let ary2: BigIntVec = ary.iter().map(|value| {
                value.to_bigint().unwrap()
            }).collect();
            assert_eq!(checker.check(&ary2), true);
        }
        {
            let ary: Vec<i64> = vec!(0,1,1,2,3);
            let ary2: BigIntVec = ary.iter().map(|value| {
                value.to_bigint().unwrap()
            }).collect();
            assert_eq!(checker.check(&ary2), true);
        }


        {
            let ary: Vec<i64> = vec!(1,2,3,4,5);
            let ary2: BigIntVec = ary.iter().map(|value| {
                value.to_bigint().unwrap()
            }).collect();
            assert_eq!(checker.check(&ary2), false);
        }
    }

    #[test]
    fn test_10004_save_load() {
        let filename: String = "cache/checkfixedlengthsequence_5terms.json".to_string();
        let checker1: CheckFixedLengthSequence = create_bloom_from_file();
        checker1.save(&filename);

        let checker2: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&filename);
    }
}
