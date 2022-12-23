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
    let recent_program_csv = loda_rust_repository.join(Path::new("resources/program_creation_dates.csv"));

    let instruction_trigram_csv: PathBuf = analytics_directory.histogram_instruction_trigram_file();
    let line_trigram_csv: PathBuf = analytics_directory.histogram_line_trigram_file();
    let source_trigram_csv: PathBuf = analytics_directory.histogram_source_trigram_file();
    let target_trigram_csv: PathBuf = analytics_directory.histogram_target_trigram_file();
    let histogram_instruction_constant_csv: PathBuf = analytics_directory.histogram_instruction_constant_file();
    let popular_program_csv: PathBuf = analytics_directory.program_popularity_file();
    let valid_program_ids_csv: PathBuf = analytics_directory.programs_valid_file();
    let invalid_program_ids_csv: PathBuf = analytics_directory.programs_invalid_file();
    let indirect_memory_access_csv: PathBuf = analytics_directory.indirect_memory_access_file();

    let mut creator = CreateGenomeMutateContext::new();
    creator.init_suggest_instruction(&instruction_trigram_csv)?;
    creator.init_suggest_line(&line_trigram_csv)?;
    creator.init_suggest_source(&source_trigram_csv)?;
    creator.init_suggest_target(&target_trigram_csv)?;
    creator.init_recent_program_container(&recent_program_csv)?;
    creator.init_popular_program_container(&popular_program_csv)?;
    creator.init_histogram_instruction_constant(&histogram_instruction_constant_csv)?;
    creator.init_valid_program_ids(&valid_program_ids_csv)?;
    creator.init_invalid_program_ids(&invalid_program_ids_csv)?;
    creator.init_indirect_memory_access_program_ids(&indirect_memory_access_csv)?;

    // Programs for initializing the genome. Remove all invalid program.
    let mut invalid_program_ids_hashset = HashSet::<u32>::new();
    if let Some(hashset) = &creator.invalid_program_ids_hashset {
        invalid_program_ids_hashset = hashset.clone();
    }
    let mut initial_genome_program_ids = Vec::<u32>::new();
    if let Some(valid_program_ids) = &creator.valid_program_ids {
        for program_id in valid_program_ids {
            if invalid_program_ids_hashset.contains(program_id) {
                debug!("initial_genome_program_ids: removed invalid program: {:?}", program_id);
                continue;
            }
            initial_genome_program_ids.push(*program_id);
        }
    }

    let context = GenomeMutateContext::new(
        creator.valid_program_ids.unwrap_or_default(),
        initial_genome_program_ids,
        creator.indirect_memory_access_program_ids.unwrap_or_default(),
        creator.invalid_program_ids_hashset.unwrap_or_default(),
        creator.popular_program_container,
        creator.recent_program_container,
        creator.histogram_instruction_constant,
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
    popular_program_container: Option<PopularProgramContainer>,
    histogram_instruction_constant: Option<HistogramInstructionConstant>,
    valid_program_ids: Option<Vec<u32>>,
    invalid_program_ids_hashset: Option<HashSet<u32>>,
    indirect_memory_access_program_ids: Option<Vec<u32>>,
}

impl CreateGenomeMutateContext {
    fn new() -> Self {
        Self {
            suggest_instruction: None,
            suggest_line: None,
            suggest_source: None,
            suggest_target: None,
            recent_program_container: None,
            popular_program_container: None,
            histogram_instruction_constant: None,
            valid_program_ids: None,
            invalid_program_ids_hashset: None,
            indirect_memory_access_program_ids: None,
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
        debug!("recent_program_container. number of clusters: {:?}", instance.cluster_program_ids().len());
        self.recent_program_container = Some(instance);
        Ok(())
    }

    /// Load the clusters with popular/unpopular program ids
    fn init_popular_program_container(&mut self, popular_program_csv: &Path) -> anyhow::Result<()> {
        let instance = PopularProgramContainer::load(popular_program_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load popular_program_csv error: {:?}", e))?;
        debug!("popular_program_container. number of clusters: {:?}", instance.cluster_program_ids().len());
        self.popular_program_container = Some(instance);
        Ok(())
    }

    fn init_histogram_instruction_constant(&mut self, histogram_instruction_constant_csv: &Path) -> anyhow::Result<()> {
        let instance = HistogramInstructionConstant::load_csv_file(histogram_instruction_constant_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load histogram_instruction_constant_csv error: {:?}", e))?;
        debug!("histogram_instruction_constant. number of items: {:?}", instance.number_of_items());
        self.histogram_instruction_constant = Some(instance);
        Ok(())
    }

    /// The programs that can execute.
    fn init_valid_program_ids(&mut self, valid_program_ids_csv: &Path) -> anyhow::Result<()> {
        let program_ids: Vec<u32> = load_program_ids_csv_file(valid_program_ids_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load valid_program_ids_csv error: {:?}", e))?;
        debug!("valid_program_ids. number of program ids: {:?}", program_ids.len());
        self.valid_program_ids = Some(program_ids);
        Ok(())
    }

    /// The invalid program_ids, that are defunct, such as cannot execute, cyclic-dependency.
    fn init_invalid_program_ids(&mut self, invalid_program_ids_csv: &Path) -> anyhow::Result<()> {
        let program_ids: Vec<u32> = load_program_ids_csv_file(invalid_program_ids_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load invalid_program_ids_csv error: {:?}", e))?;
        debug!("invalid_program_ids. number of program ids: {:?}", program_ids.len());
        let program_ids_hashset: HashSet<u32> = program_ids.into_iter().collect();
        self.invalid_program_ids_hashset = Some(program_ids_hashset);
        Ok(())
    }

    /// The programs that makes use of indirect memory access.
    /// 
    /// These programs trend to use several memory cells.
    fn init_indirect_memory_access_program_ids(&mut self, indirect_memory_access_csv: &Path) -> anyhow::Result<()> {
        let program_ids: Vec<u32> = load_program_ids_csv_file(indirect_memory_access_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load indirect_memory_access_csv error: {:?}", e))?;
        debug!("indirect_memory_access_program_ids. number of program ids: {:?}", program_ids.len());
        self.indirect_memory_access_program_ids = Some(program_ids);
        Ok(())
    }
}
