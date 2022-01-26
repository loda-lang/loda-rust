use std::time::Instant;
use std::rc::Rc;
use core::cell::RefCell;
use crate::mine::{DontMine, HistogramInstructionConstantAnalyzer, HistogramInstructionNgramAnalyzer, PopulateBloomfilter, ValidatePrograms};
use crate::mine::{ProgramIterator, ProgramIteratorPlugin};

pub fn subcommand_update() {
    let start_time = Instant::now();
    println!("update begin");

    let plugin_ngram = Rc::new(RefCell::new(HistogramInstructionNgramAnalyzer::new()));
    let plugin_constant = Rc::new(RefCell::new(HistogramInstructionConstantAnalyzer::new()));

    let plugin2 = Rc::clone(&plugin_ngram);
    let plugin3 = Rc::clone(&plugin_constant);

    let mut iterator = ProgramIterator::new();
    iterator.install_plugin(plugin_ngram);
    iterator.install_plugin(plugin_constant);

    iterator.analyze_all_program_files();

    plugin2.borrow_mut().save();
    plugin3.borrow_mut().save();

    /*
    DontMine::run();
    HistogramInstructionConstantAnalyzer::run();
    // HistogramInstructionNgramAnalyzer::run();
    ValidatePrograms::run();
    PopulateBloomfilter::run(); */
    println!("update end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
