use super::{Config, MinerCPUStrategy};

extern crate num_cpus;

pub trait NumberOfWorkers {
    fn resolve_number_of_miner_workers(&self) -> usize;
}

impl NumberOfWorkers for Config {    
    fn resolve_number_of_miner_workers(&self) -> usize {
        let miner_cpu_strategy = self.miner_cpu_strategy();
        if miner_cpu_strategy == MinerCPUStrategy::Min {
            return 1;
        }
        let number_of_available_cpus: usize = num_cpus::get();
        assert!(number_of_available_cpus >= 1_usize);
        assert!(number_of_available_cpus < 1000_usize);
        let number_of_threads: usize = match miner_cpu_strategy {
            MinerCPUStrategy::Min => 1,
            MinerCPUStrategy::Half => number_of_available_cpus / 2,
            MinerCPUStrategy::Max => number_of_available_cpus,
            MinerCPUStrategy::CPU { count } => count as usize,
        };
        // Ensures that zero is never returned
        number_of_threads.max(1)
    }
}
