use std::fmt;
use loda_rust_core::util::BigIntVec;
use loda_rust_core::oeis::{OeisId, OeisIdHashSet};
use super::{FunnelConfig, WildcardChecker};
use crate::config::Config;
use crate::analytics::AnalyticsDirectory;
use crate::common::{load_program_ids_csv_file, SimpleLog};
use crate::oeis::{ProcessStrippedFile, StrippedRow};
use num_bigint::{BigInt, ToBigInt};
use serde::{Serialize, Deserialize};
use bloomfilter::*;
use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

#[derive(Clone)]
pub struct CheckFixedLengthSequence {
    bloom: Bloom::<BigIntVec>,
    bloomfilter_wildcard_magic_value: BigInt,
}

impl CheckFixedLengthSequence {
    pub fn new(bloom: Bloom::<BigIntVec>) -> Self {
        let wildcard_magic_value: BigInt = FunnelConfig::WILDCARD_MAGIC_VALUE.to_bigint().unwrap();
        Self {
            bloom: bloom,
            bloomfilter_wildcard_magic_value: wildcard_magic_value
        }
    }
    
    pub fn new_empty() -> Self {
        let bloom_items_count = 10;
        let false_positive_rate = 0.5;
        let bloom = Bloom::<BigIntVec>::new_for_fp_rate(bloom_items_count, false_positive_rate);
        let wildcard_magic_value: BigInt = FunnelConfig::WILDCARD_MAGIC_VALUE.to_bigint().unwrap();
        Self {
            bloom,
            bloomfilter_wildcard_magic_value: wildcard_magic_value
        }
    }

    /// Returns `false` if the integer sequence is unknown.
    /// The caller doesn't have to do any more checks.
    ///
    /// Returns `true` if the integer sequence is known or unknown.
    /// The caller will have to do more time consuming checks in order to determine 
    /// if the integer sequences is known or unknown.
    pub fn check(&self, bigint_vec_ref: &BigIntVec) -> bool {
        self.bloom.check(bigint_vec_ref)
    }

    fn to_representation(&self) -> CheckFixedLengthSequenceInternalRepresentation {
        CheckFixedLengthSequenceInternalRepresentation {
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

impl WildcardChecker for CheckFixedLengthSequence {
    fn bloomfilter_check(&self, bigint_vec_ref: &BigIntVec) -> bool {
        self.check(bigint_vec_ref)
    }

    fn bloomfilter_wildcard_magic_value(&self) -> &BigInt {
        &self.bloomfilter_wildcard_magic_value
    }
}

impl fmt::Debug for CheckFixedLengthSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CheckFixedLengthSequence")
            .field("bloomfilter_wildcard_magic_value", &self.bloomfilter_wildcard_magic_value)
            .finish_non_exhaustive()
    }
}

// I cannot compile the dependency "bloomfilter" with "serde" feature enabled.
// This is my kludgy workaround that can serialize/deserialize 
// all the fields of the bloomfilter.
#[derive(Serialize, Deserialize)]
struct CheckFixedLengthSequenceInternalRepresentation {
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
        let wildcard_magic_value: BigInt = FunnelConfig::WILDCARD_MAGIC_VALUE.to_bigint().unwrap();
        CheckFixedLengthSequence {
            bloom: bloom,
            bloomfilter_wildcard_magic_value: wildcard_magic_value,
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
    Funnel10All,
    Funnel20All,
    Funnel30All,
    Funnel40All,
    Funnel10New,
    Funnel20New,
    Funnel30New,
    Funnel40New,
}

impl NamedCacheFile {
    pub fn group_all() -> [NamedCacheFile; 4] {
        [Self::Funnel10All, Self::Funnel20All, Self::Funnel30All, Self::Funnel40All]
    }

    pub fn group_new() -> [NamedCacheFile; 4] {
        [Self::Funnel10New, Self::Funnel20New, Self::Funnel30New, Self::Funnel40New]
    }

    pub fn resolve_path(&self, parent_dir: &Path) -> PathBuf {
        parent_dir.join(Path::new(self.filename()))
    }

