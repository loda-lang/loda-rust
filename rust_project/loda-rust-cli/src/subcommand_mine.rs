//! The `loda-rust mine` subcommand, runs the miner daemon process.
use crate::mine::{FunnelConfig, MinerThreadMessageToCoordinator, start_miner_loop, MovingAverage, MetricsPrometheus, Recorder, SinkRecorder};
use crate::config::{Config, MinerCPUStrategy};
use bastion::prelude::*;
use loda_rust_core::control::{DependencyManager, DependencyManagerFileSystemMode};
use loda_rust_core::execute::ProgramCache;
use num_bigint::{BigInt, ToBigInt};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver};
use std::time::Instant;
use std::convert::TryFrom;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::sync::{Arc, Mutex};
use indicatif::HumanDuration;
use crate::oeis::{load_terms_to_program_id_set, TermsToProgramIdSet};
use crate::mine::{PreventFlooding, prevent_flooding_populate};
use crate::common::find_asm_files_recursively;

extern crate num_cpus;

const PREVENT_FLOODING_CACHE_CAPACITY: usize = 300000;

#[derive(Debug)]
pub enum SubcommandMineMetricsMode {
    NoMetricsServer,
    RunMetricsServer,
}

type MyRegistry = std::sync::Arc<std::sync::Mutex<prometheus_client::registry::Registry<std::boxed::Box<dyn prometheus_client::encoding::text::SendEncodeMetric>>>>;

pub struct SubcommandMine {
    metrics_mode: SubcommandMineMetricsMode,
    number_of_workers: usize,
    config: Config,
    prevent_flooding: Arc<Mutex<PreventFlooding>>,
}

impl SubcommandMine {
    pub async fn run(
        metrics_mode: SubcommandMineMetricsMode
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Bastion::init();
        Bastion::start();
        
        let mut instance = SubcommandMine::new(metrics_mode);
        instance.check_prerequisits()?;
        instance.print_info();
        instance.populate_prevent_flooding_mechanism()?;
        instance.run_miner_workers().await?;

        Bastion::stop();
        return Ok(());
    }

    fn new(
        metrics_mode: SubcommandMineMetricsMode
    ) -> Self {
        let config = Config::load();
        let number_of_workers: usize = Self::number_of_workers(config.miner_cpu_strategy());
        Self {
            metrics_mode: metrics_mode,
            number_of_workers: number_of_workers,
            config: config,
            prevent_flooding: Arc::new(Mutex::new(PreventFlooding::new())),
        }
    }

    fn number_of_workers(miner_cpu_strategy: MinerCPUStrategy) -> usize {
        if miner_cpu_strategy == MinerCPUStrategy::Min {
            return 1;
        }
        let number_of_available_cpus: usize = num_cpus::get();
        assert!(number_of_available_cpus >= 1_usize);
        assert!(number_of_available_cpus < 1000_usize);
        let number_of_threads: usize = match miner_cpu_strategy {
            MinerCPUStrategy::Min => 1,
            MinerCPUStrategy::Half => number_of_available_cpus / 2,
            MinerCPUStrategy::Max => number_of_available_cpus,
            MinerCPUStrategy::CPU { count } => count as usize,
        };
        // Ensures that zero is never returned
        number_of_threads.max(1)
    }

    fn check_prerequisits(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let path0 = self.config.analytics_dir();
        if !path0.is_dir() {
            error!("Could not find analytics dir at path: {:?}. Please run 'loda-rust analytics' to create this dir.", path0);
            panic!("check_prerequisits - missing analytics_dir");
        }
        let path1 = self.config.mine_event_dir();
        if !path1.is_dir() {
            error!("Could not find mine_event_dir at path: {:?}. Please rerun 'loda-rust install' to create this dir.", path1);
            panic!("check_prerequisits - missing mine_event_dir");
        }
        Ok(())
    }

    fn print_info(&self) {
        println!("metrics mode: {:?}", self.metrics_mode);
        println!("Number of workers: {}", self.number_of_workers);
        print_info_about_start_conditions();
    }

