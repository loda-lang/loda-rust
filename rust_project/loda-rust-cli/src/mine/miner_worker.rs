use crate::config::Config;
use super::{ExecuteBatchResult, RunMinerLoop, MetricEvent};
use super::{Funnel, GenomeMutateContext, PreventFlooding};
use super::CoordinatorWorkerQuestion;
use crate::oeis::TermsToProgramIdSet;
use loda_rust_core::control::{DependencyManager, DependencyManagerFileSystemMode, ExecuteProfile};
use bastion::prelude::*;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rand::{RngCore, thread_rng};

#[derive(Debug, Clone)]
pub enum MinerWorkerMessage {
    StartMining,
}

#[derive(Clone)]
pub struct MinerWorkerMessageWithAnalytics {
    funnel: Funnel,
    genome_mutate_context: GenomeMutateContext,
    terms_to_program_id_arc: Arc<TermsToProgramIdSet>,
}

impl MinerWorkerMessageWithAnalytics {
    pub fn new(funnel: Funnel, genome_mutate_context: GenomeMutateContext, terms_to_program_id_arc: Arc<TermsToProgramIdSet>) -> Self {
        Self {
            funnel,
            genome_mutate_context,
            terms_to_program_id_arc,
        }
    }
}

impl fmt::Debug for MinerWorkerMessageWithAnalytics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MinerWorkerMessageWithAnalytics")
    }
}

#[derive(Debug, Clone)]
pub enum MinerWorkerQuestion {
    #[allow(dead_code)]
    Ping,
}

pub async fn miner_worker(
    ctx: BastionContext,
    prevent_flooding: Arc<Mutex<PreventFlooding>>,
    config: Config,
) -> Result<(), ()> {
    debug!("miner_worker - started, {:?}", ctx.current().id());
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();

    let coordinator_worker_distributor = Distributor::named("coordinator_worker");
    let metrics_worker_distributor = Distributor::named("metrics_worker");

    let initial_random_seed: u64 = {
        let mut rng = thread_rng();
        rng.next_u64()
    };

    let mut rml = RunMinerLoop::new(
        &config,
        prevent_flooding,
        initial_random_seed,
    );
    let callback = move |metric_event: MetricEvent| {
        let tell_result = metrics_worker_distributor.tell_everyone(metric_event.clone());
        if let Err(error) = tell_result {
            error!("miner_worker: Unable to send MetricEvent to metrics_worker_distributor. error: {:?}", error);
        }
    }; 
    rml.set_metrics_callback(callback);

    loop {
        let mut execute_one_batch = false;
        MessageHandler::new(ctx.recv().await?)
            .on_tell(|message: MinerWorkerMessage, _| {
                // println!(
                //     "miner_worker {}, received broadcast MinerWorkerMessage: {:?}",
                //     ctx.current().id(),
                //     message
                // );
                match message {
                    MinerWorkerMessage::StartMining => {
                        debug!("miner_worker: StartMining");
                        execute_one_batch = true;
                    },
                }
            })
            .on_question(|message: std::sync::Arc<MinerWorkerMessageWithAnalytics>, sender| {
                debug!(
                    "miner_worker {}, received question MinerWorkerMessageWithAnalytics: {:?}",
                    ctx.current().id(),
                    message
                );
                rml.set_funnel(message.funnel.clone());
                rml.set_genome_mutate_context(message.genome_mutate_context.clone());
                rml.set_terms_to_program_id(message.terms_to_program_id_arc.clone());
                match sender.reply("miner_worker_updated_ok".to_string()) {
                    Ok(value) => {
                        debug!("miner_worker: reply ok: {:?}", value);
                    },
                    Err(error) => {
                        error!("miner_worker: reply error: {:?}", error);
                    }
                };
            })
            .on_question(|message: MinerWorkerQuestion, sender| {
                println!("miner_worker {}, received a question: \n{:?}", 
                    ctx.current().id(),
                    message
                );
                match sender.reply("Next month!".to_string()) {
                    Ok(value) => {
                        debug!("miner_worker: reply ok: {:?}", value);
                    },
                    Err(error) => {
                        error!("miner_worker: reply error: {:?}", error);
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

        // Repeat executing batches as long as the coordinator replies with "continue"
        while execute_one_batch {
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
                    Bastion::stop();
                    panic!("the miner_worker is in a broken state");
                }
            };
            // println!("execute_batch stats: {:?}", result);

            // tell coordinator that batch has ended, with the stats
            // and the coordinator will decide what should happen
            let answer: Answer = coordinator_worker_distributor
                .ask_one(CoordinatorWorkerQuestion::MinerWorkerExecutedOneBatch { execute_batch_result: result })
                .expect("couldn't perform MinerWorkerExecutedOneBatch request");
            let reply_optional: Option<String> = run!(blocking! {
                let mut response_text: String = "no response".to_string();
                MessageHandler::new(answer.await.expect("couldn't receive reply"))
                    .on_tell(|response: String, _| {
                        response_text = response.clone();
                    })
                    .on_fallback(|unknown, _sender_addr| {
                        error!(
                            "miner_worker: uh oh, I received a message I didn't understand\n {:?}",
                            unknown
                        );
                    });
                response_text
            });

            let s: String = match reply_optional {
                Some(value) => value,
                None => "stop".to_string()
            };
            match s.as_str() {
                "continue" => {
                    debug!("miner_worker: continue mining");
                    execute_one_batch = true;
                },
                "stop" => {
                    debug!("miner_worker: stop mining");
                    execute_one_batch = false;
                },
                _ => {
                    error!("miner_worker: Unknown reply from MinerWorkerExecutedOneBatch. {:?}", s);
                    execute_one_batch = false;
                }
            }
        }
    }
}
