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
        too_short: u64,
        no_output: u64,
        no_mutation: u64,
        compute_error: u64,
    },
    General {
        prevent_flooding: u64,
        reject_self_dependency: u64,
        candidate_program: u64,
    }
}

pub trait Recorder: RecorderClone {
    fn record(&self, event: &MetricEvent);
}

pub trait RecorderClone {
    fn clone_box(&self) -> Box<dyn Recorder + Send>;
}

impl<T> RecorderClone for T
where
    T: 'static + Recorder + Clone + Send,
{
    fn clone_box(&self) -> Box<dyn Recorder + Send> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Recorder + Send> {
    fn clone(&self) -> Box<dyn Recorder + Send> { 
        self.clone_box() 
    }
}

#[derive(Clone)]
pub struct SinkRecorder {}

impl Recorder for SinkRecorder {
    fn record(&self, _event: &MetricEvent) {
        // print!("sink recorder")
    }
}