    pub fn filename(&self) -> &str {
        match self {
            Self::Funnel10All => "funnel_10_all.json",
            Self::Funnel20All => "funnel_20_all.json",
            Self::Funnel30All => "funnel_30_all.json",
            Self::Funnel40All => "funnel_40_all.json",
            Self::Funnel10New => "funnel_10_new.json",
            Self::Funnel20New => "funnel_20_new.json",
            Self::Funnel30New => "funnel_30_new.json",
            Self::Funnel40New => "funnel_40_new.json",
        }
    }
}

fn create_cache_files(
    simple_log: SimpleLog,
    oeis_stripped_file_reader: &mut dyn io::BufRead, 
    filesize: usize,
    bloom_items_count: usize,
    oeis_ids_to_ignore: &OeisIdHashSet,
    funnel10_path: &Path,
    funnel20_path: &Path,
    funnel30_path: &Path,
    funnel40_path: &Path,
) -> usize {
    let start = Instant::now();
    let mut processor = SequenceProcessor::new();
    let x = &mut processor;

    let false_positive_rate: f64 = FunnelConfig::BLOOMFILTER_FALSE_POSITIVE_RATE;
    let mut bloom10 = Bloom::<BigIntVec>::new_for_fp_rate(bloom_items_count, false_positive_rate);
    let mut bloom20 = Bloom::<BigIntVec>::new_for_fp_rate(bloom_items_count, false_positive_rate);
    let mut bloom30 = Bloom::<BigIntVec>::new_for_fp_rate(bloom_items_count, false_positive_rate);
    let mut bloom40 = Bloom::<BigIntVec>::new_for_fp_rate(bloom_items_count, false_positive_rate);
    let bloom10_ref = &mut bloom10;
    let bloom20_ref = &mut bloom20;
    let bloom30_ref = &mut bloom30;
    let bloom40_ref = &mut bloom40;

    simple_log.println(format!("oeis 'stripped' file size: {} bytes", filesize));
    let pb = ProgressBar::new(filesize as u64);
    let process_callback = |stripped_sequence: &StrippedRow, count_bytes: usize| {
        pb.set_position(count_bytes as u64);
        
        let all_vec: &BigIntVec = stripped_sequence.terms();
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
        (*x).counter += 1;
    };
    let mut stripped_sequence_processor = ProcessStrippedFile::new();
    let padding_value: BigInt = FunnelConfig::WILDCARD_MAGIC_VALUE.to_bigint().unwrap();
    stripped_sequence_processor.execute(
        oeis_stripped_file_reader,
        FunnelConfig::MINIMUM_NUMBER_OF_REQUIRED_TERMS,
        FunnelConfig::TERM_COUNT,
        oeis_ids_to_ignore, 
        &padding_value,
        true,
        process_callback
    );
    stripped_sequence_processor.print_summary(simple_log.clone());
    simple_log.println(format!("number of sequences stored in bloomfilter: {:?}", processor.counter));
    pb.finish_and_clear();

    let green_bold = Style::new().green().bold();        
    println!(
        "{:>12} populated bloomfilter in {}",
        green_bold.apply_to("Finished"),
        HumanDuration(start.elapsed())
    );

    println!("Saving bloomfilter data");
    let start2 = Instant::now();
    let pb = ProgressBar::new(4);
    {
        let instance = CheckFixedLengthSequence::new(bloom10);
        instance.save(funnel10_path);
        pb.inc(1);
    }
    {
        let instance = CheckFixedLengthSequence::new(bloom20);
        instance.save(funnel20_path);
        pb.inc(1);
    }
    {
        let instance = CheckFixedLengthSequence::new(bloom30);
        instance.save(funnel30_path);
        pb.inc(1);
    }
    {
        let instance = CheckFixedLengthSequence::new(bloom40);
        instance.save(funnel40_path);
        pb.finish_and_clear();
    }
    println!(
        "{:>12} saved bloomfilter data in {}",
        green_bold.apply_to("Finished"),
        HumanDuration(start2.elapsed())
    );

    // Number of sequences processed
    processor.counter
}

pub struct PopulateBloomfilter {
    analytics_directory: AnalyticsDirectory,
    config: Config,
    simple_log: SimpleLog,
}

impl PopulateBloomfilter {
    pub fn run(analytics_directory: AnalyticsDirectory, simple_log: SimpleLog) -> Result<(), Box<dyn Error>> {
        let config = Config::load();
        let instance = Self {
            analytics_directory,
            config,
            simple_log
        };
        instance.populate_bloomfilter_all()?;
        instance.populate_bloomfilter_new()?;
        Ok(())
    }

