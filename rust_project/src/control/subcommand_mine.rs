use super::DependencyManager;
use crate::config::Config;
use crate::mine::check_fixed_length_sequence::CheckFixedLengthSequence;
use crate::parser::{Instruction, InstructionId, InstructionParameter, ParameterType, parse_program, ParseProgramError, ParsedProgram};
use crate::execute::{EvalError, ProgramCache, ProgramId, ProgramRunner, ProgramSerializer, RegisterValue, RunMode};
use crate::oeis::stripped_sequence::BigIntVec;
use crate::util::Analyze;
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
        initial_random_seed,
    );
}

fn terms_to_string(terms: &BigIntVec) -> String {
    let term_strings: Vec<String> = terms.iter().map(|term| {
        term.to_string()
    }).collect();
    let term_strings_joined: String = term_strings.join(",");
    term_strings_joined
}

enum MutateValue {
    Increment,
    Decrement,
    Assign(i32),
}

struct GenomeItem {
    enabled: bool,
    instruction_id: InstructionId,
    target_value: i32,
    source_type: ParameterType,
    source_value: i32,
}

impl GenomeItem {
    fn new(instruction_id: InstructionId, target_value: i32, source_type: ParameterType, source_value: i32) -> Self {
        Self {
            enabled: true,
            instruction_id: instruction_id,
            target_value: target_value,
            source_type: source_type,
            source_value: source_value,
        }
    }

    fn new_move_register(target_value: i32, source_value: i32) -> Self {
        Self {
            enabled: true,
            instruction_id: InstructionId::Move,
            target_value: target_value,
            source_type: ParameterType::Register,
            source_value: source_value,
        }
    }

    fn new_instruction_with_const(instruction_id: InstructionId, target_value: i32, source_value: i32) -> Self {
        Self {
            enabled: true,
            instruction_id: instruction_id,
            target_value: target_value,
            source_type: ParameterType::Constant,
            source_value: source_value,
        }
    }

    fn mutate_trigger_division_by_zero(&mut self) {
        self.instruction_id = InstructionId::Divide;
        self.source_type = ParameterType::Constant;
        self.source_value = 0;
    }

    fn mutate_randomize_instruction<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // If there is a Call instruction then don't touch it.
        let is_call = self.instruction_id == InstructionId::Call;
        if is_call {
            return false;
        }

        // Prevent messing up loop begin/end.
        let is_loop = 
            self.instruction_id == InstructionId::LoopBegin || 
            self.instruction_id == InstructionId::LoopEnd;
        if is_loop {
            return false;
        }

