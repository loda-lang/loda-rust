use super::DependencyManager;
use crate::config::Config;
use crate::mine::{CheckFixedLengthSequence, Funnel, GenomeItem, load_program_ids_csv_file, MutateValue};
use crate::parser::{Instruction, InstructionId, InstructionParameter, ParameterType, parse_program, ParseProgramError, ParsedProgram};
use crate::execute::{EvalError, ProgramCache, ProgramId, ProgramRunner, ProgramSerializer, RegisterValue, RunMode};
use crate::util::{BigIntVec, bigintvec_to_string};
use std::fs;
use std::fmt;
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use rand::{Rng,RngCore,SeedableRng};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use chrono::{DateTime, Utc};

pub fn subcommand_mine() {

    // Print info about start conditions
    let build_mode: &str;
    if cfg!(debug_assertions) {
        error!("Debugging enabled. Wasting cpu cycles. Not good for mining!");
        build_mode = "'DEBUG'  # Terrible inefficient for mining!";
    } else {
        build_mode = "'RELEASE'  # Good";
    }
    println!("[mining info]");
    println!("build_mode = {}", build_mode);

    // Load config file
    let config = Config::load();
    let loda_program_rootdir: PathBuf = config.loda_program_rootdir();
    let cache_dir: PathBuf = config.cache_dir();
    let mine_event_dir: PathBuf = config.mine_event_dir();
    let loda_lab_repository: PathBuf = config.loda_lab_repository();

    // Load cached data
    debug!("step1");
    let file10 = cache_dir.join(Path::new("fixed_length_sequence_10terms.json"));
    let checker10: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&file10);
    let file20 = cache_dir.join(Path::new("fixed_length_sequence_20terms.json"));
    let checker20: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&file20);
    let file30 = cache_dir.join(Path::new("fixed_length_sequence_30terms.json"));
    let checker30: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&file30);
    let file40 = cache_dir.join(Path::new("fixed_length_sequence_40terms.json"));
    let checker40: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&file40);
    debug!("step2");

    // Load the program_ids to cycle through
    let available_program_ids_file = loda_lab_repository.join(Path::new("resources/mine_program_ids.csv"));
    let available_program_ids: Vec<u32> = match load_program_ids_csv_file(&available_program_ids_file) {
        Ok(value) => value,
        Err(error) => {
            panic!("Unable to load file. path: {:?} error: {:?}", available_program_ids_file, error);
        }
    };
    println!("number_of_available_programs = {}", available_program_ids.len());

    // Pick a random seed
    let mut rng = thread_rng();
    let initial_random_seed: u64 = rng.next_u64();
    println!("random_seed = {}", initial_random_seed);

    // Launch the miner
    run_experiment0(
        &loda_program_rootdir, 
        &checker10, 
        &checker20,
        &checker30,
        &checker40,
        &mine_event_dir,
        &available_program_ids,
        initial_random_seed,
    );
}


// Ideas for more mutations
// append random row
enum MutateGenome {
    Instruction,
    SourceConstant,
    SourceType,
    SwapRegisters,
    SourceRegister,
    TargetRegister,
    ToggleEnabled,
    SwapRows,
    SwapAdjacentRows,
    InsertLoopBeginEnd,
}

struct Genome {
    genome_vec: Vec<GenomeItem>
}

impl Genome {
    fn new_from_parsed_program(parsed_program: &ParsedProgram) -> Self {
        let mut genome_vec: Vec<GenomeItem> = vec!();

        for instruction in &parsed_program.instruction_vec {

            let mut target_parameter_value: i32 = 0;
            let mut source_parameter_type: ParameterType = ParameterType::Constant;
            let mut source_parameter_value: i32 = 0;
            for (index, parameter) in instruction.parameter_vec.iter().enumerate() {
                if index == 0 {
                    target_parameter_value = parameter.parameter_value as i32;
                }
                if index == 1 {
                    source_parameter_value = parameter.parameter_value as i32;
                    source_parameter_type = parameter.parameter_type.clone();
                }
            }
        
            let genome_item = GenomeItem::new(
                instruction.instruction_id.clone(),
                target_parameter_value,
                source_parameter_type,
                source_parameter_value,
            );
            genome_vec.push(genome_item);
        }

        Self {
            genome_vec: genome_vec,
        }
    }

