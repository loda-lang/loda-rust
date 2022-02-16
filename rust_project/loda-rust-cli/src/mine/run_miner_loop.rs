use super::{CheckFixedLengthSequence, Funnel, Genome, GenomeMutateContext, PopularProgramContainer, RecentProgramContainer, save_candidate_program};
use super::{PreventFlooding, prevent_flooding_populate};
use super::HistogramInstructionConstant;
use super::RecordTrigram;
use super::SuggestInstruction;
use super::SuggestSource;
use super::SuggestTarget;
use super::find_asm_files_recursively;
use super::{MinerThreadMessageToCoordinator, KeyMetricU32};
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use loda_rust_core::execute::{EvalError, NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, ProgramSerializer, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::execute::node_binomial::NodeBinomialLimit;
use loda_rust_core::execute::node_power::NodePowerLimit;
use loda_rust_core::util::{BigIntVec, bigintvec_to_string};
use std::time::Instant;
use std::path::{Path, PathBuf};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::sync::mpsc::Sender;
use std::convert::TryFrom;

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

pub fn run_miner_loop(
    tx: Sender<MinerThreadMessageToCoordinator>,
    loda_programs_oeis_dir: &PathBuf, 
    checker10: &CheckFixedLengthSequence, 
    checker20: &CheckFixedLengthSequence,
    checker30: &CheckFixedLengthSequence,
    checker40: &CheckFixedLengthSequence,
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

    let mut metric_number_of_miner_loop_iterations: u32 = 0;
    let mut metric_number_of_prevented_floodings: u32 = 0;
    let mut metric_number_of_failed_genome_loads: u32 = 0;
    let mut metric_number_of_failed_mutations: u32 = 0;
    let mut metric_number_of_programs_that_cannot_parse: u32 = 0;
    let mut metric_number_of_programs_without_output: u32 = 0;
    let mut metric_number_of_programs_that_cannot_run: u32 = 0;

    let mut progress_time = Instant::now();
    let mut iteration: usize = 0;
    let mut reload: bool = true;
    loop {
        metric_number_of_miner_loop_iterations += 1;

        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed >= 1000 {
            {
                let y: u32 = metric_number_of_miner_loop_iterations;
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::NumberOfMinerLoopIterations, y);
                tx.send(message).unwrap();
            }
            {
                let x: u64 = funnel.metric_number_of_candidates_with_basiccheck();
                let y: u32 = u32::try_from(x).unwrap_or(0);
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::Funnel10TermsPassingBasicCheck, y);
                tx.send(message).unwrap();
            }
            {
                let x: u64 = funnel.metric_number_of_candidates_with_10terms();
                let y: u32 = u32::try_from(x).unwrap_or(0);
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::Funnel10TermsInBloomfilter, y);
                tx.send(message).unwrap();
            }
            {
                let x: u64 = funnel.metric_number_of_candidates_with_20terms();
                let y: u32 = u32::try_from(x).unwrap_or(0);
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::Funnel20TermsInBloomfilter, y);
                tx.send(message).unwrap();
            }
            {
                let x: u64 = funnel.metric_number_of_candidates_with_30terms();
                let y: u32 = u32::try_from(x).unwrap_or(0);
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::Funnel30TermsInBloomfilter, y);
                tx.send(message).unwrap();
            }
            {
                let x: u64 = funnel.metric_number_of_candidates_with_40terms();
                let y: u32 = u32::try_from(x).unwrap_or(0);
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::Funnel40TermsInBloomfilter, y);
                tx.send(message).unwrap();
            }
            {
                let y: u32 = metric_number_of_prevented_floodings;
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::PreventedFlooding, y);
                tx.send(message).unwrap();
            }
            {
                let y: u32 = metric_number_of_failed_mutations;
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::NumberOfFailedMutations, y);
                tx.send(message).unwrap();
            }
            {
                let y: u32 = metric_number_of_programs_that_cannot_parse;
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::NumberOfProgramsThatCannotParse, y);
                tx.send(message).unwrap();
            }
            {
                let y: u32 = metric_number_of_programs_without_output;
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::NumberOfProgramsWithoutOutput, y);
                tx.send(message).unwrap();
            }
            {
                let y: u32 = metric_number_of_programs_that_cannot_run;
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::NumberOfProgramsThatCannotRun, y);
                tx.send(message).unwrap();
            }
            {
                let y: u32 = metric_number_of_failed_genome_loads;
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::NumberOfFailedGenomeLoads, y);
                tx.send(message).unwrap();
            }
            {
                let x: u64 = cache.metric_hit();
                let y: u32 = u32::try_from(x).unwrap_or(0);
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::CacheHit, y);
                tx.send(message).unwrap();
            }
            {
                let x: u64 = cache.metric_miss_for_program_oeis();
                let y: u32 = u32::try_from(x).unwrap_or(0);
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::CacheMissForProgramOeis, y);
                tx.send(message).unwrap();
            }
            {
                let x: u64 = cache.metric_miss_for_program_without_id();
                let y: u32 = u32::try_from(x).unwrap_or(0);
                let message = MinerThreadMessageToCoordinator::MetricU32(KeyMetricU32::CacheMissForProgramWithoutId, y);
                tx.send(message).unwrap();
            }

            funnel.reset_metrics();
            cache.reset_metrics();
            metric_number_of_miner_loop_iterations = 0;
            metric_number_of_prevented_floodings = 0;
            metric_number_of_failed_mutations = 0;
            metric_number_of_programs_that_cannot_parse = 0;
            metric_number_of_programs_without_output = 0;
            metric_number_of_programs_that_cannot_run = 0;
            metric_number_of_failed_genome_loads = 0;

            progress_time = Instant::now();
        }

        if (iteration % 10) == 0 {
            reload = true;
        }
        if reload {
            genome.clear_message_vec();
            let load_ok: bool = genome.load_random_program(&mut rng, &dm, &context);
            if !load_ok {
                metric_number_of_failed_genome_loads += 1;
                continue;
            }
            reload = false;
        }

        iteration += 1;
        
        if !genome.mutate(&mut rng, &context) {
            metric_number_of_failed_mutations += 1;
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
                metric_number_of_programs_that_cannot_parse += 1;
                continue;
            }
        };

        // If the program has no live output register, then pick the lowest live register.
        if !runner.mining_trick_attempt_fixing_the_output_register() {
            metric_number_of_programs_without_output += 1;
            continue;
        }

        // Execute program
        let mut term_computer = TermComputer::create();
        let terms10: BigIntVec = match term_computer.compute(&mut cache, &runner, 10) {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                metric_number_of_programs_that_cannot_run += 1;
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
                metric_number_of_programs_that_cannot_run += 1;
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
                metric_number_of_programs_that_cannot_run += 1;
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
                metric_number_of_programs_that_cannot_run += 1;
                continue;
            }
        };
        if !funnel.check40(&terms40) {
            continue;
        }

        if prevent_flooding.try_register(&terms40).is_err() {
            // debug!("prevented flooding");
            metric_number_of_prevented_floodings += 1;
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
        }
    }
}
