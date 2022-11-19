//! The `loda-rust mine` subcommand, runs the miner daemon process.
use crate::config::{Config, NumberOfWorkers};
use crate::common::PendingProgramsWithPriority;
use crate::mine::{analytics_worker, AnalyticsWorkerMessage};
use crate::mine::{MineEventDirectoryState, MetricsWorker, MinerWorkerMessage, MinerWorkerQuestion};
use crate::mine::{CreateFunnel, Funnel, FunnelConfig};
use crate::mine::{create_genome_mutate_context, GenomeMutateContext};
use crate::mine::{create_prevent_flooding, PreventFlooding};
use crate::mine::{cronjob_worker, CronjobWorkerMessage};
use crate::mine::{miner_worker};
use crate::mine::{postmine_worker, SharedWorkerState};
use crate::mine::upload_worker;
use crate::oeis::{load_terms_to_program_id_set, TermsToProgramIdSet};
use bastion::prelude::*;
use num_bigint::{BigInt, ToBigInt};
use anyhow::Context;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum SubcommandMineMetricsMode {
    NoMetricsServer,
    RunMetricsServer,
}

pub struct SubcommandMine {
    metrics_mode: SubcommandMineMetricsMode,
    number_of_workers: usize,
    config: Config,
    prevent_flooding: Arc<Mutex<PreventFlooding>>,
    mine_event_dir_state: Arc<Mutex<MineEventDirectoryState>>,
    shared_worker_state: Arc<Mutex<SharedWorkerState>>,
}

impl SubcommandMine {
    pub async fn run(
        metrics_mode: SubcommandMineMetricsMode
    ) -> anyhow::Result<()> {
        Bastion::init();
        
        let mut instance = SubcommandMine::new(metrics_mode);
        instance.prepare_mineevent_dir()?;
        instance.print_info();
        instance.reload_mineevent_directory_state()?;
        instance.populate_prevent_flooding_mechanism()?;
        instance.start_metrics_worker()?;
        instance.start_upload_worker()?;
        instance.start_postmine_worker()?;
        instance.start_miner_workers()?;
        instance.start_analytics_worker()?;
        instance.start_cronjob_worker()?;

        Bastion::start();

        instance.perform_analytics()?;

        instance.experiment_contact_miner_workers1();
        instance.experiment_contact_miner_workers2();

        Bastion::block_until_stopped();
        return Ok(());
    }

    /// Regenerate analytics if it has expired.
    /// 
    /// Load analytics data and pass it on to the `miner_worker` instances.
    fn perform_analytics(&self) -> anyhow::Result<()> {
        thread::sleep(Duration::from_millis(1000));
        let miner_worker_distributor = Distributor::named("analytics_worker");
        let tell_result = miner_worker_distributor.tell_everyone(AnalyticsWorkerMessage::StartAnalyticsJob);
        if let Err(error) = tell_result {
            return Err(anyhow::anyhow!("Unable to send StartAnalyticsJob to analytics_worker_distributor. error: {:?}", error));
        }
        Ok(())
    }

    fn experiment_contact_miner_workers1(&self) {
        thread::sleep(Duration::from_millis(10000));
        let miner_worker_distributor = Distributor::named("miner_worker");
        let tell_result = miner_worker_distributor.tell_everyone(MinerWorkerMessage::InvalidateAnalytics);
        if let Err(error) = tell_result {
            error!("miner_worker: Unable to send InvalidateAnalytics to miner_worker_distributor. error: {:?}", error);
        }
    }

    fn experiment_contact_miner_workers2(&self) {
        thread::sleep(Duration::from_millis(10000));
        let miner_worker_distributor = Distributor::named("miner_worker");

        let ask_result = miner_worker_distributor.ask_everyone(MinerWorkerQuestion::Launch);
        let answers: Vec<Answer> = ask_result.expect("boom");
        for answer in answers.into_iter() {
            run!(async move {
                MessageHandler::new(answer.await.expect("couldn't receive reply"))
                    .on_tell(|response: String, _| {
                        println!("response: {:?}", response);
                    })
                    .on_fallback(|unknown, _sender_addr| {
                        error!(
                            "distributor_test: uh oh, I received a message I didn't understand\n {:?}",
                            unknown
                        );
                    });
            });
        }
        // TODO: wait for response from all
    }

