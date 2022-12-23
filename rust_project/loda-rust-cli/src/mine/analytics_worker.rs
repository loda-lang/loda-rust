use crate::analytics::{Analytics, AnalyticsDirectory};
use crate::common::PendingProgramsWithPriority;
use crate::config::Config;
use crate::mine::{CoordinatorWorkerMessage, MineEventDirectoryState};
use crate::oeis::{load_terms_to_program_id_set, TermsToProgramIdSet};
use super::{CreateFunnel, Funnel, FunnelConfig};
use super::{CreateGenomeMutateContextMode, create_genome_mutate_context, GenomeMutateContext};
use super::MinerWorkerMessageWithAnalytics;
use super::{create_prevent_flooding, PreventFlooding};
use super::{MinerSyncExecute, MinerSyncExecuteStatus};
use bastion::prelude::*;
use num_bigint::{BigInt, ToBigInt};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnalyticsWorkerMessage {
    PerformSyncAndAnalytics,
}

pub async fn analytics_worker(
    ctx: BastionContext,
    config: Config,
    prevent_flooding: Arc<Mutex<PreventFlooding>>,
) -> Result<(), ()> {
    loop {
        let message: SignedMessage = match ctx.recv().await {
            Ok(message) => message,
            Err(error) => {
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
                    AnalyticsWorkerMessage::PerformSyncAndAnalytics => {
                        perform_sync_and_analytics(&config, prevent_flooding.clone());
                    },
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

fn perform_sync_and_analytics(
    config: &Config,
    prevent_flooding: Arc<Mutex<PreventFlooding>>,
) {
    let command_windows: String = config.miner_sync_executable_command_windows();
    let executable_path: PathBuf = config.miner_sync_executable();
    let sync_status: MinerSyncExecuteStatus = match MinerSyncExecute::execute(&command_windows, &executable_path) {
        Ok(value) => value,
        Err(error) => {
            Bastion::stop();
            panic!("Problem executing MinerSyncExecute: {:?}", error);
        }
    };
    println!("Successfully executed MinerSyncExecute. status: {:?}", sync_status);

    let analytics_run_result: anyhow::Result<()> = match sync_status {
        MinerSyncExecuteStatus::NoChange => {
            // Data is already uptodate, then skip no need to regenerate analytics.
            println!("BEFORE analytics - run_if_expired");
            Analytics::oeis_run_if_expired()
        },
        MinerSyncExecuteStatus::Changed => {
            // Data has been modified, then analytics needs to be regenerated.
            println!("BEFORE analytics - run_force");
            Analytics::oeis_run_force()
        }
    };
    match analytics_run_result {
        Ok(()) => {},
        Err(error) => {
            Bastion::stop();
            panic!("AFTER analytics. error: {:?}", error);
        }
    }

    let prevent_flooding_x: PreventFlooding = match create_prevent_flooding(&config) {
        Ok(value) => value,
        Err(error) => {
            Bastion::stop();
            panic!("analytics_worker: create_prevent_flooding failed. error: {:?}", error);
        }
    };
    match prevent_flooding.lock() {
        Ok(mut instance) => {
            *instance = prevent_flooding_x;
        },
        Err(error) => {
            Bastion::stop();
            panic!("analytics_worker: Unable to populate PreventFlooding mechanism. error: {:?}", error);
        }
    }

    println!("populating terms_to_program_id");
    let oeis_stripped_file: PathBuf = config.oeis_stripped_file();
    let padding_value: BigInt = FunnelConfig::WILDCARD_MAGIC_VALUE.to_bigint().unwrap();
    let terms_to_program_id_result = load_terms_to_program_id_set(
        &oeis_stripped_file, 
        FunnelConfig::MINIMUM_NUMBER_OF_REQUIRED_TERMS, 
        FunnelConfig::TERM_COUNT,
        &padding_value
    );
    let terms_to_program_id: TermsToProgramIdSet = match terms_to_program_id_result {
        Ok(value) => value,
        Err(error) => {
            Bastion::stop();
            panic!("analytics_worker: Unable to load terms for program ids. error: {:?}", error);
        }
    };
    let terms_to_program_id_arc: Arc<TermsToProgramIdSet> = Arc::new(terms_to_program_id);

    println!("populating funnel");
    let funnel: Funnel = Funnel::create_funnel_with_file_data(&config);

    println!("populating genome_mutate_context");
    let analytics_directory = AnalyticsDirectory::new(
        config.analytics_oeis_dir()
    ).expect("unable to create AnalyticsDirectory instance");
    let genome_mutate_context: GenomeMutateContext = create_genome_mutate_context(CreateGenomeMutateContextMode::OEIS, &config, analytics_directory)
        .expect("analytics_worker couldn't create GenomeMutateContext");
    
    // Pass on funnel+genome_mutate_context to miner_workers
    println!("analytics_worker: sending analytics data to miner_workers");
    let instance = MinerWorkerMessageWithAnalytics::new(
        funnel,
        genome_mutate_context,
        terms_to_program_id_arc,
    );
    let arc_instance = Arc::new(instance);
    debug!("analytics_worker: miner_workers.ask_everyone(MinerWorkerMessageWithAnalytics)");
    let ask_result = Distributor::named("miner_worker").ask_everyone(arc_instance);

    // wait for all the "miner_worker" instances to have received the data. 
    let answers: Vec<Answer> = ask_result
        .expect("analytics_worker miner_workers.ask_everyone(MinerWorkerMessageWithAnalytics) couldn't ask everyone");
    let mut count_answers: usize = 0;
    let mut count_success: usize = 0;
    for answer in answers.into_iter() {
        count_answers += 1;

        // Wait for a single miner_worker to respond to the question
        let response_result: Option<bool> = run!(blocking! {
            let mut success = false;
            MessageHandler::new(answer.await.expect("couldn't receive reply"))
                .on_tell(|response: String, _| {
                    if response == "miner_worker_updated_ok" {
                        success = true;
                    } else {
                        error!("analytics_worker: Unexpected response: {:?}", response);
                    }
                })
                .on_fallback(|unknown, _sender_addr| {
                    error!(
                        "analytics_worker: uh oh, I received a message I didn't understand\n {:?}",
                        unknown
                    );
                });
            success
        });

        // Only increment, if the miner_worker replied successfully
        if let Some(value) = response_result {
            if value {
                count_success += 1;
            }
        }
    }
    if count_answers != count_success {
        Bastion::stop();
        panic!("analytics_worker: Expected same number of answers as there are workers. {} != {}", count_answers, count_success);
    }
    debug!("analytics_worker: received answers from all miner_workers");
    // All the miner_workers have now received data

    // Determine how many files are in the "~/.loda-rust/mine-event" directory
    let mineevent_dir_state: MineEventDirectoryState = match PendingProgramsWithPriority::create(config) {
        Ok(pending) => {
            let mut instance = MineEventDirectoryState::new();
            instance.set_number_of_mined_high_prio(pending.paths_high_prio().len());
            instance.set_number_of_mined_low_prio(pending.paths_low_prio().len());
            instance
        },
        Err(error) => {
            error!("analytics_worker: Unable to determine the number of pending programs. {:?}", error);
            MineEventDirectoryState::new()
        }
    };

    let tell_result = Distributor::named("coordinator_worker").tell_everyone(
        CoordinatorWorkerMessage::SyncAndAnalyticsIsComplete { mineevent_dir_state }
    );
    if let Err(error) = tell_result {
        Bastion::stop();
        panic!("analytics_worker: Unable to send SyncAndAnalyticsIsComplete to coordinator_worker. error: {:?}", error);
    }
}
