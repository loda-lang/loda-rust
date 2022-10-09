use super::{GenomeItem, GenomeMutateContext, MutateEvalSequenceCategory, MutateValue, SourceValue, TargetValue};
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::RegisterType;
use loda_rust_core::parser::{Instruction, InstructionId, InstructionParameter, ParameterType};
use loda_rust_core::parser::ParsedProgram;
use std::collections::HashSet;
use std::fmt;
use rand::Rng;
use rand::seq::SliceRandom;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
#[allow(dead_code)]
pub enum MutateGenome {
    ReplaceInstructionWithoutHistogram,
    ReplaceInstructionWithHistogram,
    InsertInstructionWithConstant,
    IncrementSourceValueWhereTypeIsConstant,
    DecrementSourceValueWhereTypeIsConstant,
    ReplaceSourceConstantWithHistogram,
    SourceType,
    SwapRegisters,
    ReplaceSourceRegisterWithoutHistogram,
    ReplaceSourceRegisterWithHistogram,
    ReplaceTargetWithoutHistogram,
    ReplaceTargetWithHistogram,
    ToggleEnabled,
    SwapRows,
    SwapAdjacentRows,
    InsertLoopBeginEnd,
    CallProgramWeightedByPopularity,
    CallMostPopularProgram,
    CallMediumPopularProgram,
    CallLeastPopularProgram,
    CallRecentProgram,
    CallProgramThatUsesIndirectMemoryAccess,
}

pub struct Genome {
    genome_vec: Vec<GenomeItem>,
    message_vec: Vec<String>,
}

impl Genome {
    pub fn new() -> Self {
        Self {
            genome_vec: vec!(),
            message_vec: vec!(),
        }
    }

    #[allow(dead_code)]
    pub fn contains_indirect_memory_access(&self) -> bool {
        for genome_item in &self.genome_vec {
            if genome_item.contains_indirect_memory_access() {
                return true;
            }
        }
        false
    }

    pub fn depends_on_program_ids(&self) -> HashSet<u32> {
        let mut program_ids = HashSet::<u32>::new();
        for genome_item in &self.genome_vec {
            if !genome_item.is_enabled() {
                continue;
            }
            if *genome_item.instruction_id() != InstructionId::EvalSequence {
                continue;
            }
            let program_id_raw: i32 = genome_item.source_value();
            if program_id_raw < 0 {
                continue;
            }
            program_ids.insert(program_id_raw as u32);
        }
        program_ids
    }

    #[allow(dead_code)]
    pub fn load_random_program<R: Rng + ?Sized>(&mut self, rng: &mut R, dm: &DependencyManager, context: &GenomeMutateContext) -> bool {
        let program_id_u32: u32 = match context.choose_available_program(rng) {
            Some(value) => value,
            None => {
                error!("cannot load random program. The list of available programs is empty");
                return false;
            }
        };
        let program_id: u64 = program_id_u32 as u64;
        let parsed_program: ParsedProgram = match self.load_program_with_id(dm, program_id) {
            Some(value) => value,
            None => {
                return false;
            }
        };

        return self.insert_program(program_id, &parsed_program);
    }

