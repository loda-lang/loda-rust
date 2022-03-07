#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KeyMetricU32 {
    NumberOfMinerLoopIterations,
}

pub enum MetricEvent {
    Funnel { 
        basic: u64,
        terms10: u64,
        terms20: u64,
        terms30: u64,
        terms40: u64,
    },
    Cache { 
        hit: u64,
        miss_program_oeis: u64,
        miss_program_without_id: u64,
    },
    Genome {
        cannot_load: u64,
        cannot_parse: u64,
        no_output: u64,
        no_mutation: u64,
        compute_error: u64,
    },
    General {
        prevent_flooding: u64,
        candidate_program: u64,
    }
}

pub trait Recorder<Event> {
    fn record(&self, event: &Event);
}
