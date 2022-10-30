//! The `loda-rust mine` subcommand, runs the miner daemon process.
use crate::config::{Config, NumberOfWorkers};
use crate::common::PendingProgramsWithPriority;
use crate::mine::{ExecuteBatchResult, FunnelConfig, MinerCoordinatorMessage, MineEventDirectoryState, MinerCoordinator, Recorder, RunMinerLoop};
use crate::mine::{create_funnel, Funnel};
use crate::mine::{create_genome_mutate_context, GenomeMutateContext};
use crate::mine::{create_prevent_flooding, PreventFlooding};
use crate::oeis::{load_terms_to_program_id_set, TermsToProgramIdSet};
use crate::postmine::PostMine;
use loda_rust_core::control::{DependencyManager, DependencyManagerFileSystemMode, ExecuteProfile};
use bastion::prelude::*;
use num_bigint::{BigInt, ToBigInt};
use anyhow::Context;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use rand::{RngCore, thread_rng};

#[derive(Debug)]
pub enum SubcommandMineMetricsMode {
    NoMetricsServer,
    RunMetricsServer,
}

pub struct SubcommandMine {
    metrics_mode: SubcommandMineMetricsMode,
    number_of_workers: usize,
    config: Config,
    prevent_flooding: Arc<Mutex<PreventFlooding>>,
    mine_event_dir_state: Arc<Mutex<MineEventDirectoryState>>,
    shared_miner_worker_state: Arc<Mutex<SharedMinerWorkerState>>,
}

impl SubcommandMine {
    pub async fn run(
        metrics_mode: SubcommandMineMetricsMode
    ) -> anyhow::Result<()> {
        Bastion::init();
        
        let mut instance = SubcommandMine::new(metrics_mode);
        instance.check_prerequisits()?;
        instance.print_info();
        instance.reload_mineevent_directory_state()?;
        instance.populate_prevent_flooding_mechanism()?;
        instance.spawn_all_threads().await?;

        Bastion::stop();
        Bastion::block_until_stopped();
        return Ok(());
    }

    fn new(
        metrics_mode: SubcommandMineMetricsMode
    ) -> Self {
        let config = Config::load();
        let number_of_workers: usize = config.resolve_number_of_miner_workers();
        Self {
            metrics_mode: metrics_mode,
            number_of_workers: number_of_workers,
            config: config,
            prevent_flooding: Arc::new(Mutex::new(PreventFlooding::new())),
            mine_event_dir_state: Arc::new(Mutex::new(MineEventDirectoryState::new())),
            shared_miner_worker_state: Arc::new(Mutex::new(SharedMinerWorkerState::Mining)),
        }
    }

    fn check_prerequisits(&self) -> anyhow::Result<()> {
        let path0 = self.config.analytics_dir();
        if !path0.is_dir() {
            return Err(anyhow::anyhow!("check_prerequisits - missing analytics_dir. Cannot find analytics_dir at path: {:?}. Please run 'loda-rust analytics' to create this dir.", path0));
        }
        let path1 = self.config.mine_event_dir();
        if !path1.is_dir() {
            return Err(anyhow::anyhow!("check_prerequisits - missing mine_event_dir. Cannot find mine_event_dir at path: {:?}. Please rerun 'loda-rust install' to create this dir.", path1));
        }
        Ok(())
    }

    fn print_info(&self) {
        println!("metrics mode: {:?}", self.metrics_mode);
        println!("number of workers: {}", self.number_of_workers);

        let build_mode: &str;
        if cfg!(debug_assertions) {
            build_mode = "DEBUG  <-  Terrible inefficient for mining!";
        } else {
            build_mode = "RELEASE";
        }
        println!("build mode: {}", build_mode);
    }

