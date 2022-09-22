use loda_rust_core;
use crate::config::Config;
use crate::common::RecordTrigram;
use crate::common::load_program_ids_csv_file;
use crate::oeis::TermsToProgramIdSet;
use super::{CheckFixedLengthSequence, Funnel, NamedCacheFile, PopularProgramContainer, RecentProgramContainer, RunMinerLoop, HistogramInstructionConstant, MinerThreadMessageToCoordinator, Recorder};
use super::PreventFlooding;
use super::{GenomeMutateContext, Genome};
use super::SuggestInstruction;
use super::SuggestSource;
use super::SuggestTarget;
use loda_rust_core::control::{DependencyManager, DependencyManagerFileSystemMode, ExecuteProfile};
use std::path::{Path, PathBuf};
use rand::{RngCore, thread_rng};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::collections::HashSet;

pub fn start_miner_loop(
    tx: Sender<MinerThreadMessageToCoordinator>, 
    recorder: Box<dyn Recorder + Send>,
    terms_to_program_id: Arc<TermsToProgramIdSet>,
    prevent_flooding: Arc<Mutex<PreventFlooding>>
) {
    // Load config file
    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let analytics_dir: PathBuf = config.analytics_dir();
    let mine_event_dir: PathBuf = config.mine_event_dir();
    let loda_rust_repository: PathBuf = config.loda_rust_repository();
    let instruction_trigram_csv: PathBuf = config.analytics_dir_histogram_instruction_trigram_file();
    let source_trigram_csv: PathBuf = config.analytics_dir_histogram_source_trigram_file();
    let target_trigram_csv: PathBuf = config.analytics_dir_histogram_target_trigram_file();

    // Load cached data
    debug!("step1");
    let filename10: &str = NamedCacheFile::Bloom10Terms.filename();
    let filename20: &str = NamedCacheFile::Bloom20Terms.filename();
    let filename30: &str = NamedCacheFile::Bloom30Terms.filename();
    let filename40: &str = NamedCacheFile::Bloom40Terms.filename();
    let path10 = analytics_dir.join(Path::new(filename10));
    let path20 = analytics_dir.join(Path::new(filename20));
    let path30 = analytics_dir.join(Path::new(filename30));
    let path40 = analytics_dir.join(Path::new(filename40));
    let checker10: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path10);
    let checker20: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path20);
    let checker30: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path30);
    let checker40: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path40);
    let funnel = Funnel::new(
        checker10,
        checker20,
        checker30,
        checker40,
    );

    debug!("step2");
    let path_histogram: PathBuf = config.analytics_dir_histogram_instruction_constant_file();
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

    // Load the valid program_ids, that can execute.
    let programs_valid_file = config.analytics_dir_programs_valid_file();
    let valid_program_ids: Vec<u32> = match load_program_ids_csv_file(&programs_valid_file) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", programs_valid_file, error);
        }
    };
    println!("number_of_valid_program_ids = {}", valid_program_ids.len());

    // Load the invalid program_ids, that are defunct, such as cannot execute, cyclic-dependency.
    let programs_invalid_file = config.analytics_dir_programs_invalid_file();
    let invalid_program_ids: Vec<u32> = match load_program_ids_csv_file(&programs_invalid_file) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", programs_invalid_file, error);
        }
    };
    println!("number_of_invalid_program_ids = {}", invalid_program_ids.len());
    let invalid_program_ids_hashset: HashSet<u32> = invalid_program_ids.into_iter().collect();

    // Load the clusters with popular/unpopular program ids
    let program_popularity_file = config.analytics_dir_program_popularity_file();
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

    let instruction_trigram_vec: Vec<RecordTrigram> = RecordTrigram::parse_csv(&instruction_trigram_csv).expect("Unable to load instruction trigram csv");
    let mut suggest_instruction = SuggestInstruction::new();
    suggest_instruction.populate(&instruction_trigram_vec);

    let source_trigram_vec: Vec<RecordTrigram> = RecordTrigram::parse_csv(&source_trigram_csv).expect("Unable to load source trigram csv");
    let mut suggest_source = SuggestSource::new();
    suggest_source.populate(&source_trigram_vec);

    let target_trigram_vec: Vec<RecordTrigram> = RecordTrigram::parse_csv(&target_trigram_csv).expect("Unable to load target trigram csv");
    let mut suggest_target = SuggestTarget::new();
    suggest_target.populate(&target_trigram_vec);

    let mut dependency_manager = DependencyManager::new(
        DependencyManagerFileSystemMode::System,
        loda_programs_oeis_dir,
    );
    dependency_manager.set_execute_profile(ExecuteProfile::SmallLimits);

    // Pick a random seed
    let mut rng = thread_rng();
    let initial_random_seed: u64 = rng.next_u64();
    let rng: StdRng = StdRng::seed_from_u64(initial_random_seed);
    println!("random_seed = {}", initial_random_seed);

    let context = GenomeMutateContext::new(
        valid_program_ids,
        invalid_program_ids_hashset,
        popular_program_container,
        recent_program_container,
        histogram_instruction_constant,
        Some(suggest_instruction),
        Some(suggest_source),
        Some(suggest_target)
    );
    assert_eq!(context.has_available_programs(), true);

    let genome = Genome::new();

    let val2 = MinerThreadMessageToCoordinator::ReadyForMining;
    tx.send(val2).unwrap();

    // Launch the miner
    let mut rml = RunMinerLoop::new(
        tx,
        recorder,
        dependency_manager,
        funnel,
        &mine_event_dir,
        prevent_flooding,
        context,
        genome,
        rng,
        terms_to_program_id,
    );
    rml.loop_forever();
}
