use super::{Genome, GenomeItem, GenomeMutateContext, save_candidate_program, ToGenomeItemVec};
use super::{CreateFunnel, Funnel};
use super::{PreventFlooding, TermComputer};
use super::{PerformanceClassifierResult, PerformanceClassifier};
use super::MetricEvent;
use super::metrics_run_miner_loop::MetricsRunMinerLoop;
use crate::oeis::TermsToProgramIdSet;
use crate::config::{Config, MinerFilterMode};
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::{ProgramCache, ProgramId, ProgramRunner, ProgramSerializer};
use loda_rust_core::util::{BigIntVec, BigIntVecToString};
use loda_rust_core::parser::ParsedProgram;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::sync::{Arc, Mutex};

const EXECUTE_BATCH_TIME_LIMIT: u128 = 2000;
const INTERVAL_UNTIL_NEXT_METRIC_SYNC: u128 = 100;
const MINIMUM_PROGRAM_LENGTH: usize = 6;
const LOAD_INITIAL_GENOME_MINIMUM_PROGRAM_LENGTH: usize = 8;
const LOAD_INITIAL_GENOME_RETRIES: usize = 1000;
const MINER_CACHE_CAPACITY: usize = 3000;
const ITERATIONS_BETWEEN_PICKING_A_NEW_INITIAL_GENOME: usize = 300;
const ITERATIONS_BETWEEN_RELOADING_CURRENT_GENOME: usize = 5;

#[derive(Clone, Debug)]
pub struct ExecuteBatchResult {
    number_of_mined_high_prio: usize,
    number_of_mined_low_prio: usize,
}

impl ExecuteBatchResult {
    pub fn new() -> Self {
        Self {
            number_of_mined_high_prio: 0, 
            number_of_mined_low_prio: 0,
        }
    }

    pub fn number_of_mined_high_prio(&self) -> usize {
        self.number_of_mined_high_prio
    }

    pub fn number_of_mined_low_prio(&self) -> usize {
        self.number_of_mined_low_prio
    }

    pub fn increment_number_of_mined_high_prio(&mut self) {
        self.number_of_mined_high_prio += 1;
    }

    pub fn increment_number_of_mined_low_prio(&mut self) {
        self.number_of_mined_low_prio += 1;
    }
}

pub struct RunMinerLoop {
    metrics_callback: Option<Box<dyn Fn(MetricEvent) + Send>>,
    funnel: Funnel,
    mine_event_dir: PathBuf,
    cache: ProgramCache,
    prevent_flooding: Arc<Mutex<PreventFlooding>>,
    context: GenomeMutateContext,
    genome: Genome,
    rng: StdRng,
    metric: MetricsRunMinerLoop,
    current_program_id: u64,
    current_genome_vec: Vec<GenomeItem>,
    current_message_vec: Vec<String>,
    iteration: usize,
    reload: bool,
    term_computer: TermComputer,
    terms_to_program_id: Arc<TermsToProgramIdSet>,
    suppress_low_priority_programs: bool,
}

impl RunMinerLoop {
    pub fn new(
        config: &Config,
        prevent_flooding: Arc<Mutex<PreventFlooding>>,
        initial_random_seed: u64,
    ) -> Self {
        let rng: StdRng = StdRng::seed_from_u64(initial_random_seed);

        let mine_event_dir: PathBuf = config.mine_event_dir();

        let suppress_low_priority_programs: bool = match config.miner_filter_mode() {
            MinerFilterMode::All => false,
            MinerFilterMode::New => true
        };
    
        let capacity = NonZeroUsize::new(MINER_CACHE_CAPACITY).unwrap();
        Self {
            metrics_callback: None,
            funnel: Funnel::create_empty_funnel(),
            mine_event_dir: PathBuf::from(mine_event_dir),
            cache: ProgramCache::with_capacity(capacity),
            prevent_flooding: prevent_flooding,
            context: GenomeMutateContext::new_empty(),
            genome: Genome::new(),
            rng: rng,
            metric: MetricsRunMinerLoop::new(),
            current_program_id: 0,
            current_genome_vec: vec!(),
            current_message_vec: vec!(),
            iteration: 0,
            reload: true,
            term_computer: TermComputer::new(),
            terms_to_program_id: Arc::new(TermsToProgramIdSet::new()),
            suppress_low_priority_programs: suppress_low_priority_programs,
        }
    }

    pub fn execute_batch(&mut self, dependency_manager: &mut DependencyManager) -> anyhow::Result<ExecuteBatchResult> {
        let start = Instant::now();
        let mut progress_time: Instant = start;
        let mut execute_batch_result = ExecuteBatchResult::new();
        loop {
            self.execute_one_iteration(dependency_manager, &mut execute_batch_result);
            let elapsed: u128 = progress_time.elapsed().as_millis();
            if elapsed < INTERVAL_UNTIL_NEXT_METRIC_SYNC {
                continue;
            }
            self.submit_metrics();
            self.submit_metrics_for_dependency_manager(dependency_manager);
            progress_time = Instant::now();
            let elapsed_since_start: u128 = start.elapsed().as_millis();
            if elapsed_since_start < EXECUTE_BATCH_TIME_LIMIT {
                continue;
            }
            return Ok(execute_batch_result);
        }
    }