    fn reload_mineevent_directory_state(&mut self) -> anyhow::Result<()> {
        let pending = PendingProgramsWithPriority::create(&self.config)
            .context("reload_mineevent_directory_state")?;
        match self.mine_event_dir_state.lock() {
            Ok(mut state) => {
                state.set_number_of_mined_high_prio(pending.paths_high_prio().len());
                state.set_number_of_mined_low_prio(pending.paths_low_prio().len());
            },
            Err(error) => {
                error!("reload_mineevent_directory_state: mine_event_dir_state.lock() failed. {:?}", error);
            }
        }
        Ok(())
    }

    fn populate_prevent_flooding_mechanism(&mut self) -> anyhow::Result<()> {
        let prevent_flooding: PreventFlooding = create_prevent_flooding(&self.config)?;
        self.prevent_flooding = Arc::new(Mutex::new(prevent_flooding));
        Ok(())
    }

    async fn spawn_all_threads(&self) -> anyhow::Result<()> {
        let (sender, receiver) = channel::<MinerCoordinatorMessage>();

        let mc: MinerCoordinator = match self.metrics_mode {
            SubcommandMineMetricsMode::NoMetricsServer => {
                MinerCoordinator::run_without_metrics_server(receiver)?
            },
            SubcommandMineMetricsMode::RunMetricsServer => {
                let listen_on_port: u16 = self.config.miner_metrics_listen_port();
                MinerCoordinator::run_with_metrics_server(receiver, listen_on_port, self.number_of_workers as u64)?
            }
        };

        self.spawn_workers(sender, mc.recorder)?;

        println!("\nPress CTRL-C to stop the miner.");
        // Run forever until user kills the process (CTRL-C).
        mc.minercoordinator_thread.await
            .map_err(|e| anyhow::anyhow!("spawn_all_threads - minercoordinator_thread failed with error: {:?}", e))?;

        Ok(())
    }
    
    fn spawn_workers(
        &self, 
        sender: std::sync::mpsc::Sender<MinerCoordinatorMessage>, 
        recorder: Box<dyn Recorder + Send>
    ) -> anyhow::Result<()> {
        println!("populating terms_to_program_id");
        let oeis_stripped_file: PathBuf = self.config.oeis_stripped_file();
        let padding_value: BigInt = FunnelConfig::WILDCARD_MAGIC_VALUE.to_bigint().unwrap();
        let terms_to_program_id: TermsToProgramIdSet = load_terms_to_program_id_set(
            &oeis_stripped_file, 
            FunnelConfig::MINIMUM_NUMBER_OF_REQUIRED_TERMS, 
            FunnelConfig::TERM_COUNT,
            &padding_value
        )
        .map_err(|e| anyhow::anyhow!("Unable to load terms for program ids. error: {:?}", e))?;

        let terms_to_program_id_arc: Arc<TermsToProgramIdSet> = Arc::new(terms_to_program_id);

        println!("populating funnel");
        let funnel: Funnel = create_funnel(&self.config);
        
        println!("populating genome_mutate_context");
        let genome_mutate_context: GenomeMutateContext = create_genome_mutate_context(&self.config);
        
        let config_original: Config = self.config.clone();
        let prevent_flooding = self.prevent_flooding.clone();
        let mine_event_dir_state = self.mine_event_dir_state.clone();
        let shared_miner_worker_state = self.shared_miner_worker_state.clone();

        let mine_event_dir_state2 = self.mine_event_dir_state.clone();
        let shared_miner_worker_state2 = self.shared_miner_worker_state.clone();

        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(self.number_of_workers)
                    .with_distributor(Distributor::named("miner_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        let sender_clone = sender.clone();
                        let recorder_clone: Box<dyn Recorder + Send> = recorder.clone();
                        let terms_to_program_id_arc_clone = terms_to_program_id_arc.clone();
                        let prevent_flooding_clone = prevent_flooding.clone();
                        let mine_event_dir_state_clone = mine_event_dir_state.clone();
                        let shared_miner_worker_state_clone = shared_miner_worker_state.clone();
                        let config_clone = config_original.clone();
                        let funnel_clone = funnel.clone();
                        let genome_mutate_context_clone = genome_mutate_context.clone();
                        async move {
                            miner_worker(
                                ctx,
                                sender_clone, 
                                recorder_clone, 
                                terms_to_program_id_arc_clone,
                                prevent_flooding_clone,
                                mine_event_dir_state_clone,
                                shared_miner_worker_state_clone,
                                config_clone,
                                funnel_clone,
                                genome_mutate_context_clone,
                            ).await
                        }
                    })
            })
        })
        .and_then(|_| {
            Bastion::supervisor(|supervisor| {
                supervisor.children(|children| {
                    children
                        .with_redundancy(1)
                        .with_distributor(Distributor::named("postmine_worker"))
                        .with_exec(move |ctx: BastionContext| {
                            let mine_event_dir_state_clone = mine_event_dir_state2.clone();
                            let shared_miner_worker_state_clone = shared_miner_worker_state2.clone();
                            async move {
                                postmine_worker(
                                    ctx,
                                    mine_event_dir_state_clone,
                                    shared_miner_worker_state_clone,
                                ).await
                            }
                        })
    
                })
            })
        })
        .map_err(|e| anyhow::anyhow!("couldn't setup bastion. error: {:?}", e))?;

        Bastion::start();

        Ok(())
    }
}

