use crate::mine::AnalyticsWorkerMessage;
use bastion::prelude::*;
use std::time::Duration;

const RECEIVE_TIMEOUT_SECONDS: u64 = 1 * 60; // 1 minute

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoordinatorWorkerMessage {
    RunLaunchProcedure,
    CronjobTriggerSync,
}

pub async fn coordinator_worker(
    ctx: BastionContext,
) -> Result<(), ()> {
    let timeout = Duration::from_secs(RECEIVE_TIMEOUT_SECONDS);
    loop {
        let message: SignedMessage = match ctx.try_recv_timeout(timeout).await {
            Ok(message) => message,
            Err(error) => {
                if let ReceiveError::Timeout(_duration) = error {
                    error!("state_worker: timeout happened. No activity for a while.");
                    continue;
                }
                error!("state_worker: Unknown error happened. error: {:?}", error);
                continue;
            }
        };
        let mut should_run_launch_procedure: bool = false;
        MessageHandler::new(message)
            .on_tell(|message: CoordinatorWorkerMessage, _| {
                println!(
                    "state_worker: child {}, received message: {:?}",
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
                }
            })
            .on_fallback(|unknown, _sender_addr| {
                error!(
                    "state_worker {}, received an unknown message!:\n{:?}",
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
    }
}
