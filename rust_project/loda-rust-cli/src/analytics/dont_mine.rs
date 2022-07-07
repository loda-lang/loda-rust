use crate::common::SimpleLog;
use crate::config::Config;
use std::error::Error;
use std::path::PathBuf;
use std::collections::HashSet;
use super::load_program_ids_from_deny_file;
use crate::common::{load_program_ids_csv_file, save_program_ids_csv_file};

/// Generate the `dontmine.csv` file.
/// 
/// These are the program ids that should NOT be mined.
pub struct DontMine {
    simple_log: SimpleLog,
    config: Config,
    program_ids: HashSet<u32>
}

impl DontMine {
    pub fn run(simple_log: SimpleLog) -> Result<(), Box<dyn Error>> {
        simple_log.println("\nDontMine");
        let mut instance = Self {
            simple_log: simple_log,
            config: Config::load(),
            program_ids: HashSet::new()
        };
        instance.extend_program_ids_with_loda_programs_deny_txt()?;
        instance.extend_program_ids_with_dont_optimize_csv()?;
        instance.remove_invalid_programs()?;
        instance.save()?;
        Ok(())
    }

    /// Ignore simple programs, that doesn't have complex loops nor dependencies to other sequences.
    /// It doesn't make sense trying to optimize a program that is already well-optimized.
    fn extend_program_ids_with_dont_optimize_csv(&mut self) -> Result<(), Box<dyn Error>> {
        let path: PathBuf = self.config.analytics_dir_complexity_dont_optimize_file();
        let program_ids: Vec<u32> = load_program_ids_csv_file(&path)?;
        let content = format!("number of program ids in the 'dont_optimize.csv' file: {:?}", program_ids.len());
        self.simple_log.println(content);
        self.program_ids.extend(program_ids);
        Ok(())
    }

    /// If a program is on the loda-programs `deny.txt` list, then ignore it.
    /// This is for duplicates in oeis and things to be ignored.
    fn extend_program_ids_with_loda_programs_deny_txt(&mut self) -> Result<(), Box<dyn Error>> {
        let path = self.config.loda_programs_oeis_deny_file();
        let program_ids: Vec<u32> = load_program_ids_from_deny_file(&path)?;
        let content = format!("number of program ids in the 'deny.txt' file: {:?}", program_ids.len());
        self.simple_log.println(content);
        self.program_ids.extend(program_ids);
        Ok(())
    }

    /// If the `loda-programs` repo contains a program that is invalid,
    /// then we want to mine for the program anyways.
    fn remove_invalid_programs(&mut self) -> Result<(), Box<dyn Error>> {
        let path: PathBuf = self.config.analytics_dir_programs_invalid_file();
        let program_ids: Vec<u32> = load_program_ids_csv_file(&path)?;
        let mut remove_count = 0;
        for program_id in program_ids {
            if self.program_ids.remove(&program_id) {
                remove_count += 1;
            }
        }
        let content = format!("number of program ids removed because they are in the 'programs_invalid.csv' file: {}", remove_count);
        self.simple_log.println(content);
        Ok(())
    }

    fn sorted_vec(program_ids: &HashSet<u32>) -> Vec<u32> {
        let mut program_ids_sorted: Vec<u32> = program_ids.clone().into_iter().collect();
        program_ids_sorted.sort();
        program_ids_sorted
    }

    fn save(&self) -> Result<(), Box<dyn Error>> {
        let program_ids_sorted: Vec<u32> = Self::sorted_vec(&self.program_ids);
        let content = format!("number of program ids in the 'dontmine.csv' file: {:?}", program_ids_sorted.len());
        self.simple_log.println(content);
        let output_path: PathBuf = self.config.analytics_dir_dont_mine_file();
        save_program_ids_csv_file(&program_ids_sorted, &output_path)
    }
}
