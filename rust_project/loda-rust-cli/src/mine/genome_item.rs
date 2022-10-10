use loda_rust_core::execute::RegisterType;
use loda_rust_core::parser::{InstructionId, InstructionParameter, ParameterType};
use super::GenomeMutateContext;
use rand::Rng;
use rand::seq::SliceRandom;
use std::fmt;

const SANITIZE_MAX_NUMBER_OF_REGISTERS: i32 = 20;

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

#[derive(Debug)]
pub struct GenomeItem {
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

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn instruction_id(&self) -> &InstructionId {
        &self.instruction_id
    }

    #[allow(dead_code)]
    pub fn target_type(&self) -> &RegisterType {
        &self.target_type
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

    pub fn source_type(&self) -> &ParameterType {
        &self.source_type
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

    pub fn mutate_randomize_instruction<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // If there is a Call instruction then don't touch it.
        let is_call = self.instruction_id == InstructionId::EvalSequence;
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

    pub fn set_instruction(&mut self, new_instruction_id: InstructionId) -> bool {
        // Is the new instruction identical to the original instruction.
        if self.instruction_id == new_instruction_id {
            return false;
        }

        // If there is a Call instruction then don't touch it.
        let is_call = 
            self.instruction_id == InstructionId::EvalSequence ||
            new_instruction_id == InstructionId::EvalSequence;
        if is_call {
            return false;
        }    

        // Prevent messing up loop begin/end.
        let is_loop = 
            self.instruction_id == InstructionId::LoopBegin || 
            self.instruction_id == InstructionId::LoopEnd ||
            new_instruction_id == InstructionId::LoopBegin || 
            new_instruction_id == InstructionId::LoopEnd;
        if is_loop {
            return false;
        }    

        self.instruction_id = new_instruction_id;
        true
    }

    pub fn mutate_source_type(&mut self) -> bool {
        let is_call = self.instruction_id == InstructionId::EvalSequence;
        if is_call {
            return false;
        }
        match self.source_type {
            ParameterType::Constant => {
                self.source_type = ParameterType::Direct;
            },
            ParameterType::Direct => {
                self.source_type = ParameterType::Constant;
            },
            ParameterType::Indirect => {
                // Don't mutate a row with indirect memory access
                return false;
            },
        }
        true
    }

    pub fn mutate_enabled(&mut self) -> bool {
        // Prevent messing up loop begin/end.
        let is_loop = 
            self.instruction_id == InstructionId::LoopBegin || 
            self.instruction_id == InstructionId::LoopEnd;
        if is_loop {
            return false;
        }

        // Prevent messing up programs that use ParameterType::Indirect
        let is_indirect = 
            self.source_type == ParameterType::Indirect ||
            self.target_type == RegisterType::Indirect;
        if is_indirect {
            return false;
        }

        self.enabled = !self.enabled;
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
        let current_soruce_value: i32 = self.source_value();
        if current_soruce_value >= 0 {
            let is_same = (current_soruce_value as u32) == new_program_id;
            if is_same {
                // Failed to pick a different program
                return false;
            }
        }
        // Successfully picked a new program
        self.source_value = new_program_id as i32;
        true
    }

    pub fn mutate_sanitize_program_row(&mut self) -> bool {
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
            let new_register = self.target_value % SANITIZE_MAX_NUMBER_OF_REGISTERS;
            if self.target_value != new_register {
                self.target_value = new_register;
                status = false;
            }
        }

        // Prevent too extreme register index for source
        let sanitize_source_value: bool = match self.source_type {
            ParameterType::Constant => false,
            ParameterType::Direct | ParameterType::Indirect => true
        };
        if sanitize_source_value {
            let new_register = self.source_value % SANITIZE_MAX_NUMBER_OF_REGISTERS;
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
                    ParameterType::Direct => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    },
                    ParameterType::Indirect => {
                        // don't mutate a row that uses indirect
                        return false;
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
                    ParameterType::Direct => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    },
                    ParameterType::Indirect => {
                        // don't mutate a row that uses indirect
                        return false;
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
                    ParameterType::Direct => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    },
                    ParameterType::Indirect => {
                        // don't mutate a row that uses indirect
                        return false;
                    }
                }
            },
            InstructionId::Multiply => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                        if self.source_value < -1 {
                            self.source_value = -1;
                            return false;
                        }
                        if self.source_value < 2 {
                            self.source_value = 2;
                            return false;
                        }
                    },
                    ParameterType::Direct => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    },
                    ParameterType::Indirect => {
                        // don't mutate a row that uses indirect
                        return false;
                    }
                }
            },
            InstructionId::Subtract => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value < 0 {
                            self.source_value = 1;
                            return false;
                        }
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                    },
                    ParameterType::Direct => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    },
                    ParameterType::Indirect => {
                        // don't mutate a row that uses indirect
                        return false;
                    }
                }
            },
            InstructionId::Add => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value < 1 {
                            self.source_value = 1;
                            return false;
                        }
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                    },
                    ParameterType::Direct => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    },
                    ParameterType::Indirect => {
                        // don't mutate a row that uses indirect
                        return false;
                    }
                }
            },
            InstructionId::Move => {
                match self.source_type {
                    ParameterType::Constant => {
                        if self.source_value < 0 {
                            self.source_value = 1;
                            return false;
                        }
                        if self.source_value > 16 {
                            self.source_value = 16;
                            return false;
                        }
                    },
                    ParameterType::Direct => {
                        if self.source_value == self.target_value {
                            self.source_value = (self.target_value + 1) % 5;
                            return false;
                        }
                    },
                    ParameterType::Indirect => {
                        // don't mutate a row that uses indirect
                        return false;
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
                    ParameterType::Direct => {
                        self.source_type = ParameterType::Constant;
                        return false;
                    },
                    ParameterType::Indirect => {
                        // don't mutate a row that uses indirect
                        return false;
                    }
                }
            },
            _ => {
                return false;
            }
        }
        return status;
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
                let parameter1 = InstructionParameter {
                    parameter_type: self.source_type.clone(),
                    parameter_value: (self.source_value.abs()) as i64,
                };
                return vec![parameter0, parameter1];
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
        if self.source_type == ParameterType::Indirect {
            println!("!!!!!! GenomeItem.fmt indirect source");
        }
        if self.target_type == RegisterType::Indirect {
            println!("!!!!!! GenomeItem.fmt indirect target");
        }
        let line_prefix: &str;
        if self.enabled {
            line_prefix = "";
        } else {
            line_prefix = "; ";
        }
        write!(f, "{}{} {}{},{}{}", 
            line_prefix,
            self.instruction_id.shortname(), 
            self.target_type.prefix(),
            self.target_value, 
            self.source_type.prefix(), 
            self.source_value
        )
    }
}
