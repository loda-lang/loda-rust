use super::AnalyticsWorkerMessage;
use super::ExecuteBatchResult;
use super::MineEventDirectoryState;
use super::MinerWorkerMessage;
use super::PostmineWorkerMessage;
use bastion::prelude::*;
use std::time::Duration;
use std::thread;

const RECEIVE_TIMEOUT_SECONDS: u64 = 1 * 60; // 1 minute

#[derive(Clone, Debug)]
pub enum CoordinatorWorkerMessage {
    RunLaunchProcedure,
    SyncAndAnalyticsIsComplete,
    PostmineJobComplete,
    CronjobTriggerSync,
}

#[derive(Debug, Clone)]
pub enum CoordinatorWorkerQuestion {
    MinerWorkerExecutedOneBatch { execute_batch_result: ExecuteBatchResult },
}


pub async fn coordinator_worker(
    ctx: BastionContext,
) -> Result<(), ()> {
    let timeout = Duration::from_secs(RECEIVE_TIMEOUT_SECONDS);
    let mut mineevent_dir_state = MineEventDirectoryState::new();
    let mut is_postmine_running = false;
    loop {
        let message: SignedMessage = match ctx.try_recv_timeout(timeout).await {
            Ok(message) => message,
            Err(error) => {
                if let ReceiveError::Timeout(_duration) = error {
                    error!("coordinator_worker: timeout happened. No activity for a while.");
                    continue;
                }
                error!("coordinator_worker: Unknown error happened. error: {:?}", error);
                continue;
            }
        };
        let mut trigger_start_postmine_job: bool = false;
        MessageHandler::new(message)
            .on_tell(|message: CoordinatorWorkerMessage, _| {
                println!(
                    "coordinator_worker: child {}, received message: {:?}",
                    ctx.current().id(),
                    message
                );
                match message {
                    CoordinatorWorkerMessage::RunLaunchProcedure => {
                        run_launch_procedure();
                    },
                    CoordinatorWorkerMessage::CronjobTriggerSync => {
                        println!("!!!!!!!!! trigger sync")
                    },
                    CoordinatorWorkerMessage::SyncAndAnalyticsIsComplete => {
                        start_mining();
                    },
                    CoordinatorWorkerMessage::PostmineJobComplete => {
                        is_postmine_running = false;
                        mineevent_dir_state.reset();
                        println!("coordinator_worker: postmine job is complete. Resume mining again");
                        start_mining();
                    }
                }
            })
            .on_question(|message: CoordinatorWorkerQuestion, sender| {
                // debug!("coordinator_worker {}, received a question: \n{:?}", 
                //     ctx.current().id(),
                //     message
                // );
                match message {
                    CoordinatorWorkerQuestion::MinerWorkerExecutedOneBatch { execute_batch_result } => {
                        debug!("coordinator_worker: executed one batch: {:?}", execute_batch_result);
                        mineevent_dir_state.accumulate_stats(&execute_batch_result);
                        let reply: String;
                        if mineevent_dir_state.has_reached_mining_limit() {
                            // debug!("reached mining limit. {:?}", state);
                            // don't schedule another batch execute. Instead trigger the postmine job to run.
                            trigger_start_postmine_job = true;
                            reply = "stop".to_string();
                        } else {
                            // the discovery count is lower than the limit
                            // tell the worker to execute another batch
                            reply = "continue".to_string();
                        }
                        match sender.reply(reply) {
                            Ok(value) => {
                                debug!("coordinator_worker: reply ok: {:?}", value);
                            },
                            Err(error) => {
                                error!("coordinator_worker: reply error: {:?}", error);
                            }
                        };
                    },
                }
            })
            .on_fallback(|unknown, _sender_addr| {
                error!(
                    "coordinator_worker {}, received an unknown message!:\n{:?}",
                    ctx.current().id(),
                    unknown
                );
            });
        if trigger_start_postmine_job {
            // ensure it only gets triggered once. 
            // if postmine is already in progress, and it gets triggered over and over, then it's a mess.
            if is_postmine_running {
                // do nothing                
            } else {
                is_postmine_running = true;
                // TODO: if the discovery count is greater than the limit, wait for all workers to finish, tell postmine to run
                println!("coordinator_worker: trigger start postmine - pretend we are waiting for all workers to finish");
                thread::sleep(Duration::from_millis(1000));
    
                println!("coordinator_worker: trigger start postmine - pretend all miner workers have finished");
                let distributor = Distributor::named("postmine_worker");
                let tell_result = distributor
                    .tell_everyone(PostmineWorkerMessage::StartPostmineJob);
                if let Err(error) = tell_result {
                    error!("coordinator_worker: Unable to send StartPostmineJob. error: {:?}", error);
                }
            }
        }
    }
}

/// tell the `analytics_worker` instance to perform the launch procedure
fn run_launch_procedure() {
    let distributor = Distributor::named("analytics_worker");
    let tell_result = distributor.tell_everyone(AnalyticsWorkerMessage::RunLaunchProcedure);
    if let Err(error) = tell_result {
        error!("coordinator_worker: Unable to send RunLaunchProcedure to analytics_worker_distributor. error: {:?}", error);
    }
}

/// tell all `miner_worker` instances to start mining
fn start_mining() {
    let distributor = Distributor::named("miner_worker");
    let tell_result = distributor.tell_everyone(MinerWorkerMessage::StartMining);
    if let Err(error) = tell_result {
        Bastion::stop();
        panic!("coordinator_worker: Unable to send StartMining to miner_worker_distributor. error: {:?}", error);
    }
}
