use std::error::Error;
use std::time::Instant;
use std::rc::Rc;
use core::cell::RefCell;
use crate::analytics::{AnalyzeDependencies, AnalyzeInstructionConstant, AnalyzeInstructionNgram, AnalyzeProgramComplexity, AnalyzeSourceNgram, AnalyzeTargetNgram, BatchProgramAnalyzer, BatchProgramAnalyzerPluginItem, DontMine, ValidatePrograms, compute_program_rank};
use crate::mine::PopulateBloomfilter;

fn run_batch_program_analyzer() -> Result<(), Box<dyn Error>> {
    println!("run_batch_program_analyzer");
    let plugin_dependencies = Rc::new(RefCell::new(AnalyzeDependencies::new()));
    let plugin_instruction_ngram = Rc::new(RefCell::new(AnalyzeInstructionNgram::new()));
    let plugin_instruction_constant = Rc::new(RefCell::new(AnalyzeInstructionConstant::new()));
    let plugin_program_complexity = Rc::new(RefCell::new(AnalyzeProgramComplexity::new()));
    let plugin_source_ngram = Rc::new(RefCell::new(AnalyzeSourceNgram::new()));
    let plugin_target_ngram = Rc::new(RefCell::new(AnalyzeTargetNgram::new()));
    let plugin_vec: Vec<BatchProgramAnalyzerPluginItem> = vec![
        plugin_dependencies,
        plugin_instruction_ngram,
        plugin_instruction_constant,
        plugin_source_ngram,
        plugin_target_ngram,
        plugin_program_complexity,
    ];
    let mut analyzer = BatchProgramAnalyzer::new(plugin_vec)?;
    return analyzer.run();
}

pub fn subcommand_analytics() -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();
    run_batch_program_analyzer()?;
    compute_program_rank();
    DontMine::run()?;
    ValidatePrograms::run()?;
    PopulateBloomfilter::run();
    println!("analytics end, elapsed: {:?} ms", start_time.elapsed().as_millis());
    Ok(())
}
