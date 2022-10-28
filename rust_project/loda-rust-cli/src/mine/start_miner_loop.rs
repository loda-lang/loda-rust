use crate::config::Config;
use crate::oeis::TermsToProgramIdSet;
use super::{RunMinerLoop, MinerThreadMessageToCoordinator, Recorder};
use super::Funnel;
use super::PreventFlooding;
use super::{GenomeMutateContext, Genome};
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
    config: Config,
    funnel: Funnel,
    genome_mutate_context: GenomeMutateContext,
) -> RunMinerLoop {
    let mine_event_dir: PathBuf = config.mine_event_dir();

    // Pick a random seed
    let mut rng = thread_rng();
    let initial_random_seed: u64 = rng.next_u64();
    let rng: StdRng = StdRng::seed_from_u64(initial_random_seed);

    let genome = Genome::new();

    let val2 = MinerThreadMessageToCoordinator::ReadyForMining;
    tx.send(val2).unwrap();

    RunMinerLoop::new(
        tx,
        recorder,
        funnel,
        &mine_event_dir,
        prevent_flooding,
        genome_mutate_context,
        genome,
        rng,
        terms_to_program_id,
    )
}
