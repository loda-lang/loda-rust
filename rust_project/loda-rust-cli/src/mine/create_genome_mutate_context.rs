use anyhow::Context;

use crate::config::Config;
use crate::common::RecordTrigram;
use crate::common::load_program_ids_csv_file;
use crate::analytics::AnalyticsDirectory;
use super::{PopularProgramContainer, RecentProgramContainer, HistogramInstructionConstant};
use super::GenomeMutateContext;
use super::SuggestInstruction;
use super::SuggestLine;
use super::SuggestSource;
use super::SuggestTarget;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

pub fn create_genome_mutate_context(config: &Config) -> anyhow::Result<GenomeMutateContext> {
    let analytics_directory = AnalyticsDirectory::new(
        config.analytics_dir()
    ).with_context(||"unable to create AnalyticsDirectory instance")?;

    let loda_rust_repository: PathBuf = config.loda_rust_repository();
    let instruction_trigram_csv: PathBuf = analytics_directory.histogram_instruction_trigram_file();
    let line_trigram_csv: PathBuf = analytics_directory.histogram_line_trigram_file();
    let source_trigram_csv: PathBuf = analytics_directory.histogram_source_trigram_file();
    let target_trigram_csv: PathBuf = analytics_directory.histogram_target_trigram_file();

    let path_histogram: PathBuf = analytics_directory.histogram_instruction_constant_file();
    let histogram_instruction_constant: Option<HistogramInstructionConstant>;
    if path_histogram.is_file() {
        histogram_instruction_constant = match HistogramInstructionConstant::load_csv_file(&path_histogram) {
            Ok(value) => {
                debug!("Optional histogram: loaded successful");
                Some(value)
            },
            Err(error) => {
                error!("Optional histogram: {:?} error: {:?}", path_histogram, error);
                None
            }
        };
    } else {
        println!("Optional histogram: Not found at path {:?}", path_histogram);
        histogram_instruction_constant = None;
    }

    // Load the valid program_ids, that can execute.
    let programs_valid_file = analytics_directory.programs_valid_file();
    let valid_program_ids: Vec<u32> = match load_program_ids_csv_file(&programs_valid_file) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", programs_valid_file, error);
        }
    };
    debug!("number_of_valid_program_ids = {}", valid_program_ids.len());

    // Load the valid program_ids, that can execute.
    let indirect_memory_access_csv: PathBuf = analytics_directory.indirect_memory_access_file();
    let indirect_memory_access_program_ids: Vec<u32> = match load_program_ids_csv_file(&indirect_memory_access_csv) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", indirect_memory_access_csv, error);
        }
    };
    debug!("indirect_memory_access_program_ids = {}", indirect_memory_access_program_ids.len());

    // Load the invalid program_ids, that are defunct, such as cannot execute, cyclic-dependency.
    let programs_invalid_file = analytics_directory.programs_invalid_file();
    let invalid_program_ids: Vec<u32> = match load_program_ids_csv_file(&programs_invalid_file) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", programs_invalid_file, error);
        }
    };
    debug!("number_of_invalid_program_ids = {}", invalid_program_ids.len());
    let invalid_program_ids_hashset: HashSet<u32> = invalid_program_ids.into_iter().collect();

    // Determine the complex programs that are to be optimized
    let mut optimize_program_ids = Vec::<u32>::new();
    for program_id in &valid_program_ids {
        if invalid_program_ids_hashset.contains(program_id) {
            continue;
        }
        optimize_program_ids.push(*program_id);
    }

    // Load the clusters with popular/unpopular program ids
    let program_popularity_file = analytics_directory.program_popularity_file();
    let popular_program_container: PopularProgramContainer = match PopularProgramContainer::load(&program_popularity_file) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", program_popularity_file, error);
        }
    };

    // Load the clusters with newest/oldest program ids
    let recent_program_file = loda_rust_repository.join(Path::new("resources/program_creation_dates.csv"));
    let recent_program_container: RecentProgramContainer = match RecentProgramContainer::load(&recent_program_file) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", recent_program_file, error);
        }
    };

    let instruction_trigram_vec: Vec<RecordTrigram> = RecordTrigram::parse_csv(&instruction_trigram_csv).expect("Unable to load instruction trigram csv");
    let mut suggest_instruction = SuggestInstruction::new();
    suggest_instruction.populate(&instruction_trigram_vec);

    let line_trigram_vec: Vec<RecordTrigram> = RecordTrigram::parse_csv(&line_trigram_csv).expect("Unable to load line trigram csv");
    let mut suggest_line = SuggestLine::new();
    suggest_line.populate(&line_trigram_vec);

    let source_trigram_vec: Vec<RecordTrigram> = RecordTrigram::parse_csv(&source_trigram_csv).expect("Unable to load source trigram csv");
    let mut suggest_source = SuggestSource::new();
    suggest_source.populate(&source_trigram_vec);

    let target_trigram_vec: Vec<RecordTrigram> = RecordTrigram::parse_csv(&target_trigram_csv).expect("Unable to load target trigram csv");
    let mut suggest_target = SuggestTarget::new();
    suggest_target.populate(&target_trigram_vec);

    let initial_genome_program_ids = optimize_program_ids;
    
    let context = GenomeMutateContext::new(
        valid_program_ids,
        initial_genome_program_ids,
        indirect_memory_access_program_ids,
        invalid_program_ids_hashset,
        popular_program_container,
        recent_program_container,
        histogram_instruction_constant,
        Some(suggest_instruction),
        Some(suggest_line),
        Some(suggest_source),
        Some(suggest_target)
    );
    assert_eq!(context.has_available_programs(), true);
    Ok(context)
}
