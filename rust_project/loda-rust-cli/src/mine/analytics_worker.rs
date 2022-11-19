use super::{MineEventDirectoryState, SharedWorkerState};
use bastion::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;

const ANALYTICS_INTERVAL_MILLIS: u64 = 10000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnalyticsWorkerMessage {
    DoSomething,
}

pub async fn analytics_worker(
    ctx: BastionContext,
    mine_event_dir_state: Arc<Mutex<MineEventDirectoryState>>,
    shared_worker_state: Arc<Mutex<SharedWorkerState>>,
) -> Result<(), ()> {
    let mut progress_time = Instant::now();
    loop {
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed >= (ANALYTICS_INTERVAL_MILLIS as u128) {
            println!("analytics");
            progress_time = Instant::now();
        }

        let timeout = Duration::from_millis(ANALYTICS_INTERVAL_MILLIS);
        let message: SignedMessage = match ctx.try_recv_timeout(timeout).await {
            Ok(message) => message,
            Err(error) => {
                if let ReceiveError::Timeout(_duration) = error {
                    debug!("analytics_worker: timeout happened");
                    continue;
                }
                error!("analytics_worker: Unknown error happened. error: {:?}", error);
                continue;
            }
        };
        MessageHandler::new(message)
            .on_tell(|message: AnalyticsWorkerMessage, _| {
                println!(
                    "analytics_worker: child {}, received broadcast message: {:?}",
                    ctx.current().id(),
                    message
                );
                match message {
                    AnalyticsWorkerMessage::DoSomething => {
                        println!("BEFORE analytics");
                        println!("AFTER analytics");

                        println!("trigger resume analytics");
                        thread::sleep(Duration::from_millis(1000));
                    }
                }
            })
            .on_fallback(|unknown, _sender_addr| {
                error!(
                    "analytics_worker {}, received an unknown message!:\n{:?}",
                    ctx.current().id(),
                    unknown
                );
            });
    }
}
