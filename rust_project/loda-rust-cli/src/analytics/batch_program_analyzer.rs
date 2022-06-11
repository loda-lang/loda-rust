use crate::common::{find_asm_files_recursively, program_id_from_path};
use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::ParsedProgram;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::fs;
use std::time::Instant;
use std::rc::Rc;
use core::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::io::LineWriter;
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
}

pub type BatchProgramAnalyzerPluginItem = Rc<RefCell<dyn BatchProgramAnalyzerPlugin>>;

pub struct BatchProgramAnalyzer {
    config: Config,
    number_of_program_files_that_could_not_be_loaded: u32,
    plugin_vec: Vec<BatchProgramAnalyzerPluginItem>,
    line_writer: LineWriter<File>,
}

impl BatchProgramAnalyzer {
    pub fn new(plugin_vec: Vec<BatchProgramAnalyzerPluginItem>) -> Result<BatchProgramAnalyzer, Box<dyn Error>> {
        let config = Config::load();

        let path: PathBuf = config.analytics_dir().join(Path::new("batch_program_analyzer.txt"));
        let file = File::create(path)?;
        let line_writer: LineWriter<File> = LineWriter::new(file);

        let instance = BatchProgramAnalyzer {
            config: config,
            number_of_program_files_that_could_not_be_loaded: 0,
            plugin_vec: plugin_vec,
            line_writer: line_writer,
        };
        Ok(instance)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.analyze_all_program_files()?;
        self.save_result_files()?;
        self.save_summary()?;
        Ok(())
    }

    fn analyze_all_program_files(&mut self) -> Result<(), Box<dyn Error>> {
        let dir_containing_programs: PathBuf = self.config.loda_programs_oeis_dir();
        let paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_programs);
        let number_of_paths = paths.len();

        let content = format!("Overview\nnumber of paths to be analyzed: {:?}\n", number_of_paths);
        self.line_writer.write_all(content.as_bytes())?;

        if number_of_paths <= 0 {
            error!("Expected 1 or more programs, but there are no programs to analyze");
            return Ok(());
        }

        let pb = ProgressBar::new(number_of_paths as u64);
        let start = Instant::now();
        for path in paths {
            self.analyze_program_file(&path)?;
            pb.inc(1);
        }
        pb.finish_and_clear();

        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} batch-program-analyzer in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        let content = format!("number of program files that could not be loaded: {:?}\n\n", self.number_of_program_files_that_could_not_be_loaded);
        self.line_writer.write_all(content.as_bytes())?;

        Ok(())
    }

    fn analyze_program_file(&mut self, path_to_program: &PathBuf) -> Result<(), Box<dyn Error>> {
        let program_id: u32 = match program_id_from_path(&path_to_program) {
            Some(program_id) => program_id,
            None => {
                debug!("Unable to extract program_id from {:?}", path_to_program);
                self.number_of_program_files_that_could_not_be_loaded += 1;
                return Ok(());
            }
        };
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
            let name: &str = plugin.borrow().plugin_name();
            let summary: String = plugin.borrow().human_readable_summary();
            let content = format!("{}\n{}\n\n", name.trim(), summary.trim());
            self.line_writer.write_all(content.as_bytes())?;
        }
        Ok(())
    }
}
