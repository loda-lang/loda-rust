use crate::common::{find_asm_files_recursively, program_id_from_path};
use crate::common::RecordBigram;
use crate::similar::Word;
use crate::similar::WordPair;
use crate::similar::WordsFromProgram;
use loda_rust_core::parser::ParsedProgram;
use loda_rust_core::config::Config;
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::collections::HashSet;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;
use bit_set::BitSet;
use csv::WriterBuilder;
use std::error::Error;
use serde::Serialize;

const SIGNATURE_LENGTH: u8 = 30;
const IGNORE_INPUT_PROGRAM_IF_INSTRUCTION_COUNT_EXCEEDS: usize = 300;
const MAX_NUMBER_OF_ROWS_IN_OUTPUT_CSV_FILE: usize = 25;
const INTERVAL_UNTIL_NEXT_PROGRESS: u128 = 1000;

// What is LSH
// https://en.wikipedia.org/wiki/Locality-sensitive_hashing
//
// My first attempt was slow. It was coded in ruby and took 8 hours to compute similarity 
// for 1600 programs out of 41000 programs.
// For this I used the unix `diff` tool for identifying the number of identical lines.
// If there were more than 80% identical lines, then I added it to the csv file.
// 
// My second attempt was using LSH/minhash. It was coded in ruby.
// All 41000 programs in the loda-programs repository can be traversed in about 2 hours.
// 
// My third attempt at finding similarity between programs. This time using Rust for better preformance.
// And using the LSH/minhash algorithm.
// All 67000 programs in the loda-programs repository can be traversed in about 7 minutes.
pub fn subcommand_similar() {
    let start_time = Instant::now();

    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let output_rootdir: PathBuf = config.similarity_repository_oeis();

    let instruction_bigram_csv: PathBuf = config.cache_dir_histogram_instruction_bigram_file();
    let instruction_vec: Vec<RecordBigram> = RecordBigram::parse_csv(&instruction_bigram_csv).expect("Unable to load instruction bigram csv");
    let number_of_bigram_rows: usize = instruction_vec.len();
    debug!("number of bigram rows: {}", number_of_bigram_rows);
    let wordpair_vec = WordPair::convert(instruction_vec);
    assert!(wordpair_vec.len() == number_of_bigram_rows);
    let mut wordpair_to_index = HashMap::<WordPair,u16>::new();
    for (index, wordpair) in wordpair_vec.iter().enumerate() {
        wordpair_to_index.insert(*wordpair, index as u16);
    }
    assert!(wordpair_to_index.len() == number_of_bigram_rows);

    let indexes_array = IndexesArray::new(number_of_bigram_rows as u16, SIGNATURE_LENGTH);


    let mut paths: Vec<PathBuf> = find_asm_files_recursively(&loda_programs_oeis_dir);
    paths.sort();
    let number_of_paths = paths.len();
    if number_of_paths <= 0 {
        error!("Expected 1 or more programs, but there are no programs to analyze");
        return;
    }
    println!("will process {} programs", number_of_paths);

    
    let mut program_meta_vec = Vec::<ProgramMeta>::new();
    for path in paths {
        let program_meta = match analyze_program(&path, &wordpair_to_index, &indexes_array) {
            Some(value) => value,
            None => {
                continue;
            }
        };
        program_meta_vec.push(program_meta);
    }
    println!("number of program_meta items: {}", program_meta_vec.len());

    let mut progress_time = Instant::now();
    let max_index0: usize = program_meta_vec.len();
    let mut bitset = BitSet::with_capacity(number_of_bigram_rows);
    let mut comparison_results = Vec::<ComparisonResult>::new();
    comparison_results.reserve(program_meta_vec.len());
    for (index0, program0) in program_meta_vec.iter().enumerate() {
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed >= INTERVAL_UNTIL_NEXT_PROGRESS {
            let percent: usize = (index0 * 100) / max_index0;
            println!("progress: {}%  {} of {}", percent, index0 + 1, max_index0);
            progress_time = Instant::now();
        }
        comparison_results.clear();
        for (index1, program1) in program_meta_vec.iter().enumerate() {
            if index0 == index1 {
                continue;
            }
            bitset.clone_from(&program0.signature);
            bitset.intersect_with(&program1.signature);
            let overlap_count: u16 = bitset.len() as u16;
            if overlap_count == 0 {
                continue;
            }
            let comparison_result = ComparisonResult::new(program1.program_id, overlap_count);
            comparison_results.push(comparison_result);
        }
        comparison_results.sort_by(|a, b| {
            a.overlap_count.cmp(&b.overlap_count).reverse()
                .then(a.program_id.cmp(&b.program_id))
        });
        comparison_results.truncate(MAX_NUMBER_OF_ROWS_IN_OUTPUT_CSV_FILE);
    
        match OutputManager::create_csv_file(&comparison_results, program0.program_id, &output_rootdir) {
            Ok(_) => {},
            Err(error) => {
                error!("Unable to create csv file. error: {:?}", error);
                continue;
            }
        }
    }

    println!("similar end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}

fn analyze_program(
    path: &Path, 
    wordpair_to_index: &HashMap<WordPair,u16>, 
    indexes_array: &IndexesArray
) -> Option<ProgramMeta> {
    let program_id: u32 = match program_id_from_path(path) {
        Some(value) => value,
        None => {
            return None;
        }
    };
    let parsed_program: ParsedProgram = match load_program(path) {
        Some(value) => value,
        None => {
            return None;
        }
    };

    let line_count_raw: usize = parsed_program.instruction_vec.len();
    if line_count_raw > IGNORE_INPUT_PROGRAM_IF_INSTRUCTION_COUNT_EXCEEDS {
        error!("Skipped a program that is too long. path: {:?}", path);
        return None;
    }
    let line_count = line_count_raw as u16;

    let words: Vec<Word> = parsed_program.as_words();
    let n = words.len();
    if n < 2 {
        return None;
    }
    let mut match_set = HashSet::<u16>::new();
    for i in 1..n {
        let word0: Word = words[i-1];
        let word1: Word = words[i];
        let wordpair = WordPair { word0: word0, word1: word1 };
        let index: u16 = match wordpair_to_index.get(&wordpair) {
            Some(value) => *value,
            None => {
                error!("Unrecognized bigram, not found in vocabulary. Skipping. {:?}", wordpair);
                continue;
            }
        };
        match_set.insert(index);
    }

    let signature: BitSet = indexes_array.compute_signature(&match_set);

    let program_meta = ProgramMeta::new(
        program_id,
        PathBuf::from(path),
        line_count,
        signature
    );
    Some(program_meta)
}

struct ProgramMeta {
    program_id: u32,

    #[allow(dead_code)]
    path_input: PathBuf,

    #[allow(dead_code)]
    line_count: u16,

    signature: BitSet,
}

impl ProgramMeta {
    fn new(program_id: u32, path_input: PathBuf, line_count: u16, signature: BitSet) -> Self {
        Self {
            program_id: program_id,
            path_input: path_input,
            line_count: line_count,
            signature: signature
        }
    }
}

struct IndexesArray {
    indexes_array: Vec<Vec<u16>>
}

impl IndexesArray {
    fn new(vocabulary_size: u16, signature_length: u8) -> Self {
        // Create permutations of the numbers between (0 .. vocabulary_size-1)
        let mut indexes_array: Vec<Vec<u16>> = vec!();
        let original_indexes: Vec<u16> = (0..vocabulary_size).collect();
        for i in 0..signature_length {
            let mut rng = StdRng::seed_from_u64(i as u64);
            let mut indexes: Vec<u16> = original_indexes.clone();
            indexes.shuffle(&mut rng);
            indexes_array.push(indexes);
        }
        Self {
            indexes_array: indexes_array
        }
    }

    fn compute_signature(&self, match_set: &HashSet<u16>) -> BitSet {
        let mut result = BitSet::new();
        for indexes in &self.indexes_array {
            for (key, value) in indexes.iter().enumerate() {
                if match_set.contains(value) {
                    result.insert(key);
                    break;
                }
            }
        }
        result
    }
}

fn load_program(path: &Path) -> Option<ParsedProgram> {
    let contents: String = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(error) => {
            error!("load program, error: {:?} path: {:?}", error, path);
            return None;
        }
    };
    let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&contents) {
        Ok(value) => value,
        Err(error) => {
            error!("load program, error: {:?} path: {:?}", error, path);
            return None;
        }
    };
    Some(parsed_program)
}