    fn new() -> Self {
        let mut genome_vec: Vec<GenomeItem> = vec!();
        {
            let item = GenomeItem::new_move_register(1, 0);
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_move_register(2, 0);
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_move_register(3, 0);
            genome_vec.push(item);
        }
        // append instructions that doesn't do anything to the output register
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::Add, 1, 1);
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::Subtract, 1, 1);
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::Multiply, 1, 10);
            genome_vec.push(item);
        }
        // {
        //     let item = GenomeItem::new_instruction_with_const(InstructionId::Divide, 1, 2);
        //     genome_vec.push(item);
        // }
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::Multiply, 2, 10);
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::DivideIf, 1, 10);
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::Modulo, 2, 10);
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new(
                InstructionId::Call,
                1,
                ParameterType::Constant,
                80578,
            );
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::Modulo, 3, 2);
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::Add, 1, 1);
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::Subtract, 1, 1);
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::Multiply, 1, 6);
            genome_vec.push(item);
        }
        // for _ in 0..4 {
        //     {
        //         let item = GenomeItem::new_instruction_with_const(InstructionId::Add, 1, 1);
        //         genome_vec.push(item);
        //     }
        //     {
        //         let item = GenomeItem::new_instruction_with_const(InstructionId::Subtract, 1, 1);
        //         genome_vec.push(item);
        //     }
        // }
        // genome_vec[2].mutate_trigger_division_by_zero();
        Self {
            genome_vec: genome_vec,
        }
    }

    fn to_parsed_program(&self) -> ParsedProgram {
        let mut instruction_vec: Vec<Instruction> = vec!();

        let mut line_number: usize = 0;
        for genome_item in self.genome_vec.iter() {
            if !genome_item.is_enabled() {
                continue;
            }

            let instruction_id: InstructionId = 
                genome_item.instruction_id().clone();
    
            let parameter_vec: Vec<InstructionParameter> = 
                genome_item.to_parameter_vec();
    
            let instruction = Instruction {
                instruction_id: instruction_id,
                parameter_vec: parameter_vec,
                line_number: line_number,
            };
            instruction_vec.push(instruction);
            line_number += 1;
        }

        ParsedProgram {
            instruction_vec: instruction_vec
        }
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as no instructions that use a constant, underflow, overflow.
    fn mutate_source_value_constant<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use constants
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.source_type == ParameterType::Constant {
                indexes.push(index);
            }
        }
        if indexes.is_empty() {
            return false;
        }

        // Pick a random mutation
        let mutation_vec: Vec<MutateValue> = vec![
            MutateValue::Increment,
            MutateValue::Decrement,
            MutateValue::Assign(2),
            MutateValue::Assign(6),
            MutateValue::Assign(10),
            // MutateValue::Assign(100),
            // MutateValue::Assign(1000),
        ];
        let mutation: &MutateValue = mutation_vec.choose(rng).unwrap();

        // Mutate one of the instructions that use a constant
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        if !genome_item.mutate_source_value(mutation) {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as no instructions that use a source_type=register, underflow, overflow.
    fn mutate_source_register<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use source_type=register
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.source_type == ParameterType::Register {
                indexes.push(index);
            }
        }
        if indexes.is_empty() {
            return false;
        }

        // Pick a random mutation
        let mutation_vec: Vec<MutateValue> = vec![
            MutateValue::Increment,
            MutateValue::Decrement,
        ];
        let mutation: &MutateValue = mutation_vec.choose(rng).unwrap();

        // Mutate one of the instructions that use a constant
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        if !genome_item.mutate_source_value(mutation) {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_target_register<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        assert!(length > 0);
        let index: usize = rng.gen_range(0..length);

        // Pick a random mutation
        let mutation_vec: Vec<MutateValue> = vec![
            MutateValue::Increment,
            MutateValue::Decrement,
        ];
        let mutation: &MutateValue = mutation_vec.choose(rng).unwrap();

        // Mutate one of the instructions
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];
        if !genome_item.mutate_target_value(mutation) {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_instruction<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        assert!(length > 0);
        let index: usize = rng.gen_range(0..length);
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];

        if !genome_item.mutate_randomize_instruction(rng) {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_source_type<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        assert!(length > 0);
        let index: usize = rng.gen_range(0..length);
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];

        if !genome_item.mutate_source_type() {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_swap_registers<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use two registers
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.source_type == ParameterType::Register {
                indexes.push(index);
            }
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions that use two registers
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        if !genome_item.mutate_swap_source_target_value() {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_enabled<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        assert!(length > 0);
        let index: usize = rng.gen_range(0..length);
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];

        if !genome_item.mutate_enabled() {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_swap_rows<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        if length < 2 {
            return false;
        }
        let index0: usize = rng.gen_range(0..length);
        let index1: usize = rng.gen_range(0..length);
        if index0 == index1 {
            return false;
        }
        let instruction0: &InstructionId = self.genome_vec[index0].instruction_id();
        let instruction1: &InstructionId = self.genome_vec[index1].instruction_id();
        // Prevent messing with loop begin/end instructions.
        let is_loop = 
            *instruction0 == InstructionId::LoopBegin || 
            *instruction0 == InstructionId::LoopEnd ||
            *instruction1 == InstructionId::LoopBegin || 
            *instruction1 == InstructionId::LoopEnd;
        if is_loop {
            return false;
        }
        self.genome_vec.swap(index0, index1);
        true
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_swap_adjacent_rows<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        if length < 3 {
            return false;
        }
        let index0: usize = rng.gen_range(0..length-1);
        let index1: usize = index0 + 1;
        let instruction0: &InstructionId = self.genome_vec[index0].instruction_id();
        let instruction1: &InstructionId = self.genome_vec[index1].instruction_id();
        // Prevent reversing the order of the loop begin/end instructions.
        let is_loop = 
            *instruction0 == InstructionId::LoopBegin && 
            *instruction1 == InstructionId::LoopEnd;
        if is_loop {
            return false;
        }
        self.genome_vec.swap(index0, index1);
        true
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_insert_loop<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        if length < 2 {
            return false;
        }
        let index0: usize = rng.gen_range(0..length);
        let index1: usize = rng.gen_range(0..length);
        if index0 == index1 {
            return false;
        }

        // first insert loop-end
        {
            let index: usize = index0.max(index1);
            let item = GenomeItem::new(
                InstructionId::LoopEnd, 
                0, 
                ParameterType::Constant, 
                0
            );
            self.genome_vec.insert(index, item);
        }

        // last insert loop-begin
        {
            let index: usize = index0.min(index1);
            let item = GenomeItem::new(
                InstructionId::LoopBegin,
                rng.gen_range(0..5) as i32,
                ParameterType::Constant,
                1
            );
            self.genome_vec.insert(index, item);
        }

        true
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure.
    fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mutation_vec: Vec<MutateGenome> = vec![
            MutateGenome::Instruction,
            MutateGenome::SourceConstant,
            MutateGenome::SourceType,
            MutateGenome::SwapRegisters,
            MutateGenome::SourceRegister,
            MutateGenome::TargetRegister,
            MutateGenome::ToggleEnabled,
            // MutateGenome::SwapRows,
            MutateGenome::SwapAdjacentRows,
            // MutateGenome::InsertLoopBeginEnd,
        ];
        let mutation: &MutateGenome = mutation_vec.choose(rng).unwrap();
        match mutation {
            MutateGenome::Instruction => {
                return self.mutate_instruction(rng);
            },
            MutateGenome::SourceConstant => {
                return self.mutate_source_value_constant(rng);
            },
            MutateGenome::SourceType => {
                return self.mutate_source_type(rng);
            },
            MutateGenome::SwapRegisters => {
                return self.mutate_swap_registers(rng);
            },
            MutateGenome::SourceRegister => {
                return self.mutate_source_register(rng);
            },
            MutateGenome::TargetRegister => {
                return self.mutate_target_register(rng);
            },
            MutateGenome::ToggleEnabled => {
                return self.mutate_enabled(rng);
            },
            MutateGenome::SwapRows => {
                return self.mutate_swap_rows(rng);
            },
            MutateGenome::SwapAdjacentRows => {
                return self.mutate_swap_adjacent_rows(rng);
            },
            MutateGenome::InsertLoopBeginEnd => {
                return self.mutate_insert_loop(rng);
            }
        }
    }
}