        let instructions: Vec<InstructionId> = vec![
            InstructionId::Add,
            // InstructionId::Binomial,
            // InstructionId::Compare,
            InstructionId::Divide,
            InstructionId::DivideIf,
            InstructionId::GCD,
            InstructionId::Logarithm,
            // InstructionId::Max,
            // InstructionId::Min,
            InstructionId::Modulo,
            InstructionId::Move,
            InstructionId::Multiply,
            InstructionId::Power,
            InstructionId::Subtract,
            InstructionId::Truncate,
        ];
        let instruction: &InstructionId = instructions.choose(rng).unwrap();
        self.instruction_id = instruction.clone();
        true
    }

    fn mutate_source_value(&mut self, mutation: &MutateValue) -> bool {
        let is_call = self.instruction_id == InstructionId::Call;
        if is_call {
            return false;
        }
        let (status, new_value) = self.mutate_value(mutation, self.source_value);
        self.source_value = new_value;
        status
    }

    fn mutate_target_value(&mut self, mutation: &MutateValue) -> bool {
        let (status, new_value) = self.mutate_value(mutation, self.target_value);
        self.target_value = new_value;
        status
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as underflow, overflow.
    fn mutate_value(&mut self, mutation: &MutateValue, mut value: i32) -> (bool, i32) {
        match mutation {
            MutateValue::Increment => {
                if value >= 255 {
                    return (false, value);
                }
                value += 1;
            },
            MutateValue::Decrement => {
                if value <= 0 {
                    return (false, value);
                }
                value -= 1;
            },
            MutateValue::Assign(v) => {
                value = *v;
            },
        }
        (true, value)
    }

    fn mutate_source_type(&mut self) -> bool {
        let is_call = self.instruction_id == InstructionId::Call;
        if is_call {
            return false;
        }
        match self.source_type {
            ParameterType::Constant => {
                self.source_type = ParameterType::Register;
            },
            ParameterType::Register => {
                self.source_type = ParameterType::Constant;
            },
        }
        true
    }

    fn mutate_enabled(&mut self) -> bool {
        let is_call = self.instruction_id == InstructionId::Call;
        if is_call {
            return false;
        }

        // Prevent messing up loop begin/end.
        let is_loop = 
            self.instruction_id == InstructionId::LoopBegin || 
            self.instruction_id == InstructionId::LoopEnd;
        if is_loop {
            return false;
        }

        self.enabled = !self.enabled;
        true
    }

    fn mutate_swap_source_target_value(&mut self) -> bool {
        let is_call = self.instruction_id == InstructionId::Call;
        if is_call {
            return false;
        }
        let tmp = self.source_value;
        self.source_value = self.target_value;
        self.target_value = tmp;
        true
    }

    fn mutate_sanitize_program_row(&mut self) -> bool {
        // Things to prevent 
        // division by zero
        // multiply by zero
        // raise to power 0
        // move/max/min/sub/mod/div/dif with same register
        // too huge constants
        // too huge register indexes
        // call to a non-existing program
        let mut status = true;

        // Prevent too extreme register index for target
        {
            let new_register = self.target_value % 5;
            if self.target_value != new_register {
                self.target_value = new_register;
                status = false;
            }
        }

        // Prevent too extreme register index for source
        if self.source_type == ParameterType::Register {
            let new_register = self.source_value % 5;
            if self.source_value != new_register {
                self.source_value = new_register;
                status = false;
            }
        }

        match self.instruction_id {
            InstructionId::Divide => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value < 2 {
                            self.source_value = 2;
                            return false;
                        }
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                    },
                    ParameterType::Register => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    }
                }
            },
            InstructionId::DivideIf => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value < 2 {
                            self.source_value = 2;
                            return false;
                        }
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                    },
                    ParameterType::Register => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    }
                }
            },
            InstructionId::Modulo => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value < 2 {
                            self.source_value = 2;
                            return false;
                        }
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                    },
                    ParameterType::Register => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    }
                }
            },
            InstructionId::Multiply => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value < 2 {
                            self.source_value = 2;
                            return false;
                        }
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                    },
                    ParameterType::Register => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    }
                }
            },
            InstructionId::Logarithm => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value < 2 {
                            self.source_value = 2;
                            return false;
                        }
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                    },
                    ParameterType::Register => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    }
                }
            },
            InstructionId::Subtract => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value == 0 {
                            self.source_value = 1;
                            return false;
                        }
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                    },
                    ParameterType::Register => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    }
                }
            },
            InstructionId::Add => {
                if self.source_type == ParameterType::Constant {
                    if self.source_value == 0 {
                        self.source_value = 1;
                        return false;
                    }
                    if self.source_value > 16 {
                        self.source_value = 16;
                        return false;
                    }
                }
            },
            InstructionId::Move => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value == 0 {
                            self.source_value = 1;
                            return false;
                        }
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                    },
                    ParameterType::Register => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    }
                }
            },
            InstructionId::Power => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value < 2 {
                            self.source_value = 2;
                            return false;
                        }
                        if self.source_value > 4 {
                            self.source_value = 4;
                            return false;
                        }
                    },
                    ParameterType::Register => {
                        self.source_type = ParameterType::Constant;
                        return false;
                    }
                }
            },
            _ => {}
        }
        return status;
    }

    fn to_parameter_vec(&self) -> Vec<InstructionParameter> {
        match &self.instruction_id {
            InstructionId::LoopBegin => {
                // For now don't care about the source type/value.
                // Maybe in the future support source type/value.
                let parameter = InstructionParameter {
                    parameter_type: ParameterType::Register,
                    parameter_value: self.target_value.abs() as i64,
                };
                return vec![parameter];
            },
            InstructionId::LoopEnd => {
                return vec!();
            },
            _ => {
                let parameter0 = InstructionParameter {
                    parameter_type: ParameterType::Register,
                    parameter_value: self.target_value.abs() as i64,
                };

                let parameter1: InstructionParameter;
                match self.source_type {
                    ParameterType::Constant => {
                        parameter1 = InstructionParameter {
                            parameter_type: ParameterType::Constant,
                            parameter_value: self.source_value as i64,
                        };
                    },
                    ParameterType::Register => {
                        parameter1 = InstructionParameter {
                            parameter_type: ParameterType::Register,
                            parameter_value: (self.source_value.abs()) as i64,
                        };
                    }
                }
                return vec![parameter0, parameter1];
            }
        }
    }
}

