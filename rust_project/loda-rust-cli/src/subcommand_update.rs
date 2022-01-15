use std::time::Instant;
use crate::mine::{DontMine, HistogramInstructionConstantAnalyzer, HistogramInstructionNgramAnalyzer, PopulateBloomfilter, ValidatePrograms};

pub fn subcommand_update() {
    let start_time = Instant::now();
    println!("update begin");
    DontMine::run();
    HistogramInstructionConstantAnalyzer::run();
    HistogramInstructionNgramAnalyzer::run();
    ValidatePrograms::run();
    PopulateBloomfilter::run();
    println!("update end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
