use loda_rust_core;
use loda_rust_core::config::Config;
use crate::mine::{CheckFixedLengthSequence, NamedCacheFile, load_program_ids_csv_file, PopularProgramContainer, RecentProgramContainer, run_miner_loop, HistogramInstructionConstant};
use std::path::{Path, PathBuf};
use rand::{RngCore, thread_rng};
use std::thread;
use std::mem;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};

extern crate num_cpus;

#[derive(Debug)]
pub enum KeyMetricU32 {
    Funnel10TermsPassingBasicCheck,
    Funnel10TermsInBloomfilter,
    Funnel20TermsInBloomfilter,
    Funnel30TermsInBloomfilter,
    Funnel40TermsInBloomfilter,
}

#[derive(Debug)]
pub enum MinerThreadMessageToCoordinator {
    ReadyForMining,
    MetricU32(KeyMetricU32, u32),
}

pub fn subcommand_mine() {
    let mut number_of_threads: usize = 1;

    number_of_threads = num_cpus::get();
    assert!(number_of_threads >= 1_usize);
    assert!(number_of_threads < 1000_usize);

    number_of_threads = number_of_threads / 2;
    number_of_threads = number_of_threads.max(1);

    let (sender, receiver) = channel::<MinerThreadMessageToCoordinator>();

    let builder = thread::Builder::new().name("minercoordinator".to_string());
    let join_handle: thread::JoinHandle<_> = builder.spawn(move || {
        miner_coordinator_inner(receiver);
    }).unwrap();

    for j in 0..number_of_threads {
        println!("start thread {} of {}", j, number_of_threads);
        let name = format!("miner{}", j);
        let sender_clone = sender.clone();
        let _ = thread::Builder::new().name(name).spawn(move || {
            subcommand_mine_inner(sender_clone);
        });
        thread::sleep(Duration::from_millis(2000));
    }

    // Drop the original sender that is not being used
    mem::drop(sender);

    join_handle.join().expect("The minercoordinator thread being joined has panicked");
}

fn miner_coordinator_inner(rx: Receiver<MinerThreadMessageToCoordinator>) {
    loop {
        println!("coordinator iteration");
        loop {
            match rx.try_recv() {
                Ok(message) => {
                    println!("received message: {:?}", message);
                    continue;
                },
                Err(_) => {
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(1000));
    }
}

fn subcommand_mine_inner(tx: Sender<MinerThreadMessageToCoordinator>) {
    // Print info about start conditions
    let build_mode: &str;
    if cfg!(debug_assertions) {
        error!("Debugging enabled. Wasting cpu cycles. Not good for mining!");
        build_mode = "'DEBUG'  # Terrible inefficient for mining!";
    } else {
        build_mode = "'RELEASE'  # Good";
    }
    println!("[mining info]");
    println!("build_mode = {}", build_mode);

    // Load config file
    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let cache_dir: PathBuf = config.cache_dir();
    let mine_event_dir: PathBuf = config.mine_event_dir();
    let loda_rust_repository: PathBuf = config.loda_rust_repository();
    let loda_rust_mismatches: PathBuf = config.loda_rust_mismatches();
    let instruction_trigram_csv: PathBuf = config.cache_dir_histogram_instruction_trigram_file();
    let source_trigram_csv: PathBuf = config.cache_dir_histogram_source_trigram_file();
    let target_trigram_csv: PathBuf = config.cache_dir_histogram_target_trigram_file();

    // Load cached data
    debug!("step1");
    let filename10: &str = NamedCacheFile::Bloom10Terms.filename();
    let filename20: &str = NamedCacheFile::Bloom20Terms.filename();
    let filename30: &str = NamedCacheFile::Bloom30Terms.filename();
    let filename40: &str = NamedCacheFile::Bloom40Terms.filename();
    let path10 = cache_dir.join(Path::new(filename10));
    let path20 = cache_dir.join(Path::new(filename20));
    let path30 = cache_dir.join(Path::new(filename30));
    let path40 = cache_dir.join(Path::new(filename40));
    let checker10: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path10);
    let checker20: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path20);
    let checker30: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path30);
    let checker40: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path40);

    debug!("step2");
    let path_histogram: PathBuf = config.cache_dir_histogram_instruction_constant_file();
    let histogram_instruction_constant: Option<HistogramInstructionConstant>;
    if path_histogram.is_file() {
        histogram_instruction_constant = match HistogramInstructionConstant::load_csv_file(&path_histogram) {
            Ok(value) => {
                println!("Optional histogram: loaded successful");
                Some(value)
            },
            Err(error) => {
                println!("Optional histogram: {:?} error: {:?}", path_histogram, error);
                None
            }
        };
    } else {
        println!("Optional histogram: Not found at path {:?}", path_histogram);
        histogram_instruction_constant = None;
    }

    debug!("step3");

    // Load the program_ids available for mining
    let available_program_ids_file = cache_dir.join(Path::new("programs_valid.csv"));
    let available_program_ids: Vec<u32> = match load_program_ids_csv_file(&available_program_ids_file) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", available_program_ids_file, error);
        }
    };
    println!("number_of_available_programs = {}", available_program_ids.len());

    // Load the clusters with popular/unpopular program ids
    let program_popularity_file = loda_rust_repository.join(Path::new("resources/program_popularity.csv"));
    let popular_program_container: PopularProgramContainer = match PopularProgramContainer::load(&program_popularity_file) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", program_popularity_file, error);
        }
    };

    // Load the clusters with newest/oldest program ids
    let recent_program_file = loda_rust_repository.join(Path::new("resources/program_creation_dates.csv"));
    let recent_program_container: RecentProgramContainer = match RecentProgramContainer::load(&recent_program_file) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", recent_program_file, error);
        }
    };

    // Pick a random seed
    let mut rng = thread_rng();
    let initial_random_seed: u64 = rng.next_u64();
    println!("random_seed = {}", initial_random_seed);

    let val2 = MinerThreadMessageToCoordinator::ReadyForMining;
    tx.send(val2).unwrap();

    // Launch the miner
    run_miner_loop(
        &loda_programs_oeis_dir, 
        &checker10, 
        &checker20,
        &checker30,
        &checker40,
        histogram_instruction_constant,
        &mine_event_dir,
        &loda_rust_mismatches,
        &instruction_trigram_csv,
        &source_trigram_csv,
        &target_trigram_csv,
        available_program_ids,
        initial_random_seed,
        popular_program_container,
        recent_program_container,
    );
}
