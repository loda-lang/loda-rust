use crate::mine::PopularProgramContainer;

pub struct GenomeMutateContext {
    available_program_ids: Vec<u32>,
    program_program_container: PopularProgramContainer,
}

impl GenomeMutateContext {
    pub fn new(available_program_ids: Vec<u32>, program_program_container: PopularProgramContainer) -> Self {
        Self {
            available_program_ids: available_program_ids,
            program_program_container: program_program_container,
        }
    }

    pub fn available_program_ids(&self) -> &Vec<u32> {
        &self.available_program_ids
    }
}
