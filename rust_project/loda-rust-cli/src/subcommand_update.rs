use std::time::Instant;
use std::rc::Rc;
use core::cell::RefCell;
use crate::mine::{AnalyzeInstructionConstant, AnalyzeInstructionNgram, AnalyzeProgramComplexity, AnalyzeSourceNgram, AnalyzeTargetNgram, BatchProgramAnalyzer, DontMine, PopulateBloomfilter, ValidatePrograms};

fn run_batch_program_analyzer() {
    let plugin_instruction_ngram = Rc::new(RefCell::new(AnalyzeInstructionNgram::new()));
    let plugin_instruction_constant = Rc::new(RefCell::new(AnalyzeInstructionConstant::new()));
    let plugin_program_complexity = Rc::new(RefCell::new(AnalyzeProgramComplexity::new()));
    let plugin_source_ngram = Rc::new(RefCell::new(AnalyzeSourceNgram::new()));
    let plugin_target_ngram = Rc::new(RefCell::new(AnalyzeTargetNgram::new()));
    let mut analyzer = BatchProgramAnalyzer::new();
    analyzer.register(plugin_instruction_ngram);
    analyzer.register(plugin_instruction_constant);
    analyzer.register(plugin_source_ngram);
    analyzer.register(plugin_target_ngram);
    analyzer.register(plugin_program_complexity);
    analyzer.run();
}

pub fn subcommand_update() {
    let start_time = Instant::now();
    println!("update begin");
    run_batch_program_analyzer();
    DontMine::run();
    ValidatePrograms::run();
    PopulateBloomfilter::run();
    println!("update end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
