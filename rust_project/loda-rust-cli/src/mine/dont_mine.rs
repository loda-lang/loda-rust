use crate::common::{find_asm_files_recursively, program_ids_from_paths};
use loda_rust_core;
use loda_rust_core::config::Config;
use std::path::{Path, PathBuf};
use std::error::Error;
use super::load_program_ids_from_deny_file;

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
pub struct DontMine {
    config: Config,
    program_ids: Vec<u32>
}

impl DontMine {
    pub fn run() {
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
        instance.save();
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
        match Self::create_csv_file(&self.program_ids, &output_path) {
            Ok(_) => {
                println!("save ok");
            },
            Err(error) => {
                println!("save error: {:?}", error);
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
}
