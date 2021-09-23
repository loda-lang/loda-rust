use crate::mine::{PopularProgramContainer, RecentProgramContainer};
use rand::Rng;
use rand::seq::SliceRandom;

pub struct GenomeMutateContext {
    available_program_ids: Vec<u32>,
    popular_program_container: PopularProgramContainer,
    recent_program_container: RecentProgramContainer,
}

impl GenomeMutateContext {
    pub fn new(
        available_program_ids: Vec<u32>, 
        popular_program_container: PopularProgramContainer, 
        recent_program_container: RecentProgramContainer) -> Self 
    {
        Self {
            available_program_ids: available_program_ids,
            popular_program_container: popular_program_container,
            recent_program_container: recent_program_container,
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
}
