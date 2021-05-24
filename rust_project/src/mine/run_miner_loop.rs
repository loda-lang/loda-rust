use crate::control::DependencyManager;
use crate::mine::{CheckFixedLengthSequence, Funnel, Genome, GenomeMutateContext, PopularProgramContainer, PreventFlooding, RecentProgramContainer, save_candidate_program};
use crate::parser::{parse_program, ParsedProgram};
use crate::execute::{EvalError, NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, ProgramSerializer, RegisterValue, RunMode};
use crate::execute::node_binomial::NodeBinomialLimit;
use crate::execute::node_power::NodePowerLimit;
use crate::util::{BigIntVec, bigintvec_to_string};
use std::fs;
use std::time::Instant;
use std::path::{Path, PathBuf};
use rand::SeedableRng;
use rand::rngs::StdRng;

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

impl ProgramRunner {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) -> Result<BigIntVec, EvalError> {
        let mut terms: BigIntVec = vec!();
        let step_count_limit: u64 = 10000;
        let node_binomial_limit = NodeBinomialLimit::LimitN(20);
        let node_loop_limit = NodeLoopLimit::LimitCount(1000);
        let node_power_limit = NodePowerLimit::LimitBits(30);
        let mut _step_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = self.run(
                &input, 
                RunMode::Silent, 
                &mut _step_count, 
                step_count_limit, 
                node_binomial_limit.clone(),
                node_loop_limit.clone(),
                node_power_limit.clone(),
                cache
            )?;
            terms.push(output.0.clone());
            if index == 0 {
                // print!("{}", output.0);
                continue;
            }
            // print!(",{}", output.0);
        }
        // print!("\n");
        // print!("stats: step_count: {}", step_count);
        Ok(terms)
    }
}

fn asm_files_in_the_mine_event_dir(mine_event_dir: &Path) -> Vec<PathBuf> {
    let readdir_iterator: fs::ReadDir = match fs::read_dir(mine_event_dir) {
        Ok(values) => values,
        Err(err) => {
            panic!("Unable to obtain paths for mine_event_dir. error: {:?}", err);
        }
    };

    let mut paths: Vec<PathBuf> = vec!();
    for path in readdir_iterator {
        let direntry: fs::DirEntry = match path {
            Ok(value) => value,
            Err(_) => {
                continue;
            }
        };
        let path: PathBuf = direntry.path();
        let extension = match path.extension() {
            Some(value) => value,
            None => {
                continue;
            }
        };
        if extension != "asm" {
            continue;
        }
        if !path.is_file() {
            continue;
        }
        paths.push(path);
    }
    paths
}

impl PreventFlooding {
    fn load(&mut self, dependency_manager: &mut DependencyManager, cache: &mut ProgramCache, paths: Vec<PathBuf>) {
        let mut number_of_read_errors: usize = 0;
        let mut number_of_parse_errors: usize = 0;
        let mut number_of_runtime_errors: usize = 0;
        let mut number_of_already_registered_programs: usize = 0;
        let mut number_of_successfully_registered_programs: usize = 0;
        for path in paths {
            let contents: String = match fs::read_to_string(&path) {
                Ok(value) => value,
                Err(error) => {
                    debug!("Something went wrong reading the file: {:?}  error: {:?}", path, error);
                    number_of_read_errors += 1;
                    continue;
                }
            };
            let runner: ProgramRunner = match dependency_manager.parse(ProgramId::ProgramWithoutId, &contents) {
                Ok(value) => value,
                Err(error) => {
                    debug!("Something went wrong when parsing the file: {:?}  error: {:?}", path, error);
                    number_of_parse_errors += 1;
                    continue;
                }
            };
            let number_of_terms: u64 = 40;
            let terms: BigIntVec = match runner.compute_terms(number_of_terms, cache) {
                Ok(value) => value,
                Err(error) => {
                    debug!("program cannot be run. path: {:?}  error: {:?}", path, error);
                    number_of_runtime_errors += 1;
                    continue;
                }
            };
            if self.try_register(&terms).is_err() {
                number_of_already_registered_programs += 1;
                continue;
            }
            number_of_successfully_registered_programs += 1;
        }
        let junk_count: usize = number_of_read_errors + number_of_parse_errors + number_of_runtime_errors + number_of_already_registered_programs;
        debug!("prevent flooding. Registered {} programs. Ignoring {} junk programs.", number_of_successfully_registered_programs, junk_count);
    }
}


