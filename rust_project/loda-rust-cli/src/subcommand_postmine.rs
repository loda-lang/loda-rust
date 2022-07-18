//! The `loda-rust postmine` subcommand, checks the mined programs for correctness and performance.
use crate::config::Config;
use crate::common::{find_asm_files_recursively, load_program_ids_csv_file};
use crate::postmine::{CandidateProgram, find_pending_programs, PathUtil};
use std::error::Error;
use std::path::PathBuf;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::process::Command;
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

struct SubcommandPostMine {
    config: Config,
    paths_for_processing: Vec<PathBuf>,
    dontmine_hashset: HashSet<u32>,
    invalid_program_ids_hashset: HashSet<u32>,
}

impl SubcommandPostMine {
    fn new() -> Self {
        Self {
            config: Config::load(),
            paths_for_processing: vec!(),
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

    fn eval_using_loda_cpp(&self) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();

        let loda_cpp_executable: PathBuf = self.config.loda_cpp_executable();
        assert!(loda_cpp_executable.is_absolute());
        assert!(loda_cpp_executable.is_file());

        let number_of_pending_programs: usize = self.paths_for_processing.len();
        let pb = ProgressBar::new(number_of_pending_programs as u64);

        let mut candidate_programs = Vec::<CandidateProgram>::with_capacity(number_of_pending_programs);
        for path in &self.paths_for_processing {
            let candidate_program = CandidateProgram::new(path)?;
            candidate_programs.push(candidate_program);
        }

        let mut count_success: usize = 0;
        let mut count_failure: usize = 0;
        for mut candidate_program in candidate_programs {

            let output = Command::new(&loda_cpp_executable)
                .arg("eval")
                .arg(candidate_program.path())
                .arg("-t")
                .arg("40")
                .output()
                .expect("failed to execute process: loda-cpp");

            let output_stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
            let trimmed_output: String = output_stdout.trim_end().to_string();
            // println!("status: {}", output.status);
            // println!("stdout: {:?}", trimmed_output);
            // println!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));

            if !output.status.success() {
                let msg = format!("Rejecting {} Couldn't eval program with loda-cpp, this can happen if the program has a missing dependency.", candidate_program);
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
}

pub fn subcommand_postmine() -> Result<(), Box<dyn Error>> {
    let mut instance = SubcommandPostMine::new();
    instance.obtain_paths_for_processing()?;    
    println!("Will process {} programs", instance.paths_for_processing.len());

    instance.obtain_dontmine_program_ids()?;
    println!("Number of programs in dontmine.csv: {}", instance.dontmine_hashset.len());
    
    instance.obtain_invalid_program_ids()?;
    println!("Number of programs in invalid programs id.csv: {}", instance.invalid_program_ids_hashset.len());

    instance.eval_using_loda_cpp()?;

    Ok(())
}
