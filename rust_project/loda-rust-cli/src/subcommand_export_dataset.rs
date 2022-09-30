//! The `loda-rust test-xyz` subcommands, runs automated tests.
use crate::config::Config;
use crate::common::{find_asm_files_recursively, load_program_ids_csv_file, oeis_id_from_path, SimpleLog};
use crate::postmine::{ParentDirAndChildFile, path_for_oeis_program};
use loda_rust_core::oeis::OeisId;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::time::Duration;
use anyhow::Context;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

pub struct SubcommandExportDataset {
    config: Config,
    number_of_program_files_ignored: usize,
}

impl SubcommandExportDataset {
    pub fn export_dataset() -> Result<(), Box<dyn Error>> {
        let mut instance = Self {
            config: Config::load(),
            number_of_program_files_ignored: 0,
        };
        instance.run()?;
        Ok(())
    }

    fn run(&mut self) -> anyhow::Result<()> {
        let programs_invalid_file = self.config.analytics_dir_programs_invalid_file();
        let invalid_program_ids: Vec<u32> = match load_program_ids_csv_file(&programs_invalid_file) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Unable to load csv file with invalid programs. {:?}", error));
            }
        };
        let ignore_program_ids: HashSet<u32> = invalid_program_ids.into_iter().collect();

        let dir_containing_programs: PathBuf = self.config.loda_programs_oeis_dir();
        let mut paths: Vec<PathBuf> = find_asm_files_recursively(&dir_containing_programs);
        paths.truncate(100);
        let number_of_paths = paths.len();
        if number_of_paths <= 0 {
            return Err(anyhow::anyhow!("Expected 1 or more programs, but there are no programs to analyze"));
        }

        println!("Exporting {} LODA programs", paths.len());

        let pb = ProgressBar::new(number_of_paths as u64);
        let start = Instant::now();
        for path in paths {
            self.process_program_file(&path, &ignore_program_ids)?;
            pb.inc(1);
        }
        pb.finish_and_clear();

        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} exported dataset in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        println!("number_of_program_files_ignored: {}", self.number_of_program_files_ignored);

        Ok(())
    }

    fn process_program_file(&mut self, path_to_program: &Path, ignore_program_ids: &HashSet<u32>) -> anyhow::Result<()> {
        let program_id: u32 = match oeis_id_from_path(path_to_program) {
            Some(oeis_id) => oeis_id.raw(),
            None => {
                return Err(anyhow::anyhow!("Unable to extract program_id from {:?}", path_to_program));
            }
        };
        if ignore_program_ids.contains(&program_id) {
            self.number_of_program_files_ignored += 1;
            return Ok(());
        }
        let contents: String = match fs::read_to_string(&path_to_program) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("loading program_id: {:?}, something went wrong reading the file: {:?}", program_id, error));
            }
        };

        Ok(())
    }
}
