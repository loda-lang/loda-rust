use crate::mine::{MinerThreadMessageToCoordinator, start_miner_loop, KeyMetricU32, MovingAverage, Metrics};
use std::thread;
use std::mem;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver};
use std::collections::HashMap;
use std::time::Instant;
use std::convert::TryFrom;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::sync::{Arc, Mutex};

extern crate num_cpus;

pub enum SubcommandMineParallelComputingMode {
    SingleInstance,
    ParallelInstancces,
}

impl SubcommandMineParallelComputingMode {
    fn number_of_threads(&self) -> usize {
        match self {
            Self::SingleInstance => {
                return 1;
            },
            Self::ParallelInstancces => {
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

pub async fn subcommand_mine(parallel_computing_mode: SubcommandMineParallelComputingMode) 
    -> std::result::Result<(), Box<dyn std::error::Error>> 
{
    print_info_about_start_conditions();

    let number_of_minerworkers: usize = parallel_computing_mode.number_of_threads();
    println!("Number of parallel miner instances: {}", number_of_minerworkers);

    let (sender, receiver) = channel::<MinerThreadMessageToCoordinator>();

    let mut registry = <Registry>::default();
    let metrics = Metrics::new(&mut registry);
    metrics.number_of_workers.set(number_of_minerworkers as u64);

    let registry2: MyRegistry = Arc::new(Mutex::new(registry));

    let _ = tokio::spawn(async move {
        let result = webserver_with_metrics(registry2).await;
        if let Err(error) = result {
            error!("webserver thread failed with error: {:?}", error);
        }
    });

    let minercoordinator_metrics = metrics.clone();
    let minercoordinator_thread = tokio::spawn(async move {
        miner_coordinator_inner(receiver, minercoordinator_metrics);
    });

    for worker_id in 0..number_of_minerworkers {
        println!("Spawn worker id: {}", worker_id);
        let sender_clone = sender.clone();
        let metrics_clone = metrics.clone();
        let _ = tokio::spawn(async move {
            start_miner_loop(sender_clone, metrics_clone);
        });
        thread::sleep(Duration::from_millis(2000));
    }

    // Drop the original sender that is not being used
    mem::drop(sender);

    // Run forever, press CTRL-C to stop.
    minercoordinator_thread.await?;

    Ok(())
}

async fn webserver_with_metrics(registry: MyRegistry) -> std::result::Result<(), Box<dyn std::error::Error>> {
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
    app.listen("127.0.0.1:8090").await?;
    Ok(())
}

#[derive(Clone)]
struct State {
    registry: MyRegistry,
}

fn miner_coordinator_inner(rx: Receiver<MinerThreadMessageToCoordinator>, metrics: Metrics) {
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
                println!("didn't receive any messages. error: {:?}", error);
                continue;
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
        let metric0: u32 = message_processor.metric_u32_this_iteration.metric_u32(KeyMetricU32::NumberOfMinerLoopIterations);
        metrics.number_of_iterations.inc_by(metric0 as u64);
        accumulated_iterations += metric0 as u64;

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

type HashMapMetricU32 = HashMap<KeyMetricU32, u32>;

struct MessageProcessor {
    metric_u32_this_iteration: HashMapMetricU32,
    metric_u32_across_all_iterations: HashMapMetricU32,
}

impl MessageProcessor {
    fn new() -> Self {
        Self {
            metric_u32_this_iteration: HashMap::new(),
            metric_u32_across_all_iterations: HashMap::new(),
        }
    }

    fn process_message(&mut self, message: MinerThreadMessageToCoordinator) {
        // println!("received message: {:?}", message);
        match message {
            MinerThreadMessageToCoordinator::ReadyForMining => {
                println!("Miner instance is ready");
            },
            MinerThreadMessageToCoordinator::MetricU32(key, value) => {
                let counter0 = self.metric_u32_this_iteration.entry(key).or_insert(0);
                *counter0 += value;
                let counter1 = self.metric_u32_across_all_iterations.entry(key).or_insert(0);
                *counter1 += value;
            }
        }
    }

    fn metrics_summary(&self) {
        let summary0 = self.format_summary(&self.metric_u32_across_all_iterations);
        let summary1 = self.format_summary(&self.metric_u32_this_iteration);
        println!("total: {} delta: {}", summary0, summary1);
    }

    fn format_summary(&self, provider: &dyn ProvideMetricU32) -> String {
        let metric0: u32 = provider.metric_u32(KeyMetricU32::NumberOfMinerLoopIterations);
        let metric1: u32 = provider.metric_u32(KeyMetricU32::PreventedFlooding);
        let metric2: u32 = provider.metric_u32(KeyMetricU32::NumberOfFailedMutations);
        let metric3: u32 = provider.metric_u32(KeyMetricU32::NumberOfFailedGenomeLoads);
        let s: String = format!(
            "[{},{},{},{}]",
            metric0,
            metric1,
            metric2,
            metric3,
        );
        s
    }

    fn reset_iteration_metrics(&mut self) {
        self.metric_u32_this_iteration.clear();
    }
}

trait ProvideMetricU32 {
    fn metric_u32(&self, key: KeyMetricU32) -> u32;
}

impl ProvideMetricU32 for HashMapMetricU32 {
    fn metric_u32(&self, key: KeyMetricU32) -> u32 {
        self.get(&key).map_or(0, |value| *value )
    }
}
