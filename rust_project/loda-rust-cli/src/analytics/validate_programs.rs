use crate::common::{find_asm_files_recursively, oeis_ids_from_paths, ToOeisIdVec, SimpleLog};
use loda_rust_core::oeis::{OeisId, OeisIdHashSet};
use loda_rust_core;
use crate::config::Config;
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::UnofficialFunctionRegistry;
use std::path::PathBuf;
use std::collections::HashSet;
use std::error::Error;
use std::time::Instant;
use std::rc::Rc;
use std::fs::File;
use std::io::Write;
use std::io::LineWriter;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

const NUMBER_OF_TERMS_TO_VALIDATE: u64 = 1;

/*
Identify the programs that can safely be used by the miner.

Mining is computationally expensive.
The purpose of this function is to make mining less expensive,
by eliminating defunct programs before mining begins, 
so that defunct programs doesn't sporadic cause havoc during mining.

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

This function outputs three files: 
- `programs_valid.csv`
- `programs_invalid.csv`
- `programs_invalid_verbose.csv`

This function runs all the programs inside the `loda-programs` repository.
If a program can execute, then it gets appended to the valid file.

If a program cannot execute or failes, then it goes to the invalid file.
- programs that cannot parse.
- programs with cyclic dependencies.
- programs that fails to compute NUMBER_OF_TERMS_TO_VALIDATE terms.

The outputted file: `programs_valid.csv` has this format:

    program id
    4
    5
    7
    8
    10

The outputted file: `programs_invalid.csv` has this format:

    program id
    21020
    21100
    21148
    306326

The outputted file: `programs_invalid_verbose.csv` has this format:

    program id;error category;error message
    21020;LODA;ParseProgram(ParseParameters(UnrecognizedParameterType(4)))
    21100;LODA;ParseProgram(ParseParameters(UnrecognizedParameterType(6)))
    21148;LODA;ParseProgram(ParseParameters(UnrecognizedParameterType(7)))
    306326;COMPUTE;EvalSequenceWithNegativeParameter

*/
pub struct ValidatePrograms {}

impl ValidatePrograms {
    pub fn run(simple_log: SimpleLog) -> anyhow::Result<()> {
        let start = Instant::now();
        simple_log.println("\nValidatePrograms");
        println!("Validate programs");
        let config = Config::load();
        let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
        let programs_valid_csv_file: PathBuf = config.analytics_dir_programs_valid_file();
        let programs_invalid_csv_file: PathBuf = config.analytics_dir_programs_invalid_file();
        let programs_invalid_verbose_csv_file: PathBuf = config.analytics_dir_programs_invalid_verbose_file();

        // Obtain paths to loda asm files
        let paths: Vec<PathBuf> = find_asm_files_recursively(&loda_programs_oeis_dir);
        let number_of_paths = paths.len();
        if number_of_paths <= 0 {
            return Err(anyhow::anyhow!("ValidatePrograms::run - Expected 1 or more programs, cannot validate"));
        }
        // Extract oeis_ids from paths
        let oeis_ids_hashset: OeisIdHashSet = oeis_ids_from_paths(&paths);
        let oeis_ids: Vec<OeisId> = oeis_ids_hashset.sorted_vec();
        let content = format!("number of programs to validate: {:?}", oeis_ids.len());
        simple_log.println(content);

        // Create CSV file for valid program ids
        let file0 = File::create(programs_valid_csv_file)?;
        let mut programs_valid_csv = LineWriter::new(file0);
        programs_valid_csv.write_all(b"program id\n")?;

        // Create CSV file for invalid program ids
        let file1 = File::create(programs_invalid_csv_file)?;
        let mut programs_invalid_csv = LineWriter::new(file1);
        programs_invalid_csv.write_all(b"program id\n")?;

        // Create CSV file for invalid programs and their error message
        let file2 = File::create(programs_invalid_verbose_csv_file)?;
        let mut programs_invalid_verbose_csv = LineWriter::new(file2);
        programs_invalid_verbose_csv.write_all(b"program id;error category;error message\n")?;

        // Run all the programs.
        // Reject the programs that is having difficulties running.
        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            loda_programs_oeis_dir,
            UnofficialFunctionRegistry::new(),
        );
        let mut cache = ProgramCache::new();
        let oeis_ids_len: usize = oeis_ids.len();
        let mut number_of_invalid_programs: u32 = 0;
        let mut valid_program_ids: OeisIdHashSet = HashSet::new();
        let pb = ProgressBar::new(oeis_ids_len as u64);
        for oeis_id in oeis_ids {
            let program_id64 = oeis_id.raw() as u64;
            let program_runner: Rc::<ProgramRunner> = match dm.load(program_id64) {
                Ok(value) => value,
                Err(error) => {
                    // error!("Cannot load program {:?}: {:?}", oeis_id, error);
                    let row_simple = format!("{:?}\n", oeis_id.raw());
                    programs_invalid_csv.write_all(row_simple.as_bytes())?;
                    let row_verbose = format!("{:?};LOAD;{:?}\n", oeis_id.raw(), error);
                    programs_invalid_verbose_csv.write_all(row_verbose.as_bytes())?;
                    number_of_invalid_programs += 1;
                    pb.inc(1);
                    continue;
                }
            };
            match program_runner.compute_terms(NUMBER_OF_TERMS_TO_VALIDATE, &mut cache) {
                Ok(_) => {},
                Err(error) => {
                    // error!("Cannot run program {:?}: {:?}", oeis_id, error);
                    let row_simple = format!("{:?}\n", oeis_id.raw());
                    programs_invalid_csv.write_all(row_simple.as_bytes())?;
                    let row_verbose = format!("{:?};COMPUTE;{:?}\n", oeis_id.raw(), error);
                    programs_invalid_verbose_csv.write_all(row_verbose.as_bytes())?;
                    number_of_invalid_programs += 1;
                    pb.inc(1);
                    continue;
                }
            }

            // Append status for programs to the csv file.
            let row = format!("{:?}\n", oeis_id.raw());
            programs_valid_csv.write_all(row.as_bytes())?;
            valid_program_ids.insert(oeis_id);
            pb.inc(1);
        }
        pb.finish_and_clear();

        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} validated programs in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        let content = format!("number of valid programs: {:?}", valid_program_ids.len());
        simple_log.println(content);
        let content = format!("number of invalid programs: {:?}\n", number_of_invalid_programs);
        simple_log.println(content);

        return Ok(());
    }
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
                NodeLoopLimit::Unlimited,
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
