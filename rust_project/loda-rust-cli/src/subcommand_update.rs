use loda_rust_core;
use loda_rust_core::config::Config;
use crate::mine::{create_cache_files, load_program_ids_csv_file};
use crate::mine::find_asm_files_recursively;
use crate::mine::program_id_from_path;
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::node_binomial::NodeBinomialLimit;
use loda_rust_core::execute::node_power::NodePowerLimit;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Instant;
use std::rc::Rc;
use std::fs::{self, File};
use std::io::Write;
use std::io::LineWriter;

fn identify_all_valid_programs() -> std::io::Result<()> {
    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();

    let available_program_ids_file: PathBuf = config.cache_dir().join(Path::new("mine_program_ids2.csv"));

    // Obtain paths to loda asm files
    let paths: Vec<PathBuf> = find_asm_files_recursively(&loda_programs_oeis_dir);
    debug!("number of paths: {:?}", paths.len());

    // Extract program_ids from paths
    let mut program_ids: Vec<u32> = vec!();
    for path in paths {
        let program_id: u32 = match program_id_from_path(&path) {
            Some(program_id) => program_id,
            None => {
                warn!("Unable to extract program_id from {:?}", path);
                continue;
            }
        };
        program_ids.push(program_id);
    }
    program_ids.sort();
    debug!("number of program_ids: {:?}", program_ids.len());
    if program_ids.len() > 10 {
        let first_items = &program_ids[0..10];
        debug!("program_ids first: {:?}", first_items);
        let last_items = &program_ids[program_ids.len()-10..];
        debug!("program_ids last: {:?}", last_items);
    }

    // Create CSV file
    let file = File::create(available_program_ids_file)?;
    let mut line_writer = LineWriter::new(file);
    line_writer.write_all(b"program id;terms\n")?;

    // Run all the programs.
    // Reject the programs that is having difficulties running.
    let mut dm = DependencyManager::new(
        DependencyManagerFileSystemMode::System,
        loda_programs_oeis_dir,
    );
    let mut cache = ProgramCache::new();
    let mut progress_time = Instant::now();
    let program_ids_len: usize = program_ids.len();
    for (index, program_id) in program_ids.iter().enumerate() {
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed >= 1000 {
            let percent: f32 = ((index * 100) as f32) / (program_ids_len as f32);
            debug!("progress: {:.2}%  {:?} / {:?}", percent, index, program_ids_len);
            progress_time = Instant::now();
        }
        let program_id64 = *program_id as u64;
        let program_runner: Rc::<ProgramRunner> = match dm.load(program_id64) {
            Ok(value) => value,
            Err(error) => {
                // error!("Cannot load program {:?}: {:?}", program_id, error);
                let row = format!("{:?};ERROR {:?}\n", program_id, error);
                line_writer.write_all(row.as_bytes())?;
                continue;
            }
        };
        program_runner.compute_terms(10, &mut cache);

        // Append status for programs to the csv file.
        let row = format!("{:?};ok\n", program_id);
        line_writer.write_all(row.as_bytes())?;
    }

    return Ok(());
}

trait ComputeTerms {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache);
}

impl ComputeTerms for ProgramRunner {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        if count < 1 {
            panic!("Expected number of terms to be 1 or greater.");
        }
        let step_count_limit: u64 = 1000000000;
        let mut step_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let result_run = self.run(
                &input, 
                RunMode::Silent, 
                &mut step_count, 
                step_count_limit,
                NodeRegisterLimit::Unlimited,
                NodeBinomialLimit::Unlimited,
                NodeLoopLimit::Unlimited,
                NodePowerLimit::Unlimited,
                cache
            );
            let output: RegisterValue = match result_run {
                Ok(value) => value,
                Err(error) => {
                    error!("Failure while computing term {}, error: {:?}", index, error);
                    return;
                }
            };
            // if index == 0 {
            //     print!("{}", output.0);
            //     continue;
            // }
            // print!(",{}", output.0);
        }
        // print!("\n");
        // debug!("steps: {}", step_count);
    }
}

fn obtain_dontmine_program_ids(loda_rust_repository: &Path) -> HashSet<u32> {
    let relative_path = Path::new("resources/dont_mine.csv");
    let path = loda_rust_repository.join(relative_path);

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

pub fn subcommand_update() {
    let start_time = Instant::now();
    let config = Config::load();
    let oeis_stripped_file: PathBuf = config.oeis_stripped_file();
    let cache_dir: PathBuf = config.cache_dir();
    let loda_rust_repository: PathBuf = config.loda_rust_repository();

    println!("update begin");
    
    // let _ = identify_all_valid_programs();
    let program_ids_to_ignore: HashSet<u32> = obtain_dontmine_program_ids(&loda_rust_repository);
    create_cache_files(&oeis_stripped_file, &cache_dir, &program_ids_to_ignore);

    println!("update end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