    pub fn set_metrics_callback(&mut self, c: impl Fn(MetricEvent) + Send + 'static) {
        self.metrics_callback = Some(Box::new(c));
    }

    fn submit_metric_event(&mut self, metric_event: MetricEvent) {
        match &self.metrics_callback {
            Some(callback) => {
                callback(metric_event);
            },
            None => {}
        }
    }

    fn submit_metrics(&mut self) {
        self.submit_metric_event(MetricEvent::Funnel { 
            terms10: self.funnel.metric_number_of_candidates_with_10terms(),
            terms20: self.funnel.metric_number_of_candidates_with_20terms(),
            terms30: self.funnel.metric_number_of_candidates_with_30terms(),
            terms40: self.funnel.metric_number_of_candidates_with_40terms(),
            false_positives: self.metric.number_of_bloomfilter_false_positive,
        });
        self.submit_metric_event(MetricEvent::Genome { 
            cannot_load: self.metric.number_of_failed_genome_loads,
            cannot_parse: self.metric.number_of_programs_that_cannot_parse,
            too_short: self.metric.number_of_too_short_programs,
            no_output: self.metric.number_of_programs_without_output,
            no_mutation: self.metric.number_of_failed_mutations,
            compute_error: self.metric.number_of_compute_errors,
        });
        self.submit_metric_event(MetricEvent::Cache { 
            hit: self.cache.metric_hit(),
            miss_program_oeis: self.cache.metric_miss_for_program_oeis(),
            miss_program_without_id: self.cache.metric_miss_for_program_without_id(),
        });
        self.submit_metric_event(MetricEvent::General { 
            number_of_iterations: self.metric.number_of_iterations,
            prevent_flooding: self.metric.number_of_prevented_floodings,
            reject_self_dependency: self.metric.number_of_self_dependencies,
            candidate_program: self.metric.number_of_candidate_programs,
        });
        self.funnel.reset_metrics();
        self.cache.reset_metrics();
        self.metric.reset_metrics();
    }

    fn submit_metrics_for_dependency_manager(&mut self, dependency_manager: &mut DependencyManager) {
        self.submit_metric_event(MetricEvent::DependencyManager {
            read_success: dependency_manager.metric_read_success(),
            read_error: dependency_manager.metric_read_error(),
        });
        dependency_manager.reset_metrics();
    }

    pub fn set_funnel(&mut self, funnel: Funnel) {
        self.funnel = funnel;
    }

    pub fn set_genome_mutate_context(&mut self, genome_mutate_context: GenomeMutateContext) {
        self.context = genome_mutate_context;
    }

    pub fn set_terms_to_program_id(&mut self, terms_to_program_id: Arc<TermsToProgramIdSet>) {
        self.terms_to_program_id = terms_to_program_id;
    }

    pub fn load_initial_genome_program(&mut self, dependency_manager: &mut DependencyManager) -> anyhow::Result<()> {
        for _ in 0..LOAD_INITIAL_GENOME_RETRIES {
            let program_id: u32 = match self.context.choose_initial_genome_program(&mut self.rng) {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("choose_initial_genome_program() returned None, seems like data model is empty"));
                }
            };
            let parsed_program: ParsedProgram = match Genome::load_program_with_id(dependency_manager, program_id as u64) {
                Ok(value) => value,
                Err(error) => {
                    error!("Unable to load program. {:?}", error);
                    continue;
                }
            };
            if parsed_program.instruction_vec.len() < LOAD_INITIAL_GENOME_MINIMUM_PROGRAM_LENGTH {
                continue;
            }
            self.current_program_id = program_id as u64;

            let mut genome_vec: Vec<GenomeItem> = parsed_program.to_genome_item_vec();

            let inline_probability_vec: Vec<(bool,usize)> = vec![
                (false, 95),
                (true, 5),
            ];
            let should_inline_seq: &bool = &inline_probability_vec.choose_weighted(&mut self.rng, |item| item.1).unwrap().0;

            let message_vec: Vec<String>;
            if *should_inline_seq {
                let did_mutate_ok = Genome::mutate_inline_seq(&mut self.rng, dependency_manager, &mut genome_vec);
                let mutate_message: String;
                if did_mutate_ok {
                    mutate_message = "mutate: mutate_inline_seq".to_string();
                } else {
                    mutate_message = "mutate: mutate_inline_seq, no change".to_string();
                }
                message_vec = vec![
                    format!("template {}", program_id),
                    mutate_message
                ];
            } else {
                message_vec = vec![
                    format!("template {}", program_id)
                ];
            }

