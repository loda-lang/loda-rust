use super::{MineEventDirectoryState, SharedWorkerState};
use bastion::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;

const CRONJOB_INTERVAL_MILLIS: u64 = 10000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CronjobWorkerMessage {
    DoSomething,
}

pub async fn cronjob_worker(
    ctx: BastionContext,
    mine_event_dir_state: Arc<Mutex<MineEventDirectoryState>>,
    shared_worker_state: Arc<Mutex<SharedWorkerState>>,
) -> Result<(), ()> {
    let mut progress_time = Instant::now();
    loop {
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed >= (CRONJOB_INTERVAL_MILLIS as u128) {
            println!("cron");
            progress_time = Instant::now();
        }

        let timeout = Duration::from_millis(CRONJOB_INTERVAL_MILLIS);
        let message: SignedMessage = match ctx.try_recv_timeout(timeout).await {
            Ok(message) => message,
            Err(error) => {
                if let ReceiveError::Timeout(_duration) = error {
                    debug!("cronjob_worker: timeout happened");
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
                        println!("BEFORE cronjob");
                        println!("AFTER cronjob");

                        println!("trigger resume cronjob");
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
