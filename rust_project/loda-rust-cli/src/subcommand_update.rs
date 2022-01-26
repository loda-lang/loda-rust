use std::time::Instant;
use std::rc::Rc;
use core::cell::RefCell;
use crate::mine::{BatchProgramAnalyzer, DontMine, HistogramInstructionConstantAnalyzer, HistogramInstructionNgramAnalyzer, PopulateBloomfilter, ValidatePrograms};

fn run_batch_program_analyzer() {
    let plugin_ngram = Rc::new(RefCell::new(HistogramInstructionNgramAnalyzer::new()));
    let plugin_constant = Rc::new(RefCell::new(HistogramInstructionConstantAnalyzer::new()));
    let mut analyzer = BatchProgramAnalyzer::new();
    analyzer.register(plugin_ngram);
    analyzer.register(plugin_constant);
    analyzer.run();
}

pub fn subcommand_update() {
    let start_time = Instant::now();
    println!("update begin");
    DontMine::run();
    run_batch_program_analyzer();
    ValidatePrograms::run();
    PopulateBloomfilter::run();
    println!("update end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
