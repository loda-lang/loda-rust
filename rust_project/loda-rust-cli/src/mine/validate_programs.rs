use loda_rust_core;
use loda_rust_core::config::Config;
use super::find_asm_files_recursively;
use super::program_id_from_path;
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::node_binomial::NodeBinomialLimit;
use loda_rust_core::execute::node_power::NodePowerLimit;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::error::Error;
use std::time::Instant;
use std::rc::Rc;
use std::fs::File;
use std::io::Write;
use std::io::LineWriter;

/*
Identify the programs that can safely be used by the miner.

Mining is computationally expensive.
The purpose of this function is to make mining less expensive,
by identifying defunct programs, so that these program doesn't break during mining.

Usecase A:
During mining, when mutating a `seq` instruction and assigning it a 
random program id. Here we want to be certain that it's a meaningful program id.
Assigning a program id that doesn't exist, wastes time trying to resolve it.
Assigning a program id that is defunct, may compute a few terms, but then fails.
For best performance, it's best up front to know what program ids makes sense to use.

Usecase B:
During mining, when choosing a random program as template for mutations.
Choosing a bad template, and much time can be wasted on error handling.
Choosing a good template, and time is well spent.

This function outputs two files: `programs_valid.csv`, `programs_invalid.csv`.

This function runs all the programs inside the `loda-programs` repository.
If a program can execute, then it gets appended to the valid file.

If a program cannot execute or failes, then it goes to the invalid file.
- programs that cannot parse.
- programs with cyclic dependencies.
- programs that fails to compute 10 terms.

The outputted file: `programs_valid.csv` has this format:

    program id
    4
    5
    7
    8
    10

The outputted file: `programs_invalid.csv` has this format:

    program id;error
    21020;ParseProgram(ParseParameters(UnrecognizedParameterType(4)))
    21100;ParseProgram(ParseParameters(UnrecognizedParameterType(6)))
    21148;ParseProgram(ParseParameters(UnrecognizedParameterType(7)))
    21292;ParseProgram(ParseParameters(UnrecognizedParameterType(5)))

*/
pub fn validate_programs() -> std::io::Result<()> {
    let start_time = Instant::now();
    println!("validate_programs begin");
    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();

    let programs_valid_csv_file: PathBuf = config.cache_dir().join(Path::new("programs_valid.csv"));
    let programs_invalid_csv_file: PathBuf = config.cache_dir().join(Path::new("programs_invalid.csv"));

    // Obtain paths to loda asm files
    let paths: Vec<PathBuf> = find_asm_files_recursively(&loda_programs_oeis_dir);
    // debug!("number of paths: {:?}", paths.len());

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
    println!("validate_programs, will analyze {:?} programs", program_ids.len());

    // Create CSV file for valid program ids
    let file0 = File::create(programs_valid_csv_file)?;
    let mut programs_valid_csv = LineWriter::new(file0);
    programs_valid_csv.write_all(b"program id\n")?;

    // Create CSV file for invalid programs and their error message
    let file1 = File::create(programs_invalid_csv_file)?;
    let mut programs_invalid_csv = LineWriter::new(file1);
    programs_invalid_csv.write_all(b"program id;error\n")?;

    // Run all the programs.
    // Reject the programs that is having difficulties running.
    let mut dm = DependencyManager::new(
        DependencyManagerFileSystemMode::System,
        loda_programs_oeis_dir,
    );
    let mut cache = ProgramCache::new();
    let mut progress_time = Instant::now();
    let program_ids_len: usize = program_ids.len();
    let mut number_of_invalid_programs: u32 = 0;
    let mut valid_program_ids: HashSet<u32> = HashSet::new();
    for (index, program_id) in program_ids.iter().enumerate() {
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed >= 1000 {
            let percent: f32 = ((index * 100) as f32) / (program_ids_len as f32);
            println!("progress: {:.2}%  {:?} / {:?}", percent, index, program_ids_len);
            progress_time = Instant::now();
        }
        let program_id64 = *program_id as u64;
        let program_runner: Rc::<ProgramRunner> = match dm.load(program_id64) {
            Ok(value) => value,
            Err(error) => {
                // error!("Cannot load program {:?}: {:?}", program_id, error);
                let row = format!("{:?};{:?}\n", program_id, error);
                programs_invalid_csv.write_all(row.as_bytes())?;
                number_of_invalid_programs += 1;
                continue;
            }
        };
        match program_runner.compute_terms(10, &mut cache) {
            Ok(_) => {},
            Err(error) => {
                // error!("Cannot run program {:?}: {:?}", program_id, error);
                let row = format!("{:?};{:?}\n", program_id, error);
                programs_invalid_csv.write_all(row.as_bytes())?;
                number_of_invalid_programs += 1;
                continue;
            }
        }

        // Append status for programs to the csv file.
        let row = format!("{:?}\n", program_id);
        programs_valid_csv.write_all(row.as_bytes())?;
        valid_program_ids.insert(*program_id);
    }
    println!("number of valid programs: {:?}", valid_program_ids.len());
    println!("number of invalid programs: {:?}", number_of_invalid_programs);
    println!("validate_programs end. elapsed: {:?} ms", start_time.elapsed().as_millis());

    return Ok(());
}

trait ComputeTerms {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) -> Result<(), Box<dyn Error>>;
}

impl ComputeTerms for ProgramRunner {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) -> Result<(), Box<dyn Error>> {
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
            let _ = match result_run {
                Ok(value) => value,
                Err(error) => {
                    debug!("Failure while computing term {}, error: {:?}", index, error);
                    return Err(Box::new(error));
                }
            };
        }
        return Ok(());
    }
}
