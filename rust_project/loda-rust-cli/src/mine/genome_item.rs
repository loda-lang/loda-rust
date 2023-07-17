use loda_rust_core::execute::RegisterType;
use loda_rust_core::parser::{Instruction, InstructionId, InstructionParameter, ParameterType};
use loda_rust_core::parser::ParsedProgram;
use super::GenomeMutateContext;
use rand::Rng;
use rand::seq::SliceRandom;
use std::fmt;

// Ideas for more categories:
// Pick a recently created program.
// Pick a recently modified program.
// Pick a program that has not been modified for a long time.
// Increment the program_id, to get to the next available program_id.
// Pick a program with a similar name.
// Pick a program that executes fast.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum MutateEvalSequenceCategory {
    WeightedByPopularity,
    MostPopular,
    MediumPopular,
    LeastPopular,
    Recent,
    ProgramThatUsesIndirectMemoryAccess,
}

#[derive(Clone, Debug)]
pub struct GenomeItem {
    /// The `Genome` avoids modifying `GenomeItem`s that have `mutation_locked=true`.
    /// this is when a program follows a rigid pattern,
    /// where narrow areas in the program are to be mutated.
    mutation_locked: bool,

    enabled: bool,
    instruction_id: InstructionId,
    target_type: RegisterType,
    target_value: i32,
    source_type: ParameterType,
    source_value: i32,
}

impl GenomeItem {
    pub fn new(instruction_id: InstructionId, target_type: RegisterType, target_value: i32, source_type: ParameterType, source_value: i32) -> Self {
        Self {
            mutation_locked: false,
            enabled: true,
            instruction_id: instruction_id,
            target_type: target_type,
            target_value: target_value,
            source_type: source_type,
            source_value: source_value,
        }
    }

    pub fn contains_indirect_memory_access(&self) -> bool {
        if !self.enabled {
            return false;
        }
        if self.target_type == RegisterType::Indirect {
            return true;
        }
        if self.source_type == ParameterType::Indirect {
            return true;
        }
        false
    }

    pub fn is_mutation_locked(&self) -> bool {
        self.mutation_locked
    }

    #[allow(dead_code)]
    pub fn set_mutation_locked(&mut self, mutation_locked: bool) {
        self.mutation_locked = mutation_locked;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn instruction_id(&self) -> InstructionId {
        self.instruction_id
    }

    pub fn target_type(&self) -> RegisterType {
        self.target_type
    }

    pub fn set_target_type(&mut self, target_type: RegisterType) {
        self.target_type = target_type;
    }

    #[allow(dead_code)]
    pub fn target_value(&self) -> i32 {
        self.target_value
    }

    pub fn set_target_value(&mut self, value: i32) -> bool {
        if self.target_value() == value {
            return false;
        }
        if value < 0 {
            return false;
        }
        self.target_value = value;
        return true;
    }

    pub fn source_type(&self) -> ParameterType {
        self.source_type
    }

    pub fn set_source_type(&mut self, source_type: ParameterType) {
        self.source_type = source_type;
    }

    pub fn source_value(&self) -> i32 {
        self.source_value
    }    

    pub fn set_source_value(&mut self, value: i32) {
        self.source_value = value;
    }

    #[allow(dead_code)]
    pub fn mutate_trigger_division_by_zero(&mut self) {
        self.instruction_id = InstructionId::Divide;
        self.source_type = ParameterType::Constant;
        self.source_value = 0;
    }

    pub fn set_instruction(&mut self, new_instruction_id: InstructionId) -> bool {
        // Is the new instruction identical to the original instruction.
        if self.instruction_id == new_instruction_id {
            return false;
        }

        // Abort if the current instruction is special
        match self.instruction_id {
            InstructionId::EvalSequence | 
            InstructionId::LoopBegin | 
            InstructionId::LoopEnd |
            InstructionId::UnofficialFunction { .. } |
            InstructionId::UnofficialLoopBeginSubtract => {
                return false;
            },
            _ => {}
        }

        // Abort if the new instruction is special
        match new_instruction_id {
            InstructionId::EvalSequence | 
            InstructionId::LoopBegin | 
            InstructionId::LoopEnd |
            InstructionId::UnofficialFunction { .. } |
            InstructionId::UnofficialLoopBeginSubtract => {
                return false;
            },
            _ => {}
        }

        self.instruction_id = new_instruction_id;
        true
    }

    pub fn mutate_swap_source_target_value(&mut self) -> bool {
        if self.target_value == self.source_value {
            // No mutation happened
            return false;
        }
        let tmp = self.source_value;
        self.source_value = self.target_value;
        self.target_value = tmp;
        true
    }

    /// Mutate the `seq` instruction, so it invokes the next program in the list.
    /// 
    /// If it reaches the end, then it picks the first program from the list.
    #[allow(dead_code)]
    pub fn mutate_pick_next_program<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        let is_seq = self.instruction_id == InstructionId::EvalSequence;
        if !is_seq {
            // Only a `seq` instruction can be modified.
            return false;
        }
        let available_program_ids: &Vec<u32> = context.available_program_ids();
        if available_program_ids.is_empty() {
            // There are no program_ids to pick from.
            return false;
        }
        let current_program_id: u32 = self.source_value().abs() as u32;
        let mut iter = available_program_ids.iter();
        let index: Option<usize> = iter.position(|&program_id| program_id == current_program_id);

        // If the program wasn't found among the available programs,
        // then pick a random program.
        if index.is_none() {
            let new_program_id: &u32 = available_program_ids.choose(rng).unwrap();
            self.source_value = *new_program_id as i32;
            return true;
        }
        
        // If the program was found among the available programs,
        // then pick the next available program.
        if let Some(new_program_id) = iter.next() {
            self.source_value = *new_program_id as i32;
            return true;   
        }
        
        // Wraparound when reaching the end of the available programs.
        match available_program_ids.first() {
            Some(new_program_id) => {
                self.source_value = *new_program_id as i32;
                return true;   
            },
            None => {
                // If everything fails, fallback to fibonacci, A000045
                self.source_value = 45;
                return false;
            }
        }
    }

