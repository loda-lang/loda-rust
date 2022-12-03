use super::PreventFlooding;
use loda_rust_core::control::{DependencyManager, DependencyManagerFileSystemMode, ExecuteProfile};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::util::BigIntVec;
use crate::common::find_asm_files_recursively;
use crate::config::{Config, MinerFilterMode};
use std::fs;
use std::path::PathBuf;
use indicatif::{HumanDuration, ProgressBar};
use std::time::{Instant, Duration};
use std::num::NonZeroUsize;

const PREVENT_FLOODING_CACHE_CAPACITY: usize = 3000;

trait ComputeTerms {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) -> anyhow::Result<BigIntVec>;
}

impl ComputeTerms for ProgramRunner {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) -> anyhow::Result<BigIntVec> {
        let mut terms: BigIntVec = BigIntVec::with_capacity(count as usize);
        let step_count_limit: u64 = 10000;
        let node_register_limit = NodeRegisterLimit::LimitBits(8); // 256 registers
        let node_loop_limit = NodeLoopLimit::LimitCount(1000);
        let max_number_of_bits: u64 = 100000; // 100k bits / 8 = 12.5kbytes
        let max_duration_seconds: u64 = 5;
        let mut step_count: u64 = 0;
        let start_time = Instant::now();
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = self.run(
                input, 
                RunMode::Silent, 
                &mut step_count, 
                step_count_limit, 
                node_register_limit.clone(),
                node_loop_limit.clone(),
                cache
            )?;
            let elapsed: Duration = start_time.elapsed();
            if elapsed.as_secs() >= max_duration_seconds {
                return Err(anyhow::anyhow!("ignoring program. elapsed time {} exceeded the limit of {} seconds.", HumanDuration(elapsed), max_duration_seconds));
            }
            let bits: u64 = output.0.bits();
            if bits >= max_number_of_bits {
                return Err(anyhow::anyhow!("ignoring program. term bit size {}, exceeded the limit of {}", bits, max_number_of_bits));
            }
            terms.push(output.0.clone());
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
    dependency_manager.set_execute_profile(ExecuteProfile::SmallLimits);
    let capacity = NonZeroUsize::new(PREVENT_FLOODING_CACHE_CAPACITY).unwrap();
    let mut cache = ProgramCache::with_capacity(capacity);
    let mut prevent_flooding = PreventFlooding::new();
    prevent_flooding_populate(&mut prevent_flooding, &mut dependency_manager, &mut cache, paths);
    println!("PreventFlooding: number of programs added: {}", prevent_flooding.len());
    println!("PreventFlooding: elapsed: {}", HumanDuration(start.elapsed()));
    Ok(prevent_flooding)
}