    fn populate_prevent_flooding_mechanism(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        let loda_programs_oeis_dir: PathBuf = self.config.loda_programs_oeis_dir();
        let mine_event_dir: PathBuf = self.config.mine_event_dir();
        let oeis_divergent_dir: PathBuf = self.config.loda_outlier_programs_repository_oeis_divergent();
    
        let mut paths0: Vec<PathBuf> = find_asm_files_recursively(&mine_event_dir);
        println!("PreventFlooding: number of .asm files in mine_event_dir: {:?}", paths0.len());
        let mut paths1: Vec<PathBuf> = find_asm_files_recursively(&oeis_divergent_dir);
        println!("PreventFlooding: number of .asm files in oeis_divergent_dir: {:?}", paths1.len());
        let mut paths: Vec<PathBuf> = vec!();
        paths.append(&mut paths0);
        paths.append(&mut paths1);
        println!("PreventFlooding: number of .asm files in total: {:?}", paths.len());
    
        let mut dependency_manager = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            loda_programs_oeis_dir,
        );
        let capacity = NonZeroUsize::new(PREVENT_FLOODING_CACHE_CAPACITY).unwrap();
        let mut cache = ProgramCache::with_capacity(capacity);
        let mut prevent_flooding = PreventFlooding::new();
        prevent_flooding_populate(&mut prevent_flooding, &mut dependency_manager, &mut cache, paths);
        println!("PreventFlooding: number of programs added: {}", prevent_flooding.len());
        println!("PreventFlooding: elapsed: {}", HumanDuration(start.elapsed()));
        self.prevent_flooding = Arc::new(Mutex::new(prevent_flooding));
        Ok(())
    }

    async fn run_miner_workers(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        match self.metrics_mode {
            SubcommandMineMetricsMode::NoMetricsServer => {
                return self.run_without_metrics().await
            },
            SubcommandMineMetricsMode::RunMetricsServer => {
                return self.run_with_prometheus_metrics().await
            }
        }
    }

    async fn run_without_metrics(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let (sender, receiver) = channel::<MinerThreadMessageToCoordinator>();

        let minercoordinator_thread = tokio::spawn(async move {
            coordinator_thread_metrics_sink(receiver);
        });

        let recorder: Box<dyn Recorder + Send> = Box::new(SinkRecorder {});

        self.spawn_workers(sender, recorder);

        // Run forever, press CTRL-C to stop.
        minercoordinator_thread.await?;

        Ok(())
    }

    async fn run_with_prometheus_metrics(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let listen_on_port: u16 = self.config.miner_metrics_listen_port();
        println!("miner metrics can be downloaded here: http://localhost:{}/metrics", listen_on_port);

        let (sender, receiver) = channel::<MinerThreadMessageToCoordinator>();

        let mut registry = <Registry>::default();
        let metrics = MetricsPrometheus::new(&mut registry);
        metrics.number_of_workers.set(self.number_of_workers as u64);

        let registry2: MyRegistry = Arc::new(Mutex::new(registry));

        let _ = tokio::spawn(async move {
            let result = webserver_with_metrics(registry2, listen_on_port).await;
            if let Err(error) = result {
                error!("webserver thread failed with error: {:?}", error);
            }
        });

        let minercoordinator_metrics = metrics.clone();
        let minercoordinator_thread = tokio::spawn(async move {
            coordinator_thread_metrics_prometheus(receiver, minercoordinator_metrics);
        });

        let recorder: Box<dyn Recorder + Send> = Box::new(metrics);
        self.spawn_workers(sender, recorder);

        // Run forever, press CTRL-C to stop.
        minercoordinator_thread.await?;

        Ok(())
    }

    fn spawn_workers(
        &self, 
        sender: std::sync::mpsc::Sender<MinerThreadMessageToCoordinator>, 
        recorder: Box<dyn Recorder + Send>
    ) {
        let oeis_stripped_file: PathBuf = self.config.oeis_stripped_file();
        let padding_value: BigInt = FunnelConfig::WILDCARD_MAGIC_VALUE.to_bigint().unwrap();
        let load_result = load_terms_to_program_id_set(
            &oeis_stripped_file, 
            FunnelConfig::MINIMUM_NUMBER_OF_REQUIRED_TERMS, 
            FunnelConfig::TERM_COUNT,
            &padding_value
        );
        let terms_to_program_id: TermsToProgramIdSet = match load_result {
            Ok(value) => value,
            Err(error) => {
                panic!("Unable to load program ids. {:?}", error);
            }
        };
        let terms_to_program_id_arc: Arc<TermsToProgramIdSet> = Arc::new(terms_to_program_id);
    
        for worker_id in 0..self.number_of_workers {
            println!("Spawn worker id: {}", worker_id);
            let sender_clone = sender.clone();
            let recorder_clone: Box<dyn Recorder + Send> = recorder.clone();
            let terms_to_program_id_arc_clone = terms_to_program_id_arc.clone();
            let prevent_flooding_clone = self.prevent_flooding.clone();
            let _ = tokio::spawn(async move {
                start_miner_loop(
                    sender_clone, 
                    recorder_clone, 
                    terms_to_program_id_arc_clone,
                    prevent_flooding_clone
                );
            });
            thread::sleep(Duration::from_millis(2000));
        }
        println!("\nPress CTRL-C to stop the miner.");
    }
}

