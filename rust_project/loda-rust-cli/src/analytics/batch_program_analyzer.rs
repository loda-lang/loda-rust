use crate::common::{find_asm_files_recursively, load_program_ids_csv_file, oeis_id_from_path, SimpleLog};
use loda_rust_core;
use super::AnalyticsError;
use crate::config::Config;
use loda_rust_core::parser::ParsedProgram;
use std::collections::HashSet;
use std::path::PathBuf;
use std::error::Error;
use std::fs;
use std::time::Instant;
use std::rc::Rc;
use core::cell::RefCell;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

pub struct BatchProgramAnalyzerContext {
    pub program_id: u32,
    pub parsed_program: ParsedProgram,
}

pub trait BatchProgramAnalyzerPlugin {
    fn plugin_name(&self) -> &'static str;
    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> Result<(), Box<dyn Error>>;
    fn save(&self) -> Result<(), Box<dyn Error>>;
    fn human_readable_summary(&self) -> String;

    fn format_summary(&self) -> String {
        let name: &str = self.plugin_name();
        let summary: String = self.human_readable_summary();
        format!("\n{}\n{}\n", name.trim(), summary.trim())
    }
}

pub type BatchProgramAnalyzerPluginItem = Rc<RefCell<dyn BatchProgramAnalyzerPlugin>>;

pub struct BatchProgramAnalyzer {
    simple_log: SimpleLog,
    config: Config,
    number_of_program_files_that_could_not_be_loaded: usize,
    number_of_program_files_ignored: usize,
    number_of_program_files_successfully_analyzed: usize,
    plugin_vec: Vec<BatchProgramAnalyzerPluginItem>,
}

impl BatchProgramAnalyzer {
    pub fn new(plugin_vec: Vec<BatchProgramAnalyzerPluginItem>, simple_log: SimpleLog) -> Self {
        Self {
            simple_log: simple_log,
            config: Config::load(),
            number_of_program_files_that_could_not_be_loaded: 0,
            number_of_program_files_ignored: 0,
            number_of_program_files_successfully_analyzed: 0,
            plugin_vec: plugin_vec,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        println!("Run batch-program-analyzer");
        self.analyze_the_valid_program_files()
            .map_err(|e| anyhow::anyhow!("BatchProgramAnalyzer.run. analyze_the_valid_program_files. error: {:?}", e))?;

        self.save_result_files()
            .map_err(|e| anyhow::anyhow!("BatchProgramAnalyzer.run. save_result_files. error: {:?}", e))?;

        self.save_summary()
            .map_err(|e| anyhow::anyhow!("BatchProgramAnalyzer.run. save_summary. error: {:?}", e))?;
        Ok(())
    }

    fn analyze_the_valid_program_files(&mut self) -> Result<(), Box<dyn Error>> {
        self.simple_log.println("BatchProgramAnalyzer");

        let programs_invalid_file = self.config.analytics_dir_programs_invalid_file();
        let invalid_program_ids: Vec<u32> = load_program_ids_csv_file(&programs_invalid_file)?;
        let ignore_program_ids: HashSet<u32> = invalid_program_ids.into_iter().collect();
    
        let dir_containing_programs: PathBuf = self.config.loda_programs_oeis_dir();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_programs);
        let number_of_paths = paths.len();
        if number_of_paths <= 0 {
            let message = "Expected 1 or more programs, but there are no programs to analyze";
            return Err(Box::new(AnalyticsError::BatchProgramAnalyzer(message.to_string())));
        }

        let pb = ProgressBar::new(number_of_paths as u64);
        let start = Instant::now();
        for path in paths {
            self.analyze_program_file(&path, &ignore_program_ids)?;
            pb.inc(1);
        }
        pb.finish_and_clear();

        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} batch-program-analyzer in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        let content = format!("number of program files successfully analyzed: {:?}", self.number_of_program_files_successfully_analyzed);
        self.simple_log.println(content);
        let content = format!("number of program files that could not be loaded: {:?}", self.number_of_program_files_that_could_not_be_loaded);
        self.simple_log.println(content);
        let content = format!("number of program files that was ignored: {:?}", self.number_of_program_files_ignored);
        self.simple_log.println(content);

        Ok(())
    }

    fn analyze_program_file(
        &mut self, 
        path_to_program: &PathBuf,
        ignore_program_ids: &HashSet<u32>
    ) -> Result<(), Box<dyn Error>> {
        let program_id: u32 = match oeis_id_from_path(&path_to_program) {
            Some(oeis_id) => oeis_id.raw(),
            None => {
                debug!("Unable to extract program_id from {:?}", path_to_program);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return Ok(());
            }
        };
        if ignore_program_ids.contains(&program_id) {
            debug!("Ignoring program_id {:?}", program_id);
            self.number_of_program_files_ignored += 1;
            return Ok(());
        }
        let contents: String = match fs::read_to_string(&path_to_program) {
            Ok(value) => value,
            Err(error) => {
                debug!("loading program_id: {:?}, something went wrong reading the file: {:?}", program_id, error);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return Ok(());
            }
        };
        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&contents) {
            Ok(value) => value,
            Err(error) => {
                debug!("loading program_id: {:?}, something went wrong parsing the program: {:?}", program_id, error);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return Ok(());
            }
        };
        let context = BatchProgramAnalyzerContext {
            program_id: program_id,
            parsed_program: parsed_program,
        };
        for plugin in self.plugin_vec.iter() {
            plugin.borrow_mut().analyze(&context)?;
        }
        self.number_of_program_files_successfully_analyzed += 1;
        Ok(())
    }

    fn save_result_files(&self) -> Result<(), Box<dyn Error>> {
        for plugin in self.plugin_vec.iter() {
            plugin.borrow().save()?;
        }
        Ok(())
    }

    fn save_summary(&mut self) -> Result<(), Box<dyn Error>> {
        for plugin in self.plugin_vec.iter() {
            let summary: String = plugin.borrow().format_summary();
            self.simple_log.print(&summary)?;
        }
        Ok(())
    }
}