            self.current_genome_vec = genome_vec;
            self.current_message_vec = message_vec;
            return Ok(());
        }
        return Err(anyhow::anyhow!("Unable to pick among available programs"));
    }

    fn execute_one_iteration(
        &mut self, 
        dependency_manager: &mut DependencyManager, 
        execute_batch_result: &mut ExecuteBatchResult
    ) {
        self.metric.number_of_iterations += 1;
        if (self.iteration % ITERATIONS_BETWEEN_RELOADING_CURRENT_GENOME) == 0 {
            self.reload = true;
        }
        if (self.iteration % ITERATIONS_BETWEEN_PICKING_A_NEW_INITIAL_GENOME) == 0 {
            match self.load_initial_genome_program(dependency_manager) {
                Ok(_) => {},
                Err(error) => {
                    error!("Failed loading initial genome. {:?}", error);
                    panic!("Failed loading initial genome. {:?}", error);
                }
            }
        }
        if self.reload {
            self.genome.set_message_vec(self.current_message_vec.clone());
            self.genome.set_genome_vec(self.current_genome_vec.clone());
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
        let result_parse = dependency_manager.parse_stage2(
            ProgramId::ProgramWithoutId, 
            &genome_parsed_program
        );
        let runner: ProgramRunner = match result_parse {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be parsed. {}", iteration, error);
                self.metric.number_of_programs_that_cannot_parse += 1;
                return;
            }
        };

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
        {
            let prevent_flooding = self.prevent_flooding.lock().unwrap();
            if prevent_flooding.contains(&terms40_original) {
                // debug!("prevented flooding");
                self.metric.number_of_prevented_floodings += 1;
                self.reload = true;
                return;
            }
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
        for program_id in &depends_on_program_ids {
            let program_runner: Rc::<ProgramRunner> = match dependency_manager.load(*program_id as u64) {
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
        let key: String = terms40_wildcard.to_compact_comma_string();
        let corresponding_program_id_set: &HashSet<u32> = match self.terms_to_program_id.get(&key) {
            Some(value) => value,
            None => {
                debug!("Ignoring false-positive in bloomfilter funnel. Could not find the candiate in the oeis stripped file. funnel20_number_of_wildcards: {:?} funnel30_number_of_wildcards: {:?} funnel40_number_of_wildcards: {:?} key: {:?}", funnel20_number_of_wildcards, funnel30_number_of_wildcards, funnel40_number_of_wildcards, key);
                self.metric.number_of_bloomfilter_false_positive += 1;
                self.reload = true;
                return
            }
        };
        let intersection: HashSet<&u32> = depends_on_program_ids.intersection(corresponding_program_id_set).collect();
        if !intersection.is_empty() {
            debug!("Ignoring self-dependency. There is this intersection: {:?}", intersection);
            self.metric.number_of_self_dependencies += 1;
            self.reload = true;
            return
        }
        debug!("Found corresponding program_id's: {:?} funnel20_number_of_wildcards: {:?} funnel30_number_of_wildcards: {:?} funnel40_number_of_wildcards: {:?}", corresponding_program_id_set, funnel20_number_of_wildcards, funnel30_number_of_wildcards, funnel40_number_of_wildcards);

        let steps: &Vec<u64> = &self.term_computer.steps;
        let steps_len: usize = steps.len();
        let performance_classifier = PerformanceClassifier::new(10);
        let mut maybe_a_new_program = false;
        let mut is_existing_program_with_better_performance = false;
        let mut priority = ProgramCandidatePriority::Low;
        for program_id in corresponding_program_id_set {
            if self.context.is_program_id_invalid(*program_id) {
                debug!("Keep. Maybe a new program. The program id {} is contained in 'programs_invalid.csv'", program_id);
                self.genome.append_message(format!("keep: maybe a new program. The program id {} is contained in 'programs_invalid.csv'", program_id));
                maybe_a_new_program = true;
                break;
            }
            let program_runner: Rc::<ProgramRunner> = match dependency_manager.load(*program_id as u64) {
                Ok(value) => value,
                Err(error) => {
                    debug!("Keep. Maybe a new program. Cannot verify, failed to load program id {}, {:?}", program_id, error);
                    self.genome.append_message(format!("keep: maybe a new program. cannot load program {:?} with the same initial terms. error: {:?}", program_id, error));
                    self.genome.append_message(format!("priority: high"));
                    maybe_a_new_program = true;
                    priority = ProgramCandidatePriority::High;
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

        {
            let mut prevent_flooding = self.prevent_flooding.lock().unwrap();
            if prevent_flooding.try_register(&terms40_original).is_err() {
                debug!("already contained in prevent flooding dictionary");
            }
        }

        if self.suppress_low_priority_programs {
            if priority == ProgramCandidatePriority::Low {
                debug!("suppressing low priority program");
                return;
            }
        }

        // Yay, this candidate program seems to be good.
        // It's either an entirely new program.
        // Or it's faster than the existing program.
        // Save a snapshot of this program to `$HOME/.loda-rust/mine-even/`
        let mut serializer = ProgramSerializer::new();
        serializer.append_comment(terms40_original.to_compact_comma_string());
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

        match priority {
            ProgramCandidatePriority::Low => {
                execute_batch_result.increment_number_of_mined_low_prio();
            },
            ProgramCandidatePriority::High => {
                execute_batch_result.increment_number_of_mined_high_prio();
            }
        }

    }
}

#[derive(Debug, PartialEq)]
enum ProgramCandidatePriority {
    Low,
    High,
}