    fn new(
        metrics_mode: SubcommandMineMetricsMode
    ) -> Self {
        let config = Config::load();
        let number_of_workers: usize = config.resolve_number_of_miner_workers();
        Self {
            metrics_mode: metrics_mode,
            number_of_workers: number_of_workers,
            config: config,
            prevent_flooding: Arc::new(Mutex::new(PreventFlooding::new())),
            mine_event_dir_state: Arc::new(Mutex::new(MineEventDirectoryState::new())),
            shared_worker_state: Arc::new(Mutex::new(SharedWorkerState::Analytics)),
        }
    }

    /// Create the `~/.loda-rust/mine-event` dir if needed
    fn prepare_mineevent_dir(&self) -> anyhow::Result<()> {
        let path = self.config.mine_event_dir();
        if !path.is_dir() {
            fs::create_dir(&path)
                .with_context(|| format!("Unable to create mine-event dir: {:?}", &path))?;

        }
        assert!(path.is_dir());
        Ok(())
    }

    fn print_info(&self) {
        // Ascii art generated using:
        // https://patorjk.com/software/taag/#p=display&f=ANSI%20Shadow&t=LODA-RUST
        // Font="ANSI Shadow"
        let banner = r#"
██╗      ██████╗ ██████╗  █████╗       ██████╗ ██╗   ██╗███████╗████████╗
██║     ██╔═══██╗██╔══██╗██╔══██╗      ██╔══██╗██║   ██║██╔════╝╚══██╔══╝
██║     ██║   ██║██║  ██║███████║█████╗██████╔╝██║   ██║███████╗   ██║   
██║     ██║   ██║██║  ██║██╔══██║╚════╝██╔══██╗██║   ██║╚════██║   ██║   
███████╗╚██████╔╝██████╔╝██║  ██║      ██║  ██║╚██████╔╝███████║   ██║   
╚══════╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝      ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝"#;
        println!("{}", banner);

        const VERSION: &str = env!("CARGO_PKG_VERSION");
        let build_mode: &str;
        if cfg!(debug_assertions) {
            build_mode = "DEBUG (terrible performance!)";
        } else {
            build_mode = "RELEASE";
        }
        println!("LODA-RUST version: {}, build: {}\n", VERSION, build_mode);

        println!("metrics mode: {:?}", self.metrics_mode);
        println!("number of workers: {}", self.number_of_workers);

        println!("Press CTRL-C to stop the miner.\n\n");
    }

    fn reload_mineevent_directory_state(&mut self) -> anyhow::Result<()> {
        let pending = PendingProgramsWithPriority::create(&self.config)
            .context("reload_mineevent_directory_state")?;
        match self.mine_event_dir_state.lock() {
            Ok(mut state) => {
                state.set_number_of_mined_high_prio(pending.paths_high_prio().len());
                state.set_number_of_mined_low_prio(pending.paths_low_prio().len());
            },
            Err(error) => {
                error!("reload_mineevent_directory_state: mine_event_dir_state.lock() failed. {:?}", error);
            }
        }
        Ok(())
    }

    fn populate_prevent_flooding_mechanism(&mut self) -> anyhow::Result<()> {
        let prevent_flooding: PreventFlooding = create_prevent_flooding(&self.config)?;
        self.prevent_flooding = Arc::new(Mutex::new(prevent_flooding));
        Ok(())
    }

    fn start_metrics_worker(&self) -> anyhow::Result<()> {
        match self.metrics_mode {
            SubcommandMineMetricsMode::NoMetricsServer => {
                MetricsWorker::start_without_server()?;
            },
            SubcommandMineMetricsMode::RunMetricsServer => {
                let listen_on_port: u16 = self.config.miner_metrics_listen_port();
                MetricsWorker::start_with_server(listen_on_port, self.number_of_workers as u64)?;
            }
        };
        Ok(())
    }
    
