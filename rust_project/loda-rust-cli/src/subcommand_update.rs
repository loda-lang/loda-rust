use std::time::Instant;
use std::rc::Rc;
use core::cell::RefCell;
use crate::mine::{DontMine, HistogramInstructionConstantAnalyzer, HistogramInstructionNgramAnalyzer, PopulateBloomfilter, ValidatePrograms};
use crate::mine::ProgramIterator;

pub fn subcommand_update() {
    let start_time = Instant::now();
    println!("update begin");

    DontMine::run();

    let plugin_ngram = Rc::new(RefCell::new(HistogramInstructionNgramAnalyzer::new()));
    let plugin_constant = Rc::new(RefCell::new(HistogramInstructionConstantAnalyzer::new()));
    let mut iterator = ProgramIterator::new();
    iterator.install_plugin(plugin_ngram);
    iterator.install_plugin(plugin_constant);
    iterator.run();

    ValidatePrograms::run();
    PopulateBloomfilter::run();
    println!("update end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