#[derive(Debug, Clone)]
enum MinerWorkerMessage {
    #[allow(dead_code)]
    Ping,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SharedMinerWorkerState {
    Mining,
    Paused,
}

async fn miner_worker(
    ctx: BastionContext,
    tx: Sender<MinerCoordinatorMessage>, 
    recorder: Box<dyn Recorder + Send>,
    terms_to_program_id: Arc<TermsToProgramIdSet>,
    prevent_flooding: Arc<Mutex<PreventFlooding>>,
    mine_event_dir_state: Arc<Mutex<MineEventDirectoryState>>,
    shared_miner_worker_state: Arc<Mutex<SharedMinerWorkerState>>,
    config: Config,
    funnel: Funnel,
    genome_mutate_context: GenomeMutateContext,    
) -> Result<(), ()> {
    println!("miner_worker - started, {:?}", ctx.current().id());
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();

    let postmine_worker_distributor = Distributor::named("postmine_worker");

    let initial_random_seed: u64 = {
        let mut rng = thread_rng();
        rng.next_u64()
    };

    let mut rml = RunMinerLoop::new(
        tx,
        recorder,
        funnel,
        &config,
        prevent_flooding,
        genome_mutate_context,
        initial_random_seed,
        terms_to_program_id,
    );

    loop {
        // try receive, if there is no pending message, then continue working
        // this way the worker, is kept busy, until there is an incoming message.
        let optional_message: Option<SignedMessage> = ctx.try_recv().await;
        match optional_message {
            Some(message) => {
                MessageHandler::new(message)
                    .on_tell(|miner_worker_message: MinerWorkerMessage, _| {
                        println!(
                            "miner_worker {}, received broadcast MinerWorkerMessage!:\n{:?}",
                            ctx.current().id(),
                            miner_worker_message
                        );
                        match miner_worker_message {
                            MinerWorkerMessage::Ping => {
                                println!("Ping");
                            }
                        }
                    })
                    .on_fallback(|unknown, _sender_addr| {
                        error!(
                            "miner_worker {}, received an unknown message!:\n{:?}",
                            ctx.current().id(),
                            unknown
                        );
                    });
            },
            None => {
                let the_state: SharedMinerWorkerState = match shared_miner_worker_state.lock() {
                    Ok(state) => *state,
                    Err(error) => {
                        error!("miner_worker. shared_miner_worker_state. Unable to lock mutex. {:?}", error);
                        thread::sleep(Duration::from_millis(200));
                        continue;
                    }
                };
                if the_state == SharedMinerWorkerState::Paused {
                    // Not mining, sleep for a while, and poll again
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }

                // We are mining
                // println!("miner-worker {}: execute_batch", ctx.current().id());

                // TODO: preserve the content of the dependency manager, and pass it on to next iteration.
                // Currently the dependency manager gets wiped, it's time consuming to load programs from disk.
                // The `Rc<ProgramRunner>` cannot be passed across thread-boundaries such as async/await.
                // 
                let mut dependency_manager = DependencyManager::new(
                    DependencyManagerFileSystemMode::System,
                    loda_programs_oeis_dir.clone(),
                );
                dependency_manager.set_execute_profile(ExecuteProfile::SmallLimits);
            
                let result: ExecuteBatchResult = match rml.execute_batch(&mut dependency_manager) {
                    Ok(value) => value,
                    Err(error) => {
                        error!(
                            "miner_worker {}, execute_batch error: {:?}",
                            ctx.current().id(),
                            error
                        );
                        thread::sleep(Duration::from_millis(200));
                        continue;
                    }
                };
                // println!("execute_batch stats: {:?}", result);

                let mut has_reached_mining_limit = false;
                match mine_event_dir_state.lock() {
                    Ok(mut state) => {
                        state.accumulate_stats(&result);
                        if state.has_reached_mining_limit() {
                            // debug!("reached mining limit. {:?}", state);
                            has_reached_mining_limit = true;
                        }
                    },
                    Err(error) => {
                        error!("miner_worker: mine_event_dir_state.lock() failed. {:?}", error);
                    }
                }

                if has_reached_mining_limit {
                    let mut trigger_start_postmine = false;
                    match shared_miner_worker_state.lock() {
                        Ok(mut state) => {
                            if *state == SharedMinerWorkerState::Mining {
                                trigger_start_postmine = true;
                            }
                            *state = SharedMinerWorkerState::Paused;
                        },
                        Err(error) => {
                            error!("miner_worker: Unable to Pause all miner_workers. error: {:?}", error);
                        }
                    }

                    if trigger_start_postmine {
                        println!("trigger start postmine");
                        thread::sleep(Duration::from_millis(1000));
                        let tell_result = postmine_worker_distributor
                            .tell_everyone(PostmineWorkerMessage::StartPostmineJob);
                        if let Err(error) = tell_result {
                            error!("miner_worker: Unable to send StartPostmineJob. error: {:?}", error);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PostmineWorkerMessage {
    StartPostmineJob,
}

async fn postmine_worker(
    ctx: BastionContext,
    mine_event_dir_state: Arc<Mutex<MineEventDirectoryState>>,
    shared_miner_worker_state: Arc<Mutex<SharedMinerWorkerState>>,
) -> Result<(), ()> {
    loop {
        MessageHandler::new(ctx.recv().await?)
            .on_tell(|message: PostmineWorkerMessage, _| {
                println!(
                    "postmine_worker: child {}, received broadcast message: {:?}",
                    ctx.current().id(),
                    message
                );
                match message {
                    PostmineWorkerMessage::StartPostmineJob => {
                        println!("BEFORE PostMine::run()");
                        let result = PostMine::run();
                        println!("AFTER PostMine::run()");
                        match result {
                            Ok(()) => {
                                println!("postmine Ok");
                            },
                            Err(error) => {
                                error!("postmine error: {:?}", error);
                            }
                        }
                        // TODO: loop multiple times until all pending have been processed.

                        // update/reset mine_event_dir_state, so counters are 0.
                        match mine_event_dir_state.lock() {
                            Ok(mut state) => {
                                state.reset();
                            },
                            Err(error) => {
                                error!("postmine_worker: mine_event_dir_state.lock() failed. {:?}", error);
                            }
                        }

                        // Resume the miner_workers
                        println!("trigger resume mining");
                        thread::sleep(Duration::from_millis(1000));
                        match shared_miner_worker_state.lock() {
                            Ok(mut state) => {
                                *state = SharedMinerWorkerState::Mining;
                            },
                            Err(error) => {
                                error!("postmine_worker: Unable to change state=Mining. error: {:?}", error);
                            }
                        }
                    }
                }
            });
    }
}
