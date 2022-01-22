use super::{PopularProgramContainer, RecentProgramContainer, HistogramInstructionConstant};
use super::SuggestInstruction;
use loda_rust_core::parser::InstructionId;
use rand::Rng;
use rand::seq::SliceRandom;

pub struct GenomeMutateContext {
    available_program_ids: Vec<u32>,
    popular_program_container: PopularProgramContainer,
    recent_program_container: RecentProgramContainer,
    histogram_instruction_constant: Option<HistogramInstructionConstant>,
    suggest_instruction: Option<SuggestInstruction>,
}

impl GenomeMutateContext {
    pub fn new(
        available_program_ids: Vec<u32>, 
        popular_program_container: PopularProgramContainer, 
        recent_program_container: RecentProgramContainer,
        histogram_instruction_constant: Option<HistogramInstructionConstant>,
        suggest_instruction: Option<SuggestInstruction>
    ) -> Self {
        Self {
            available_program_ids: available_program_ids,
            popular_program_container: popular_program_container,
            recent_program_container: recent_program_container,
            histogram_instruction_constant: histogram_instruction_constant,
            suggest_instruction: suggest_instruction
        }
    }

    pub fn available_program_ids(&self) -> &Vec<u32> {
        &self.available_program_ids
    }

    pub fn choose_available_program<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        let program_id: u32 = match self.available_program_ids.choose(rng) {
            Some(program_id) => *program_id,
            None => {
                // For a non-empty vector, this shouldn't happen.
                return None;
            }
        };
        Some(program_id)
    }

    pub fn choose_popular_program<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        self.popular_program_container.choose(rng)
    }

    pub fn choose_recent_program<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        self.recent_program_container.choose(rng)
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

    pub fn suggest_instruction<R: Rng + ?Sized>(&self, rng: &mut R, prev_instruction: Option<InstructionId>, next_instruction: Option<InstructionId>) -> Option<InstructionId> {
        let suggest_instruction: &SuggestInstruction = match &self.suggest_instruction {
            Some(value) => value,
            None => {
                return None;
            }
        };
        suggest_instruction.choose_weighted(rng, prev_instruction, next_instruction)
    }
}