    fn start_upload_worker(&self) -> anyhow::Result<()> {
        let miner_program_upload_endpoint: String = self.config.miner_program_upload_endpoint().clone();
        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(1)
                    .with_distributor(Distributor::named("upload_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        let miner_program_upload_endpoint_clone = miner_program_upload_endpoint.clone();
                        async move {
                            upload_worker(
                                ctx,
                                miner_program_upload_endpoint_clone,
                            ).await
                        }
                    })

            })
        })
        .map_err(|e| anyhow::anyhow!("couldn't start upload_worker. error: {:?}", e))?;
        Ok(())
    }
    
    fn start_postmine_worker(&self) -> anyhow::Result<()> {
        let mine_event_dir_state = self.mine_event_dir_state.clone();
        let shared_worker_state = self.shared_worker_state.clone();
        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(1)
                    .with_distributor(Distributor::named("postmine_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        let mine_event_dir_state_clone = mine_event_dir_state.clone();
                        let shared_worker_state_clone = shared_worker_state.clone();
                        async move {
                            postmine_worker(
                                ctx,
                                mine_event_dir_state_clone,
                                shared_worker_state_clone,
                            ).await
                        }
                    })
            })
        })
        .map_err(|e| anyhow::anyhow!("couldn't start postmine_worker. error: {:?}", e))?;
        Ok(())
    }

    fn start_miner_workers(&self) -> anyhow::Result<()> {
        println!("populating terms_to_program_id");
        let oeis_stripped_file: PathBuf = self.config.oeis_stripped_file();
        let padding_value: BigInt = FunnelConfig::WILDCARD_MAGIC_VALUE.to_bigint().unwrap();
        let terms_to_program_id: TermsToProgramIdSet = load_terms_to_program_id_set(
            &oeis_stripped_file, 
            FunnelConfig::MINIMUM_NUMBER_OF_REQUIRED_TERMS, 
            FunnelConfig::TERM_COUNT,
            &padding_value
        )
        .map_err(|e| anyhow::anyhow!("Unable to load terms for program ids. error: {:?}", e))?;

        let terms_to_program_id_arc: Arc<TermsToProgramIdSet> = Arc::new(terms_to_program_id);

        let funnel: Funnel = Funnel::create_empty_funnel();
        let genome_mutate_context: GenomeMutateContext = GenomeMutateContext::new_empty();
        
        let config_original: Config = self.config.clone();
        let prevent_flooding = self.prevent_flooding.clone();
        let mine_event_dir_state = self.mine_event_dir_state.clone();
        let shared_worker_state = self.shared_worker_state.clone();

        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(self.number_of_workers)
                    .with_distributor(Distributor::named("miner_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        let terms_to_program_id_arc_clone = terms_to_program_id_arc.clone();
                        let prevent_flooding_clone = prevent_flooding.clone();
                        let mine_event_dir_state_clone = mine_event_dir_state.clone();
                        let shared_worker_state_clone = shared_worker_state.clone();
                        let config_clone = config_original.clone();
                        let funnel_clone = funnel.clone();
                        let genome_mutate_context_clone = genome_mutate_context.clone();
                        async move {
                            miner_worker(
                                ctx,
                                terms_to_program_id_arc_clone,
                                prevent_flooding_clone,
                                mine_event_dir_state_clone,
                                shared_worker_state_clone,
                                config_clone,
                                funnel_clone,
                                genome_mutate_context_clone,
                            ).await
                        }
                    })
            })
        })
        .map_err(|e| anyhow::anyhow!("couldn't start miner_workers. error: {:?}", e))?;
        Ok(())
    }
    
    fn start_analytics_worker(&self) -> anyhow::Result<()> {
        let config_original: Config = self.config.clone();
        let mine_event_dir_state = self.mine_event_dir_state.clone();
        let shared_worker_state = self.shared_worker_state.clone();
        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(1)
                    .with_distributor(Distributor::named("analytics_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        let config_clone: Config = config_original.clone();
                        let mine_event_dir_state_clone = mine_event_dir_state.clone();
                        let shared_worker_state_clone = shared_worker_state.clone();
                        async move {
                            analytics_worker(
                                ctx,
                                config_clone,
                                mine_event_dir_state_clone,
                                shared_worker_state_clone,
                            ).await
                        }
                    })
            })
        })
        .map_err(|e| anyhow::anyhow!("couldn't start analytics_worker. error: {:?}", e))?;
        Ok(())
    }
    
    fn start_cronjob_worker(&self) -> anyhow::Result<()> {
        let mine_event_dir_state = self.mine_event_dir_state.clone();
        let shared_worker_state = self.shared_worker_state.clone();
        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(1)
                    .with_distributor(Distributor::named("cronjob_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        let mine_event_dir_state_clone = mine_event_dir_state.clone();
                        let shared_worker_state_clone = shared_worker_state.clone();
                        async move {
                            cronjob_worker(
                                ctx,
                                mine_event_dir_state_clone,
                                shared_worker_state_clone,
                            ).await
                        }
                    })
            })
        })
        .map_err(|e| anyhow::anyhow!("couldn't start cronjob_worker. error: {:?}", e))?;
        Ok(())
    }
}
