use crate::config::Config;
use crate::common::SimpleLog;
use crate::mine::PopulateBloomfilter;
use super::{AnalyticsMode, AnalyticsDirectory};
use super::{AnalyzeDependencies, AnalyzeIndirectMemoryAccess, AnalyzeInstructionConstant, AnalyzeInstructionNgram};
use super::{AnalyzeProgramComplexity, AnalyzeLineNgram, AnalyzeSourceNgram, AnalyzeTargetNgram, BatchProgramAnalyzer, BatchProgramAnalyzerPluginItem, DontMine, HistogramStrippedFile, AnalyticsTimestampFile, ValidatePrograms, compute_program_rank};
use anyhow::Context;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;
use core::cell::RefCell;

const ANALYTICS_TIMESTAMP_FILE_EXPIRE_AFTER_MINUTES: u32 = 30;

pub struct Analytics {
    analytics_mode: AnalyticsMode,
    analytics_directory: AnalyticsDirectory,
}

impl Analytics {
    pub fn oeis_run_if_expired() -> anyhow::Result<()> {
        let instance = Self::new(AnalyticsMode::OEIS)?;
        instance.run_if_expired()?;
        Ok(())
    }

    pub fn oeis_run_force() -> anyhow::Result<()> {
        let instance = Self::new(AnalyticsMode::OEIS)?;
        instance.run_force()?;
        Ok(())
    }

    pub fn arc_run_force() -> anyhow::Result<()> {
        let instance = Self::new(AnalyticsMode::ARC)?;
        instance.run_force()?;
        Ok(())
    }

    pub fn new(analytics_mode: AnalyticsMode) -> anyhow::Result<Self> {
        let config = Config::load();
        let analytics_directory = AnalyticsDirectory::new(
            config.analytics_dir()
        ).with_context(||"unable to create AnalyticsDirectory instance")?;
        let instance = Self {
            analytics_mode,
            analytics_directory,
        };
        Ok(instance)
    }

    /// If data is still somewhat up to date, then do nothing.
    /// 
    /// If data is too old then regenerate the `~/.loda-rust/analytics` directory.
    pub fn run_if_expired(&self) -> anyhow::Result<()> {
        let timestamp_file_path: PathBuf = self.analytics_directory.last_analytics_timestamp_file();
        let expire_minutes = ANALYTICS_TIMESTAMP_FILE_EXPIRE_AFTER_MINUTES;
        if !AnalyticsTimestampFile::is_expired(&timestamp_file_path, expire_minutes) {
            println!("The \"analytics\" dir is newer than {} minutes. No need to regenerate analytics.", expire_minutes);
            return Ok(());
        }
        println!("Generating the \"analytics\" dir.");
        self.run_force()
    }

    /// Always generate content of the `~/.loda-rust/analytics` directory.
    pub fn run_force(&self) -> anyhow::Result<()> {
        let start_time = Instant::now();
        let timestamp_file_path: PathBuf = self.analytics_directory.last_analytics_timestamp_file();
        let logfile_path: PathBuf = self.analytics_directory.analytics_log_file();

        self.analytics_directory.create_if_needed()?;

        let simple_log = SimpleLog::new(&logfile_path)
            .map_err(|e| anyhow::anyhow!("Analytics.run_force - simple_log error: {:?}", e))?;
        
        HistogramStrippedFile::run(simple_log.clone())?;
        ValidatePrograms::run(simple_log.clone())?;
        self.run_batch_program_analyzer(simple_log.clone())?;
        // compute_program_rank();

        DontMine::run(simple_log.clone())
            .map_err(|e| anyhow::anyhow!("Analytics.run_force. DontMine::run. error: {:?}", e))?;

        PopulateBloomfilter::run(simple_log.clone())
            .map_err(|e| anyhow::anyhow!("Analytics.run_force. PopulateBloomfilter::run. error: {:?}", e))?;
    
        AnalyticsTimestampFile::save_now(&timestamp_file_path)?;
        let content = format!("\nsubcommand_analytics finished, elapsed: {:?} ms", start_time.elapsed().as_millis());
        simple_log.println(content);
    
        Ok(())
    }

    fn run_batch_program_analyzer(&self, simple_log: SimpleLog) -> anyhow::Result<()> {
        let plugin_dependencies = Rc::new(RefCell::new(AnalyzeDependencies::new()));
        let plugin_indirect_memory_access = Rc::new(RefCell::new(AnalyzeIndirectMemoryAccess::new()));
        let plugin_instruction_constant = Rc::new(RefCell::new(AnalyzeInstructionConstant::new()));
        let plugin_instruction_ngram = Rc::new(RefCell::new(AnalyzeInstructionNgram::new()));
        let plugin_source_ngram = Rc::new(RefCell::new(AnalyzeSourceNgram::new()));
        let plugin_line_ngram = Rc::new(RefCell::new(AnalyzeLineNgram::new(self.analytics_mode)));
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
