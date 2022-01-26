use std::time::Instant;
use std::rc::Rc;
use core::cell::RefCell;
use crate::mine::{AnalyzeInstructionNgram, AnalyzeTargetNgram, BatchProgramAnalyzer, DontMine, HistogramInstructionConstantAnalyzer, PopulateBloomfilter, ValidatePrograms};

fn run_batch_program_analyzer() {
    let plugin_instruction_ngram = Rc::new(RefCell::new(AnalyzeInstructionNgram::new()));
    let plugin_instruction_constant = Rc::new(RefCell::new(HistogramInstructionConstantAnalyzer::new()));
    let plugin_target_ngram = Rc::new(RefCell::new(AnalyzeTargetNgram::new()));
    let mut analyzer = BatchProgramAnalyzer::new();
    analyzer.register(plugin_instruction_ngram);
    analyzer.register(plugin_instruction_constant);
    analyzer.register(plugin_target_ngram);
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
