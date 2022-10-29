use super::ExecuteBatchResult;

const MINEEVENTDIRECTORY_HIGH_PRIORITY_LIMIT: usize = 10;

#[derive(Debug)]
pub struct MineEventDirectoryState {
    number_of_mined_high_prio: usize,
    number_of_mined_low_prio: usize,
}

impl MineEventDirectoryState {
    pub fn new() -> Self {
        Self {
            number_of_mined_high_prio: 0, 
            number_of_mined_low_prio: 0,
        }
    }

    pub fn reset(&mut self) {
        self.number_of_mined_high_prio = 0;
        self.number_of_mined_low_prio = 0;
    }

    #[allow(dead_code)]
    pub fn number_of_mined_high_prio(&self) -> usize {
        self.number_of_mined_high_prio
    }

    #[allow(dead_code)]
    pub fn number_of_mined_low_prio(&self) -> usize {
        self.number_of_mined_low_prio
    }

    pub fn accumulate_stats(&mut self, execute_batch_result: &ExecuteBatchResult) {
        self.number_of_mined_low_prio += execute_batch_result.number_of_mined_low_prio();
        self.number_of_mined_high_prio += execute_batch_result.number_of_mined_high_prio();
    }

    pub fn has_reached_mining_limit(&self) -> bool {
        self.number_of_mined_high_prio >= MINEEVENTDIRECTORY_HIGH_PRIORITY_LIMIT
    }
}
