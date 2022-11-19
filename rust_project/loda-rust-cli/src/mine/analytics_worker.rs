use crate::analytics::Analytics;
use crate::config::{Config, NumberOfWorkers};
use super::{CreateFunnel, Funnel, FunnelConfig};
use super::{create_genome_mutate_context, GenomeMutateContext};
use super::{MineEventDirectoryState, SharedWorkerState};
use super::{MinerWorkerMessage, MinerWorkerMessageWithAnalytics, MinerWorkerQuestion};
use bastion::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;

const ANALYTICS_INTERVAL_MILLIS: u64 = 10000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnalyticsWorkerMessage {
    StartAnalyticsJob,
}

pub async fn analytics_worker(
    ctx: BastionContext,
    config: Config,
    mine_event_dir_state: Arc<Mutex<MineEventDirectoryState>>,
    shared_worker_state: Arc<Mutex<SharedWorkerState>>,
) -> Result<(), ()> {
    let miner_worker_distributor = Distributor::named("miner_worker");
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
        let mut perform_analytics: bool = false;
        MessageHandler::new(message)
            .on_tell(|message: AnalyticsWorkerMessage, _| {
                println!(
                    "analytics_worker: child {}, received broadcast message: {:?}",
                    ctx.current().id(),
                    message
                );
                match message {
                    AnalyticsWorkerMessage::StartAnalyticsJob => {
                        perform_analytics = true;
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

        if perform_analytics {
            println!("BEFORE analytics");
            match Analytics::run_if_expired() {
                Ok(()) => {},
                Err(error) => {
                    error!("AFTER analytics. error: {:?}", error);
                    Bastion::stop();
                }
            }

            println!("populating funnel");
            let funnel: Funnel = Funnel::create_funnel_with_file_data(&config);
            println!("populating genome_mutate_context");
            let genome_mutate_context: GenomeMutateContext = create_genome_mutate_context(&config);
            
            // Pass on funnel+genome_mutate_context to miner_workers
            println!("analytics_worker: sending analytics data to miner_workers");
            let instance = MinerWorkerMessageWithAnalytics::new(
                funnel,
                genome_mutate_context,
            );
            let arc_instance = Arc::new(instance);
            let tell_result = miner_worker_distributor.tell_everyone(arc_instance);
            if let Err(error) = tell_result {
                error!("analytics_worker: Unable to send MinerWorkerMessageWithAnalytics to miner_worker_distributor. error: {:?}", error);
            }
    
            thread::sleep(Duration::from_millis(1000));
            println!("AFTER analytics. ok");
        }
    }
}
