use super::AnalyticsWorkerMessage;
use super::ExecuteBatchResult;
use super::MinerWorkerMessage;
use super::SharedWorkerState;
use bastion::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const RECEIVE_TIMEOUT_SECONDS: u64 = 1 * 60; // 1 minute

#[derive(Clone, Debug)]
pub enum CoordinatorWorkerMessage {
    RunLaunchProcedure,
    SyncAndAnalyticsIsComplete,
    CronjobTriggerSync,
    MinerWorkerExecutedOneBatch { execute_batch_result: ExecuteBatchResult },
}

pub async fn coordinator_worker(
    ctx: BastionContext,
    shared_worker_state: Arc<Mutex<SharedWorkerState>>,
) -> Result<(), ()> {
    let timeout = Duration::from_secs(RECEIVE_TIMEOUT_SECONDS);
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
        let mut should_run_launch_procedure: bool = false;
        let mut is_sync_and_analytics_complete: bool = false;
        MessageHandler::new(message)
            .on_tell(|message: CoordinatorWorkerMessage, _| {
                println!(
                    "coordinator_worker: child {}, received message: {:?}",
                    ctx.current().id(),
                    message
                );
                match message {
                    CoordinatorWorkerMessage::RunLaunchProcedure => {
                        should_run_launch_procedure = true;
                    },
                    CoordinatorWorkerMessage::CronjobTriggerSync => {
                        println!("!!!!!!!!! trigger sync")
                    },
                    CoordinatorWorkerMessage::SyncAndAnalyticsIsComplete => {
                        is_sync_and_analytics_complete = true;
                    },
                    CoordinatorWorkerMessage::MinerWorkerExecutedOneBatch { execute_batch_result } => {
                        println!("coordinator_worker: executed one batch: {:?}", execute_batch_result);
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
        if should_run_launch_procedure {
            println!("coordinator_worker: Run launch procedure");
            let distributor = Distributor::named("analytics_worker");
            let tell_result = distributor.tell_everyone(AnalyticsWorkerMessage::RunLaunchProcedure);
            if let Err(error) = tell_result {
                error!("coordinator_worker: Unable to send RunLaunchProcedure to analytics_worker_distributor. error: {:?}", error);
            }
        }
        if is_sync_and_analytics_complete {
            match shared_worker_state.lock() {
                Ok(mut state) => {
                    *state = SharedWorkerState::Mining;
                },
                Err(error) => {
                    Bastion::stop();
                    panic!("coordinator_worker: Unable to change state=Mining. error: {:?}", error);
                }
            }
        }
    }
}
