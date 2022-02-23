use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

pub struct Metrics {
    pub number_of_workers: Gauge::<u64>,
    pub number_of_iterations: Counter,
    pub number_of_iteration_now: Gauge::<u64>,
    pub number_of_candidate_programs: Gauge::<u64>,
    pub cache_hit: Counter,
    pub cache_miss: Counter,
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
            "Number of cache hits",
            Box::new(cache_hit.clone()),
        );

        let cache_miss = Counter::default();
        sub_registry.register(
            "cache_miss",
            "Number of cache hits",
            Box::new(cache_miss.clone()),
        );

        Self {
            number_of_workers: number_of_workers,
            number_of_iterations: number_of_iterations,
            number_of_iteration_now: number_of_iteration_now,
            cache_hit: cache_hit,
            cache_miss: cache_miss,
            number_of_candidate_programs: number_of_candidate_programs,
        }
    }
}
