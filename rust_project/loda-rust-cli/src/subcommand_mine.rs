use crate::mine::{MinerThreadMessageToCoordinator, start_miner_loop, KeyMetricU32};
use std::thread;
use std::mem;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver};
use std::collections::HashMap;

extern crate num_cpus;

pub fn subcommand_mine() {
    print_info_about_start_conditions();

    let mut number_of_threads: usize = 1;

    number_of_threads = num_cpus::get();
    assert!(number_of_threads >= 1_usize);
    assert!(number_of_threads < 1000_usize);

    number_of_threads = number_of_threads / 2;
    number_of_threads = number_of_threads.max(1);

    let (sender, receiver) = channel::<MinerThreadMessageToCoordinator>();

    let builder = thread::Builder::new().name("minercoordinator".to_string());
    let join_handle: thread::JoinHandle<_> = builder.spawn(move || {
        miner_coordinator_inner(receiver);
    }).unwrap();

    for j in 0..number_of_threads {
        println!("start thread {} of {}", j, number_of_threads);
        let name = format!("minerworker{}", j);
        let sender_clone = sender.clone();
        let _ = thread::Builder::new().name(name).spawn(move || {
            start_miner_loop(sender_clone);
        });
        thread::sleep(Duration::from_millis(2000));
    }

    // Drop the original sender that is not being used
    mem::drop(sender);

    // Run forever, press CTRL-C to stop.
    join_handle.join().expect("The minercoordinator thread being joined has panicked");
}

fn miner_coordinator_inner(rx: Receiver<MinerThreadMessageToCoordinator>) {
    let mut message_processor = MessageProcessor::new();
    loop {
        // println!("coordinator iteration");
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
        message_processor.metrics_summary();
        message_processor.reset_iteration_metrics();
        thread::sleep(Duration::from_millis(1000));
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
                println!("Ready");
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
        let metric1: u32 = provider.metric_u32(KeyMetricU32::Funnel10TermsPassingBasicCheck);
        let metric2: u32 = provider.metric_u32(KeyMetricU32::Funnel10TermsInBloomfilter);
        let metric3: u32 = provider.metric_u32(KeyMetricU32::Funnel20TermsInBloomfilter);
        let metric4: u32 = provider.metric_u32(KeyMetricU32::Funnel30TermsInBloomfilter);
        let metric5: u32 = provider.metric_u32(KeyMetricU32::Funnel40TermsInBloomfilter);
        let metric6: u32 = provider.metric_u32(KeyMetricU32::PreventedFlooding);
        let metric7: u32 = provider.metric_u32(KeyMetricU32::NumberOfFailedMutations);
        let s: String = format!(
            "[{},{},{},{},{},{},{},{}]",
            metric0,
            metric1,
            metric2,
            metric3,
            metric4,
            metric5,
            metric6,
            metric7,
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