impl fmt::Display for Genome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rows: Vec<String> = self.genome_vec.iter().map(|genome_item| {
            genome_item.to_string()
        }).collect();
        let joined_rows: String = rows.join("\n");
        write!(f, "{}", joined_rows)
    }
}


impl ProgramRunner {
    fn compute_terms(&self, count: u64, cache: &mut ProgramCache) -> Result<BigIntVec, EvalError> {
        let mut terms: BigIntVec = vec!();
        let step_count_limit: u64 = 10000;
        let mut _step_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = self.run(
                input, 
                RunMode::Silent, 
                &mut _step_count, 
                step_count_limit, 
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

fn save_candidate_program(
    mine_event_dir: &Path,
    iteration: usize,
    content: &String,
) -> std::io::Result<()> 
{
    // Format filename as "19841231-235959-1234.asm"
    let now: DateTime<Utc> = Utc::now();
    let filename: String = format!("{}-{}.asm", now.format("%Y%m%d-%H%M%S"), iteration);

    // Write the file to the output dir
    let path = mine_event_dir.join(Path::new(&filename));
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;

    println!("candidate: {:?}", filename);
    Ok(())
}

fn run_experiment0(
    loda_program_rootdir: &PathBuf, 
    checker10: &CheckFixedLengthSequence, 
    checker20: &CheckFixedLengthSequence,
    checker30: &CheckFixedLengthSequence,
    checker40: &CheckFixedLengthSequence,
    mine_event_dir: &Path,
    available_program_ids: &Vec<u32>,
    initial_random_seed: u64,
) {
    let mut rng = StdRng::seed_from_u64(initial_random_seed);

    let mut dm = DependencyManager::new(
        loda_program_rootdir.clone(),
    );

    let path_to_program: PathBuf = dm.path_to_program(112456);
    let contents: String = match fs::read_to_string(&path_to_program) {
        Ok(value) => value,
        Err(error) => {
            panic!("Something went wrong reading the file: {:?}", error);
        }
    };

    let parsed_program: ParsedProgram = match parse_program(&contents) {
        Ok(value) => value,
        Err(error) => {
            panic!("Something went wrong parsing the program: {:?}", error);
        }
    };
    
    let mut genome = Genome::new_from_parsed_program(&parsed_program);
    // let mut genome = Genome::new();
    // genome.mutate_insert_loop(&mut rng);
    // debug!("Initial genome\n{}", genome);
    println!("Initial genome\n{}", genome);

    // return;

    let mut funnel = Funnel::new(
        checker10,
        checker20,
        checker30,
        checker40,
    );

    println!("\nPress CTRL-C to stop the miner.");
    let mut cache = ProgramCache::new();
    let mut iteration: usize = 0;
    let mut progress_time = Instant::now();
    let mut progress_iteration: usize = 0;
    let mut number_of_errors_parse: usize = 0;
    let mut number_of_errors_nooutput: usize = 0;
    let mut number_of_errors_run: usize = 0;
    loop {
        if (iteration % 10000) == 0 {
            let elapsed: u128 = progress_time.elapsed().as_millis();
            if elapsed >= 5000 {
                let iterations_diff: usize = iteration - progress_iteration;
                let iterations_per_second: f32 = ((1000 * iterations_diff) as f32) / (elapsed as f32);
                let iteration_info = format!(
                    "{:.0} iter/sec", iterations_per_second
                );

                let error_info = format!(
                    "[{},{},{}]",
                    number_of_errors_parse,
                    number_of_errors_nooutput,
                    number_of_errors_run
                );

                println!("#{} cache: {}   error: {}   funnel: {}   {}", 
                    iteration, 
                    cache.hit_miss_info(), 
                    error_info,
                    funnel.funnel_info(), 
                    iteration_info
                );

                progress_time = Instant::now();
                progress_iteration = iteration;
            }
        }
        iteration += 1;
        
        for _ in 0..5 {
            genome.mutate(&mut rng);
        }
    
        // Create program from genome
        dm.reset();
        let result_parse = dm.parse_stage2(
            ProgramId::ProgramWithoutId, 
            &genome.to_parsed_program()
        );
        let mut runner: ProgramRunner = match result_parse {
            Ok(value) => value,
            Err(error) => {
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
        let number_of_terms: u64 = 10;
        let terms10: BigIntVec = match runner.compute_terms(number_of_terms, &mut cache) {
            Ok(value) => value,
            Err(error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                number_of_errors_run += 1;
                continue;
            }
        };
        if !funnel.check_basic(&terms10) {
            continue;
        }
        if !funnel.check10(&terms10) {
            continue;
        }

        let terms20: BigIntVec = match runner.compute_terms(20, &mut cache) {
            Ok(value) => value,
            Err(error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                number_of_errors_run += 1;
                continue;
            }
        };
        if !funnel.check20(&terms20) {
            continue;
        }

        let terms30: BigIntVec = match runner.compute_terms(30, &mut cache) {
            Ok(value) => value,
            Err(error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                number_of_errors_run += 1;
                continue;
            }
        };
        if !funnel.check30(&terms30) {
            continue;
        }

        let terms40: BigIntVec = match runner.compute_terms(40, &mut cache) {
            Ok(value) => value,
            Err(error) => {
                // debug!("iteration: {} cannot be run. {:?}", iteration, error);
                number_of_errors_run += 1;
                continue;
            }
        };
        if !funnel.check40(&terms40) {
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
