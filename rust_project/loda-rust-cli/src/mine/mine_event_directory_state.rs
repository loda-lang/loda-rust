use super::ExecuteBatchResult;

const MINEEVENTDIRECTORY_MINING_LIMIT: usize = 40;

#[derive(Clone, Debug)]
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

    #[allow(dead_code)]
    pub fn number_of_mined_high_prio(&self) -> usize {
        self.number_of_mined_high_prio
    }

    pub fn set_number_of_mined_high_prio(&mut self, count: usize) {
        self.number_of_mined_high_prio = count;
    }

    #[allow(dead_code)]
    pub fn number_of_mined_low_prio(&self) -> usize {
        self.number_of_mined_low_prio
    }

    pub fn set_number_of_mined_low_prio(&mut self, count: usize) {
        self.number_of_mined_low_prio = count;
    }

    pub fn accumulate_stats(&mut self, execute_batch_result: &ExecuteBatchResult) {
        self.number_of_mined_low_prio += execute_batch_result.number_of_mined_low_prio();
        self.number_of_mined_high_prio += execute_batch_result.number_of_mined_high_prio();
    }

    pub fn has_reached_mining_limit(&self) -> bool {
        let sum = self.number_of_mined_high_prio + self.number_of_mined_low_prio;
        sum >= MINEEVENTDIRECTORY_MINING_LIMIT
    }
}
