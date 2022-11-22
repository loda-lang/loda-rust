//! The `loda-rust mine` subcommand, runs the miner daemon process.
use crate::config::{Config, NumberOfWorkers};
use crate::common::PendingProgramsWithPriority;
use crate::mine::analytics_worker;
use crate::mine::{MineEventDirectoryState, MetricsWorker};
use crate::mine::PreventFlooding;
use crate::mine::cronjob_worker;
use crate::mine::miner_worker;
use crate::mine::postmine_worker;
use crate::mine::upload_worker;
use crate::mine::{coordinator_worker, CoordinatorWorkerMessage};
use bastion::prelude::*;
use anyhow::Context;
use std::fs;
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
        instance.start_metrics_worker()?;
        instance.start_coordinator_worker()?;
        instance.start_upload_worker()?;
        instance.start_postmine_worker()?;
        instance.start_miner_workers()?;
        instance.start_analytics_worker()?;
        instance.start_cronjob_worker()?;

        Bastion::start();

        instance.run_launch_procedure()?;

        Bastion::block_until_stopped();
        return Ok(());
    }

    /// Regenerate analytics if it has expired.
    /// 
    /// Load analytics data and pass it on to the `miner_worker` instances.
    /// 
    /// Start mining.
    fn run_launch_procedure(&self) -> anyhow::Result<()> {
        thread::sleep(Duration::from_millis(1000));
        let distributor = Distributor::named("coordinator_worker");
        let tell_result = distributor.tell_everyone(CoordinatorWorkerMessage::RunLaunchProcedure);
        if let Err(error) = tell_result {
            return Err(anyhow::anyhow!("Unable to send RunLaunchProcedure to coordinator_worker_distributor. error: {:?}", error));
        }
        Ok(())
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
    
    fn start_coordinator_worker(&self) -> anyhow::Result<()> {
        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(1)
                    .with_distributor(Distributor::named("coordinator_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        async move {
                            coordinator_worker(
                                ctx,
                            ).await
                        }
                    })
            })
        })
        .map_err(|e| anyhow::anyhow!("couldn't start coordinator_worker. error: {:?}", e))?;
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
        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(1)
                    .with_distributor(Distributor::named("postmine_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        async move {
                            postmine_worker(
                                ctx,
                            ).await
                        }
                    })
            })
        })
        .map_err(|e| anyhow::anyhow!("couldn't start postmine_worker. error: {:?}", e))?;
        Ok(())
    }

    fn start_miner_workers(&self) -> anyhow::Result<()> {
        let config_original: Config = self.config.clone();
        let prevent_flooding = self.prevent_flooding.clone();
        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(self.number_of_workers)
                    .with_distributor(Distributor::named("miner_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        let prevent_flooding_clone = prevent_flooding.clone();
                        let config_clone = config_original.clone();
                        async move {
                            miner_worker(
                                ctx,
                                prevent_flooding_clone,
                                config_clone,
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
        let prevent_flooding = self.prevent_flooding.clone();
        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(1)
                    .with_distributor(Distributor::named("analytics_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        let config_clone: Config = config_original.clone();
                        let prevent_flooding_clone = prevent_flooding.clone();
                        async move {
                            analytics_worker(
                                ctx,
                                config_clone,
                                prevent_flooding_clone,
                            ).await
                        }
                    })
            })
        })
        .map_err(|e| anyhow::anyhow!("couldn't start analytics_worker. error: {:?}", e))?;
        Ok(())
    }
    
    fn start_cronjob_worker(&self) -> anyhow::Result<()> {
        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(1)
                    .with_distributor(Distributor::named("cronjob_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        async move {
                            cronjob_worker(
                                ctx,
                            ).await
                        }
                    })
            })
        })
        .map_err(|e| anyhow::anyhow!("couldn't start cronjob_worker. error: {:?}", e))?;
        Ok(())
    }
}
