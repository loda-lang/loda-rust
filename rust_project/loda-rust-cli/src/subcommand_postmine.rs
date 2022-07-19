//! The `loda-rust postmine` subcommand, checks the mined programs for correctness and performance.
use crate::config::Config;
use crate::common::{find_asm_files_recursively, load_program_ids_csv_file};
use crate::postmine::{CandidateProgram, find_pending_programs, PathUtil};
use crate::oeis::{ProcessStrippedSequenceFile, StrippedSequence};
use loda_rust_core::util::BigIntVec;
use num_bigint::{BigInt, ToBigInt};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

struct SubcommandPostMine {
    config: Config,
    paths_for_processing: Vec<PathBuf>,
    candidate_programs: Vec<CandidateProgram>,
    dontmine_hashset: HashSet<u32>,
    invalid_program_ids_hashset: HashSet<u32>,
}

impl SubcommandPostMine {
    const LOOKUP_TERM_COUNT: usize = 40;
    const MINIMUM_NUMBER_OF_REQUIRED_TERMS: usize = 10;

    fn new() -> Self {
        Self {
            config: Config::load(),
            paths_for_processing: vec!(),
            candidate_programs: vec!(),
            dontmine_hashset: HashSet::new(),
            invalid_program_ids_hashset: HashSet::new()
        }
    }

    fn obtain_paths_for_processing(&mut self) -> Result<(), Box<dyn Error>> {
        let mine_event_dir: PathBuf = self.config.mine_event_dir();
        let paths_all: Vec<PathBuf> = find_asm_files_recursively(&mine_event_dir);
        let paths_for_processing: Vec<PathBuf> = find_pending_programs(&paths_all, true)?;
        self.paths_for_processing = paths_for_processing;
        Ok(())
    }

    fn populate_candidate_programs(&mut self) -> Result<(), Box<dyn Error>> {
        let mut candidate_programs = Vec::<CandidateProgram>::with_capacity(self.paths_for_processing.len());
        for path in &self.paths_for_processing {
            let candidate_program = CandidateProgram::new(path)?;
            candidate_programs.push(candidate_program);
        }
        self.candidate_programs = candidate_programs;
        Ok(())
    }

    fn obtain_dontmine_program_ids(&mut self) -> Result<(), Box<dyn Error>> {
        let path = self.config.analytics_dir_dont_mine_file();
        let program_ids: Vec<u32> = load_program_ids_csv_file(&path)?;
        let hashset: HashSet<u32> = HashSet::from_iter(program_ids.iter().cloned());
        println!("loaded dontmine file. number of records: {}", hashset.len());
        self.dontmine_hashset = hashset;
        Ok(())
    }    

    fn obtain_invalid_program_ids(&mut self) -> Result<(), Box<dyn Error>> {
        let path = self.config.analytics_dir_programs_invalid_file();
        let program_ids: Vec<u32> = load_program_ids_csv_file(&path)?;
        let hashset: HashSet<u32> = HashSet::from_iter(program_ids.iter().cloned());
        println!("loaded invalid program_ids file. number of records: {}", hashset.len());
        self.invalid_program_ids_hashset = hashset;
        Ok(())
    }

    fn eval_using_loda_cpp(&mut self) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();

        let loda_cpp_executable: PathBuf = self.config.loda_cpp_executable();
        assert!(loda_cpp_executable.is_absolute());
        assert!(loda_cpp_executable.is_file());

        let number_of_pending_programs: usize = self.candidate_programs.len();
        let pb = ProgressBar::new(number_of_pending_programs as u64);

        let mut count_success: usize = 0;
        let mut count_failure: usize = 0;
        let term_count = Self::LOOKUP_TERM_COUNT.to_string();
        for candidate_program in self.candidate_programs.iter_mut() {
            let output = Command::new(&loda_cpp_executable)
                .arg("eval")
                .arg(candidate_program.path_original())
                .arg("-t")
                .arg(&term_count)
                .output()
                .expect("failed to execute process: loda-cpp");

            let output_stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
            let trimmed_output: String = output_stdout.trim_end().to_string();
            // println!("status: {}", output.status);
            // println!("stdout: {:?}", trimmed_output);
            // println!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));

            if !output.status.success() {
                let reason = "Couldn't eval program with loda-cpp, this can happen if the program has a missing dependency.";
                let msg = format!("Rejecting {}, {}", candidate_program, reason);
                candidate_program.perform_reject(reason)?;
                pb.println(msg);
                count_failure += 1;
                pb.inc(1);
                continue;
            }

            count_success += 1;
            candidate_program.update_terms40(trimmed_output);
            pb.inc(1);
        }
        pb.finish_and_clear();
    
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} Ran loda-cpp with pending programs, in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        println!("evaluate: count_success: {} count_failure: {}", count_success, count_failure);
        Ok(())
    }

    /// Look up the initial terms in the OEIS `stripped` file and gather the corresponding program ids.
    fn lookup_in_oeis_stripped_file(&mut self) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();
        println!("Looking up in the OEIS 'stripped' file");

        let oeis_stripped_file: PathBuf = self.config.oeis_stripped_file();
        assert!(oeis_stripped_file.is_absolute());
        assert!(oeis_stripped_file.is_file());
        let file = File::open(oeis_stripped_file)?;
        let filesize: usize = file.metadata()?.len() as usize;
        let mut oeis_stripped_file_reader = BufReader::new(file);

        let pb = ProgressBar::new(filesize as u64);
        let padding_value_i64: i64 = 0xC0FFEE;
        let padding_value: BigInt = padding_value_i64.to_bigint().unwrap();
        let process_callback = |stripped_sequence: &StrippedSequence, count_bytes: usize| {
            pb.set_position(count_bytes as u64);
            let all_vec: &BigIntVec = stripped_sequence.bigint_vec_ref();

        };
        let program_ids_to_ignore = HashSet::<u32>::new();
        let mut stripped_sequence_processor = ProcessStrippedSequenceFile::new();
        stripped_sequence_processor.execute(
            &mut oeis_stripped_file_reader,
            Self::MINIMUM_NUMBER_OF_REQUIRED_TERMS,
            Self::LOOKUP_TERM_COUNT,
            &program_ids_to_ignore,
            &padding_value,
            process_callback
        );
        pb.finish_and_clear();
    
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} Lookups in the OEIS 'stripped' file, in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        Ok(())
    }
}

pub fn subcommand_postmine() -> Result<(), Box<dyn Error>> {
    let mut instance = SubcommandPostMine::new();
    instance.obtain_paths_for_processing()?;    
    instance.populate_candidate_programs()?;
    instance.obtain_dontmine_program_ids()?;
    instance.obtain_invalid_program_ids()?;
    instance.eval_using_loda_cpp()?;
    instance.lookup_in_oeis_stripped_file()?;
    Ok(())
}
