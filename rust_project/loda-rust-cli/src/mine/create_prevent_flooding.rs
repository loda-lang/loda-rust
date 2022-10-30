use super::PreventFlooding;
use loda_rust_core::control::{DependencyManager, DependencyManagerFileSystemMode};
use loda_rust_core::execute::{EvalError, NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::util::BigIntVec;
use crate::common::find_asm_files_recursively;
use crate::config::{Config, MinerFilterMode};
use std::fs;
use std::path::PathBuf;
use indicatif::ProgressBar;
use std::time::Instant;
use std::num::NonZeroUsize;
use indicatif::HumanDuration;

const PREVENT_FLOODING_CACHE_CAPACITY: usize = 300000;

trait ComputeTerms {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) -> Result<BigIntVec, EvalError>;
}

impl ComputeTerms for ProgramRunner {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) -> Result<BigIntVec, EvalError> {
        let mut terms: BigIntVec = BigIntVec::with_capacity(count as usize);
        let step_count_limit: u64 = 10000;
        let node_register_limit = NodeRegisterLimit::LimitBits(32);
        let node_loop_limit = NodeLoopLimit::LimitCount(1000);
        let mut _step_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = self.run(
                &input, 
                RunMode::Silent, 
                &mut _step_count, 
                step_count_limit, 
                node_register_limit.clone(),
                node_loop_limit.clone(),
                cache
            )?;
            terms.push(output.0.clone());
            if index == 0 {
                // print!("{}", output.0);
                continue;
            }
            // print!(",{}", output.0);
        }
        // print!("\n");
        // print!("stats: step_count: {}", step_count);
        Ok(terms)
    }
}

fn prevent_flooding_populate(prevent_flooding: &mut PreventFlooding, dependency_manager: &mut DependencyManager, cache: &mut ProgramCache, paths: Vec<PathBuf>) {
    let mut number_of_read_errors: usize = 0;
    let mut number_of_parse_errors: usize = 0;
    let mut number_of_runtime_errors: usize = 0;
    let mut number_of_already_registered_programs: usize = 0;
    let mut number_of_successfully_registered_programs: usize = 0;
    let pb = ProgressBar::new(paths.len() as u64);
    for path in paths {
        pb.inc(1);
        let contents: String = match fs::read_to_string(&path) {
            Ok(value) => value,
            Err(error) => {
                debug!("Something went wrong reading the file: {:?}  error: {:?}", path, error);
                number_of_read_errors += 1;
                continue;
            }
        };
        let runner: ProgramRunner = match dependency_manager.parse(ProgramId::ProgramWithoutId, &contents) {
            Ok(value) => value,
            Err(error) => {
                debug!("Something went wrong when parsing the file: {:?}  error: {:?}", path, error);
                number_of_parse_errors += 1;
                continue;
            }
        };
        let number_of_terms: u64 = 40;
        let terms: BigIntVec = match runner.compute_terms(number_of_terms, cache) {
            Ok(value) => value,
            Err(error) => {
                debug!("program cannot be run. path: {:?}  error: {:?}", path, error);
                number_of_runtime_errors += 1;
                continue;
            }
        };
        if prevent_flooding.try_register(&terms).is_err() {
            number_of_already_registered_programs += 1;
            continue;
        }
        number_of_successfully_registered_programs += 1;
    }
    let junk_count: usize = number_of_read_errors + number_of_parse_errors + number_of_runtime_errors + number_of_already_registered_programs;
    pb.finish_and_clear();

    debug!("prevent flooding. Registered {} programs. Ignoring {} junk programs.", number_of_successfully_registered_programs, junk_count);
}

pub fn create_prevent_flooding(config: &Config) -> anyhow::Result<PreventFlooding> {
    let start = Instant::now();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let mine_event_dir: PathBuf = config.mine_event_dir();
    let oeis_divergent_dir: PathBuf = config.loda_outlier_programs_repository_oeis_divergent();

    let mut paths0: Vec<PathBuf> = find_asm_files_recursively(&mine_event_dir);
    println!("PreventFlooding: number of .asm files in mine_event_dir: {:?}", paths0.len());
    let mut paths1: Vec<PathBuf> = find_asm_files_recursively(&oeis_divergent_dir);
    println!("PreventFlooding: number of .asm files in oeis_divergent_dir: {:?}", paths1.len());
    let mut paths: Vec<PathBuf> = vec!();
    paths.append(&mut paths0);
    match config.miner_filter_mode() {
        MinerFilterMode::All => {
            paths.append(&mut paths1);     
        },
        MinerFilterMode::New => {
            // Ignore the `oeis_divergent_dir`.
        }
    }
    println!("PreventFlooding: number of .asm files in total: {:?}", paths.len());

    let mut dependency_manager = DependencyManager::new(
        DependencyManagerFileSystemMode::System,
        loda_programs_oeis_dir,
    );
    let capacity = NonZeroUsize::new(PREVENT_FLOODING_CACHE_CAPACITY).unwrap();
    let mut cache = ProgramCache::with_capacity(capacity);
    let mut prevent_flooding = PreventFlooding::new();
    prevent_flooding_populate(&mut prevent_flooding, &mut dependency_manager, &mut cache, paths);
    println!("PreventFlooding: number of programs added: {}", prevent_flooding.len());
    println!("PreventFlooding: elapsed: {}", HumanDuration(start.elapsed()));
    Ok(prevent_flooding)
}