async fn webserver_with_metrics(registry: MyRegistry, listen_port: u16) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = tide::with_state(State {
        registry: registry,
    });
    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.at("/metrics")
        .get(|req: tide::Request<State>| async move {
            let mut encoded = Vec::new();
            encode(&mut encoded, &req.state().registry.lock().unwrap()).unwrap();
            let response = tide::Response::builder(200)
                .body(encoded)
                .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
                .build();
            Ok(response)
        });
    let server_address = format!("localhost:{}", listen_port);
    app.listen(server_address).await?;
    Ok(())
}

fn coordinator_thread_metrics_sink(rx: Receiver<MinerThreadMessageToCoordinator>) {
    let mut progress_time = Instant::now();
    let mut number_of_messages: u64 = 0;
    loop {
        // Sleep until there are an incoming message
        match rx.recv() {
            Ok(_) => {
                number_of_messages += 1;
            },
            Err(error) => {
                panic!("didn't receive any messages. error: {:?}", error);
            }
        }
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed > 1000 {
            println!("number of messages: {:?}", number_of_messages);
            progress_time = Instant::now();
            number_of_messages = 0;
        }
    }
 }


#[derive(Clone)]
struct State {
    registry: MyRegistry,
}

fn coordinator_thread_metrics_prometheus(rx: Receiver<MinerThreadMessageToCoordinator>, metrics: MetricsPrometheus) {
    let mut message_processor = MessageProcessor::new();
    let mut progress_time = Instant::now();
    let mut accumulated_iterations: u64 = 0;
    let mut moving_average = MovingAverage::new();
    loop {
        // Sleep until there are an incoming message
        match rx.recv() {
            Ok(message) => {
                message_processor.process_message(message);
            },
            Err(error) => {
                panic!("didn't receive any messages. error: {:?}", error);
            }
        }
        // Fetch as many messages as possible
        loop {
            match rx.try_recv() {
                Ok(message) => {
                    message_processor.process_message(message);
                    continue;
                },
                Err(_) => {
                    break;
                }
            }
        }

        // Number of operations per second, gauge
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed > 1000 {
            let elapsed_clamped: u64 = u64::try_from(elapsed).unwrap_or(1000);
            accumulated_iterations *= 1000;
            accumulated_iterations /= elapsed_clamped;

            moving_average.insert(accumulated_iterations);
            let weighted_average: u64 = moving_average.average();
            metrics.number_of_iteration_now.set(weighted_average);
            
            progress_time = Instant::now();
            accumulated_iterations = 0;
            moving_average.rotate();
        }

        // Number of iterations per second, chart
        let metric0: u64 = message_processor.number_of_iterations();
        metrics.number_of_iterations.inc_by(metric0);
        accumulated_iterations += metric0;

        // message_processor.metrics_summary();
        message_processor.reset_iteration_metrics();
    }
}

fn print_info_about_start_conditions() {
    let build_mode: &str;
    if cfg!(debug_assertions) {
        error!("Debugging enabled. Wasting cpu cycles. Not good for mining!");
        build_mode = "'DEBUG'  # Terrible inefficient for mining!";
    } else {
        build_mode = "'RELEASE'  # Good";
    }
    println!("[mining info]");
    println!("build_mode = {}", build_mode);
}

struct MessageProcessor {
    number_of_iterations: u64,
}

impl MessageProcessor {
    fn new() -> Self {
        Self {
            number_of_iterations: 0,
        }
    }

    fn process_message(&mut self, message: MinerThreadMessageToCoordinator) {
        match message {
            MinerThreadMessageToCoordinator::ReadyForMining => {
                println!("Miner instance is ready");
            },
            MinerThreadMessageToCoordinator::NumberOfIterations(value) => {
                self.number_of_iterations += value;
            }
        }
    }

    fn reset_iteration_metrics(&mut self) {
        self.number_of_iterations = 0;
    }

    fn number_of_iterations(&self) -> u64 {
        self.number_of_iterations
    }
}
