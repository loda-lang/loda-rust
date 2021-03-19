use crate::oeis::stripped_sequence::*;
use bloomfilter::*;
use std::io;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use num_bigint::ToBigInt;

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
    

    const INPUT_STRIPPED_SEQUENCE_DATA: &str = r#"
# OEIS Sequence Data (http://oeis.org/stripped.gz)
# Last Modified: January 32 01:01 UTC 1984
# Use of this content is governed by the
# OEIS End-User License: http://oeis.org/LICENSE
A000040 ,2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,
A000045 ,0,1,1,2,3,5,8,13,21,34,55,89,144,233,377,610,987,1597,
"#;

    #[test]
    fn test_10003_populate_bloomfilter_with_oeis_data() {
        let mut input: &[u8] = INPUT_STRIPPED_SEQUENCE_DATA.as_bytes();
        let checker: CheckFixedLengthSequence = create_bloom_inner(&mut input);
        // let checker: CheckFixedLengthSequence = create_bloom_from_file();
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
}
