use super::{Funnel, Genome, GenomeMutateContext, save_candidate_program};
use super::PreventFlooding;
use super::{PerformanceClassifierResult, PerformanceClassifier};
use super::{MinerThreadMessageToCoordinator, MetricEvent, Recorder};
use super::metrics_run_miner_loop::MetricsRunMinerLoop;
use crate::oeis::TermsToProgramIdSet;
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::{EvalError, NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, ProgramSerializer, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::node_binomial::NodeBinomialLimit;
use loda_rust_core::execute::node_power::NodePowerLimit;
use loda_rust_core::util::{BigIntVec, bigintvec_to_string};
use loda_rust_core::parser::ParsedProgram;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::time::Instant;
use rand::rngs::StdRng;
use std::sync::Arc;

const INTERVAL_UNTIL_NEXT_METRIC_SYNC: u128 = 100;
const MINIMUM_PROGRAM_LENGTH: usize = 2;

struct TermComputer {
    terms: BigIntVec,
    steps: Vec<u64>,
    step_count: u64,
}

impl TermComputer {
    fn new() -> Self {
        Self {
            terms: Vec::with_capacity(40),
            steps: Vec::with_capacity(40),
            step_count: 0,
        }
    }

    fn compute(&mut self, cache: &mut ProgramCache, runner: &ProgramRunner, count: usize) -> Result<(), EvalError> {
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
            self.terms.push(output.0);
            self.steps.push(self.step_count);
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.terms.clear();
        self.steps.clear();
        self.step_count = 0;
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
    term_computer: TermComputer,
    terms_to_program_id: Arc<TermsToProgramIdSet>,
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
        terms_to_program_id: Arc<TermsToProgramIdSet>
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
            term_computer: TermComputer::new(),
            terms_to_program_id: terms_to_program_id,
        }
    }

    pub fn loop_forever(&mut self) {
        let mut progress_time = Instant::now();
        loop {
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
                false_positives: self.metric.number_of_bloomfilter_false_positive,
            };
            self.recorder.record(&event);
        }
        {
            let event = MetricEvent::Genome { 
                cannot_load: self.metric.number_of_failed_genome_loads,
                cannot_parse: self.metric.number_of_programs_that_cannot_parse,
                too_short: self.metric.number_of_too_short_programs,
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
                reject_self_dependency: self.metric.number_of_self_dependencies,
                candidate_program: self.metric.number_of_candidate_programs,
            };
            self.recorder.record(&event);
        }
        self.funnel.reset_metrics();
        self.cache.reset_metrics();
        self.metric.reset_metrics();
    }

    fn execute_one_iteration(&mut self) {
        self.metric.number_of_miner_loop_iterations += 1;
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
            let parsed_program: ParsedProgram = match self.genome.load_program_with_id(&self.dependency_manager, self.current_program_id) {
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
    
        let genome_parsed_program: ParsedProgram = self.genome.to_parsed_program();
        if genome_parsed_program.instruction_vec.len() < MINIMUM_PROGRAM_LENGTH {
            self.metric.number_of_too_short_programs += 1;
            self.reload = true;
            return;
        }

        // Create program from genome
        let result_parse = self.dependency_manager.parse_stage2(
            ProgramId::ProgramWithoutId, 
            &genome_parsed_program
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
        self.term_computer.reset();
        match self.term_computer.compute(&mut self.cache, &runner, 10) {
            Ok(_) => {},
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                self.metric.number_of_compute_errors += 1;
                return;
            }
        }
        let terms10: &BigIntVec = &self.term_computer.terms;
        // println!("terms10: {:?}", terms10);
        if !self.funnel.check_basic(terms10) {
            return;
        }
        if !self.funnel.check10(terms10) {
            return;
        }

        match self.term_computer.compute(&mut self.cache, &runner, 20) {
            Ok(_) => {},
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                self.metric.number_of_compute_errors += 1;
                return;
            }
        }
        let funnel20result: Option<usize> = self.funnel.check20_with_wildcards(&self.term_computer.terms);
        let funnel20_number_of_wildcards: usize;
        match funnel20result {
            Some(wildcard_count) => {
                funnel20_number_of_wildcards = wildcard_count;
            },
            None => {
                // terms is not contained in bloomfilter
                return;
            }
        }

        match self.term_computer.compute(&mut self.cache, &runner, 30) {
            Ok(_) => {},
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                self.metric.number_of_compute_errors += 1;
                return;
            }
        }
        let funnel30result: Option<usize> = self.funnel.check30_with_wildcards(&self.term_computer.terms);
        let funnel30_number_of_wildcards: usize;
        match funnel30result {
            Some(wildcard_count) => {
                funnel30_number_of_wildcards = wildcard_count;
            },
            None => {
                // terms is not contained in bloomfilter
                return;
            }
        }

        match self.term_computer.compute(&mut self.cache, &runner, 40) {
            Ok(_) => {},
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                self.metric.number_of_compute_errors += 1;
                return;
            }
        }
        let terms40_original: BigIntVec = self.term_computer.terms.clone();
        if self.prevent_flooding.try_register(&terms40_original).is_err() {
            // debug!("prevented flooding");
            self.metric.number_of_prevented_floodings += 1;
            self.reload = true;
            return;
        }
        let mut funnel40terms: BigIntVec = terms40_original.clone();
        let funnel40result: Option<usize> = self.funnel.mut_check40_with_wildcards(&mut funnel40terms);
        let funnel40_number_of_wildcards: usize;
        match funnel40result {
            Some(wildcard_count) => {
                funnel40_number_of_wildcards = wildcard_count;
            },
            None => {
                // terms is not contained in bloomfilter
                return;
            }
        }
        let terms40_wildcard: &BigIntVec = &funnel40terms;

        // Reject, if it's identical to one of the programs that this program depends on
        let depends_on_program_ids: HashSet<u32> = self.genome.depends_on_program_ids();
        let mut reject_self_dependency = false;
        for program_id in depends_on_program_ids {
            let program_runner: Rc::<ProgramRunner> = match self.dependency_manager.load(program_id as u64) {
                Ok(value) => value,
                Err(error) => {
                    error!("Cannot verify, failed to load program id {}, {:?}", program_id, error);
                    continue;
                }
            };
            let mut verify_term_computer = TermComputer::new();
            match verify_term_computer.compute(&mut self.cache, &program_runner, 40) {
                Ok(_) => {},
                Err(error) => {
                    debug!("Cannot verify, unable to run program id {}, {:?}", program_id, error);
                    continue;
                }
            }
            let verify_terms40: &BigIntVec = &verify_term_computer.terms;
            if terms40_original == *verify_terms40 {
                // The candidate program seems to be generating the same terms
                // as the program that it depends on.
                // debug!("Rejecting program with a dependency to itself. {}", program_id);
                reject_self_dependency = true;
                break;
            }
        }
        if reject_self_dependency {
            self.metric.number_of_self_dependencies += 1;
            self.reload = true;
            return;
        }

        // lookup in stripped.zip and find the corresponding program_ids
        let key: String = bigintvec_to_string(terms40_wildcard);
        let corresponding_program_id_set: &HashSet<u32> = match self.terms_to_program_id.get(&key) {
            Some(value) => value,
            None => {
                debug!("Ignoring false-positive in bloomfilter funnel. Could not find the candiate in the oeis stripped file. funnel20_number_of_wildcards: {:?} funnel30_number_of_wildcards: {:?} funnel40_number_of_wildcards: {:?} key: {:?}", funnel20_number_of_wildcards, funnel30_number_of_wildcards, funnel40_number_of_wildcards, key);
                self.metric.number_of_bloomfilter_false_positive += 1;
                self.reload = true;
                return
            }
        };
        debug!("Found corresponding program_id's: {:?} funnel20_number_of_wildcards: {:?} funnel30_number_of_wildcards: {:?} funnel40_number_of_wildcards: {:?}", corresponding_program_id_set, funnel20_number_of_wildcards, funnel30_number_of_wildcards, funnel40_number_of_wildcards);

        let steps: &Vec<u64> = &self.term_computer.steps;
        let steps_len: usize = steps.len();
        let performance_classifier = PerformanceClassifier::new(10);
        let mut maybe_a_new_program = false;
        let mut is_existing_program_with_better_performance = false;
        for program_id in corresponding_program_id_set {
            let program_runner: Rc::<ProgramRunner> = match self.dependency_manager.load(*program_id as u64) {
                Ok(value) => value,
                Err(error) => {
                    debug!("Keep. Maybe a new program. Cannot verify, failed to load program id {}, {:?}", program_id, error);
                    self.genome.append_message(format!("keep: maybe a new program. cannot load program {:?} with the same initial terms. error: {:?}", program_id, error));
                    maybe_a_new_program = true;
                    break;
                }
            };
            let mut verify_term_computer = TermComputer::new();
            match verify_term_computer.compute(&mut self.cache, &program_runner, 40) {
                Ok(_) => {},
                Err(error) => {
                    debug!("Keep. Maybe a new program. Cannot verify, unable to run program id {}, {:?}", program_id, error);
                    self.genome.append_message(format!("keep: maybe a new program. cannot compute program {:?} with the same initial terms. error: {:?}", program_id, error));
                    maybe_a_new_program = true;
                    break;
                }
            };
            let verify_terms40: &BigIntVec = &verify_term_computer.terms;
            if terms40_original != *verify_terms40 {
                debug!("Ignoring program with different terms. {}", program_id);
                continue;
            }
            if verify_term_computer.steps.len() != steps_len {
                error!("verify_term_computer.steps.len() {:?} should be the same as steps_len: {:?}", verify_term_computer.steps.len(), steps_len);
                panic!("integrity problem. Length of the computed terms must be the same.");
            }

            let sum_program0: u64 = self.term_computer.step_count;
            let sum_program1: u64 = verify_term_computer.step_count;
            if sum_program0 >= sum_program1 {
                debug!("Reject. The new program is slower or identical to the old program");
                continue;
            }

            let pcr: PerformanceClassifierResult = performance_classifier.analyze(&steps, &verify_term_computer.steps);
            match pcr {
                PerformanceClassifierResult::ErrorDifferentInputVectorLengths => {
                    panic!("integrity problem. Length of the computed terms must be the same.");
                },
                PerformanceClassifierResult::ErrorTooShortInputVector => {
                    panic!("integrity problem. The length of the first slice goes beyond the input length");
                },
                PerformanceClassifierResult::Identical => {
                    debug!("Reject. Identical performance as the existing program. {:?}", program_id);
                    continue;
                },
                PerformanceClassifierResult::NewProgramIsAlwaysFaster => {
                    debug!("Keep. The new program is always faster than the old program.");
                    self.genome.append_message(format!("keep: performance NewProgramIsAlwaysFaster than {:?}", program_id));
                    is_existing_program_with_better_performance = true;
                    break;
                },
                PerformanceClassifierResult::NewProgramIsEqualOrFaster => {
                    debug!("Keep. The new program is faster or similar than the old program.");
                    self.genome.append_message(format!("keep: performance NewProgramIsEqualOrFaster than {:?}", program_id));
                    is_existing_program_with_better_performance = true;
                    break;
                },
                PerformanceClassifierResult::NewProgramIsAlwaysFasterWhenSkippingTheFirstSlice => {
                    debug!("Keep. The new program is faster when skipping the first slice");
                    self.genome.append_message(format!("keep: performance NewProgramIsAlwaysFasterWhenSkippingTheFirstSlice than {:?}", program_id));
                    is_existing_program_with_better_performance = true;
                    break;
                },
                PerformanceClassifierResult::RejectNewProgram => {
                    debug!("Reject. Worse performance than the existing program. {:?}", program_id);
                    continue;
                }
            }
        }
        let keep_it = maybe_a_new_program || is_existing_program_with_better_performance;
        if !keep_it {
            debug!("Reject. Worse performance than existing programs.");
            self.reload = true;
            return;
        }

        if funnel20_number_of_wildcards > 0 {
            self.genome.append_message(format!("funnel20 number of wildcards: {:?}", funnel20_number_of_wildcards));
        }
        if funnel30_number_of_wildcards > 0 {
            self.genome.append_message(format!("funnel30 number of wildcards: {:?}", funnel30_number_of_wildcards));
        }
        if funnel40_number_of_wildcards > 0 {
            self.genome.append_message(format!("funnel40 number of wildcards: {:?}", funnel40_number_of_wildcards));
        }

        // Yay, this candidate program seems to be good.
        // It's either an entirely new program.
        // Or it's faster than the existing program.
        // Save a snapshot of this program to `$HOME/.loda-rust/mine-even/`
        let mut serializer = ProgramSerializer::new();
        serializer.append_comment(bigintvec_to_string(&terms40_original));
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
