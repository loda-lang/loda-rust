#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KeyMetricU32 {
    NumberOfMinerLoopIterations,
}

pub enum MetricEvent {
    CacheHit { 
        increment: u64
    },
    CacheMissProgramOeis { 
        increment: u64
    },
    CacheMissProgramWithoutId { 
        increment: u64
    },
    ErrorGenomeLoad {
        increment: u64
    },
}

pub trait Recorder<Event> {
    fn record(&self, event: &Event);
}
