use loda_rust_core;
use loda_rust_core::config::Config;
use crate::mine::{create_cache_files, load_program_ids_csv_file};
use crate::mine::validate_programs;
use crate::mine::load_program_ids_from_deny_file;
use crate::mine::load_program_ids_from_mismatch_dir;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Instant;
use std::io;
use std::error::Error;
use crate::mine::find_asm_files_recursively;
use crate::mine::program_id_from_path;
use crate::mine::program_ids_from_paths;

// What NOT to be mined!
//
// If a sequence already has been mined, then it's no longer a top priority to mine it again.
// This may change in the future.
//
// If a sequence is on the loda-programs deny.txt list, then ignore it.
// This is for duplicates in oeis and things to be ignored.
//
// If the same program is frequently turning up a false positive, then ignore it.
// This is done by adding the program to the "mismatches" directory.
struct DontMine {
    config: Config,
    program_ids: Vec<u32>
}

impl DontMine {
    fn create() -> Self {
        let mut instance = Self {
            config: Config::load(),
            program_ids: vec!()
        };
        {
            let program_ids: Vec<u32> = instance.process_existing_programs();
            instance.program_ids.extend(program_ids);
        }
        {
            let program_ids: Vec<u32> = instance.process_mismatches();
            instance.program_ids.extend(program_ids);
        }
        {
            let program_ids: Vec<u32> = instance.process_loda_programs_deny_file();
            instance.program_ids.extend(program_ids);
        }
        instance
    }

    fn process_existing_programs(&self) -> Vec<u32> {
        let dir_containing_programs: PathBuf = self.config.loda_programs_oeis_dir();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_programs);
        let program_ids: Vec<u32> = program_ids_from_paths(paths);
        println!("number of existing programs: {:?}", program_ids.len());
        program_ids
    }

    fn process_mismatches(&self) -> Vec<u32> {
        let dir_containing_mismatches: PathBuf = self.config.loda_rust_mismatches();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_mismatches);
        let program_ids: Vec<u32> = program_ids_from_paths(paths);
        println!("number of mismatches: {:?}", program_ids.len());
        program_ids
    }

    fn process_loda_programs_deny_file(&self) -> Vec<u32> {
        let path = self.config.loda_programs_oeis_deny_file();
        let program_ids: Vec<u32> = match load_program_ids_from_deny_file(&path) {
            Ok(value) => value,
            Err(error) => {
                error!("Unable to read the file: {:?} error: {:?}", path, error);
                return vec!();
            }
        };
        println!("number of programs in the 'deny.txt' file: {:?}", program_ids.len());
        program_ids
    }

    fn save(&self) {
        println!("saving, number of program_ids: {:?}", self.program_ids.len());
        let output_path: PathBuf = self.config.cache_dir_dont_mine_file();
        match create_csv_file(&self.program_ids, &output_path) {
            Ok(_) => {
                println!("save ok");
            },
            Err(error) => {
                println!("save error: {:?}", error);
            }
        }
    }
}


fn process_mismatches() {
    let config = Config::load();
    let dir_containing_mismatches: PathBuf = config.loda_rust_mismatches();
    let program_ids: Vec<u32> = load_program_ids_from_mismatch_dir(&dir_containing_mismatches);
    println!("number of mismatches: {:?}", program_ids.len());
}

fn process_loda_programs_deny_file() {
    let config = Config::load();
    let path = config.loda_programs_oeis_deny_file();
    let cache_dir: PathBuf = config.cache_dir();
    let output_path: PathBuf = cache_dir.join("dont-mine2.csv");
    let program_ids: Vec<u32> = match load_program_ids_from_deny_file(&path) {
        Ok(value) => value,
        Err(error) => {
            error!("Unable to read the file: {:?} error: {:?}", path, error);
            return;
        }
    };
    println!("deny.txt program_ids.len(): {:?}", program_ids.len());
    match create_csv_file(&program_ids, &output_path) {
        Ok(_) => {
            println!("ok");
        },
        Err(error) => {
            println!("error: {:?}", error);
        }
    }
}

fn create_csv_file(program_ids: &Vec<u32>, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(output_path)?;
    wtr.write_record(&["program id"])?;
    for program_id in program_ids {
        let s = format!("{:?}", program_id);
        wtr.write_record(&[s])?;
    }
    wtr.flush()?;
    Ok(())
}

fn obtain_dontmine_program_ids(loda_rust_repository: &Path) -> HashSet<u32> {
    // let relative_path = Path::new("resources/dont_mine.csv");
    // let path = loda_rust_repository.join(relative_path);
    let config = Config::load();
    let path = config.cache_dir_dont_mine_file();
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

fn populate_bloomfilter() {
    let start_time = Instant::now();
    println!("populate_bloomfilter begin");
    let config = Config::load();
    let oeis_stripped_file: PathBuf = config.oeis_stripped_file();
    let cache_dir: PathBuf = config.cache_dir();
    let loda_rust_repository: PathBuf = config.loda_rust_repository();

    let program_ids_to_ignore: HashSet<u32> = obtain_dontmine_program_ids(&loda_rust_repository);
    create_cache_files(&oeis_stripped_file, &cache_dir, &program_ids_to_ignore);

    println!("populate_bloomfilter end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}

pub fn subcommand_update() {
    let start_time = Instant::now();
    println!("update begin");
    let dontmine = DontMine::create();
    dontmine.save();
    // process_mismatches();
    // process_loda_programs_deny_file();
    let _ = validate_programs();
    populate_bloomfilter();
    println!("update end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
