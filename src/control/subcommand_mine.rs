use super::DependencyManager;
use crate::config::Config;
use crate::mine::check_fixed_length_sequence::CheckFixedLengthSequence;
use crate::parser::{InstructionId, ParameterType};
use crate::execute::{EvalError, Program, ProgramRunner, RegisterValue, RunMode};
use crate::oeis::stripped_sequence::BigIntVec;
use crate::util::Analyze;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use num_bigint::BigInt;
use num_traits::Zero;
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
    let mine_output_dir: PathBuf = config.mine_output_dir();

    // Load cached data
    debug!("step1");
    let file10 = cache_dir.join(Path::new("fixed_length_sequence_10terms.json"));
    let checker10: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&file10);
    let file20 = cache_dir.join(Path::new("fixed_length_sequence_20terms.json"));
    let checker20: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&file20);
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
        &mine_output_dir,
        initial_random_seed,
    );
}

enum MutateValue {
    Increment,
    Decrement,
    Assign(u32),
}

struct GenomeItem {
    enabled: bool,
    instruction_id: InstructionId,
    target_value: u32,
    source_type: ParameterType,
    source_value: u32,
}

impl GenomeItem {
    fn new(instruction_id: InstructionId, target_value: u32, source_type: ParameterType, source_value: u32) -> Self {
        Self {
            enabled: true,
            instruction_id: instruction_id,
            target_value: target_value,
            source_type: source_type,
            source_value: source_value,
        }
    }

    fn new_move_register(target_value: u32, source_value: u32) -> Self {
        Self {
            enabled: true,
            instruction_id: InstructionId::Move,
            target_value: target_value,
            source_type: ParameterType::Register,
            source_value: source_value,
        }
    }

