use loda_rust_core::util::BigIntVec;
use loda_rust_core::config::Config;
use super::load_program_ids_csv_file;
use crate::oeis::stripped_sequence::*;
use serde::{Serialize, Deserialize};
use bloomfilter::*;
use std::io;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::prelude::*;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Instant;

// As of january 2022, the OEIS contains around 350k sequences.
// So an approx count of 400k and there should be room for them all.
static APPROX_BLOOM_ITEMS_COUNT: usize = 400000;

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

pub enum NamedCacheFile {
    Bloom10Terms,
    Bloom20Terms,
    Bloom30Terms,
    Bloom40Terms
}

impl NamedCacheFile {
    #[allow(dead_code)]
    fn all() -> Vec<NamedCacheFile> {
        vec!(Self::Bloom10Terms, Self::Bloom20Terms, Self::Bloom30Terms, Self::Bloom40Terms)
    }

    pub fn filename(&self) -> &str {
        match self {
            Self::Bloom10Terms => "fixed_length_sequence_10terms.json",
            Self::Bloom20Terms => "fixed_length_sequence_20terms.json",
            Self::Bloom30Terms => "fixed_length_sequence_30terms.json",
            Self::Bloom40Terms => "fixed_length_sequence_40terms.json"
        }
    }
}

fn create_cache_files(oeis_stripped_file: &Path, cache_dir: &PathBuf, program_ids_to_ignore: &HashSet<u32>) {
    assert!(oeis_stripped_file.is_absolute());
    assert!(oeis_stripped_file.is_file());

    let file = File::open(oeis_stripped_file).unwrap();
    let filesize: usize = file.metadata().unwrap().len() as usize;
    let mut reader = BufReader::new(file);
    create_cache_files_inner(
        &mut reader, 
        filesize,
        APPROX_BLOOM_ITEMS_COUNT, 
        cache_dir, 
        program_ids_to_ignore
    );
}

fn create_cache_files_inner(
    oeis_stripped_file_reader: &mut dyn io::BufRead, 
    filesize: usize,
    bloom_items_count: usize,
    cache_dir: &PathBuf, 
    program_ids_to_ignore: &HashSet<u32>
) -> usize {
    let mut processor = SequenceProcessor::new();
    let x = &mut processor;

    let false_positive_rate: f64 = 0.01;
    let mut bloom10 = Bloom::<BigIntVec>::new_for_fp_rate(bloom_items_count, false_positive_rate);
    let mut bloom20 = Bloom::<BigIntVec>::new_for_fp_rate(bloom_items_count, false_positive_rate);
    let mut bloom30 = Bloom::<BigIntVec>::new_for_fp_rate(bloom_items_count, false_positive_rate);
    let mut bloom40 = Bloom::<BigIntVec>::new_for_fp_rate(bloom_items_count, false_positive_rate);
    let bloom10_ref = &mut bloom10;
    let bloom20_ref = &mut bloom20;
    let bloom30_ref = &mut bloom30;
    let bloom40_ref = &mut bloom40;

    let process_callback = |stripped_sequence: &StrippedSequence| {
        // debug!("call {:?}", stripped_sequence.sequence_number);
        (*x).counter += 1;

        let all_vec: &BigIntVec = stripped_sequence.bigint_vec_ref();
        {
            let vec: BigIntVec = all_vec[0..10].to_vec();
            (*bloom10_ref).set(&vec);
        }
        {
            let vec: BigIntVec = all_vec[0..20].to_vec();
            (*bloom20_ref).set(&vec);
        }
        {
            let vec: BigIntVec = all_vec[0..30].to_vec();
            (*bloom30_ref).set(&vec);
        }
        {
            let vec: BigIntVec = all_vec[0..40].to_vec();
            (*bloom40_ref).set(&vec);
        }
    };
    let term_count: usize = 40;
    process_stripped_sequence_file(
        oeis_stripped_file_reader, 
        filesize,
        term_count, 
        program_ids_to_ignore, 
        true, 
        process_callback
    );
    debug!("number of sequences processed: {:?}", processor.counter);

    {
        let instance = CheckFixedLengthSequence::new(bloom10, term_count);
        let filename: &str = NamedCacheFile::Bloom10Terms.filename();
        let destination_file = cache_dir.join(Path::new(filename));
        println!("saving cache file: {:?}", destination_file);
        instance.save(&destination_file);
    }
    {
        let instance = CheckFixedLengthSequence::new(bloom20, term_count);
        let filename: &str = NamedCacheFile::Bloom20Terms.filename();
        let destination_file = cache_dir.join(Path::new(filename));
        println!("saving cache file: {:?}", destination_file);
        instance.save(&destination_file);
    }
    {
        let instance = CheckFixedLengthSequence::new(bloom30, term_count);
        let filename: &str = NamedCacheFile::Bloom30Terms.filename();
        let destination_file = cache_dir.join(Path::new(filename));
        println!("saving cache file: {:?}", destination_file);
        instance.save(&destination_file);
    }
    {
        let instance = CheckFixedLengthSequence::new(bloom40, term_count);
        let filename: &str = NamedCacheFile::Bloom40Terms.filename();
        let destination_file = cache_dir.join(Path::new(filename));
        println!("saving cache file: {:?}", destination_file);
        instance.save(&destination_file);
    }

    // Number of sequences processed
    processor.counter
}

