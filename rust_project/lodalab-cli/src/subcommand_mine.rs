use lodalab_core;
use lodalab_core::config::Config;
use lodalab_core::mine::{CheckFixedLengthSequence, load_program_ids_csv_file, PopularProgramContainer, RecentProgramContainer, run_miner_loop};
use std::path::{Path, PathBuf};
use rand::{RngCore, thread_rng};

pub fn subcommand_mine() {

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
    let loda_program_rootdir: PathBuf = config.loda_program_rootdir();
    let cache_dir: PathBuf = config.cache_dir();
    let mine_event_dir: PathBuf = config.mine_event_dir();
    let loda_rust_repository: PathBuf = config.loda_rust_repository();

    // Load cached data
    debug!("step1");
    let file10 = cache_dir.join(Path::new("fixed_length_sequence_10terms.json"));
    let checker10: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&file10);
    let file20 = cache_dir.join(Path::new("fixed_length_sequence_20terms.json"));
    let checker20: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&file20);
    let file30 = cache_dir.join(Path::new("fixed_length_sequence_30terms.json"));
    let checker30: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&file30);
    let file40 = cache_dir.join(Path::new("fixed_length_sequence_40terms.json"));
    let checker40: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&file40);
    debug!("step2");

    // Load the program_ids available for mining
    let available_program_ids_file = loda_rust_repository.join(Path::new("resources/mine_program_ids.csv"));
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

    // Launch the miner
    run_miner_loop(
        &loda_program_rootdir, 
        &checker10, 
        &checker20,
        &checker30,
        &checker40,
        &mine_event_dir,
        available_program_ids,
        initial_random_seed,
        popular_program_container,
        recent_program_container,
    );
}
