use super::{PopularProgramContainer, RecentProgramContainer, HistogramInstructionConstant};
use super::SuggestInstruction;
use super::{SuggestLine, LineValue};
use super::{SuggestSource, SourceValue};
use super::{SuggestTarget, TargetValue};
use loda_rust_core::parser::InstructionId;
use std::fmt;
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::HashSet;

#[derive(Clone)]
pub struct GenomeMutateContext {
    valid_program_ids: Vec<u32>,
    initial_genome_program_ids: Vec<u32>, 
    indirect_memory_access_program_ids: Vec<u32>,
    invalid_program_ids: HashSet<u32>,
    popular_program_container: PopularProgramContainer,
    recent_program_container: Option<RecentProgramContainer>,
    histogram_instruction_constant: Option<HistogramInstructionConstant>,
    suggest_instruction: Option<SuggestInstruction>,
    suggest_line: Option<SuggestLine>,
    suggest_source: Option<SuggestSource>,
    suggest_target: Option<SuggestTarget>,
}

impl GenomeMutateContext {
    pub fn new(
        valid_program_ids: Vec<u32>, 
        initial_genome_program_ids: Vec<u32>, 
        indirect_memory_access_program_ids: Vec<u32>,
        invalid_program_ids: HashSet<u32>,
        popular_program_container: PopularProgramContainer, 
        recent_program_container: Option<RecentProgramContainer>,
        histogram_instruction_constant: Option<HistogramInstructionConstant>,
        suggest_instruction: Option<SuggestInstruction>,
        suggest_line: Option<SuggestLine>,
        suggest_source: Option<SuggestSource>,
        suggest_target: Option<SuggestTarget>
    ) -> Self {
        Self {
            valid_program_ids: valid_program_ids,
            initial_genome_program_ids: initial_genome_program_ids,
            indirect_memory_access_program_ids: indirect_memory_access_program_ids,
            invalid_program_ids: invalid_program_ids,
            popular_program_container: popular_program_container,
            recent_program_container: recent_program_container,
            histogram_instruction_constant: histogram_instruction_constant,
            suggest_instruction: suggest_instruction,
            suggest_line: suggest_line,
            suggest_source: suggest_source,
            suggest_target: suggest_target
        }
    }

    pub fn new_empty() -> Self {
        Self {
            valid_program_ids: vec!(),
            initial_genome_program_ids: vec!(),
            indirect_memory_access_program_ids: vec!(),
            invalid_program_ids: HashSet::<u32>::new(),
            popular_program_container: PopularProgramContainer::new_empty(),
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

    pub fn has_available_programs(&self) -> bool {
        !self.valid_program_ids.is_empty()
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
        self.popular_program_container.choose_weighted_by_popularity(rng)
    }

    pub fn choose_most_popular<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        self.popular_program_container.choose_most_popular(rng)
    }

    pub fn choose_medium_popular<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        self.popular_program_container.choose_medium_popular(rng)
    }

    pub fn choose_least_popular<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        self.popular_program_container.choose_least_popular(rng)
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