    fn populate_bloomfilter_all(&self) -> anyhow::Result<()> {
        println!("Populate bloomfilter - group all");
        self.simple_log.println("\nPopulateBloomfilter - group all");
        let oeis_ids_to_ignore: OeisIdHashSet = self.obtain_dontmine_program_ids()?;
        self.simple_log.println(format!("ignore total: {}", oeis_ids_to_ignore.len()));
        self.populate_bloomfilter(NamedCacheFile::group_all(), oeis_ids_to_ignore)?;
        Ok(())
    }

    fn populate_bloomfilter_new(&self) -> anyhow::Result<()> {
        println!("Populate bloomfilter - group new");
        self.simple_log.println("\nPopulateBloomfilter - group new");
        let oeis_ids_dontmine: OeisIdHashSet = self.obtain_dontmine_program_ids()?;
        let oeis_ids_invalid: OeisIdHashSet = self.obtain_invalid_program_ids()?;
        let oeis_ids_valid: OeisIdHashSet = self.obtain_valid_program_ids()?;
        let mut oeis_ids_to_ignore: OeisIdHashSet = oeis_ids_dontmine.clone();
        oeis_ids_to_ignore.extend(&oeis_ids_invalid);
        oeis_ids_to_ignore.extend(&oeis_ids_valid);
        self.simple_log.println(format!("ignore total: {} dontmine: {} valid: {} invalid: {}", oeis_ids_to_ignore.len(), oeis_ids_dontmine.len(), oeis_ids_valid.len(), oeis_ids_invalid.len()));
        self.populate_bloomfilter(NamedCacheFile::group_new(), oeis_ids_to_ignore)?;
        Ok(())
    }

    fn populate_bloomfilter(&self, names: [NamedCacheFile; 4], oeis_ids_to_ignore: OeisIdHashSet) -> anyhow::Result<()> {
        let oeis_stripped_file: PathBuf = self.config.oeis_stripped_file();
        assert!(oeis_stripped_file.is_absolute());
        assert!(oeis_stripped_file.is_file());

        let analytics_dir: PathBuf = self.config.analytics_oeis_dir();
        let funnel10_path: PathBuf = names[0].resolve_path(&analytics_dir);
        let funnel20_path: PathBuf = names[1].resolve_path(&analytics_dir);
        let funnel30_path: PathBuf = names[2].resolve_path(&analytics_dir);
        let funnel40_path: PathBuf = names[3].resolve_path(&analytics_dir);

        let file = File::open(oeis_stripped_file).unwrap();
        let filesize: usize = file.metadata().unwrap().len() as usize;
        let mut reader = BufReader::new(file);
        create_cache_files(
            self.simple_log.clone(),
            &mut reader, 
            filesize,
            FunnelConfig::BLOOMFILTER_CAPACITY,
            &oeis_ids_to_ignore,
            &funnel10_path,
            &funnel20_path,
            &funnel30_path,
            &funnel40_path,
        );
        Ok(())
    }

    fn obtain_dontmine_program_ids(&self) -> anyhow::Result<OeisIdHashSet> {
        let path = self.analytics_directory.dont_mine_file();
        let program_ids_raw: Vec<u32> = load_program_ids_csv_file(&path)
            .map_err(|e| anyhow::anyhow!("obtain_dontmine_program_ids - unable to load program_ids. error: {:?}", e))?;
        let program_ids: Vec<OeisId> = program_ids_raw.iter().map(|x| OeisId::from(*x)).collect();
        let hashset: OeisIdHashSet = HashSet::from_iter(program_ids.iter().cloned());
        debug!("loaded dontmine program_ids file. number of records: {}", hashset.len());
        Ok(hashset)
    }

    fn obtain_invalid_program_ids(&self) -> anyhow::Result<OeisIdHashSet> {
        let path = self.analytics_directory.programs_invalid_file();
        let program_ids_raw: Vec<u32> = load_program_ids_csv_file(&path)
            .map_err(|e| anyhow::anyhow!("obtain_invalid_program_ids - unable to load program_ids. error: {:?}", e))?;
        let program_ids: Vec<OeisId> = program_ids_raw.iter().map(|x| OeisId::from(*x)).collect();
        let hashset: OeisIdHashSet = HashSet::from_iter(program_ids.iter().cloned());
        debug!("loaded invalid program_ids file. number of records: {}", hashset.len());
        Ok(hashset)
    }