fn process_stripped_sequence_file<F>(
    reader: &mut dyn io::BufRead, 
    filesize: usize,
    term_count: usize, 
    program_ids_to_ignore: &HashSet<u32>, 
    print_progress: bool, 
    mut callback: F
)
    where F: FnMut(&StrippedSequence)
{
    assert!(filesize >= 1);
    assert!(term_count >= 1);
    assert!(term_count <= 100);
    if print_progress {
        println!("number of bytes to be processed: {}", filesize);
    }
    let mut count_callback: usize = 0;
    let mut count_junk: usize = 0;
    let mut count_tooshort: usize = 0;
    let mut count_ignore: usize = 0;
    let mut count_bytes: usize = 0;
    let mut progress_time = Instant::now();
    for line in reader.lines() {
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if print_progress && elapsed >= 1000 {
            let percent: usize = (count_bytes * 100) / filesize;
            println!("progress: {}%  {} of {}", percent, count_bytes, filesize);
            progress_time = Instant::now();
        }
        let line: String = line.unwrap();
        count_bytes += line.len();
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

pub struct PopulateBloomfilter {}

impl PopulateBloomfilter {
    pub fn run() {
        let start_time = Instant::now();
        println!("populate_bloomfilter begin");
        let config = Config::load();
        let oeis_stripped_file: PathBuf = config.oeis_stripped_file();
        let cache_dir: PathBuf = config.analytics_dir();
        let program_ids_to_ignore: HashSet<u32> = Self::obtain_dontmine_program_ids();
        create_cache_files(&oeis_stripped_file, &cache_dir, &program_ids_to_ignore);
    
        println!("populate_bloomfilter end, elapsed: {:?} ms", start_time.elapsed().as_millis());
    }

    fn obtain_dontmine_program_ids() -> HashSet<u32> {
        let config = Config::load();
        let path = config.analytics_dir_dont_mine_file();
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
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use rand::prelude::*;
    use std::path::PathBuf;
    use num_bigint::ToBigInt;
    use std::fs;
    
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
A000040 ,2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97,101,103,107,109,113,127,131,137,139,149,151,157,163,167,173,179,181,191,193,197,199,211,223,227,229,233,239,241,251,257,263,269,271,
A000045 ,0,1,1,2,3,5,8,13,21,34,55,89,144,233,377,610,987,1597,2584,4181,6765,10946,17711,28657,46368,75025,121393,196418,317811,514229,832040,1346269,2178309,3524578,5702887,9227465,14930352,24157817,39088169,63245986,102334155,
"#;

    fn create_checkfixedlengthsequence_inner(
        reader: &mut dyn io::BufRead, 
        filesize: usize,
        term_count: usize, 
        program_ids_to_ignore: &HashSet<u32>, 
    ) -> CheckFixedLengthSequence
    {
        let items_count: usize = 400000;
        let false_positive_rate: f64 = 0.01;
        let mut bloom = Bloom::<BigIntVec>::new_for_fp_rate(items_count, false_positive_rate);
        let bloom_ref = &mut bloom;
        let process_callback = |stripped_sequence: &StrippedSequence| {
            let vec: &BigIntVec = stripped_sequence.bigint_vec_ref();
            (*bloom_ref).set(vec);
        };
        process_stripped_sequence_file(
            reader, 
            filesize,
            term_count, 
            program_ids_to_ignore, 
            false, 
            process_callback
        );
        CheckFixedLengthSequence::new(bloom, term_count)
    }

    impl CheckFixedLengthSequence {
        fn new_mock() -> CheckFixedLengthSequence {
            let mut input: &[u8] = INPUT_STRIPPED_SEQUENCE_MOCKDATA.as_bytes();
            let filesize: usize = input.len();
            let hashset = HashSet::<u32>::new();
            create_checkfixedlengthsequence_inner(
                &mut input, 
                filesize,
                5, 
                &hashset
            )
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
        let filename = "test_20001_save_load.json";
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

    #[test]
    fn test_30000_create_cache_files() {
        let dirname = "test_30000_create_cache_files";
        let tempdir = tempfile::tempdir().unwrap();
        let mut cache_dir = PathBuf::from(&tempdir.path());
        cache_dir.push(dirname);
        fs::create_dir(&cache_dir).unwrap();

        // Create cache files
        let mut input: &[u8] = INPUT_STRIPPED_SEQUENCE_MOCKDATA.as_bytes();
        let filesize: usize = input.len();
        let hashset = HashSet::<u32>::new();
        let number_of_sequences: usize = create_cache_files_inner(
            &mut input, 
            filesize,
            10,
            &cache_dir,
            &hashset
        );
        assert_eq!(number_of_sequences, 2);

        // Check that all the cache files can be loaded
        let mut file_count: usize = 0;
        for item in NamedCacheFile::all() {
            let filename: &str = item.filename();
            let path: PathBuf = cache_dir.join(Path::new(filename));
            let _checker: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path);
            file_count += 1;
        }
        assert_eq!(file_count, 4);
    }
}
