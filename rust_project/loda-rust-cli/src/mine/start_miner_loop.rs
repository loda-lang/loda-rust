use loda_rust_core;
use crate::config::Config;
use crate::oeis::TermsToProgramIdSet;
use super::{RunMinerLoop, MinerThreadMessageToCoordinator, Recorder};
use super::Funnel;
use super::PreventFlooding;
use super::{create_genome_mutate_context, GenomeMutateContext, Genome};
use loda_rust_core::control::{DependencyManager, DependencyManagerFileSystemMode, ExecuteProfile};
use std::path::PathBuf;
use rand::{RngCore, thread_rng};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

pub fn start_miner_loop(
    tx: Sender<MinerThreadMessageToCoordinator>, 
    recorder: Box<dyn Recorder + Send>,
    terms_to_program_id: Arc<TermsToProgramIdSet>,
    prevent_flooding: Arc<Mutex<PreventFlooding>>,
    funnel: Funnel,
) {
    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let mine_event_dir: PathBuf = config.mine_event_dir();

    let context: GenomeMutateContext = create_genome_mutate_context(&config);

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