#[derive(Serialize)]
struct ComparisonResult {
    #[serde(rename = "program id")]
    program_id: u32,

    #[serde(rename = "overlap")]
    overlap_count: u16,
}

impl ComparisonResult {
    fn new(program_id: u32, overlap_count: u16) -> Self {
        Self {
            program_id: program_id,
            overlap_count: overlap_count,
        }
    }
}

struct OutputManager {}

impl OutputManager {
    fn create_csv_file(records: &Vec<ComparisonResult>, program_id: u32, output_rootdir: &Path) -> Result<(), Box<dyn Error>> {
        let (dirname_string, filename_string) = Self::output_dir_and_file(program_id);
        let dirname = Path::new(&dirname_string);
        let filename = Path::new(&filename_string);
        let path_output_dir: PathBuf = output_rootdir.join(dirname);
        let path_output_file: PathBuf = path_output_dir.join(filename);
        Self::create_csv_file_inner(records, &path_output_dir, &path_output_file)
    }

    // Used for construct a path like: "/absolute/path/123/A123456_similarity_lsh.csv"
    fn output_dir_and_file(program_id: u32) -> (String, String) {
        let dir_index: u32 = program_id / 1000;
        let dir_index_string: String = format!("{:0>3}", dir_index);
        let filename_string: String = format!("A{:0>6}_similarity_lsh.csv", program_id);
        (dir_index_string, filename_string)
    }

    fn create_csv_file_inner(records: &Vec<ComparisonResult>, output_path_dir: &Path, output_path_file: &Path) -> Result<(), Box<dyn Error>> {
        if !output_path_dir.is_dir() {
            debug!("creating dir: {:?}", output_path_dir);
            fs::create_dir(output_path_dir)?;
        }
        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_path(output_path_file)?;
        for record in records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        Ok(())
    }
}
