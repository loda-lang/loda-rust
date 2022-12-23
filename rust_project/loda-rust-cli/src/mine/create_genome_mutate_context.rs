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

pub fn create_genome_mutate_context(config: &Config, analytics_directory: AnalyticsDirectory) -> anyhow::Result<GenomeMutateContext> {
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

    let mut creator = CreateGenomeMutateContext::new();
    creator.init_suggest_instruction(&instruction_trigram_csv)?;
    creator.init_suggest_line(&line_trigram_csv)?;
    creator.init_suggest_source(&source_trigram_csv)?;
    creator.init_suggest_target(&target_trigram_csv)?;

    // Load the clusters with newest/oldest program ids
    let recent_program_csv = loda_rust_repository.join(Path::new("resources/program_creation_dates.csv"));
    creator.init_recent_program_container(&recent_program_csv)?;

    let initial_genome_program_ids = optimize_program_ids;
    
    let context = GenomeMutateContext::new(
        valid_program_ids,
        initial_genome_program_ids,
        indirect_memory_access_program_ids,
        invalid_program_ids_hashset,
        popular_program_container,
        creator.recent_program_container,
        histogram_instruction_constant,
        creator.suggest_instruction,
        creator.suggest_line,
        creator.suggest_source,
        creator.suggest_target,
    );
    assert_eq!(context.has_available_programs(), true);
    Ok(context)
}

struct CreateGenomeMutateContext {
    suggest_instruction: Option<SuggestInstruction>,
    suggest_line: Option<SuggestLine>,
    suggest_source: Option<SuggestSource>,
    suggest_target: Option<SuggestTarget>,
    recent_program_container: Option<RecentProgramContainer>,
}

impl CreateGenomeMutateContext {
    fn new() -> Self {
        Self {
            suggest_instruction: None,
            suggest_line: None,
            suggest_source: None,
            suggest_target: None,
            recent_program_container: None,
        }
    }

    fn init_suggest_instruction(&mut self, instruction_trigram_csv: &Path) -> anyhow::Result<()> {
        let records: Vec<RecordTrigram> = RecordTrigram::parse_csv(&instruction_trigram_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load instruction_trigram_csv error: {:?}", e))?;
        let mut instance = SuggestInstruction::new();
        instance.populate(&records);
        self.suggest_instruction = Some(instance);
        Ok(())
    }

    fn init_suggest_line(&mut self, line_trigram_csv: &Path) -> anyhow::Result<()> {
        let records: Vec<RecordTrigram> = RecordTrigram::parse_csv(&line_trigram_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load line_trigram_csv error: {:?}", e))?;
        let mut instance = SuggestLine::new();
        instance.populate(&records);
        self.suggest_line = Some(instance);
        Ok(())
    }

    fn init_suggest_source(&mut self, source_trigram_csv: &Path) -> anyhow::Result<()> {
        let records: Vec<RecordTrigram> = RecordTrigram::parse_csv(&source_trigram_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load source_trigram_csv error: {:?}", e))?;
        let mut instance = SuggestSource::new();
        instance.populate(&records);
        self.suggest_source = Some(instance);
        Ok(())
    }

    fn init_suggest_target(&mut self, target_trigram_csv: &Path) -> anyhow::Result<()> {
        let records: Vec<RecordTrigram> = RecordTrigram::parse_csv(target_trigram_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load target_trigram_csv error: {:?}", e))?;
        let mut instance = SuggestTarget::new();
        instance.populate(&records);
        self.suggest_target = Some(instance);
        Ok(())
    }

    /// Load the clusters with newest/oldest program ids
    fn init_recent_program_container(&mut self, recent_program_csv: &Path) -> anyhow::Result<()> {
        let instance = RecentProgramContainer::load(recent_program_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load recent_program_csv error: {:?}", e))?;
        debug!("recent_program_container: {:?}", instance.cluster_program_ids().len());
        self.recent_program_container = Some(instance);
        Ok(())
    }
}
