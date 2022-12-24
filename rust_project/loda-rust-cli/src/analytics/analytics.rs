use super::{AnalyticsMode, AnalyticsDirectory};
use super::{AnalyzeDependencies, AnalyzeIndirectMemoryAccess, AnalyzeInstructionConstant, AnalyzeInstructionNgram, AnalyzeProgramModified};
use super::{AnalyzeProgramComplexity, AnalyzeLineNgram, AnalyzeSourceNgram, AnalyzeTargetNgram, BatchProgramAnalyzer, BatchProgramAnalyzerPluginItem, DontMine, HistogramStrippedFile, AnalyticsTimestampFile, ValidatePrograms, compute_program_rank};
use crate::config::Config;
use crate::mine::PopulateBloomfilter;
use crate::common::{find_asm_files_recursively, load_program_ids_csv_file, oeis_id_from_path, SimpleLog};
use anyhow::Context;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;
use core::cell::RefCell;

const ANALYTICS_TIMESTAMP_FILE_EXPIRE_AFTER_MINUTES: u32 = 30;

pub struct Analytics {
    analytics_mode: AnalyticsMode,
    analytics_directory: AnalyticsDirectory,
    config: Config,
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

    pub fn arc_run_if_expired() -> anyhow::Result<()> {
        let instance = Self::new(AnalyticsMode::ARC)?;
        instance.run_if_expired()?;
        Ok(())
    }

    pub fn arc_run_force() -> anyhow::Result<()> {
        let instance = Self::new(AnalyticsMode::ARC)?;
        instance.run_force()?;
        Ok(())
    }

    fn new(analytics_mode: AnalyticsMode) -> anyhow::Result<Self> {
        let config = Config::load();

        let analytics_dir: PathBuf = match analytics_mode {
            AnalyticsMode::OEIS => config.analytics_oeis_dir(),
            AnalyticsMode::ARC => config.analytics_arc_dir()
        };
        
        let analytics_directory = AnalyticsDirectory::new(
            analytics_dir
        ).with_context(||"unable to create AnalyticsDirectory instance")?;
        let instance = Self {
            analytics_mode,
            analytics_directory,
            config,
        };
        Ok(instance)
    }

    /// If data is still somewhat up to date, then do nothing.
    /// 
    /// If data is too old then regenerate the `~/.loda-rust/analytics` directory.
    fn run_if_expired(&self) -> anyhow::Result<()> {
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
    fn run_force(&self) -> anyhow::Result<()> {
        let start_time = Instant::now();
        let timestamp_file_path: PathBuf = self.analytics_directory.last_analytics_timestamp_file();
        let logfile_path: PathBuf = self.analytics_directory.analytics_log_file();

        self.analytics_directory.create_if_needed()?;

        let simple_log = SimpleLog::new(&logfile_path)
            .map_err(|e| anyhow::anyhow!("Analytics.run_force - simple_log error: {:?}", e))?;
       
        match self.analytics_mode {
            AnalyticsMode::OEIS => self.run_oeis_tasks(simple_log.clone())?,
            AnalyticsMode::ARC => self.run_arc_tasks(simple_log.clone())?
        }
    
        AnalyticsTimestampFile::save_now(&timestamp_file_path)?;
        let content = format!("\nsubcommand_analytics finished, elapsed: {:?} ms", start_time.elapsed().as_millis());
        simple_log.println(content);
    
        Ok(())
    }

    fn run_oeis_tasks(&self, simple_log: SimpleLog) -> anyhow::Result<()> {
        HistogramStrippedFile::run(self.analytics_directory.clone(), simple_log.clone())?;
        ValidatePrograms::run(self.analytics_directory.clone(), simple_log.clone())?;

        let programs_invalid_file = self.analytics_directory.programs_invalid_file();
        let invalid_program_ids: Vec<u32> = load_program_ids_csv_file(&programs_invalid_file)
            .map_err(|e| anyhow::anyhow!("run_oeis_tasks: load_program_ids_csv_file -> load_program_ids_csv_file. error: {:?}", e))?;

        let ignore_program_ids: HashSet<u32> = invalid_program_ids.into_iter().collect();
    
        let dir_containing_programs: PathBuf = self.config.loda_programs_oeis_dir();
        let all_program_paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_programs);

        let mut program_paths = Vec::<PathBuf>::new();
        let mut number_of_program_files_that_could_not_be_loaded = 0;
        let mut number_of_program_files_ignored = 0;
        for path in &all_program_paths {
            let program_id = match oeis_id_from_path(path) {
                Some(oeis_id) => oeis_id.raw(),
                None => {
                    debug!("Unable to extract program_id from {:?}", path);
                    number_of_program_files_that_could_not_be_loaded += 1;
                    continue;
                }
            };

            if ignore_program_ids.contains(&program_id) {
                debug!("Ignoring program_id {:?}", program_id);
                number_of_program_files_ignored += 1;
                continue;
            }

            program_paths.push(path.clone());
        }

        let content = format!("number of program files that could not be loaded: {:?}", number_of_program_files_that_could_not_be_loaded);
        simple_log.println(content);
        let content = format!("number of program files that was ignored: {:?}", number_of_program_files_ignored);
        simple_log.println(content);

        self.run_batch_program_analyzer(simple_log.clone(), program_paths)?;

        compute_program_rank(self.analytics_directory.clone());

        DontMine::run(self.analytics_directory.clone(), simple_log.clone())
            .map_err(|e| anyhow::anyhow!("Analytics.run_force. DontMine::run. error: {:?}", e))?;

        PopulateBloomfilter::run(self.analytics_directory.clone(), simple_log.clone())
            .map_err(|e| anyhow::anyhow!("Analytics.run_force. PopulateBloomfilter::run. error: {:?}", e))?;

        Ok(())
    }