    /// Mutate the `seq` instruction, so it invokes a random program.
    pub fn mutate_instruction_seq<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext, category: MutateEvalSequenceCategory) -> bool {
        let is_seq = self.instruction_id == InstructionId::EvalSequence;
        if !is_seq {
            // Only a `seq` instruction can be modified.
            return false;
        }
        let chosen_program_id: Option<u32> = match category {
            MutateEvalSequenceCategory::WeightedByPopularity => context.choose_weighted_by_popularity(rng),
            MutateEvalSequenceCategory::MostPopular => context.choose_most_popular(rng),
            MutateEvalSequenceCategory::MediumPopular => context.choose_medium_popular(rng),
            MutateEvalSequenceCategory::LeastPopular => context.choose_least_popular(rng),
            MutateEvalSequenceCategory::Recent => context.choose_recent_program(rng),
            MutateEvalSequenceCategory::ProgramThatUsesIndirectMemoryAccess => context.choose_indirect_memory_access_program_id(rng),
        };
        let new_program_id: u32 = match chosen_program_id {
            Some(value) => value,
            None => {
                // The PopularProgramContainer is empty in some way.
                return false;
            }
        };
        let available_program_ids: &Vec<u32> = context.available_program_ids();
        if !available_program_ids.contains(&new_program_id) {
            // Picked a program that isn't among the available programs.
            // This happens when the csv files are outdated with the latest LODA repository.
            return false;
        }
        let current_source_value: i32 = self.source_value();
        if current_source_value >= 0 {
            let is_same = (current_source_value as u32) == new_program_id;
            if is_same {
                // Failed to pick a different program
                return false;
            }
        }
        // Successfully picked a new program
        self.source_value = new_program_id as i32;
        true
    }

    pub fn to_line_string(&self) -> String {
        if !self.enabled {
            return ";".to_string();
        }
        if self.instruction_id == InstructionId::LoopEnd {
            return self.instruction_id.to_string();
        }
        let parameter_vec: Vec<InstructionParameter> = self.to_parameter_vec();
        let strings: Vec<String> = parameter_vec.iter().map(|item| {
            item.to_string()
        }).collect();
        let parameter_strings: String = strings.join(",");
        format!("{} {}", self.instruction_id, parameter_strings)
    }

    pub fn to_parameter_vec(&self) -> Vec<InstructionParameter> {
        match &self.instruction_id {
            InstructionId::LoopBegin => {
                let parameter0: InstructionParameter;
                match self.target_type {
                    RegisterType::Direct => {
                        parameter0 = InstructionParameter {
                            parameter_type: ParameterType::Direct,
                            parameter_value: (self.target_value.abs()) as i64,
                        };
                    },
                    RegisterType::Indirect => {
                        parameter0 = InstructionParameter {
                            parameter_type: ParameterType::Indirect,
                            parameter_value: (self.target_value.abs()) as i64,
                        };
                    },
                }
                if self.source_type == ParameterType::Constant && self.source_value == 1 {
                    return vec![parameter0];
                }
                let parameter1 = InstructionParameter {
                    parameter_type: self.source_type.clone(),
                    parameter_value: (self.source_value.abs()) as i64,
                };
                return vec![parameter0, parameter1];
            },
            InstructionId::UnofficialLoopBeginSubtract => {
                let parameter0: InstructionParameter;
                match self.target_type {
                    RegisterType::Direct => {
                        parameter0 = InstructionParameter {
                            parameter_type: ParameterType::Direct,
                            parameter_value: (self.target_value.abs()) as i64,
                        };
                    },
                    RegisterType::Indirect => {
                        parameter0 = InstructionParameter {
                            parameter_type: ParameterType::Indirect,
                            parameter_value: (self.target_value.abs()) as i64,
                        };
                    },
                }
                return vec![parameter0];
            },
            InstructionId::LoopEnd => {
                return vec!();
            },
            InstructionId::EvalSequence => {
                let parameter0: InstructionParameter;
                match self.target_type {
                    RegisterType::Direct => {
                        parameter0 = InstructionParameter {
                            parameter_type: ParameterType::Direct,
                            parameter_value: (self.target_value.abs()) as i64,
                        };
                    },
                    RegisterType::Indirect => {
                        parameter0 = InstructionParameter {
                            parameter_type: ParameterType::Indirect,
                            parameter_value: (self.target_value.abs()) as i64,
                        };
                    },
                }
                let parameter1 = InstructionParameter {
                    parameter_type: ParameterType::Constant,
                    parameter_value: (self.source_value.abs()) as i64,
                };
                return vec![parameter0, parameter1];
            },
            _ => {
                let parameter0: InstructionParameter;
                match self.target_type {
                    RegisterType::Direct => {
                        parameter0 = InstructionParameter {
                            parameter_type: ParameterType::Direct,
                            parameter_value: (self.target_value.abs()) as i64,
                        };
                    },
                    RegisterType::Indirect => {
                        parameter0 = InstructionParameter {
                            parameter_type: ParameterType::Indirect,
                            parameter_value: (self.target_value.abs()) as i64,
                        };
                    },
                }
                let parameter1: InstructionParameter;
                match self.source_type {
                    ParameterType::Constant => {
                        parameter1 = InstructionParameter {
                            parameter_type: ParameterType::Constant,
                            parameter_value: self.source_value as i64,
                        };
                    },
                    ParameterType::Direct => {
                        parameter1 = InstructionParameter {
                            parameter_type: ParameterType::Direct,
                            parameter_value: (self.source_value.abs()) as i64,
                        };
                    },
                    ParameterType::Indirect => {
                        parameter1 = InstructionParameter {
                            parameter_type: ParameterType::Indirect,
                            parameter_value: (self.source_value.abs()) as i64,
                        };
                    },
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
        write!(f, "{}{} {}{},{}{}", 
            line_prefix,
            self.instruction_id, 
            self.target_type.prefix(),
            self.target_value, 
            self.source_type.prefix(), 
            self.source_value
        )
    }
}

pub trait ToGenomeItem {
    fn to_genome_item(&self) -> Option<GenomeItem>;
}

impl ToGenomeItem for Instruction {
    fn to_genome_item(&self) -> Option<GenomeItem> {
        let mut target_type = RegisterType::Direct;
        let mut target_value: i32 = 0;
        let mut source_type: ParameterType = ParameterType::Constant;
        let mut source_value: i32 = 0;
        if self.instruction_id == InstructionId::LoopBegin {
            // The "lpb" instruction, when there is no source parameter, then its default value is 1.
            source_value = 1;
        }
        for (index, parameter) in self.parameter_vec.iter().enumerate() {
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
            self.instruction_id,
            target_type,
            target_value,
            source_type,
            source_value,
        );
        Some(genome_item)
    }
}


pub trait ToGenomeItemVec {
    fn to_genome_item_vec(&self) -> Vec<GenomeItem>;
}

impl ToGenomeItemVec for ParsedProgram {
    fn to_genome_item_vec(&self) -> Vec<GenomeItem> {
        let mut genome_vec = Vec::<GenomeItem>::with_capacity(self.instruction_vec.len());
        for instruction in &self.instruction_vec {
            let genome_item: GenomeItem = match instruction.to_genome_item() {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            genome_vec.push(genome_item);
        }
        genome_vec
    }
}
