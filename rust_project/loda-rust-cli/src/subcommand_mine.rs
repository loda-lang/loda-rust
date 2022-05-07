use crate::mine::{MinerThreadMessageToCoordinator, start_miner_loop, MovingAverage, MetricsPrometheus, Recorder, SinkRecorder};
use loda_rust_core::config::Config;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver};
use std::time::Instant;
use std::convert::TryFrom;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::sync::{Arc, Mutex};

extern crate num_cpus;

#[derive(Debug)]
pub enum SubcommandMineMetricsMode {
    NoMetricsServer,
    RunMetricsServer,
}

pub enum SubcommandMineParallelComputingMode {
    SingleInstance,
    ParallelInstances,
}

impl SubcommandMineParallelComputingMode {
    fn number_of_threads(&self) -> usize {
        match self {
            Self::SingleInstance => {
                return 1;
            },
            Self::ParallelInstances => {
                return Self::number_of_threads_in_parallel_mode();
            }
        }
    }

    fn number_of_threads_in_parallel_mode() -> usize {
        let mut number_of_threads = num_cpus::get();
        assert!(number_of_threads >= 1_usize);
        assert!(number_of_threads < 1000_usize);

        // Spawning too many threads, causes the miner performance too drop significantly.
        // The simple solution: Use half as many threads as there are cores.
        number_of_threads = number_of_threads / 2;

        // Ensures that zero is never returned
        number_of_threads.max(1)
    }
}

type MyRegistry = std::sync::Arc<std::sync::Mutex<prometheus_client::registry::Registry<std::boxed::Box<dyn prometheus_client::encoding::text::SendEncodeMetric>>>>;

pub struct SubcommandMine {
    parallel_computing_mode: SubcommandMineParallelComputingMode,
    metrics_mode: SubcommandMineMetricsMode,
    number_of_minerworkers: usize,
    config: Option<Config>,
}

impl SubcommandMine {
    pub fn new(
        parallel_computing_mode: SubcommandMineParallelComputingMode,
        metrics_mode: SubcommandMineMetricsMode
    ) -> Self {
        Self {
            parallel_computing_mode: parallel_computing_mode,
            metrics_mode: metrics_mode,
            number_of_minerworkers: 1,
            config: None
        }
    }

    pub fn determine_number_of_minerworkers(&mut self) {
        self.number_of_minerworkers = self.parallel_computing_mode.number_of_threads();
    }

    pub fn load_config(&mut self) {
        self.config = Some(Config::load());
    }

    pub fn print_info(&self) {
        println!("metrics mode: {:?}", self.metrics_mode);
        println!("Number of parallel miner instances: {}", self.number_of_minerworkers);
        print_info_about_start_conditions();
    }

    pub async fn run(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        assert!(self.config.is_some());
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
        let config: &Config = match &self.config {
            Some(value) => &value,
            None => {
                panic!("Expected config file to have been read at this point");
            }
        };
        let listen_on_port: u16 = config.miner_metrics_listen_port();
        println!("miner metrics can be downloaded here: http://localhost:{}/metrics", listen_on_port);

        let (sender, receiver) = channel::<MinerThreadMessageToCoordinator>();

        let mut registry = <Registry>::default();
        let metrics = MetricsPrometheus::new(&mut registry);
        metrics.number_of_workers.set(self.number_of_minerworkers as u64);

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
        for worker_id in 0..self.number_of_minerworkers {
            println!("Spawn worker id: {}", worker_id);
            let sender_clone = sender.clone();
            let recorder_clone: Box<dyn Recorder + Send> = recorder.clone();
            let _ = tokio::spawn(async move {
                start_miner_loop(sender_clone, recorder_clone);
            });
            thread::sleep(Duration::from_millis(2000));
        }
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
    println!("\nPress CTRL-C to stop the miner.");
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