    fn run_arc_tasks(&self, simple_log: SimpleLog) -> anyhow::Result<()> {
        let dir_containing_programs: PathBuf = self.config.loda_arc_challenge_repository_programs();
        let program_paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_programs);

        self.run_batch_program_analyzer(simple_log.clone(), program_paths)?;
        Ok(())
    }

    fn run_batch_program_analyzer(&self, simple_log: SimpleLog, program_paths: Vec<PathBuf>) -> anyhow::Result<()> {
        if program_paths.is_empty() {
            return Err(anyhow::anyhow!("Expected 1 or more programs, but there are no programs to analyze"));
        }

        let plugin_dependencies = Rc::new(RefCell::new(AnalyzeDependencies::new(self.analytics_directory.clone())));
        let plugin_indirect_memory_access = Rc::new(RefCell::new(AnalyzeIndirectMemoryAccess::new(self.analytics_directory.clone())));
        let plugin_instruction_constant = Rc::new(RefCell::new(AnalyzeInstructionConstant::new(self.analytics_directory.clone())));
        let plugin_instruction_ngram = Rc::new(RefCell::new(AnalyzeInstructionNgram::new(self.analytics_directory.clone())));
        let plugin_source_ngram = Rc::new(RefCell::new(AnalyzeSourceNgram::new(self.analytics_directory.clone())));
        let plugin_line_ngram = Rc::new(RefCell::new(AnalyzeLineNgram::new(self.analytics_directory.clone(), self.analytics_mode)));
        let plugin_target_ngram = Rc::new(RefCell::new(AnalyzeTargetNgram::new(self.analytics_directory.clone())));
        let plugin_program_complexity = Rc::new(RefCell::new(AnalyzeProgramComplexity::new(self.analytics_directory.clone())));
        let plugin_program_modified = Rc::new(RefCell::new(AnalyzeProgramModified::new(self.analytics_directory.clone())));
        let plugin_vec: Vec<BatchProgramAnalyzerPluginItem> = vec![
            plugin_dependencies,
            plugin_indirect_memory_access,
            plugin_instruction_constant,
            plugin_instruction_ngram,
            plugin_source_ngram,
            plugin_line_ngram,
            plugin_target_ngram,
            plugin_program_complexity,
            plugin_program_modified,
        ];
        let mut analyzer = BatchProgramAnalyzer::new(
            self.analytics_mode, 
            plugin_vec, 
            simple_log,
            program_paths
        );
        return analyzer.run();
    }
}
