use crate::common::{oeis_id_from_path, SimpleLog};
use loda_rust_core;
use super::{AnalyticsError, AnalyticsMode};
use crate::arc::RunWithProgram;
use loda_rust_core::parser::ParsedProgram;
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
    analytics_mode: AnalyticsMode,
    plugin_vec: Vec<BatchProgramAnalyzerPluginItem>,
    simple_log: SimpleLog,
    program_paths: Vec<PathBuf>,
    number_of_program_files_that_could_not_be_loaded: usize,
    number_of_program_files_ignored: usize,
    number_of_program_files_successfully_analyzed: usize,
}

impl BatchProgramAnalyzer {
    pub fn new(
        analytics_mode: AnalyticsMode, 
        plugin_vec: Vec<BatchProgramAnalyzerPluginItem>, 
        simple_log: SimpleLog,
        program_paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            analytics_mode,
            plugin_vec,
            simple_log,
            program_paths,
            number_of_program_files_that_could_not_be_loaded: 0,
            number_of_program_files_ignored: 0,
            number_of_program_files_successfully_analyzed: 0,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        println!("Run batch-program-analyzer");
        self.simple_log.println("BatchProgramAnalyzer");

        self.analyze_the_valid_program_files()
            .map_err(|e| anyhow::anyhow!("BatchProgramAnalyzer.run. analyze_the_valid_program_files. error: {:?}", e))?;

        self.save_result_files()    
            .map_err(|e| anyhow::anyhow!("BatchProgramAnalyzer.run. save_result_files. error: {:?}", e))?;

        self.save_summary()    
            .map_err(|e| anyhow::anyhow!("BatchProgramAnalyzer.run. save_summary. error: {:?}", e))?;
        Ok(())    
    }    

    fn analyze_the_valid_program_files(&mut self) -> Result<(), Box<dyn Error>> {
        let number_of_paths = self.program_paths.len();
        if number_of_paths <= 0 {
            let message = "Expected 1 or more programs, but there are no programs to analyze";
            return Err(Box::new(AnalyticsError::BatchProgramAnalyzer(message.to_string())));
        }    
        
        let pb = ProgressBar::new(number_of_paths as u64);
        let start = Instant::now();
        let program_paths: Vec<PathBuf> = self.program_paths.clone();
        for program_path in program_paths {
            self.analyze_program_file(program_path)?;
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
        program_path: PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let program_id: u32;
        match self.analytics_mode {
            AnalyticsMode::OEIS => {
                // Extract OEIS id from program path
                program_id = match oeis_id_from_path(&program_path) {
                    Some(oeis_id) => oeis_id.raw(),
                    None => {
                        debug!("Unable to extract program_id from {:?}", program_path);
                        self.number_of_program_files_that_could_not_be_loaded += 1;
                        return Ok(());
                    }
                };
            },
            AnalyticsMode::ARC => {
                // ARC programs use a filename ala `39a8645d-1.asm`, so it doesn't work with an integer `program_id`.
                program_id = 1;
            },
        }

        let mut contents: String = match fs::read_to_string(&program_path) {
            Ok(value) => value,
            Err(error) => {
                debug!("loading program_id: {:?}, something went wrong reading the file: {:?}", program_id, error);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return Ok(());
            }
        };
        if self.analytics_mode == AnalyticsMode::ARC {
            // detect if it's a "simple" program, and wrap it in the "advanced" template
            let is_simple: bool = contents.contains("Program Type: simple");
            if is_simple {
                contents = RunWithProgram::convert_simple_to_full(contents);
            }
        }
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
