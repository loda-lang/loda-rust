use crate::mine::PopularProgramContainer;
use rand::Rng;

pub struct GenomeMutateContext {
    available_program_ids: Vec<u32>,
    popular_program_container: PopularProgramContainer,
}

impl GenomeMutateContext {
    pub fn new(available_program_ids: Vec<u32>, popular_program_container: PopularProgramContainer) -> Self {
        Self {
            available_program_ids: available_program_ids,
            popular_program_container: popular_program_container,
        }
    }

    pub fn available_program_ids(&self) -> &Vec<u32> {
        &self.available_program_ids
    }

    pub fn choose_popular_program_id<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        self.popular_program_container.choose(rng)
    }
}
