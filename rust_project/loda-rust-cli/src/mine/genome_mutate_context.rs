use super::{PopularProgramContainer, RecentProgramContainer, HistogramInstructionConstant};
use super::SuggestInstruction;
use super::{SuggestLine, LineValue};
use super::{SuggestSource, SourceValue};
use super::{SuggestTarget, TargetValue};
use loda_rust_core::parser::InstructionId;
use crate::common::RecordTrigram;
use crate::common::load_program_ids_csv_file;
use std::path::Path;
use std::collections::HashSet;
use std::fmt;
use rand::Rng;
use rand::seq::SliceRandom;

#[derive(Clone)]
pub struct GenomeMutateContext {
    valid_program_ids: Vec<u32>,
    initial_genome_program_ids: Vec<u32>, 
    indirect_memory_access_program_ids: Vec<u32>,
    invalid_program_ids: HashSet<u32>,
    popular_program_container: Option<PopularProgramContainer>,
    recent_program_container: Option<RecentProgramContainer>,
    histogram_instruction_constant: Option<HistogramInstructionConstant>,
    suggest_instruction: Option<SuggestInstruction>,
    suggest_line: Option<SuggestLine>,
    suggest_source: Option<SuggestSource>,
    suggest_target: Option<SuggestTarget>,
}

impl GenomeMutateContext {
    pub fn empty() -> Self {
        Self {
            valid_program_ids: vec!(),
            initial_genome_program_ids: vec!(),
            indirect_memory_access_program_ids: vec!(),
            invalid_program_ids: HashSet::<u32>::new(),
            popular_program_container: None,
            recent_program_container: None,
            histogram_instruction_constant: None,
            suggest_instruction: None,
            suggest_line: None,
            suggest_source: None,
            suggest_target: None,
        }
    }

    pub fn is_program_id_invalid(&self, program_id: u32) -> bool {
        self.invalid_program_ids.contains(&program_id)
    }

    pub fn available_program_ids(&self) -> &Vec<u32> {
        &self.valid_program_ids
    }

    pub fn choose_initial_genome_program<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        let program_id: u32 = match self.initial_genome_program_ids.choose(rng) {
            Some(program_id) => *program_id,
            None => {
                // For a non-empty vector, this shouldn't happen.
                return None;
            }
        };
        Some(program_id)
    }

    pub fn choose_indirect_memory_access_program_id<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        let program_id: u32 = match self.indirect_memory_access_program_ids.choose(rng) {
            Some(program_id) => *program_id,
            None => {
                // For a non-empty vector, this shouldn't happen.
                return None;
            }
        };
        Some(program_id)
    }

    pub fn choose_weighted_by_popularity<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        let popular_program_container: &PopularProgramContainer = match &self.popular_program_container {
            Some(value) => value,
            None => {
                return None;
            }
        };
        popular_program_container.choose_weighted_by_popularity(rng)
    }

    pub fn choose_most_popular<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        let popular_program_container: &PopularProgramContainer = match &self.popular_program_container {
            Some(value) => value,
            None => {
                return None;
            }
        };
        popular_program_container.choose_most_popular(rng)
    }

    pub fn choose_medium_popular<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        let popular_program_container: &PopularProgramContainer = match &self.popular_program_container {
            Some(value) => value,
            None => {
                return None;
            }
        };
        popular_program_container.choose_medium_popular(rng)
    }

    pub fn choose_least_popular<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        let popular_program_container: &PopularProgramContainer = match &self.popular_program_container {
            Some(value) => value,
            None => {
                return None;
            }
        };
        popular_program_container.choose_least_popular(rng)
    }

    pub fn choose_recent_program<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        let recent_program_container: &RecentProgramContainer = match &self.recent_program_container {
            Some(value) => value,
            None => {
                return None;
            }
        };
        recent_program_container.choose(rng)
    }

    pub fn has_histogram_instruction_constant(&self) -> bool {
        self.histogram_instruction_constant.is_some()
    }

    pub fn choose_constant_with_histogram<R: Rng + ?Sized>(&self, rng: &mut R, instruction_id: InstructionId) -> Option<i32> {
        let instance: &HistogramInstructionConstant = match &self.histogram_instruction_constant {
            Some(value) => value,
            None => {
                return None;
            }
        };
        instance.choose_weighted(rng, instruction_id)
    }

    pub fn has_suggest_instruction(&self) -> bool {
        self.suggest_instruction.is_some()
    }

    pub fn suggest_instruction<R: Rng + ?Sized>(&self, rng: &mut R, prev_word: Option<InstructionId>, next_word: Option<InstructionId>) -> Option<InstructionId> {
        let suggest_instruction: &SuggestInstruction = match &self.suggest_instruction {
            Some(value) => value,
            None => {
                return None;
            }
        };
        suggest_instruction.choose_weighted(rng, prev_word, next_word)
    }

    pub fn has_suggest_line(&self) -> bool {
        self.suggest_line.is_some()
    }

    pub fn suggest_line<R: Rng + ?Sized>(&self, rng: &mut R, prev_word: LineValue, next_word: LineValue) -> Option<LineValue> {
        let suggest_line: &SuggestLine = match &self.suggest_line {
            Some(value) => value,
            None => {
                return None;
            }
        };
        suggest_line.choose_weighted(rng, prev_word, next_word)
    }

    pub fn has_suggest_source(&self) -> bool {
        self.suggest_source.is_some()
    }

    pub fn suggest_source<R: Rng + ?Sized>(&self, rng: &mut R, prev_word: SourceValue, next_word: SourceValue) -> Option<SourceValue> {
        let suggest_source: &SuggestSource = match &self.suggest_source {
            Some(value) => value,
            None => {
                return None;
            }
        };
        suggest_source.choose_weighted(rng, prev_word, next_word)
    }

    pub fn has_suggest_target(&self) -> bool {
        self.suggest_target.is_some()
    }

    pub fn suggest_target<R: Rng + ?Sized>(&self, rng: &mut R, prev_word: TargetValue, next_word: TargetValue) -> Option<TargetValue> {
        let suggest_target: &SuggestTarget = match &self.suggest_target {
            Some(value) => value,
            None => {
                return None;
            }
        };
        suggest_target.choose_weighted(rng, prev_word, next_word)
    }
}

