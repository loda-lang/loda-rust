
pub struct GenomeMutateContext {
    available_program_ids: Vec<u32>,
}

impl GenomeMutateContext {
    pub fn new(available_program_ids: Vec<u32>) -> Self {
        Self {
            available_program_ids: available_program_ids,
        }
    }

    pub fn available_program_ids(&self) -> &Vec<u32> {
        &self.available_program_ids
    }
}
