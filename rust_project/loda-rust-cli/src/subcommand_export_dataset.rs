//! The `loda-rust export-dataset` subcommand, exports terms and programs to a CSV file.
use crate::config::Config;
use crate::common::{find_asm_files_recursively, load_program_ids_csv_file, oeis_id_from_path};
use crate::common::create_csv_file;
use crate::oeis::{ProcessStrippedFile, StrippedRow};
use loda_rust_core::util::BigIntVecToString;
use loda_rust_core::oeis::OeisIdHashSet;
use loda_rust_core::oeis::OeisId;
use loda_rust_core::parser::{ParsedProgram};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};
use serde::Serialize;
use num_bigint::BigInt;
use num_traits::Zero;

pub type OeisIdToTermsSet = HashMap::<OeisId, String>;

pub struct SubcommandExportDataset {
    config: Config,
    count_ignored: usize,
    count_insufficient_number_of_terms: usize,
    oeis_id_to_terms_set: OeisIdToTermsSet,
    records: Vec<Record>,
}

impl SubcommandExportDataset {
    pub fn export_dataset() -> Result<(), Box<dyn Error>> {
        let mut instance = Self {
            config: Config::load(),
            count_ignored: 0,
            count_insufficient_number_of_terms: 0,
            records: vec!(),
            oeis_id_to_terms_set: OeisIdToTermsSet::new(),
        };
        instance.run()?;
        Ok(())
    }

    fn run(&mut self) -> anyhow::Result<()> {
        self.load_stripped_file()?;
        self.process_program_files()?;
        self.save()?;
        println!("Ok");
        Ok(())
    }

    fn load_stripped_file(&mut self) -> anyhow::Result<()> {
        let mut oeis_id_to_terms_set = OeisIdToTermsSet::new();

        let callback = |row: &StrippedRow, _| {
            let value: String = row.terms().to_compact_comma_string();
            let key: OeisId = row.oeis_id();
            oeis_id_to_terms_set.insert(key, value);
        };

        let oeis_stripped_file = self.config.oeis_stripped_file();
        let file = File::open(oeis_stripped_file)?;
        let mut oeis_stripped_file_reader = BufReader::new(file);

        let minimum_number_of_required_terms = 20;
        let term_count = 20;
        let padding_value = BigInt::zero();
    
        let mut processor = ProcessStrippedFile::new();
        let oeis_ids_to_ignore = OeisIdHashSet::new();
        processor.execute(
            &mut oeis_stripped_file_reader,
            minimum_number_of_required_terms,
            term_count,
            &oeis_ids_to_ignore,
            &padding_value, 
            false,
            callback
        );

        self.oeis_id_to_terms_set = oeis_id_to_terms_set;

        Ok(())
    }

    fn process_program_files(&mut self) -> anyhow::Result<()> {
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
        // paths.truncate(1000);
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

        if self.count_ignored > 0 {
            println!("count_ignored: {}", self.count_ignored);
        }
        if self.count_insufficient_number_of_terms > 0 {
            println!("count_insufficient_number_of_terms: {}", self.count_insufficient_number_of_terms);
        }
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
            self.count_ignored += 1;
            return Ok(());
        }
        let oeis_id = OeisId::from(program_id);
        let terms: String = match self.oeis_id_to_terms_set.get(&oeis_id) {
            Some(value) => {
                value.clone()
            },
            None => {
                self.count_insufficient_number_of_terms += 1;
                return Ok(());
            }
        };
        let contents: String = match fs::read_to_string(&path_to_program) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("loading program_id: {:?}, something went wrong reading the file: {:?}", program_id, error));
            }
        };
        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&contents) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("parsing program_id: {:?}, something went wrong parsing the file: {:?}", program_id, error));
            }
        };
        let instructions: Vec<String> = parsed_program.instruction_vec.iter().map(|instruction| {
            instruction.to_string()
        }).collect();
        let instructions_joined: String = instructions.join("\\n");

        let record = Record {
            program_id: program_id,
            terms: terms,
            program: instructions_joined
        };
        self.records.push(record);

        Ok(())
    }

    /// Save as a CSV file
    fn save(&self) -> anyhow::Result<()> {
        let mut records: Vec<Record> = self.records.clone();
        records.sort_unstable_by_key(|item| (item.program_id));

        let output_path: PathBuf = self.config.analytics_dir().join("dataset.csv");
        match create_csv_file(&records, &output_path) {
            Ok(_) => {},
            Err(error) => {
                return Err(anyhow::anyhow!("Unable to save csv file at {:?}, error: {:?}", output_path, error));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Serialize)]
struct Record {
    #[serde(rename = "oeis")]
    program_id: u32,
    terms: String,
    #[serde(rename = "loda")]
    program: String,
}
