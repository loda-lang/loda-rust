use super::{Funnel, Genome, GenomeMutateContext, save_candidate_program};
use super::PreventFlooding;
use super::{MinerThreadMessageToCoordinator, MetricEvent, Recorder};
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::{EvalError, NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, ProgramSerializer, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::node_binomial::NodeBinomialLimit;
use loda_rust_core::execute::node_power::NodePowerLimit;
use loda_rust_core::util::{BigIntVec, bigintvec_to_string};
use loda_rust_core::parser::ParsedProgram;
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use rand::rngs::StdRng;

const INTERVAL_UNTIL_NEXT_METRIC_SYNC: u128 = 100;

struct TermComputer {
    terms: BigIntVec,
    step_count: u64,
}

impl TermComputer {
    fn create() -> Self {
        Self {
            terms: vec!(),
            step_count: 0,
        }
    }

    fn compute(&mut self, cache: &mut ProgramCache, runner: &ProgramRunner, count: usize) -> Result<BigIntVec, EvalError> {
        let step_count_limit: u64 = 10000;
        let node_register_limit = NodeRegisterLimit::LimitBits(32);
        let node_binomial_limit = NodeBinomialLimit::LimitN(20);
        let node_loop_limit = NodeLoopLimit::LimitCount(1000);
        let node_power_limit = NodePowerLimit::LimitBits(30);
        loop {
            let length: usize = self.terms.len();
            if length >= count {
                break;
            }
            let index = length as i64;
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = runner.run(
                &input, 
                RunMode::Silent, 
                &mut self.step_count, 
                step_count_limit, 
                node_register_limit.clone(),
                node_binomial_limit.clone(),
                node_loop_limit.clone(),
                node_power_limit.clone(),
                cache
            )?;
            self.terms.push(output.0.clone());
        }
        Ok(self.terms.clone())
    }
}

struct MetricsRunMinerLoop {
    number_of_miner_loop_iterations: u64,
    number_of_prevented_floodings: u64,
    number_of_failed_genome_loads: u64,
    number_of_failed_mutations: u64,
    number_of_programs_that_cannot_parse: u64,
    number_of_programs_without_output: u64,
    number_of_compute_errors: u64,
    number_of_candidate_programs: u64,
}

impl MetricsRunMinerLoop {
    fn new() -> Self {
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

    fn reset_metrics(&mut self) {
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

pub struct RunMinerLoop {
    tx: Sender<MinerThreadMessageToCoordinator>,
    recorder: Box<dyn Recorder>,
    dependency_manager: DependencyManager,
    funnel: Funnel,
    mine_event_dir: PathBuf,
    cache: ProgramCache,
    prevent_flooding: PreventFlooding,
    context: GenomeMutateContext,
    genome: Genome,
    rng: StdRng,
    metric: MetricsRunMinerLoop,
    current_program_id: u64,
    current_parsed_program: ParsedProgram,
    iteration: usize,
    reload: bool,
}

impl RunMinerLoop {
    pub fn new(
        tx: Sender<MinerThreadMessageToCoordinator>,
        recorder: Box<dyn Recorder>,
        dependency_manager: DependencyManager,
        funnel: Funnel,
        mine_event_dir: &Path,
        cache: ProgramCache,
        prevent_flooding: PreventFlooding,
        context: GenomeMutateContext,
        genome: Genome,
        rng: StdRng,
    ) -> Self {
        Self {
            tx: tx,
            recorder: recorder,
            dependency_manager: dependency_manager,
            funnel: funnel,
            mine_event_dir: PathBuf::from(mine_event_dir),
            cache: cache,
            prevent_flooding: prevent_flooding,
            context: context,
            genome: genome,
            rng: rng,
            metric: MetricsRunMinerLoop::new(),
            current_program_id: 0,
            current_parsed_program: ParsedProgram::new(),
            iteration: 0,
            reload: true,
        }
    }

    pub fn loop_forever(&mut self) {
        let mut progress_time = Instant::now();
        loop {
            self.metric.number_of_miner_loop_iterations += 1;
            let elapsed: u128 = progress_time.elapsed().as_millis();
            if elapsed >= INTERVAL_UNTIL_NEXT_METRIC_SYNC {
                self.submit_metrics();
                progress_time = Instant::now();
            }
            self.execute_one_iteration();
        }
    }

    fn submit_metrics(&mut self) {
        {
            let y: u64 = self.metric.number_of_miner_loop_iterations;
            let message = MinerThreadMessageToCoordinator::NumberOfIterations(y);
            self.tx.send(message).unwrap();
        }
        {
            let event = MetricEvent::Funnel { 
                basic: self.funnel.metric_number_of_candidates_with_basiccheck(),
                terms10: self.funnel.metric_number_of_candidates_with_10terms(),
                terms20: self.funnel.metric_number_of_candidates_with_20terms(),
                terms30: self.funnel.metric_number_of_candidates_with_30terms(),
                terms40: self.funnel.metric_number_of_candidates_with_40terms(),
            };
            self.recorder.record(&event);
        }
        {
            let event = MetricEvent::Genome { 
                cannot_load: self.metric.number_of_failed_genome_loads,
                cannot_parse: self.metric.number_of_programs_that_cannot_parse,
                no_output: self.metric.number_of_programs_without_output,
                no_mutation: self.metric.number_of_failed_mutations,
                compute_error: self.metric.number_of_compute_errors,
            };
            self.recorder.record(&event);
        }
        {
            let event = MetricEvent::Cache { 
                hit: self.cache.metric_hit(),
                miss_program_oeis: self.cache.metric_miss_for_program_oeis(),
                miss_program_without_id: self.cache.metric_miss_for_program_without_id(),
            };
            self.recorder.record(&event);
        }
        {
            let event = MetricEvent::General { 
                prevent_flooding: self.metric.number_of_prevented_floodings,
                candidate_program: self.metric.number_of_candidate_programs,
            };
            self.recorder.record(&event);
        }
        self.funnel.reset_metrics();
        self.cache.reset_metrics();
        self.metric.reset_metrics();
    }

    fn execute_one_iteration(&mut self) {
        if (self.iteration % 10) == 0 {
            self.reload = true;
        }
        if (self.iteration % 50000) == 0 {
            match self.context.choose_available_program(&mut self.rng) {
                Some(program_id) => { 
                    self.current_program_id = program_id as u64;
                },
                None => {
                    panic!("Unable to pick among available programs");
                }
            };
            let parsed_program: ParsedProgram = match self.genome.load_program(&self.dependency_manager, self.current_program_id) {
                Some(value) => value,
                None => {
                    error!("Unable to parse available program");
                    self.reload = true;
                    return;
                }
            };
            self.current_parsed_program = parsed_program;
        }
        if self.reload {
            self.genome.clear_message_vec();
            let load_ok: bool = self.genome.insert_program(self.current_program_id, &self.current_parsed_program);
            if !load_ok {
                self.metric.number_of_failed_genome_loads += 1;
                return;
            }
            self.reload = false;
        }

        self.iteration += 1;
        
        if !self.genome.mutate(&mut self.rng, &self.context) {
            self.metric.number_of_failed_mutations += 1;
            return;
        }

        // println!("#{} Current genome\n{}", iteration, self.genome);
    
        // Create program from genome
        self.dependency_manager.reset();
        let result_parse = self.dependency_manager.parse_stage2(
            ProgramId::ProgramWithoutId, 
            &self.genome.to_parsed_program()
        );
        let mut runner: ProgramRunner = match result_parse {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be parsed. {}", iteration, error);
                self.metric.number_of_programs_that_cannot_parse += 1;
                return;
            }
        };

        // If the program has no live output register, then pick the lowest live register.
        if !runner.mining_trick_attempt_fixing_the_output_register() {
            self.metric.number_of_programs_without_output += 1;
            return;
        }

        // Execute program
        let mut term_computer = TermComputer::create();
        let terms10: BigIntVec = match term_computer.compute(&mut self.cache, &runner, 10) {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                self.metric.number_of_compute_errors += 1;
                return;
            }
        };
        // println!("terms10: {:?}", terms10);
        if !self.funnel.check_basic(&terms10) {
            return;
        }
        if !self.funnel.check10(&terms10) {
            return;
        }

        let terms20: BigIntVec = match term_computer.compute(&mut self.cache, &runner, 20) {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                self.metric.number_of_compute_errors += 1;
                return;
            }
        };
        if !self.funnel.check20(&terms20) {
            return;
        }

        let terms30: BigIntVec = match term_computer.compute(&mut self.cache, &runner, 30) {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                self.metric.number_of_compute_errors += 1;
                return;
            }
        };
        if !self.funnel.check30(&terms30) {
            return;
        }

        let terms40: BigIntVec = match term_computer.compute(&mut self.cache, &runner, 40) {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                self.metric.number_of_compute_errors += 1;
                return;
            }
        };
        if !self.funnel.check40(&terms40) {
            return;
        }

        if self.prevent_flooding.try_register(&terms40).is_err() {
            // debug!("prevented flooding");
            self.metric.number_of_prevented_floodings += 1;
            self.reload = true;
            return;
        }

        // Yay, this candidate program has 40 terms that are good.
        // Save a snapshot of this program to `$HOME/.loda-rust/mine-even/`
        let mut serializer = ProgramSerializer::new();
        serializer.append_comment(bigintvec_to_string(&terms40));
        serializer.append_empty_line();
        runner.serialize(&mut serializer);
        serializer.append_empty_line();
        for message in self.genome.message_vec() {
            serializer.append_comment(message);
        }
        serializer.append_empty_line();
        let candidate_program: String = serializer.to_string();

        if let Err(error) = save_candidate_program(&self.mine_event_dir, self.iteration, &candidate_program) {
            println!("; GENOME\n{}", self.genome);
            error!("Unable to save candidate program: {:?}", error);
            return;
        }
        self.metric.number_of_candidate_programs += 1;
    }
}