    fn obtain_valid_program_ids(&self) -> anyhow::Result<OeisIdHashSet> {
        let path = self.analytics_directory.programs_valid_file();
        let program_ids_raw: Vec<u32> = load_program_ids_csv_file(&path)
            .map_err(|e| anyhow::anyhow!("obtain_valid_program_ids - unable to load program_ids. error: {:?}", e))?;
        let program_ids: Vec<OeisId> = program_ids_raw.iter().map(|x| OeisId::from(*x)).collect();
        let hashset: OeisIdHashSet = HashSet::from_iter(program_ids.iter().cloned());
        debug!("loaded valid program_ids file. number of records: {}", hashset.len());
        Ok(hashset)
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
    fn test_10001_bloomfilter_few_false_positives() {
        let items_count: usize = 400000;
        let false_positive_rate: f64 = 0.01;
        let bloom = Bloom::<()>::new_for_fp_rate(items_count, false_positive_rate);
        assert_eq!(bloom.number_of_bits(), 3834024);
        assert_eq!(bloom.number_of_hash_functions(), 7);
    }

    #[test]
    fn test_10002_bloomfilter_many_false_positives() {
        let items_count: usize = 400000;
        let false_positive_rate: f64 = 0.1;
        let bloom = Bloom::<()>::new_for_fp_rate(items_count, false_positive_rate);
        assert_eq!(bloom.number_of_bits(), 1917016);
        assert_eq!(bloom.number_of_hash_functions(), 4);
    }
    
    #[test]
    fn test_10003_bloomfilter_set_check_with_hash_of_string() {
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
        minimum_number_of_required_terms: usize, 
        term_count: usize, 
        oeis_ids_to_ignore: &OeisIdHashSet,
    ) -> CheckFixedLengthSequence
    {
        let items_count: usize = 400000;
        let false_positive_rate: f64 = 0.01;
        let mut bloom = Bloom::<BigIntVec>::new_for_fp_rate(items_count, false_positive_rate);
        let bloom_ref = &mut bloom;
        let process_callback = |stripped_sequence: &StrippedRow, _count_bytes: usize| {
            let vec: &BigIntVec = stripped_sequence.terms();
            (*bloom_ref).set(vec);
        };
        let mut processor = ProcessStrippedFile::new();
        let padding_value: BigInt = FunnelConfig::WILDCARD_MAGIC_VALUE.to_bigint().unwrap();
        processor.execute(
            reader, 
            minimum_number_of_required_terms,
            term_count, 
            oeis_ids_to_ignore, 
            &padding_value,
            true,
            process_callback
        );
        CheckFixedLengthSequence::new(bloom)
    }

    impl CheckFixedLengthSequence {
        fn new_mock() -> CheckFixedLengthSequence {
            let mut input: &[u8] = INPUT_STRIPPED_SEQUENCE_MOCKDATA.as_bytes();
            let hashset = HashSet::<OeisId>::new();
            create_checkfixedlengthsequence_inner(
                &mut input,
                0,
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
        // Arrange
        let dirname = "test_30000_create_cache_files";
        let tempdir = tempfile::tempdir().unwrap();
        let mut cache_dir = PathBuf::from(&tempdir.path());
        cache_dir.push(dirname);
        fs::create_dir(&cache_dir).unwrap();
        let simple_log = SimpleLog::sink();
        let mut input: &[u8] = INPUT_STRIPPED_SEQUENCE_MOCKDATA.as_bytes();
        let filesize: usize = input.len();
        let hashset = HashSet::<OeisId>::new();

        let funnel10_path = cache_dir.join(Path::new(NamedCacheFile::Funnel10All.filename()));
        let funnel20_path = cache_dir.join(Path::new(NamedCacheFile::Funnel20All.filename()));
        let funnel30_path = cache_dir.join(Path::new(NamedCacheFile::Funnel30All.filename()));
        let funnel40_path = cache_dir.join(Path::new(NamedCacheFile::Funnel40All.filename()));

        // Act
        let number_of_sequences: usize = create_cache_files(
            simple_log,
            &mut input, 
            filesize,
            10,
            &hashset,
            &funnel10_path,
            &funnel20_path,
            &funnel30_path,
            &funnel40_path,
        );

        // Assert
        assert_eq!(number_of_sequences, 2);
        // Check that all the cache files can be loaded
        let mut file_count: usize = 0;
        for item in NamedCacheFile::group_all() {
            let path: PathBuf = item.resolve_path(&cache_dir);
            let _checker: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path);
            file_count += 1;
        }
        assert_eq!(file_count, 4);
    }
}
