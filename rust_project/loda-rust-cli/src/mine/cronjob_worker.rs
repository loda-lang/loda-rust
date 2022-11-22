use super::CoordinatorWorkerMessage;
use bastion::prelude::*;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const CRONJOB_SYNC_INTERVAL_SECONDS: u64 = 60 * 60; // 1 hour
const CRONJOB_HEARTBEAT_INTERVAL_SECONDS: u64 = 5 * 60; // 5 minutes

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CronjobWorkerMessage {
    #[allow(dead_code)]
    DoSomething,
}

pub async fn cronjob_worker(
    ctx: BastionContext,
) -> Result<(), ()> {
    let mut progress_time = Instant::now();
    loop {
        let elapsed: u64 = progress_time.elapsed().as_secs();
        if elapsed >= CRONJOB_SYNC_INTERVAL_SECONDS {
            debug!("cronjob_worker: wake up");
            let distributor = Distributor::named("coordinator_worker");
            let tell_result = distributor.tell_everyone(CoordinatorWorkerMessage::CronjobTriggerSync);
            if let Err(error) = tell_result {
                error!("Unable to send CronjobTriggerSync to coordinator_worker. error: {:?}", error);
            }
            progress_time = Instant::now();
        }

        let timeout = Duration::from_secs(CRONJOB_HEARTBEAT_INTERVAL_SECONDS);
        let message: SignedMessage = match ctx.try_recv_timeout(timeout).await {
            Ok(message) => message,
            Err(error) => {
                if let ReceiveError::Timeout(_duration) = error {
                    // debug!("cronjob_worker: timeout happened");
                    continue;
                }
                error!("cronjob_worker: Unknown error happened. error: {:?}", error);
                continue;
            }
        };
        MessageHandler::new(message)
            .on_tell(|message: CronjobWorkerMessage, _| {
                println!(
                    "cronjob_worker: child {}, received broadcast message: {:?}",
                    ctx.current().id(),
                    message
                );
                match message {
                    CronjobWorkerMessage::DoSomething => {
                        println!("CronjobWorkerMessage::DoSomething");
                        thread::sleep(Duration::from_millis(1000));
                    }
                }
            })
            .on_fallback(|unknown, _sender_addr| {
                error!(
                    "cronjob_worker {}, received an unknown message!:\n{:?}",
                    ctx.current().id(),
                    unknown
                );
            });
    }
}
