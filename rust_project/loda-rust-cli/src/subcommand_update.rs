use std::time::Instant;
use crate::mine::{DontMine, HistogramInstructionConstantAnalyzer, PopulateBloomfilter, ValidatePrograms};

pub fn subcommand_update() {
    let start_time = Instant::now();
    println!("update begin");
    DontMine::run();
    HistogramInstructionConstantAnalyzer::run();
    ValidatePrograms::run();
    PopulateBloomfilter::run();
    println!("update end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
