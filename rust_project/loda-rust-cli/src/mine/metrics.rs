use super::{MetricEvent, Recorder};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

#[derive(Clone)]
pub struct Metrics {
    pub number_of_workers: Gauge::<u64>,
    pub number_of_iterations: Counter,
    pub number_of_iteration_now: Gauge::<u64>,
    pub number_of_candidate_programs: Gauge::<u64>,
    pub cache_hit: Counter,
    pub cache_miss_program_oeis: Counter,
    pub cache_miss_program_without_id: Counter,
    pub error_genome_load: Counter,
    pub reject_cannot_be_parsed: Counter,
    pub reject_no_output_register: Counter,
    pub reject_compute_error: Counter,
    pub reject_mutate_without_impact: Counter,
    pub rejected_preventing_flooding: Counter,
    pub funnel_basic: Counter,
    pub funnel_10terms: Counter,
    pub funnel_20terms: Counter,
    pub funnel_30terms: Counter,
    pub funnel_40terms: Counter,
}

impl Metrics {
    pub fn new(registry: &mut Registry) -> Self {
        let sub_registry = registry.sub_registry_with_prefix("lodarust_mine");

        let number_of_workers = Gauge::<u64>::default();
        sub_registry.register(
            "workers", 
            "Number of workers", 
            Box::new(number_of_workers.clone())
        );

        let number_of_iterations = Counter::default();
        sub_registry.register(
            "iterations",
            "Number of iterations",
            Box::new(number_of_iterations.clone()),
        );    

        let number_of_iteration_now = Gauge::<u64>::default();
        sub_registry.register(
            "iterations_now", 
            "Number of iterations right now", 
            Box::new(number_of_iteration_now.clone())
        );

        let number_of_candidate_programs = Gauge::<u64>::default();
        sub_registry.register(
            "candiate_programs", 
            "Number of candidate programs found so far", 
            Box::new(number_of_candidate_programs.clone())
        );

        let cache_hit = Counter::default();
        sub_registry.register(
            "cache_hit",
            "Cache hits",
            Box::new(cache_hit.clone()),
        );

        let cache_miss_program_oeis = Counter::default();
        sub_registry.register(
            "cache_miss_program_oeis",
            "Cache misses for oeis programs",
            Box::new(cache_miss_program_oeis.clone()),
        );

        let cache_miss_program_without_id = Counter::default();
        sub_registry.register(
            "cache_miss_program_without_id",
            "Cache misses for programs without id",
            Box::new(cache_miss_program_without_id.clone()),
        );

        let error_genome_load = Counter::default();
        sub_registry.register(
            "error_genome_load",
            "Unable to load program into genome",
            Box::new(error_genome_load.clone()),
        );

        let reject_cannot_be_parsed = Counter::default();
        sub_registry.register(
            "reject_cannot_be_parsed",
            "Rejected programs because they cannot be parsed",
            Box::new(reject_cannot_be_parsed.clone()),
        );

        let reject_no_output_register = Counter::default();
        sub_registry.register(
            "reject_no_output_register",
            "Rejected programs because they have no output register",
            Box::new(reject_no_output_register.clone()),
        );

        let reject_compute_error = Counter::default();
        sub_registry.register(
            "reject_compute_error",
            "Rejected programs because of compute error",
            Box::new(reject_compute_error.clone()),
        );

        let reject_mutate_without_impact = Counter::default();
        sub_registry.register(
            "reject_mutate_without_impact",
            "Rejected programs because mutate had no effect",
            Box::new(reject_mutate_without_impact.clone()),
        );

        let rejected_preventing_flooding = Counter::default();
        sub_registry.register(
            "rejected_preventing_flooding",
            "Rejected programs because there already exist similar candidate programs",
            Box::new(rejected_preventing_flooding.clone()),
        );

        let funnel_basic = Counter::default();
        sub_registry.register(
            "funnel_basic",
            "Number of programs that passed the basic funnel",
            Box::new(funnel_basic.clone()),
        );

        let funnel_10terms = Counter::default();
        sub_registry.register(
            "funnel_10terms",
            "Number of programs that passed the 10 terms funnel",
            Box::new(funnel_10terms.clone()),
        );

        let funnel_20terms = Counter::default();
        sub_registry.register(
            "funnel_20terms",
            "Number of programs that passed the 20 terms funnel",
            Box::new(funnel_20terms.clone()),
        );

        let funnel_30terms = Counter::default();
        sub_registry.register(
            "funnel_30terms",
            "Number of programs that passed the 30 terms funnel",
            Box::new(funnel_30terms.clone()),
        );

        let funnel_40terms = Counter::default();
        sub_registry.register(
            "funnel_40terms",
            "Number of programs that passed the 40 terms funnel",
            Box::new(funnel_40terms.clone()),
        );

        Self {
            number_of_workers: number_of_workers,
            number_of_iterations: number_of_iterations,
            number_of_iteration_now: number_of_iteration_now,
            number_of_candidate_programs: number_of_candidate_programs,
            cache_hit: cache_hit,
            cache_miss_program_oeis: cache_miss_program_oeis,
            cache_miss_program_without_id: cache_miss_program_without_id,
            error_genome_load: error_genome_load,
            reject_cannot_be_parsed: reject_cannot_be_parsed,
            reject_no_output_register: reject_no_output_register,
            reject_compute_error: reject_compute_error,
            reject_mutate_without_impact: reject_mutate_without_impact,
            rejected_preventing_flooding: rejected_preventing_flooding,
            funnel_basic: funnel_basic,
            funnel_10terms: funnel_10terms,
            funnel_20terms: funnel_20terms,
            funnel_30terms: funnel_30terms,
            funnel_40terms: funnel_40terms,
        }
    }
}

impl Recorder<MetricEvent> for Metrics {
    fn record(&self, event: &MetricEvent) {
        match event {
            MetricEvent::Funnel { basic, terms10, terms20, terms30, terms40 } => {
                self.funnel_basic.inc_by(*basic);
                self.funnel_10terms.inc_by(*terms10);
                self.funnel_20terms.inc_by(*terms20);
                self.funnel_30terms.inc_by(*terms30);
                self.funnel_40terms.inc_by(*terms40);
            },
            MetricEvent::Cache { hit, miss_program_oeis, miss_program_without_id } => {
                self.cache_hit.inc_by(*hit);
                self.cache_miss_program_oeis.inc_by(*miss_program_oeis);
                self.cache_miss_program_without_id.inc_by(*miss_program_without_id);
            },
            MetricEvent::Genome { cannot_load, cannot_parse, no_output, no_mutation, compute_error } => {
                self.error_genome_load.inc_by(*cannot_load);
                self.reject_cannot_be_parsed.inc_by(*cannot_parse);
                self.reject_no_output_register.inc_by(*no_output);
                self.reject_mutate_without_impact.inc_by(*no_mutation);
                self.reject_compute_error.inc_by(*compute_error);
            },
            MetricEvent::General { prevent_flooding, candidate_program } => {
                self.rejected_preventing_flooding.inc_by(*prevent_flooding);
                self.number_of_candidate_programs.inc_by(*candidate_program);
            },
        }
    }
}