    pub fn load_program_with_id(&self, dm: &DependencyManager, program_id: u64) -> Option<ParsedProgram> {
        let path_to_program: PathBuf = dm.path_to_program(program_id);
        let contents: String = match fs::read_to_string(&path_to_program) {
            Ok(value) => value,
            Err(error) => {
                error!("loading program_id: {:?}, something went wrong reading the file: {:?}", program_id, error);
                return None;
            }
        };
        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&contents) {
            Ok(value) => value,
            Err(error) => {
                error!("loading program_id: {:?}, something went wrong parsing the program: {:?}", program_id, error);
                return None;
            }
        };
        Some(parsed_program)
    }

    pub fn insert_program(&mut self, program_id: u64, parsed_program: &ParsedProgram) -> bool {
        self.genome_vec.clear();
        self.push_parsed_program_onto_genome(&parsed_program);
        // debug!("loaded program_id: {:?}", program_id);
        self.message_vec.push(format!("template {:?}", program_id));
        return true;
    }

    pub fn push_parsed_program_onto_genome(&mut self, parsed_program: &ParsedProgram) {
        for instruction in &parsed_program.instruction_vec {

            let mut target_type = RegisterType::Direct;
            let mut target_value: i32 = 0;
            let mut source_type: ParameterType = ParameterType::Constant;
            let mut source_value: i32 = 0;
            for (index, parameter) in instruction.parameter_vec.iter().enumerate() {
                if index == 0 {
                    target_value = parameter.parameter_value as i32;
                    if parameter.parameter_type == ParameterType::Indirect {
                        target_type = RegisterType::Indirect;
                    } else {
                        target_type = RegisterType::Direct;
                    }
                }
                if index == 1 {
                    source_value = parameter.parameter_value as i32;
                    source_type = parameter.parameter_type.clone();
                }
            }
        
            let genome_item = GenomeItem::new(
                instruction.instruction_id.clone(),
                target_type,
                target_value,
                source_type,
                source_value,
            );
            self.genome_vec.push(genome_item);
        }
    }

    pub fn to_parsed_program(&self) -> ParsedProgram {
        let mut instruction_vec = Vec::<Instruction>::with_capacity(self.genome_vec.len());

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

    pub fn message_vec(&self) -> &Vec<String> {
        &self.message_vec
    }

    pub fn clear_message_vec(&mut self) {
        self.message_vec.clear();
    }

    pub fn append_message(&mut self, message: String) {
        self.message_vec.push(message);
    }

    /// Increment the source value.
    ///
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case the mutation had no effect.
    pub fn increment_source_value_where_type_is_constant<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify the instructions
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if *genome_item.source_type() != ParameterType::Constant {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::EvalSequence {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::LoopBegin {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::LoopEnd {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::Clear {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let value: i32 = genome_item.source_value();
        if value >= i32::MAX {
            return false;
        }
        let new_value = value + 1;
        if *genome_item.instruction_id() == InstructionId::Divide && new_value == 0 {
            return false;
        }
        if *genome_item.instruction_id() == InstructionId::DivideIf && new_value == 0 {
            return false;
        }
        if *genome_item.instruction_id() == InstructionId::Modulo && new_value == 0 {
            return false;
        }
        genome_item.set_source_value(new_value);
        true
    }

    /// Decrement the source value.
    ///
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case the mutation had no effect.
    pub fn decrement_source_value_where_type_is_constant<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify the instructions
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if *genome_item.source_type() != ParameterType::Constant {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::EvalSequence {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::LoopBegin {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::LoopEnd {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::Clear {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let value: i32 = genome_item.source_value();
        if value <= i32::MIN {
            return false;
        }
        let new_value = value - 1;
        if *genome_item.instruction_id() == InstructionId::Divide && new_value == 0 {
            return false;
        }
        if *genome_item.instruction_id() == InstructionId::DivideIf && new_value == 0 {
            return false;
        }
        if *genome_item.instruction_id() == InstructionId::Modulo && new_value == 0 {
            return false;
        }
        genome_item.set_source_value(new_value);
        true
    }

    /// Assign a constant, by picking from a histogram.
    /// 
    /// The histogram has knowledge about each instruction.
    /// 
    /// If it's an `add` instruction, then the most used constant is 1.
    /// 
    /// If it's a `mul` instruction, then the most used constant is 2.
    /// 
    /// If it's a `cmp` instruction, then the most used constant is 0.
    ///
    /// There is a high probability that this function assigns a constant
    /// that is highly used across all programs.
    ///
    /// There is a low probablility that this function assigns a constant
    /// that is sporadic used across all programs.
    ///
    /// This function does not assign a constant that has never been
    /// used elsewhere. So it doesn't explore new never tried out magic constants.
    ///
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as no instructions that use a constant, underflow, overflow.
    pub fn replace_source_constant_with_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the histogram csv file hasn't been loaded.
        if !context.has_histogram_instruction_constant() {
            return false;
        }

        // Identify the instructions that use constants
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if *genome_item.source_type() != ParameterType::Constant {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::EvalSequence {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::LoopBegin {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::LoopEnd {
                continue;
            }
            if *genome_item.instruction_id() == InstructionId::Clear {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions that use a constant
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];

        let instruction_id: InstructionId = *genome_item.instruction_id();
        let picked_value: i32 = match context.choose_constant_with_histogram(rng, instruction_id) {
            Some(value) => value,
            None => {
                return false;
            }
        };

        if picked_value == genome_item.source_value() {
            return false;
        }

        genome_item.set_source_value(picked_value);
        genome_item.mutate_sanitize_program_row()
    }

    /// Return `true` when the mutation was successful.
    /// Return `false` in case of failure, such as no instructions that use a source_type=register, underflow, overflow.
    pub fn replace_source_register_without_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use source_type=Direct
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if *genome_item.source_type() == ParameterType::Direct {
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
            MutateValue::Assign(0),
            MutateValue::Assign(1),
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

    fn get_source_value(genome_item: &GenomeItem) -> SourceValue {
        let instruction_id: InstructionId = *genome_item.instruction_id();
        if instruction_id == InstructionId::LoopEnd {
            return SourceValue::None;
        }
        if instruction_id == InstructionId::LoopBegin {
            return SourceValue::None;
        }
        match genome_item.source_type() {
            ParameterType::Constant => {
                return SourceValue::Constant;            
            },
            ParameterType::Direct => {
                let value: i32 = genome_item.source_value();
                return SourceValue::Value(value);
            },
            ParameterType::Indirect => {
                return SourceValue::None;
            }
        }
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn replace_source_register_with_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_source() {
            return false;
        }
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }
        let index1: usize = rng.gen_range(0..length);
        let index0: i32 = (index1 as i32) - 1;
        let index2: usize = index1 + 1;
        let mut prev_word: SourceValue = SourceValue::ProgramStart;
        if index0 >= 0 {
            match self.genome_vec.get(index0 as usize) {
                Some(ref value) => {
                    prev_word = Self::get_source_value(value)
                },
                None => {}
            };
        }
        let mut next_word: SourceValue = SourceValue::ProgramStop;
        match self.genome_vec.get(index2) {
            Some(ref value) => {
                next_word = Self::get_source_value(value)
            },
            None => {}
        };
        if prev_word == SourceValue::None || next_word == SourceValue::None {
            return false;
        }
        if prev_word == SourceValue::ProgramStart && next_word == SourceValue::ProgramStop {
            return false;
        }
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index1];
        if *genome_item.source_type() != ParameterType::Direct {
            // Only modify when the ParameterType=Direct.
            //
            // Prevent modifying an existing instruction if it has ParameterType=Indirect
            // since the `histogram_source_trigram.csv` doesn't consider indirect.
            //
            // Prevent modifying an existing instruction if it has ParameterType=Constant
            // since the `histogram_source_trigram.csv` doesn't consider constant.
            // The CSV file has `CONST`, but none of the constants.
            return false;
        }
        let suggested_value: SourceValue = match context.suggest_source(rng, prev_word, next_word) {
            Some(value) => value,
            None => {
                return false;
            }
        };
        let suggested_value_inner: i32 = match suggested_value {
            SourceValue::Value(value) => value,
            _ => {
                return false;
            }
        };
        if suggested_value_inner == genome_item.source_value() {
            return false;
        }
        if suggested_value_inner < 0 {
            return false;
        }
        genome_item.set_source_value(suggested_value_inner);
        // No need to sanitize when using histogram
        true
    }
    
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn replace_target_without_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }
        let index: usize = rng.gen_range(0..length);

        // Pick a random mutation
        let mutation_vec: Vec<MutateValue> = vec![
            MutateValue::Increment,
            MutateValue::Decrement,
            MutateValue::Assign(0),
            MutateValue::Assign(1),
        ];
        let mutation: &MutateValue = mutation_vec.choose(rng).unwrap();

        // Mutate one of the instructions
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];
        if !genome_item.mutate_target_value(mutation) {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    fn get_target_value(genome_item: &GenomeItem) -> TargetValue {
        if *genome_item.target_type() == RegisterType::Indirect {
            return TargetValue::None;
        }
        let instruction_id: InstructionId = *genome_item.instruction_id();
        if instruction_id == InstructionId::LoopEnd {
            return TargetValue::None;
        }
        let value: i32 = genome_item.target_value();
        TargetValue::Value(value)
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn replace_target_with_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_target() {
            return false;
        }
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }
        let index1: usize = rng.gen_range(0..length);
        let index0: i32 = (index1 as i32) - 1;
        let index2: usize = index1 + 1;
        let mut prev_word: TargetValue = TargetValue::ProgramStart;
        if index0 >= 0 {
            match self.genome_vec.get(index0 as usize) {
                Some(ref value) => {
                    prev_word = Self::get_target_value(value)
                },
                None => {}
            };
        }
        let mut next_word: TargetValue = TargetValue::ProgramStop;
        match self.genome_vec.get(index2) {
            Some(ref value) => {
                next_word = Self::get_target_value(value)
            },
            None => {}
        };
        if prev_word == TargetValue::None || next_word == TargetValue::None {
            return false;
        }
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index1];
        if *genome_item.target_type() == RegisterType::Indirect {
            return false;
        }
        let suggested_value: TargetValue = match context.suggest_target(rng, prev_word, next_word) {
            Some(value) => value,
            None => {
                return false;
            }
        };
        let suggested_value_inner: i32 = match suggested_value {
            TargetValue::Value(value) => value,
            _ => {
                return false;
            }
        };
        // let old_target: i32 = genome_item.target_value();
        if !genome_item.set_target_value(suggested_value_inner) {
            return false;
        }
        // debug!("suggest target: {:?} -> {:?}", old_target, suggested_value_inner);
        // No need to sanitize when using histogram
        true
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn replace_instruction_without_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }
        let index: usize = rng.gen_range(0..length);
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];

        if !genome_item.mutate_randomize_instruction(rng) {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn replace_instruction_with_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_instruction() {
            return false;
        }
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }
        let index1: usize = rng.gen_range(0..length);
        let index0: i32 = (index1 as i32) - 1;
        let index2: usize = index1 + 1;
        let mut prev_instruction: Option<InstructionId> = None;
        if index0 >= 0 {
            match self.genome_vec.get(index0 as usize) {
                Some(ref value) => {
                    let instruction_id: InstructionId = *value.instruction_id();
                    prev_instruction = Some(instruction_id);
                },
                None => {}
            };
        }
        let next_instruction: Option<InstructionId> = match self.genome_vec.get(index2) {
            Some(ref value) => {
                let instruction_id: InstructionId = *value.instruction_id();
                Some(instruction_id)
            },
            None => None
        };
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index1];
        let suggested_instruction_id: InstructionId = match context.suggest_instruction(rng, prev_instruction, next_instruction) {
            Some(value) => value,
            None => {
                return false;
            }
        };
        // let old_instruction: InstructionId = *genome_item.instruction_id();
        if !genome_item.set_instruction(suggested_instruction_id) {
            return false;
        }
        // debug!("suggest instruction: {:?} -> {:?}", old_instruction, suggested_instruction_id);
        genome_item.mutate_sanitize_program_row()
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn insert_instruction_with_constant<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the histogram csv file hasn't been loaded.
        if !context.has_histogram_instruction_constant() {
            return false;
        }
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_instruction() {
            return false;
        }
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_target() {
            return false;
        }
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }

        // Decide on where to insert a new GenomeItem
        let index1: usize = rng.gen_range(0..length);
        let index0: i32 = (index1 as i32) - 1;
        let index2: usize = index1;

        // Gather info about the "previous" GenomeItem
        let mut prev_instruction: Option<InstructionId> = None;
        let mut prev_target: TargetValue = TargetValue::ProgramStart;
        if index0 >= 0 {
            match self.genome_vec.get(index0 as usize) {
                Some(ref value) => {
                    let instruction_id: InstructionId = *value.instruction_id();
                    prev_instruction = Some(instruction_id);
                    prev_target = Self::get_target_value(value);
                },
                None => {}
            };
        }

        // Gather info about the "next" GenomeItem
        let mut next_instruction: Option<InstructionId> = None;
        let mut next_target: TargetValue = TargetValue::ProgramStop;
        match self.genome_vec.get(index2) {
            Some(ref value) => {
                let instruction_id: InstructionId = *value.instruction_id();
                next_instruction = Some(instruction_id);
                next_target = Self::get_target_value(value)
            },
            None => {}
        }

        // Pick an instruction from the histogram
        let suggested_instruction_id: InstructionId = match context.suggest_instruction(rng, prev_instruction, next_instruction) {
            Some(value) => value,
            None => {
                return false;
            }
        };

        // Pick a source constant from the histogram
        let source_value: i32 = match context.choose_constant_with_histogram(rng, suggested_instruction_id) {
            Some(value) => value,
            None => 0
        };

        // Pick a target register from the histogram
        let suggested_target_value: Option<TargetValue> = context.suggest_target(rng, prev_target, next_target);
        let target_value: i32;
        match suggested_target_value {
            Some(TargetValue::Value(value)) => {
                target_value = value;
            },
            _ => {
                target_value = rng.gen_range(0..5);
            }
        };

        let genome_item = GenomeItem::new(
            suggested_instruction_id, 
            RegisterType::Direct,
            target_value, 
            ParameterType::Constant, 
            source_value
        );
        // No need to sanitize when using histogram
        // println!("insert at {:?} item: {:?}", index1, genome_item);
        self.genome_vec.insert(index1, genome_item);
        true
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_source_type<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }
        let index: usize = rng.gen_range(0..length);
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];

        if !genome_item.mutate_source_type() {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_swap_registers<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use two registers
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if *genome_item.target_type() != RegisterType::Direct {
                continue;
            }
            if *genome_item.source_type() != ParameterType::Direct {
                continue;
            }
            indexes.push(index);
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

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_enabled<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }
        let index: usize = rng.gen_range(0..length);
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];

        if !genome_item.mutate_enabled() {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
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

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
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

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
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
                RegisterType::Direct,
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
                RegisterType::Direct,
                rng.gen_range(0..5) as i32,
                ParameterType::Constant,
                1
            );
            self.genome_vec.insert(index, item);
        }

        true
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure.
    pub fn mutate_eval_sequence<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext, category: MutateEvalSequenceCategory) -> bool {
        // Identify GenomeItem's that use the `seq` instruction
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
        // genome_item.mutate_pick_next_program(rng, context);
        genome_item.mutate_eval_sequence_instruction(rng, context, category)
    }

    /// Apply a mutation to the genome.
    /// 
    /// Return `true` when the genome got altered.
    /// 
    /// Return `false` in case the mutation didn't change the genome.
    pub fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        let mutation_vec: Vec<(MutateGenome,usize)> = vec![
            (MutateGenome::ReplaceInstructionWithoutHistogram, 1),
            (MutateGenome::ReplaceInstructionWithHistogram, 20),
            (MutateGenome::InsertInstructionWithConstant, 10),
            (MutateGenome::IncrementSourceValueWhereTypeIsConstant, 50),
            (MutateGenome::DecrementSourceValueWhereTypeIsConstant, 50),
            (MutateGenome::ReplaceSourceConstantWithHistogram, 200),
            (MutateGenome::SourceType, 1),
            (MutateGenome::SwapRegisters, 10),
            (MutateGenome::ReplaceSourceRegisterWithoutHistogram, 1),
            (MutateGenome::ReplaceSourceRegisterWithHistogram, 50),
            (MutateGenome::ReplaceTargetWithoutHistogram, 1),
            (MutateGenome::ReplaceTargetWithHistogram, 100),
            (MutateGenome::ToggleEnabled, 30),
            (MutateGenome::SwapRows, 1),
            (MutateGenome::SwapAdjacentRows, 10),
            (MutateGenome::InsertLoopBeginEnd, 0),
            (MutateGenome::CallProgramWeightedByPopularity, 5),
            (MutateGenome::CallMostPopularProgram, 100),
            (MutateGenome::CallMediumPopularProgram, 100),
            (MutateGenome::CallLeastPopularProgram, 10),
            // (MutateGenome::CallRecentProgram, 1),
            // (MutateGenome::CallProgramThatUsesIndirectMemoryAccess, 100),
        ];
        let mutation: &MutateGenome = &mutation_vec.choose_weighted(rng, |item| item.1).unwrap().0;

        let did_mutate_ok: bool = match mutation {
            MutateGenome::ReplaceInstructionWithoutHistogram => {
                self.replace_instruction_without_histogram(rng)
            },
            MutateGenome::ReplaceInstructionWithHistogram => {
                self.replace_instruction_with_histogram(rng, context)
            },
            MutateGenome::InsertInstructionWithConstant => {
                self.insert_instruction_with_constant(rng, context)
            },
            MutateGenome::IncrementSourceValueWhereTypeIsConstant => {
                self.increment_source_value_where_type_is_constant(rng)
            },
            MutateGenome::DecrementSourceValueWhereTypeIsConstant => {
                self.decrement_source_value_where_type_is_constant(rng)
            },
            MutateGenome::ReplaceSourceConstantWithHistogram => {
                self.replace_source_constant_with_histogram(rng, context)
            },
            MutateGenome::SourceType => {
                self.mutate_source_type(rng)
            },
            MutateGenome::SwapRegisters => {
                self.mutate_swap_registers(rng)
            },
            MutateGenome::ReplaceSourceRegisterWithoutHistogram => {
                self.replace_source_register_without_histogram(rng)
            },
            MutateGenome::ReplaceSourceRegisterWithHistogram => {
                self.replace_source_register_with_histogram(rng, context)
            },
            MutateGenome::ReplaceTargetWithoutHistogram => {
                self.replace_target_without_histogram(rng)
            },
            MutateGenome::ReplaceTargetWithHistogram => {
                self.replace_target_with_histogram(rng, context)
            },
            MutateGenome::ToggleEnabled => {
                self.mutate_enabled(rng)
            },
            MutateGenome::SwapRows => {
                self.mutate_swap_rows(rng)
            },
            MutateGenome::SwapAdjacentRows => {
                self.mutate_swap_adjacent_rows(rng)
            },
            MutateGenome::InsertLoopBeginEnd => {
                self.mutate_insert_loop(rng)
            },            
            MutateGenome::CallProgramWeightedByPopularity => {
                self.mutate_eval_sequence(rng, context, MutateEvalSequenceCategory::WeightedByPopularity)
            },
            MutateGenome::CallMostPopularProgram => {
                self.mutate_eval_sequence(rng, context, MutateEvalSequenceCategory::MostPopular)
            },
            MutateGenome::CallMediumPopularProgram => {
                self.mutate_eval_sequence(rng, context, MutateEvalSequenceCategory::MediumPopular)
            },
            MutateGenome::CallLeastPopularProgram => {
                self.mutate_eval_sequence(rng, context, MutateEvalSequenceCategory::LeastPopular)
            },
            MutateGenome::CallRecentProgram => {
                self.mutate_eval_sequence(rng, context, MutateEvalSequenceCategory::Recent)
            },
            MutateGenome::CallProgramThatUsesIndirectMemoryAccess => {
                self.mutate_eval_sequence(rng, context, MutateEvalSequenceCategory::ProgramThatUsesIndirectMemoryAccess)
            }
        };

        if did_mutate_ok {
            self.message_vec.push(format!("mutate: {:?}", mutation));
        } else {
            self.message_vec.push(format!("mutate: {:?}, no change", mutation));
        }

        did_mutate_ok
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