pub fn run_miner_loop(
    loda_program_rootdir: &PathBuf, 
    checker10: &CheckFixedLengthSequence, 
    checker20: &CheckFixedLengthSequence,
    checker30: &CheckFixedLengthSequence,
    checker40: &CheckFixedLengthSequence,
    mine_event_dir: &Path,
    available_program_ids: Vec<u32>,
    initial_random_seed: u64,
    popular_program_container: PopularProgramContainer,
    recent_program_container: RecentProgramContainer,
) {
    let mut rng = StdRng::seed_from_u64(initial_random_seed);

    let mut dm = DependencyManager::new(
        loda_program_rootdir.clone(),
    );
    let mut cache = ProgramCache::new();

    let paths: Vec<PathBuf> = asm_files_in_the_mine_event_dir(mine_event_dir);
    println!("number of .asm files in the mine-event dir: {:?}", paths.len());

    let mut prevent_flooding = PreventFlooding::new();
    prevent_flooding.load(&mut dm, &mut cache, paths);
    println!("number of programs added to the PreventFlooding mechanism: {}", prevent_flooding.len());

    let path_to_program: PathBuf = dm.path_to_program(112456);
    let contents: String = match fs::read_to_string(&path_to_program) {
        Ok(value) => value,
        Err(error) => {
            panic!("Something went wrong reading the file: {:?}", error);
        }
    };

    let _parsed_program: ParsedProgram = match parse_program(&contents) {
        Ok(value) => value,
        Err(error) => {
            panic!("Something went wrong parsing the program: {:?}", error);
        }
    };
    
    // let mut genome = Genome::new_from_parsed_program(&parsed_program);
    let mut genome = Genome::new();
    // genome.mutate_insert_loop(&mut rng);
    // debug!("Initial genome\n{}", genome);
    println!("Initial genome\n{}", genome);

    // return;

    let context = GenomeMutateContext::new(
        available_program_ids,
        popular_program_container,
        recent_program_container,
    );

    let mut funnel = Funnel::new(
        checker10,
        checker20,
        checker30,
        checker40,
    );

    println!("\nPress CTRL-C to stop the miner.");
    let mut iteration: usize = 0;
    let mut progress_time = Instant::now();
    let mut progress_iteration: usize = 0;
    let mut number_of_failed_mutations: usize = 0;
    let mut number_of_errors_parse: usize = 0;
    let mut number_of_errors_nooutput: usize = 0;
    let mut number_of_errors_run: usize = 0;
    let mut number_of_prevented_floodings: usize = 0;
    loop {
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed >= 1000 {
            let iterations_diff: usize = iteration - progress_iteration;
            let iterations_per_second: f32 = ((1000 * iterations_diff) as f32) / (elapsed as f32);
            let iteration_info = format!(
                "{:.0} iter/sec", iterations_per_second
            );

            let error_info = format!(
                "[{},{},{},{}]",
                number_of_failed_mutations,
                number_of_errors_parse,
                number_of_errors_nooutput,
                number_of_errors_run
            );

            println!("#{} cache: {}   error: {}   funnel: {}  flooding: {}  {}", 
                iteration, 
                cache.hit_miss_info(), 
                error_info,
                funnel.funnel_info(),
                number_of_prevented_floodings,
                iteration_info
            );

            // println!("Current genome\n{}", genome);

            progress_time = Instant::now();
            progress_iteration = iteration;
        }

        iteration += 1;
        
        if !genome.mutate(&mut rng, &context) {
            number_of_failed_mutations += 1;
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
                number_of_errors_parse += 1;
                continue;
            }
        };

        // If the program has no live output register, then pick the lowest live register.
        if !runner.mining_trick_attempt_fixing_the_output_register() {
            number_of_errors_nooutput += 1;
            continue;
        }

        // Execute program
        let mut term_computer = TermComputer::create();
        let terms10: BigIntVec = match term_computer.compute(&mut cache, &runner, 10) {
            Ok(value) => value,
            Err(_error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                number_of_errors_run += 1;
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
                number_of_errors_run += 1;
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
                number_of_errors_run += 1;
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
                number_of_errors_run += 1;
                continue;
            }
        };
        if !funnel.check40(&terms40) {
            continue;
        }

        if prevent_flooding.try_register(&terms40).is_err() {
            // debug!("prevented flooding");
            number_of_prevented_floodings += 1;
            continue;
        }

        // Yay, this candidate program has 40 terms that are good.
        // Save a snapshot of this program to `$HOME/.loda-lab/mine-even/`
        let mut serializer = ProgramSerializer::new();
        serializer.append(format!("; {}", bigintvec_to_string(&terms40)));
        serializer.append("");
        runner.serialize(&mut serializer);
        let candidate_program: String = serializer.to_string();

        if let Err(error) = save_candidate_program(mine_event_dir, iteration, &candidate_program) {
            println!("; GENOME\n{}", genome);
            error!("Unable to save candidate program: {:?}", error);
        }
    }
}
