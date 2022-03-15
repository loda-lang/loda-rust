use crate::mine::find_asm_files_recursively;
use crate::mine::RecordBigram;
use loda_rust_core::parser::ParsedProgram;
use loda_rust_core::config::Config;
use loda_rust_core::parser::InstructionId;
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::collections::HashSet;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

const SIGNATURE_LENGTH: u8 = 30;

pub fn subcommand_similar() {
    let start_time = Instant::now();
    println!("similar begin");

    let config = Config::load();

    let instruction_bigram_csv: PathBuf = config.cache_dir_histogram_instruction_bigram_file();
    let instruction_vec: Vec<RecordBigram> = RecordBigram::parse_csv(&instruction_bigram_csv).expect("Unable to load instruction bigram csv");
    println!("number of rows in bigram.csv: {}", instruction_vec.len());
    let bigram_pairs = BigramPair::new(instruction_vec);
    println!("number of bigram pairs: {}", bigram_pairs.len());

    let mut bigram_to_index = HashMap::<BigramPair,u16>::new();
    for (index, bigram_pair) in bigram_pairs.iter().enumerate() {
        bigram_to_index.insert(*bigram_pair, index as u16);
    }
    // println!("bigram: {:?}", bigram_to_index);
    println!("bigram_to_index length: {:?}", bigram_to_index.len());

    let indexes_array = IndexesArray::new(bigram_pairs.len() as u16, SIGNATURE_LENGTH);


    let dir_containing_programs: PathBuf = config.loda_programs_oeis_dir();
    let paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_programs);
    let number_of_paths = paths.len();
    if number_of_paths <= 0 {
        error!("Expected 1 or more programs, but there are no programs to analyze");
        return;
    }
    println!("will process {} programs", number_of_paths);

    
    let mut sum = 0;
    for path in paths {
        let parsed_program: ParsedProgram = match load_program(&path) {
            Some(value) => value,
            None => {
                continue;
            }
        };
        sum += parsed_program.instruction_vec.len();

        signature(&parsed_program, &bigram_to_index, &indexes_array);
    }
    println!("number of rows total: {}", sum);

    println!("similar end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}

fn signature(parsed_program: &ParsedProgram, bigram_to_index: &HashMap<BigramPair,u16>, indexes_array: &IndexesArray) {
    let words: Vec<Word> = parsed_program.as_words();
    let n = words.len();
    if n < 2 {
        return;
    }
    let mut match_set = HashSet::<u16>::new();
    for i in 1..n {
        let word0: Word = words[i-1];
        let word1: Word = words[i];
        let pair = BigramPair { word0: word0, word1: word1 };
        let index: u16 = match bigram_to_index.get(&pair) {
            Some(value) => *value,
            None => {
                println!("Unrecognized bigram, not found in vocabulary. Skipping. {:?}", pair);
                continue;
            }
        };
        match_set.insert(index);
    }
    // println!("match_set: {:?}", match_set);

    
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
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct BigramPair {
    word0: Word,
    word1: Word,
}

impl BigramPair {
    fn new(bigram_rows: Vec<RecordBigram>) -> Vec<BigramPair> {
        let mut bigram_pairs: Vec<BigramPair> = vec!();
        let mut number_of_parse_errors = 0;
        for bigram_row in bigram_rows {
            let word0 = match Word::parse(&bigram_row.word0) {
                Some(value) => value,
                None => {
                    number_of_parse_errors += 1;
                    continue;
                }
            };
            let word1 = match Word::parse(&bigram_row.word1) {
                Some(value) => value,
                None => {
                    number_of_parse_errors += 1;
                    continue;
                }
            };
            let pair = BigramPair {
                word0: word0,
                word1: word1
            };
            bigram_pairs.push(pair);
        }
        if number_of_parse_errors > 0 {
            error!("number of parse errors: {}", number_of_parse_errors);
        }
        bigram_pairs
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

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
enum Word {
    Start,
    Stop,
    Instruction(InstructionId)
}

impl Word {
    fn parse(raw: &str) -> Option<Word> {
        match raw {
            "START" => {
                return Some(Word::Start);
            },
            "STOP" => {
                return Some(Word::Stop);
            },
            _ => {}
        }
        match InstructionId::parse(raw, 1) {
            Ok(instruction_id) => {
                return Some(Word::Instruction(instruction_id));
            },
            Err(_) => {
                return None;
            }
        }
    }
}

trait WordsFromProgram {
    fn as_words(&self) -> Vec<Word>;
}

impl WordsFromProgram for ParsedProgram {
    fn as_words(&self) -> Vec<Word> {
        let mut words: Vec<Word> = self.instruction_ids().iter().map(|instruction_id| {
            Word::Instruction(*instruction_id)
        }).collect();
        words.insert(0, Word::Start);
        words.push(Word::Stop);
        words
    }
}