impl fmt::Display for GenomeItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let line_prefix: &str;
        if self.enabled {
            line_prefix = "";
        } else {
            line_prefix = "; ";
        }
        write!(f, "{}{} ${},{}{}", 
            line_prefix,
            self.instruction_id.shortname(), 
            self.target_value, 
            self.source_type.prefix(), 
            self.source_value
        )
    }
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
            let item = GenomeItem {
                enabled: true,
                instruction_id: InstructionId::Call,
                target_value: 1,
                source_type: ParameterType::Constant,
                source_value: 80578,
            };
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
            if !genome_item.enabled {
                continue;
            }

            let instruction_id: InstructionId = 
                genome_item.instruction_id.clone();
    
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
        let instruction0: &InstructionId = &self.genome_vec[index0].instruction_id;
        let instruction1: &InstructionId = &self.genome_vec[index1].instruction_id;
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
        let instruction0: &InstructionId = &self.genome_vec[index0].instruction_id;
        let instruction1: &InstructionId = &self.genome_vec[index1].instruction_id;
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

fn is_possible_candidate_basic_checks(terms: &BigIntVec) -> bool {
    if Analyze::count_unique(&terms) < 8 {
        // there are many results where all terms are just zeros.
        // there are many results where all terms are a constant value.
        // there are many results where most of the terms is a constant value.
        // there are many results where the terms alternates between 2 values.
        // debug!("too few unique terms");
        return false;
    }
    if Analyze::is_almost_natural_numbers(&terms) {
        // there are many result that are like these
        // [0, 0, 1, 2, 3, 4, 5, 6, 7, 8]
        // [1, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        // it's the natural numbers with 1 term different
        // debug!("too close to being the natural numbers");
        return false;
    }
    if Analyze::count_zero(&terms) >= 7 {
        // debug!("there are too many zero terms");
        return false;
    }
    if Analyze::is_all_the_same_value(&terms) {
        // debug!("all terms are the same");
        return false;
    }
    if Analyze::is_constant_step(&terms) {
        // debug!("the terms use constant step");
        return false;
    }
    true
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

struct Funnel<'a> {
    checker10: &'a CheckFixedLengthSequence,
    checker20: &'a CheckFixedLengthSequence,
    checker30: &'a CheckFixedLengthSequence,
    checker40: &'a CheckFixedLengthSequence,

    number_of_candidates_with_basiccheck: u64,
    number_of_candidates_with_10terms: u64,
    number_of_candidates_with_20terms: u64,
    number_of_candidates_with_30terms: u64,
    number_of_candidates_with_40terms: u64,
}

impl<'a> Funnel<'a> {
    fn new(
        checker10: &'a CheckFixedLengthSequence, 
        checker20: &'a CheckFixedLengthSequence,
        checker30: &'a CheckFixedLengthSequence,
        checker40: &'a CheckFixedLengthSequence,
    ) -> Self {
        Self {
            checker10: checker10, 
            checker20: checker20,
            checker30: checker30,
            checker40: checker40,
            number_of_candidates_with_basiccheck: 0,
            number_of_candidates_with_10terms: 0,
            number_of_candidates_with_20terms: 0,
            number_of_candidates_with_30terms: 0,
            number_of_candidates_with_40terms: 0,
        }
    }

    fn funnel_info(&self) -> String {
        format!(
            "[{},{},{},{},{}]",
            self.number_of_candidates_with_basiccheck,
            self.number_of_candidates_with_10terms,
            self.number_of_candidates_with_20terms,
            self.number_of_candidates_with_30terms,
            self.number_of_candidates_with_40terms,
        )
    }

    fn check_basic(&mut self, terms: &BigIntVec) -> bool {
        if !is_possible_candidate_basic_checks(terms) {
            return false;
        }
        self.number_of_candidates_with_basiccheck += 1;
        true
    }

    fn check10(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker10.check(terms) {
            return false;
        }
        self.number_of_candidates_with_10terms += 1;
        true
    }

    fn check20(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker20.check(terms) {
            return false;
        }
        self.number_of_candidates_with_20terms += 1;
        true
    }

    fn check30(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker30.check(terms) {
            return false;
        }
        self.number_of_candidates_with_30terms += 1;
        true
    }

    fn check40(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker40.check(terms) {
            return false;
        }
        self.number_of_candidates_with_40terms += 1;
        true
    }
}


fn run_experiment0(
    loda_program_rootdir: &PathBuf, 
    checker10: &CheckFixedLengthSequence, 
    checker20: &CheckFixedLengthSequence,
    checker30: &CheckFixedLengthSequence,
    checker40: &CheckFixedLengthSequence,
    mine_event_dir: &Path,
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
        serializer.append(format!("; {}", terms_to_string(&terms40)));
        serializer.append("");
        runner.serialize(&mut serializer);
        let candidate_program: String = serializer.to_string();

        if let Err(error) = save_candidate_program(mine_event_dir, iteration, &candidate_program) {
            println!("; GENOME\n{}", genome);
            error!("Unable to save candidate program: {:?}", error);
        }
    }
}
