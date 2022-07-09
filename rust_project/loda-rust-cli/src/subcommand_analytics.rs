//! The `loda-rust analytics` subcommand, populates histograms/bloomfilters.
use std::error::Error;
use std::time::Instant;
use std::rc::Rc;
use core::cell::RefCell;
use std::path::{Path, PathBuf};
use crate::config::Config;
use crate::analytics::{AnalyzeDependencies, AnalyzeInstructionConstant, AnalyzeInstructionNgram, AnalyzeProgramComplexity, AnalyzeSourceNgram, AnalyzeTargetNgram, BatchProgramAnalyzer, BatchProgramAnalyzerPluginItem, DontMine, HistogramStrippedFile, ValidatePrograms, compute_program_rank};
use crate::common::SimpleLog;
use crate::mine::PopulateBloomfilter;

fn run_batch_program_analyzer(simple_log: SimpleLog) -> Result<(), Box<dyn Error>> {
    let plugin_dependencies = Rc::new(RefCell::new(AnalyzeDependencies::new()));
    let plugin_instruction_constant = Rc::new(RefCell::new(AnalyzeInstructionConstant::new()));
    let plugin_instruction_ngram = Rc::new(RefCell::new(AnalyzeInstructionNgram::new()));
    let plugin_source_ngram = Rc::new(RefCell::new(AnalyzeSourceNgram::new()));
    let plugin_target_ngram = Rc::new(RefCell::new(AnalyzeTargetNgram::new()));
    let plugin_program_complexity = Rc::new(RefCell::new(AnalyzeProgramComplexity::new()));
    let plugin_vec: Vec<BatchProgramAnalyzerPluginItem> = vec![
        plugin_dependencies,
        plugin_instruction_constant,
        plugin_instruction_ngram,
        plugin_source_ngram,
        plugin_target_ngram,
        plugin_program_complexity,
    ];
    let mut analyzer = BatchProgramAnalyzer::new(plugin_vec, simple_log);
    return analyzer.run();
}

pub fn subcommand_analytics() -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();
    let config = Config::load();
    let path: PathBuf = config.analytics_dir().join(Path::new("analytics_log.txt"));
    let simple_log = SimpleLog::new(&path)?;
    
    HistogramStrippedFile::run(simple_log.clone())?;
    ValidatePrograms::run(simple_log.clone())?;
    run_batch_program_analyzer(simple_log.clone())?;
    compute_program_rank();
    DontMine::run(simple_log.clone())?;
    PopulateBloomfilter::run(simple_log.clone())?;

    let content = format!("\nsubcommand_analytics finished, elapsed: {:?} ms", start_time.elapsed().as_millis());
    simple_log.println(content);
    Ok(())
}
