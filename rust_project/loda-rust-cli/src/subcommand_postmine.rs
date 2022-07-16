//! The `loda-rust postmine` subcommand, checks the mined programs for correctness and performance.
use crate::config::Config;
use crate::common::{find_asm_files_recursively, load_program_ids_csv_file};
use crate::postmine::find_pending_programs;
use std::error::Error;
use std::path::PathBuf;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::process::Command;

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

    fn eval_using_loda_cpp(&self) {
        let loda_cpp_executable: PathBuf = self.config.loda_cpp_executable();
        assert!(loda_cpp_executable.is_absolute());
        assert!(loda_cpp_executable.is_file());
        for path in &self.paths_for_processing {
            assert!(path.is_absolute());
            assert!(path.is_file());
            
            let output = Command::new(&loda_cpp_executable)
                .arg("eval")
                .arg(path)
                .arg("-t")
                .arg("40")
                .output()
                .expect("failed to execute process: loda-cpp");

            println!("status: {}", output.status);
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            break;
        }
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

    instance.eval_using_loda_cpp();

    Ok(())
}
