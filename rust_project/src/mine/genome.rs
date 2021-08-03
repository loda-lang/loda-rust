use crate::mine::{GenomeItem, GenomeMutateContext, MutateValue};
use crate::parser::{Instruction, InstructionId, InstructionParameter, ParameterType, ParsedProgram};
use std::fmt;
use rand::Rng;
use rand::seq::SliceRandom;

// Ideas for more mutations
// append random row
#[allow(dead_code)]
pub enum MutateGenome {
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
    CallAnotherProgram,
}

pub struct Genome {
    genome_vec: Vec<GenomeItem>
}

impl Genome {
    #[allow(dead_code)]
    pub fn new_from_parsed_program(parsed_program: &ParsedProgram) -> Self {
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

    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut genome_vec: Vec<GenomeItem> = vec!();
        {
            let item = GenomeItem::new_move_register(1, 0);
            genome_vec.push(item);
        }
        // {
        //     let item = GenomeItem::new_move_register(2, 0);
        //     genome_vec.push(item);
        // }
        // {
        //     let item = GenomeItem::new_move_register(3, 0);
        //     genome_vec.push(item);
        // }
        // append instructions that doesn't do anything to the output register
        // {
        //     let item = GenomeItem::new_instruction_with_const(InstructionId::Add, 1, 1);
        //     genome_vec.push(item);
        // }
        // {
        //     let item = GenomeItem::new_instruction_with_const(InstructionId::Subtract, 1, 1);
        //     genome_vec.push(item);
        // }
        // {
        //     let item = GenomeItem::new_instruction_with_const(InstructionId::Multiply, 1, 10);
        //     genome_vec.push(item);
        // }
        // {
        //     let item = GenomeItem::new_instruction_with_const(InstructionId::Divide, 1, 2);
        //     genome_vec.push(item);
        // }
        // {
        //     let item = GenomeItem::new_instruction_with_const(InstructionId::Multiply, 2, 10);
        //     genome_vec.push(item);
        // }
        // {
        //     let item = GenomeItem::new_instruction_with_const(InstructionId::DivideIf, 1, 10);
        //     genome_vec.push(item);
        // }
        // {
        //     let item = GenomeItem::new_instruction_with_const(InstructionId::Modulo, 2, 10);
        //     genome_vec.push(item);
        // }
        {
            let item = GenomeItem::new(
                InstructionId::EvalSequence,
                1,
                ParameterType::Constant,
                40,
            );
            genome_vec.push(item);
        }
        {
            let item = GenomeItem::new_instruction_with_const(InstructionId::Modulo, 1, 10);
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

    pub fn to_parsed_program(&self) -> ParsedProgram {
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
    #[allow(dead_code)]
    pub fn mutate_source_value_constant<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use constants
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if *genome_item.source_type() == ParameterType::Constant {
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
    #[allow(dead_code)]
    pub fn mutate_source_register<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use source_type=register
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if *genome_item.source_type() == ParameterType::Register {
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
    pub fn mutate_target_register<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
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
    #[allow(dead_code)]
    pub fn mutate_instruction<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
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
    #[allow(dead_code)]
    pub fn mutate_source_type<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
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
    #[allow(dead_code)]
    pub fn mutate_swap_registers<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use two registers
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if *genome_item.source_type() == ParameterType::Register {
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
    #[allow(dead_code)]
    pub fn mutate_enabled<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
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
    #[allow(dead_code)]
    pub fn mutate_swap_rows<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
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
    #[allow(dead_code)]
    pub fn mutate_swap_adjacent_rows<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
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
    #[allow(dead_code)]
    pub fn mutate_insert_loop<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
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
    #[allow(dead_code)]
    pub fn mutate_call<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Identify GenomeItem's that use the `cal` instruction
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if *genome_item.instruction_id() == InstructionId::EvalSequence {
                indexes.push(index);
            }
        }
        if indexes.is_empty() {
            return false;
        }

        // Pick one of the GenomeItem's 
        let index: &usize = indexes.choose(rng).unwrap();

        // Mutate the call instruction, so it invokes the next program in the list.
        // If it reaches the end, then it picks the first program from the list.
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        // genome_item.mutate_pick_next_program(rng, context)
        // genome_item.mutate_pick_popular_program(rng, context)
        genome_item.mutate_pick_recent_program(rng, context)
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure.
    #[allow(dead_code)]
    pub fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        let mutation_vec: Vec<(MutateGenome,usize)> = vec![
            (MutateGenome::Instruction, 10),
            (MutateGenome::SourceConstant, 200),
            (MutateGenome::SourceType, 100),
            (MutateGenome::SwapRegisters, 100),
            (MutateGenome::SourceRegister, 100),
            (MutateGenome::TargetRegister, 100),
            (MutateGenome::ToggleEnabled, 20),
            (MutateGenome::SwapRows, 1),
            (MutateGenome::SwapAdjacentRows, 10),
            (MutateGenome::InsertLoopBeginEnd, 0),
            (MutateGenome::CallAnotherProgram, 50),
        ];
        let mutation: &MutateGenome = &mutation_vec.choose_weighted(rng, |item| item.1).unwrap().0;
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
            },
            MutateGenome::CallAnotherProgram => {
                return self.mutate_call(rng, context);
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
