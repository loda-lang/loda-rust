use crate::config::Config;
use super::{ExecuteBatchResult, MineEventDirectoryState, RunMinerLoop, MetricEvent};
use super::{Funnel, GenomeMutateContext, PreventFlooding, PostmineWorkerMessage, SharedWorkerState};
use crate::oeis::TermsToProgramIdSet;
use loda_rust_core::control::{DependencyManager, DependencyManagerFileSystemMode, ExecuteProfile};
use bastion::prelude::*;
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rand::{RngCore, thread_rng};

#[derive(Debug, Clone)]
pub enum MinerWorkerMessage {
    #[allow(dead_code)]
    Ping,
    #[allow(dead_code)]
    InvalidateAnalytics,
}

#[derive(Debug, Clone)]
pub struct MinerWorkerMessageWithAnalytics {
    funnel: Funnel,
    genome_mutate_context: GenomeMutateContext,
}

impl MinerWorkerMessageWithAnalytics {
    pub fn new(funnel: Funnel, genome_mutate_context: GenomeMutateContext) -> Self {
        Self {
            funnel,
            genome_mutate_context,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MinerWorkerQuestion {
    Launch,
}

pub async fn miner_worker(
    ctx: BastionContext,
    terms_to_program_id: Arc<TermsToProgramIdSet>,
    prevent_flooding: Arc<Mutex<PreventFlooding>>,
    mine_event_dir_state: Arc<Mutex<MineEventDirectoryState>>,
    shared_worker_state: Arc<Mutex<SharedWorkerState>>,
    config: Config,
    funnel: Funnel,
    genome_mutate_context: GenomeMutateContext,    
) -> Result<(), ()> {
    debug!("miner_worker - started, {:?}", ctx.current().id());
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();

    let postmine_worker_distributor = Distributor::named("postmine_worker");
    let metrics_worker_distributor = Distributor::named("metrics_worker");

    let initial_random_seed: u64 = {
        let mut rng = thread_rng();
        rng.next_u64()
    };

    let mut rml = RunMinerLoop::new(
        funnel,
        &config,
        prevent_flooding,
        genome_mutate_context,
        initial_random_seed,
        terms_to_program_id,
    );
    let callback = move |metric_event: MetricEvent| {
        let tell_result = metrics_worker_distributor.tell_everyone(metric_event.clone());
        if let Err(error) = tell_result {
            error!("miner_worker: Unable to send MetricEvent to metrics_worker_distributor. error: {:?}", error);
        }
    }; 
    rml.set_metrics_callback(callback);

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
                            },
                            MinerWorkerMessage::InvalidateAnalytics => {
                                println!("InvalidateAnalytics");
                            }
                        }
                    })
                    .on_tell(|miner_worker_message: std::sync::Arc<MinerWorkerMessageWithAnalytics>, _| {
                        println!(
                            "miner_worker {}, received broadcast MinerWorkerMessageWithAnalytics!:\n{:?}",
                            ctx.current().id(),
                            miner_worker_message
                        );
                        let funnel: Funnel = miner_worker_message.funnel.clone();
                        rml.set_funnel(funnel);
                        let genome_mutate_context: GenomeMutateContext = miner_worker_message.genome_mutate_context.clone();
                        rml.set_genome_mutate_context(genome_mutate_context);
                    })
                    .on_question(|message: MinerWorkerQuestion, sender| {
                        println!("miner_worker {}, received a question: \n{:?}", 
                            ctx.current().id(),
                            message
                        );
                        match sender.reply("Next month!".to_string()) {
                            Ok(value) => {
                                println!("reply ok: {:?}", value);
                            },
                            Err(error) => {
                                error!("reply error: {:?}", error);
                            }
                        };
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
                let the_state: SharedWorkerState = match shared_worker_state.lock() {
                    Ok(state) => *state,
                    Err(error) => {
                        error!("miner_worker. shared_miner_worker_state. Unable to lock mutex. {:?}", error);
                        thread::sleep(Duration::from_millis(200));
                        continue;
                    }
                };
                if the_state != SharedWorkerState::Mining {
                    // Not mining, sleep for a while, and poll again
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }

                // We are mining
                // debug!("miner-worker {}: execute_batch", ctx.current().id());

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
                    match shared_worker_state.lock() {
                        Ok(mut state) => {
                            if *state == SharedWorkerState::Mining {
                                trigger_start_postmine = true;
                            }
                            *state = SharedWorkerState::Postmine;
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