    fn new_instruction_with_const(instruction_id: InstructionId, target_value: u32, source_value: u32) -> Self {
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
    fn mutate_value(&mut self, mutation: &MutateValue, mut value: u32) -> (bool, u32) {
        match mutation {
            MutateValue::Increment => {
                if value >= 255 {
                    return (false, value);
                }
                value += 1;
            },
            MutateValue::Decrement => {
                if value == 0 {
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

    fn to_program_row(&self) -> String {
        if self.enabled {
            self.to_program_row_inner()
        } else {
            format!("; {}", self.to_program_row_inner())
        }
    }

    fn to_program_row_inner(&self) -> String {
        match &self.instruction_id {
            InstructionId::LoopBegin => {
                // For now don't care about the source type/value.
                // Maybe in the future support source type/value.
                return format!("{} ${}", 
                    self.instruction_id.shortname(), 
                    self.target_value 
                );
            },
            InstructionId::LoopEnd => {
                return self.instruction_id.shortname().to_string();
            },
            _ => {
                return format!("{} ${},{}{}", 
                    self.instruction_id.shortname(), 
                    self.target_value, 
                    self.source_type.prefix(), 
                    self.source_value
                );
            }
        }
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
        // {
        //     let item = GenomeItem {
        //         enabled: true,
        //         instruction_id: InstructionId::Call,
        //         target_value: 1,
        //         source_type: ParameterType::Constant,
        //         source_value: 230980,
        //     };
        //     genome_vec.push(item);
        // }
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

    fn to_program_string_vec(&self) -> Vec<String> {
        let program_rows: Vec<String> = self.genome_vec.iter().map(|genome_item| {
            genome_item.to_program_row()
        }).collect();
        program_rows
    }

    fn to_program_string(&self) -> String {
        self.to_program_string_vec().join("\n")
    }

    fn print(&self) {
        println!("program:\n{}", self.to_program_string());
    }

    fn program_string_with_terms(&self, terms: &BigIntVec) -> String {
        let term_strings: Vec<String> = terms.iter().map(|term| {
            term.to_string()
        }).collect();
        let term_strings_joined: String = term_strings.join(",");
        let prefix_rows = format!("; {}\n\n", term_strings_joined);
        let program_rows = self.to_program_string();
        prefix_rows + &program_rows
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
                rng.gen_range(0..5) as u32,
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

impl ProgramRunner {
    fn compute_terms(&self, count: u64) -> Result<BigIntVec, EvalError> {
        let mut terms: BigIntVec = vec!();
        let eval_count_limit: u64 = 10000;
        let mut _eval_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = self.run(input, RunMode::Silent, &mut _eval_count, eval_count_limit)?;
            terms.push(output.0.clone());
            if index == 0 {
                // print!("{}", output.0);
                continue;
            }
            // print!(",{}", output.0);
        }
        // print!("\n");
        // print!("stats: eval_count: {}", eval_count);
        Ok(terms)
    }
}

impl CheckFixedLengthSequence {
    fn is_possible_candidate(&self, terms: &BigIntVec) -> bool {
        if Analyze::count_unique(&terms) < 8 {
            // there are many results where all terms are just zeros.
            // there are many results where all terms are a constant value.
            // there are many results where most of the terms is a constant value.
            // there are many results where the terms alternates between 2 values.
            debug!("too few unique terms");
            return false;
        }
        if Analyze::is_almost_natural_numbers(&terms) {
            // there are many result that are like these
            // [0, 0, 1, 2, 3, 4, 5, 6, 7, 8]
            // [1, 1, 2, 3, 4, 5, 6, 7, 8, 9]
            // it's the natural numbers with 1 term different
            debug!("too close to being the natural numbers");
            return false;
        }
        if Analyze::count_zero(&terms) >= 7 {
            debug!("there are too many zero terms");
            return false;
        }
        if Analyze::is_all_the_same_value(&terms) {
            debug!("all terms are the same");
            return false;
        }
        if Analyze::is_constant_step(&terms) {
            debug!("the terms use constant step");
            return false;
        }
        if !self.check(&terms) {
            debug!("not found in bloom filter");
            return false;
        }
        // println!("contained in bloom filter: {:?}", terms);
        true
        // self.check(&terms)
    }
}

fn save_candidate_program(
    mine_output_dir: &Path,
    iteration: usize,
    content: &String,
) -> std::io::Result<()> 
{
    // Format filename as "19841231-235959-1234.asm"
    let now: DateTime<Utc> = Utc::now();
    let filename: String = format!("{}-{}.asm", now.format("%Y%m%d-%H%M%S"), iteration);

    // Write the file to the output dir
    let path = mine_output_dir.join(Path::new(&filename));
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;

    println!("candidate: {:?}", filename);
    Ok(())
}

fn run_experiment0(
    loda_program_rootdir: &PathBuf, 
    checker10: &CheckFixedLengthSequence, 
    checker20: &CheckFixedLengthSequence,
    mine_output_dir: &Path,
    initial_random_seed: u64,
) {
    let mut rng = StdRng::seed_from_u64(initial_random_seed);

    let mut dm = DependencyManager::new(
        loda_program_rootdir.clone(),
    );
    let mut genome = Genome::new();
    // genome.mutate_insert_loop(&mut rng);
    // genome.print();

    println!("\nPress CTRL-C to stop the miner.");
    let mut iteration: usize = 0;
    loop {
        if (iteration % 1000) == 0 {
            println!("iteration: {}", iteration);
        }
        iteration += 1;
        
        for _ in 0..20 {
            genome.mutate(&mut rng);
        }
    
        let program: Program = match dm.parse(&genome.to_program_string()) {
            Ok(value) => value,
            Err(error) => {
                debug!("iteration: {} cannot be parsed. {}", iteration, error);
                continue;
            }
        };
        let runner = ProgramRunner::new(program);
        let number_of_terms: u64 = 10;
        let terms10: BigIntVec = match runner.compute_terms(number_of_terms) {
            Ok(value) => value,
            Err(error) => {
                debug!("iteration: {} cannot be run. {:?}", iteration, error);
                continue;
            }
        };
    
        let check10_result: bool = checker10.is_possible_candidate(&terms10);
        if !check10_result {
            debug!("iteration: {} no match in oeis", iteration);
            continue;
        }
        // println!("iteration: {} candidate. terms: {:?}", iteration, terms10);

        let terms20: BigIntVec = match runner.compute_terms(20) {
            Ok(value) => value,
            Err(error) => {
                debug!("iteration: {} cannot be run. {:?}", iteration, error);
                continue;
            }
        };
        let check20_result: bool = checker20.is_possible_candidate(&terms20);
        if !check20_result {
            debug!("iteration: {} no match in oeis", iteration);
            continue;
        }

        let terms40: BigIntVec = match runner.compute_terms(40) {
            Ok(value) => value,
            Err(error) => {
                debug!("iteration: {} cannot be run. {:?}", iteration, error);
                continue;
            }
        };
        // println!("iteration: {} candidate. terms: {:?}", iteration, terms40);
        // genome.print();

        let candidate_program: String = genome.program_string_with_terms(&terms40);
        if let Err(error) = save_candidate_program(mine_output_dir, iteration, &candidate_program) {
            genome.print();
            error!("Unable to save candidate program: {:?}", error);
        }
    }
}
