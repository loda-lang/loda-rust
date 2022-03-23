pub struct MetricsRunMinerLoop {
    pub number_of_miner_loop_iterations: u64,
    pub number_of_prevented_floodings: u64,
    pub number_of_failed_genome_loads: u64,
    pub number_of_failed_mutations: u64,
    pub number_of_programs_that_cannot_parse: u64,
    pub number_of_programs_without_output: u64,
    pub number_of_compute_errors: u64,
    pub number_of_candidate_programs: u64,
}

impl MetricsRunMinerLoop {
    pub fn new() -> Self {
        Self {
            number_of_miner_loop_iterations: 0,
            number_of_prevented_floodings: 0,
            number_of_failed_genome_loads: 0,
            number_of_failed_mutations: 0,
            number_of_programs_that_cannot_parse: 0,
            number_of_programs_without_output: 0,
            number_of_compute_errors: 0,
            number_of_candidate_programs: 0
        }
    }

    pub fn reset_metrics(&mut self) {
        self.number_of_miner_loop_iterations = 0;
        self.number_of_prevented_floodings = 0;
        self.number_of_failed_mutations = 0;
        self.number_of_programs_that_cannot_parse = 0;
        self.number_of_programs_without_output = 0;
        self.number_of_compute_errors = 0;
        self.number_of_failed_genome_loads = 0;
        self.number_of_candidate_programs = 0;
    }  
}
