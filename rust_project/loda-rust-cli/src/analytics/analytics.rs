use crate::config::Config;
use crate::common::SimpleLog;
use crate::mine::PopulateBloomfilter;
use super::{AnalyzeDependencies, AnalyzeIndirectMemoryAccess, AnalyzeInstructionConstant, AnalyzeInstructionNgram, AnalyzeProgramComplexity, AnalyzeLineNgram, AnalyzeSourceNgram, AnalyzeTargetNgram, BatchProgramAnalyzer, BatchProgramAnalyzerPluginItem, DontMine, HistogramStrippedFile, AnalyticsTimestampFile, ValidatePrograms, compute_program_rank};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;
use core::cell::RefCell;

pub struct Analytics {}

impl Analytics {
    pub fn run() -> Result<(), Box<dyn Error>> {
        let start_time = Instant::now();
        let config = Config::load();

        // Ensure that the `analytics` dir exist
        let analytics_dir_path: PathBuf = config.analytics_dir();
        if !analytics_dir_path.is_dir() {
            fs::create_dir(&analytics_dir_path)?;
        }
        assert!(analytics_dir_path.is_dir());

        let timestamp_file_path: PathBuf = config.analytics_dir_last_analytics_timestamp_file();
        match AnalyticsTimestampFile::load(&timestamp_file_path) {
            Ok(value) => {
                println!("success: {:?}", value);
            },
            Err(error) => {
                println!("error: {:?}", error);
            }
        }

        let logfile_path: PathBuf = config.analytics_dir_analytics_log_file();
        let simple_log = SimpleLog::new(&logfile_path)?;
        
        HistogramStrippedFile::run(simple_log.clone())?;
        ValidatePrograms::run(simple_log.clone())?;
        Self::run_batch_program_analyzer(simple_log.clone())?;
        compute_program_rank();
        DontMine::run(simple_log.clone())?;
        PopulateBloomfilter::run(simple_log.clone())?;
    
        AnalyticsTimestampFile::save_now(&timestamp_file_path)?;
        let content = format!("\nsubcommand_analytics finished, elapsed: {:?} ms", start_time.elapsed().as_millis());
        simple_log.println(content);
    
        Ok(())
    }

    fn run_batch_program_analyzer(simple_log: SimpleLog) -> Result<(), Box<dyn Error>> {
        let plugin_dependencies = Rc::new(RefCell::new(AnalyzeDependencies::new()));
        let plugin_indirect_memory_access = Rc::new(RefCell::new(AnalyzeIndirectMemoryAccess::new()));
        let plugin_instruction_constant = Rc::new(RefCell::new(AnalyzeInstructionConstant::new()));
        let plugin_instruction_ngram = Rc::new(RefCell::new(AnalyzeInstructionNgram::new()));
        let plugin_source_ngram = Rc::new(RefCell::new(AnalyzeSourceNgram::new()));
        let plugin_line_ngram = Rc::new(RefCell::new(AnalyzeLineNgram::new()));
        let plugin_target_ngram = Rc::new(RefCell::new(AnalyzeTargetNgram::new()));
        let plugin_program_complexity = Rc::new(RefCell::new(AnalyzeProgramComplexity::new()));
        let plugin_vec: Vec<BatchProgramAnalyzerPluginItem> = vec![
            plugin_dependencies,
            plugin_indirect_memory_access,
            plugin_instruction_constant,
            plugin_instruction_ngram,
            plugin_source_ngram,
            plugin_line_ngram,
            plugin_target_ngram,
            plugin_program_complexity,
        ];
        let mut analyzer = BatchProgramAnalyzer::new(plugin_vec, simple_log);
        return analyzer.run();
    }
}
