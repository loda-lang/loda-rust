use crate::common::find_asm_files_recursively;
use crate::common::RecordTrigram;
use super::{CheckFixedLengthSequence, Funnel, Genome, GenomeMutateContext, PopularProgramContainer, RecentProgramContainer, save_candidate_program};
use super::{PreventFlooding, prevent_flooding_populate};
use super::HistogramInstructionConstant;
use super::SuggestInstruction;
use super::SuggestSource;
use super::SuggestTarget;
use super::{MinerThreadMessageToCoordinator, MetricEvent, Recorder};
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use loda_rust_core::execute::{EvalError, NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, ProgramSerializer, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::node_binomial::NodeBinomialLimit;
use loda_rust_core::execute::node_power::NodePowerLimit;
use loda_rust_core::util::{BigIntVec, bigintvec_to_string};
use loda_rust_core::parser::ParsedProgram;
use std::time::Instant;
use std::path::{Path, PathBuf};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::sync::mpsc::Sender;

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

struct LoopMetrics {
    number_of_miner_loop_iterations: u64,
    number_of_prevented_floodings: u64,
    number_of_failed_genome_loads: u64,
    number_of_failed_mutations: u64,
    number_of_programs_that_cannot_parse: u64,
    number_of_programs_without_output: u64,
    number_of_compute_errors: u64,
    number_of_candidate_programs: u64,
}

impl LoopMetrics {
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

pub fn run_miner_loop(
    tx: Sender<MinerThreadMessageToCoordinator>,
    recorder: Box<dyn Recorder>,
    loda_programs_oeis_dir: &PathBuf, 
    checker10: CheckFixedLengthSequence, 
    checker20: CheckFixedLengthSequence,
    checker30: CheckFixedLengthSequence,
    checker40: CheckFixedLengthSequence,
    histogram_instruction_constant: Option<HistogramInstructionConstant>,
    mine_event_dir: &Path,
    loda_rust_mismatches: &Path,
    instruction_trigram_csv: &Path,
    source_trigram_csv: &Path,
    target_trigram_csv: &Path,
    available_program_ids: Vec<u32>,
    initial_random_seed: u64,
    popular_program_container: PopularProgramContainer,
    recent_program_container: RecentProgramContainer,
) {
    let mut rng = StdRng::seed_from_u64(initial_random_seed);

    let mut dm = DependencyManager::new(
        DependencyManagerFileSystemMode::System,
        loda_programs_oeis_dir.clone(),
    );

    let instruction_trigram_vec: Vec<RecordTrigram> = RecordTrigram::parse_csv(instruction_trigram_csv).expect("Unable to load instruction trigram csv");
    let mut suggest_instruction = SuggestInstruction::new();
    suggest_instruction.populate(&instruction_trigram_vec);

    let source_trigram_vec: Vec<RecordTrigram> = RecordTrigram::parse_csv(source_trigram_csv).expect("Unable to load source trigram csv");
    let mut suggest_source = SuggestSource::new();
    suggest_source.populate(&source_trigram_vec);

    let target_trigram_vec: Vec<RecordTrigram> = RecordTrigram::parse_csv(target_trigram_csv).expect("Unable to load target trigram csv");
    let mut suggest_target = SuggestTarget::new();
    suggest_target.populate(&target_trigram_vec);

    let mut cache = ProgramCache::new();

    let mut paths0: Vec<PathBuf> = find_asm_files_recursively(mine_event_dir);
    let mut paths1: Vec<PathBuf> = find_asm_files_recursively(loda_rust_mismatches);
    let mut paths: Vec<PathBuf> = vec!();
    paths.append(&mut paths0);
    paths.append(&mut paths1);
    println!("number of .asm files in total: {:?}", paths.len());

    let mut prevent_flooding = PreventFlooding::new();
    prevent_flooding_populate(&mut prevent_flooding, &mut dm, &mut cache, paths);
    println!("number of programs added to the PreventFlooding mechanism: {}", prevent_flooding.len());

    let mut genome = Genome::new();

    let context = GenomeMutateContext::new(
        available_program_ids,
        popular_program_container,
        recent_program_container,
        histogram_instruction_constant,
        Some(suggest_instruction),
        Some(suggest_source),
        Some(suggest_target)
    );

    let mut funnel = Funnel::new(
        checker10,
        checker20,
        checker30,
        checker40,
    );

    let mut metric = LoopMetrics::new();

    let mut progress_time = Instant::now();
    let mut iteration: usize = 0;
    let mut reload: bool = true;

    let mut current_program_id: u64 = 0;
    let mut current_parsed_program = ParsedProgram::new();

    assert_eq!(context.has_available_programs(), true);
    loop {
        metric.number_of_miner_loop_iterations += 1;

        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed >= INTERVAL_UNTIL_NEXT_METRIC_SYNC {
            {
                let y: u64 = metric.number_of_miner_loop_iterations;
                let message = MinerThreadMessageToCoordinator::NumberOfIterations(y);
                tx.send(message).unwrap();
            }
            {
                let event = MetricEvent::Funnel { 
                    basic: funnel.metric_number_of_candidates_with_basiccheck(),
                    terms10: funnel.metric_number_of_candidates_with_10terms(),
                    terms20: funnel.metric_number_of_candidates_with_20terms(),
                    terms30: funnel.metric_number_of_candidates_with_30terms(),
                    terms40: funnel.metric_number_of_candidates_with_40terms(),
                };
                recorder.record(&event);
            }
            {
                let event = MetricEvent::Genome { 
                    cannot_load: metric.number_of_failed_genome_loads,
                    cannot_parse: metric.number_of_programs_that_cannot_parse,
                    no_output: metric.number_of_programs_without_output,
                    no_mutation: metric.number_of_failed_mutations,
                    compute_error: metric.number_of_compute_errors,
                };
                recorder.record(&event);
            }
            {
                let event = MetricEvent::Cache { 
                    hit: cache.metric_hit(),
                    miss_program_oeis: cache.metric_miss_for_program_oeis(),
                    miss_program_without_id: cache.metric_miss_for_program_without_id(),
                };
                recorder.record(&event);
            }
            {
                let event = MetricEvent::General { 
                    prevent_flooding: metric.number_of_prevented_floodings,
                    candidate_program: metric.number_of_candidate_programs,
                };
                recorder.record(&event);
            }

            funnel.reset_metrics();
            cache.reset_metrics();
            metric.reset_metrics();

            progress_time = Instant::now();
        }

        if (iteration % 10) == 0 {
            reload = true;
        }
        if (iteration % 50000) == 0 {
            match context.choose_available_program(&mut rng) {
                Some(program_id) => { 
                    current_program_id = program_id as u64;
                },
                None => {
                    panic!("Unable to pick among available programs");
                }
            };
            let parsed_program: ParsedProgram = match genome.load_program(&dm, current_program_id) {
                Some(value) => value,
                None => {
                    error!("Unable to parse available program");
                    reload = true;
                    continue;
                }
            };
            current_parsed_program = parsed_program;
        }
        if reload {
            genome.clear_message_vec();
            let load_ok: bool = genome.insert_program(current_program_id, &current_parsed_program);
            if !load_ok {
                metric.number_of_failed_genome_loads += 1;
                continue;
            }
            reload = false;
        }

        iteration += 1;
        
        if !genome.mutate(&mut rng, &context) {
            metric.number_of_failed_mutations += 1;
            continue;
        }

        // println!("#{} Current genome\n{}", iteration, genome);
    
        // Create program from genome
        dm.reset();
        let result_parse = dm.parse_stage2(
            ProgramId::ProgramWithoutId, 
            &genome.to_parsed_program()
        );
        let mut runner: ProgramRunner = match result_parse {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be parsed. {}", iteration, error);
                metric.number_of_programs_that_cannot_parse += 1;
                continue;
            }
        };

        // If the program has no live output register, then pick the lowest live register.
        if !runner.mining_trick_attempt_fixing_the_output_register() {
            metric.number_of_programs_without_output += 1;
            continue;
        }

        // Execute program
        let mut term_computer = TermComputer::create();
        let terms10: BigIntVec = match term_computer.compute(&mut cache, &runner, 10) {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                metric.number_of_compute_errors += 1;
                continue;
            }
        };
        // println!("terms10: {:?}", terms10);
        if !funnel.check_basic(&terms10) {
            continue;
        }
        if !funnel.check10(&terms10) {
            continue;
        }

        let terms20: BigIntVec = match term_computer.compute(&mut cache, &runner, 20) {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                metric.number_of_compute_errors += 1;
                continue;
            }
        };
        if !funnel.check20(&terms20) {
            continue;
        }

        let terms30: BigIntVec = match term_computer.compute(&mut cache, &runner, 30) {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                metric.number_of_compute_errors += 1;
                continue;
            }
        };
        if !funnel.check30(&terms30) {
            continue;
        }

        let terms40: BigIntVec = match term_computer.compute(&mut cache, &runner, 40) {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                metric.number_of_compute_errors += 1;
                continue;
            }
        };
        if !funnel.check40(&terms40) {
            continue;
        }

        if prevent_flooding.try_register(&terms40).is_err() {
            // debug!("prevented flooding");
            metric.number_of_prevented_floodings += 1;
            reload = true;
            continue;
        }

        // Yay, this candidate program has 40 terms that are good.
        // Save a snapshot of this program to `$HOME/.loda-rust/mine-even/`
        let mut serializer = ProgramSerializer::new();
        serializer.append_comment(bigintvec_to_string(&terms40));
        serializer.append_empty_line();
        runner.serialize(&mut serializer);
        serializer.append_empty_line();
        for message in genome.message_vec() {
            serializer.append_comment(message);
        }
        serializer.append_empty_line();
        let candidate_program: String = serializer.to_string();

        if let Err(error) = save_candidate_program(mine_event_dir, iteration, &candidate_program) {
            println!("; GENOME\n{}", genome);
            error!("Unable to save candidate program: {:?}", error);
            continue;
        }
        metric.number_of_candidate_programs += 1;
    }
}
