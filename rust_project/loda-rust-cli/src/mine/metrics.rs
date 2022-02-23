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
            funnel_basic: funnel_basic,
            funnel_10terms: funnel_10terms,
            funnel_20terms: funnel_20terms,
            funnel_30terms: funnel_30terms,
            funnel_40terms: funnel_40terms,
        }
    }
}