impl fmt::Debug for GenomeMutateContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GenomeMutateContext")
    }
}

pub struct GenomeMutateContextBuilder {
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

impl GenomeMutateContextBuilder {
    pub fn new() -> Self {
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

    pub fn build(self) -> anyhow::Result<GenomeMutateContext> {
        // Programs for initializing the genome. Remove all invalid program.
        let mut invalid_program_ids_hashset = HashSet::<u32>::new();
        if let Some(hashset) = &self.invalid_program_ids_hashset {
            invalid_program_ids_hashset = hashset.clone();
        }
        let mut initial_genome_program_ids = Vec::<u32>::new();
        if let Some(valid_program_ids) = &self.valid_program_ids {
            for program_id in valid_program_ids {
                if invalid_program_ids_hashset.contains(program_id) {
                    debug!("initial_genome_program_ids: removed invalid program: {:?}", program_id);
                    continue;
                }
                initial_genome_program_ids.push(*program_id);
            }
        }

        let instance = GenomeMutateContext {
            valid_program_ids: self.valid_program_ids.unwrap_or_default(),
            initial_genome_program_ids: initial_genome_program_ids,
            indirect_memory_access_program_ids: self.indirect_memory_access_program_ids.unwrap_or_default(),
            invalid_program_ids: self.invalid_program_ids_hashset.unwrap_or_default(),
            popular_program_container: self.popular_program_container,
            recent_program_container: self.recent_program_container,
            histogram_instruction_constant: self.histogram_instruction_constant,
            suggest_instruction: self.suggest_instruction,
            suggest_line: self.suggest_line,
            suggest_source: self.suggest_source,
            suggest_target: self.suggest_target,
        };
        Ok(instance)
    }

    pub fn init_suggest_instruction(&mut self, instruction_trigram_csv: &Path) -> anyhow::Result<()> {
        let records: Vec<RecordTrigram> = RecordTrigram::parse_csv(&instruction_trigram_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load instruction_trigram_csv error: {:?}", e))?;
        let mut instance = SuggestInstruction::new();
        instance.populate(&records);
        self.suggest_instruction = Some(instance);
        Ok(())
    }

    pub fn init_suggest_line(&mut self, line_trigram_csv: &Path) -> anyhow::Result<()> {
        let records: Vec<RecordTrigram> = RecordTrigram::parse_csv(&line_trigram_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load line_trigram_csv error: {:?}", e))?;
        let mut instance = SuggestLine::new();
        instance.populate(&records);
        self.suggest_line = Some(instance);
        Ok(())
    }

    pub fn init_suggest_source(&mut self, source_trigram_csv: &Path) -> anyhow::Result<()> {
        let records: Vec<RecordTrigram> = RecordTrigram::parse_csv(&source_trigram_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load source_trigram_csv error: {:?}", e))?;
        let mut instance = SuggestSource::new();
        instance.populate(&records);
        self.suggest_source = Some(instance);
        Ok(())
    }

    pub fn init_suggest_target(&mut self, target_trigram_csv: &Path) -> anyhow::Result<()> {
        let records: Vec<RecordTrigram> = RecordTrigram::parse_csv(target_trigram_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load target_trigram_csv error: {:?}", e))?;
        let mut instance = SuggestTarget::new();
        instance.populate(&records);
        self.suggest_target = Some(instance);
        Ok(())
    }

    /// Load the clusters with newest/oldest program ids
    pub fn init_recent_program_container(&mut self, recent_program_csv: &Path) -> anyhow::Result<()> {
        let instance = RecentProgramContainer::load(recent_program_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load recent_program_csv error: {:?}", e))?;
        debug!("recent_program_container. number of clusters: {:?}", instance.cluster_program_ids().len());
        self.recent_program_container = Some(instance);
        Ok(())
    }

    /// Load the clusters with popular/unpopular program ids
    pub fn init_popular_program_container(&mut self, popular_program_csv: &Path) -> anyhow::Result<()> {
        let instance = PopularProgramContainer::load(popular_program_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load popular_program_csv error: {:?}", e))?;
        debug!("popular_program_container. number of clusters: {:?}", instance.cluster_program_ids().len());
        self.popular_program_container = Some(instance);
        Ok(())
    }

    pub fn init_histogram_instruction_constant(&mut self, histogram_instruction_constant_csv: &Path) -> anyhow::Result<()> {
        let instance = HistogramInstructionConstant::load_csv_file(histogram_instruction_constant_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load histogram_instruction_constant_csv error: {:?}", e))?;
        debug!("histogram_instruction_constant. number of items: {:?}", instance.number_of_items());
        self.histogram_instruction_constant = Some(instance);
        Ok(())
    }

    /// The programs that can execute.
    pub fn init_valid_program_ids(&mut self, valid_program_ids_csv: &Path) -> anyhow::Result<()> {
        let program_ids: Vec<u32> = load_program_ids_csv_file(valid_program_ids_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load valid_program_ids_csv error: {:?}", e))?;
        debug!("valid_program_ids. number of program ids: {:?}", program_ids.len());
        self.valid_program_ids = Some(program_ids);
        Ok(())
    }

    /// The invalid program_ids, that are defunct, such as cannot execute, cyclic-dependency.
    pub fn init_invalid_program_ids(&mut self, invalid_program_ids_csv: &Path) -> anyhow::Result<()> {
        let program_ids: Vec<u32> = load_program_ids_csv_file(invalid_program_ids_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load invalid_program_ids_csv error: {:?}", e))?;
        debug!("invalid_program_ids. number of program ids: {:?}", program_ids.len());
        let program_ids_hashset: HashSet<u32> = program_ids.into_iter().collect();
        self.invalid_program_ids_hashset = Some(program_ids_hashset);
        Ok(())
    }

    /// The programs that makes use of indirect memory access.
    /// 
    /// These programs trend to have a big memory foot print.
    pub fn init_indirect_memory_access_program_ids(&mut self, indirect_memory_access_csv: &Path) -> anyhow::Result<()> {
        let program_ids: Vec<u32> = load_program_ids_csv_file(indirect_memory_access_csv)
            .map_err(|e| anyhow::anyhow!("Unable to load indirect_memory_access_csv error: {:?}", e))?;
        debug!("indirect_memory_access_program_ids. number of program ids: {:?}", program_ids.len());
        self.indirect_memory_access_program_ids = Some(program_ids);
        Ok(())
    }
}
