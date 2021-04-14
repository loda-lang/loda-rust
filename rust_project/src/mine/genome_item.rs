use crate::parser::{InstructionId, InstructionParameter, ParameterType};
use crate::mine::GenomeMutateContext;
use rand::Rng;
use rand::seq::SliceRandom;
use std::fmt;

pub enum MutateValue {
    Increment,
    Decrement,
    Assign(i32),
}

pub struct GenomeItem {
    enabled: bool,
    instruction_id: InstructionId,
    target_value: i32,
    source_type: ParameterType,
    source_value: i32,
}

impl GenomeItem {
    pub fn new(instruction_id: InstructionId, target_value: i32, source_type: ParameterType, source_value: i32) -> Self {
        Self {
            enabled: true,
            instruction_id: instruction_id,
            target_value: target_value,
            source_type: source_type,
            source_value: source_value,
        }
    }

    pub fn new_move_register(target_value: i32, source_value: i32) -> Self {
        Self {
            enabled: true,
            instruction_id: InstructionId::Move,
            target_value: target_value,
            source_type: ParameterType::Register,
            source_value: source_value,
        }
    }

    pub fn new_instruction_with_const(instruction_id: InstructionId, target_value: i32, source_value: i32) -> Self {
        Self {
            enabled: true,
            instruction_id: instruction_id,
            target_value: target_value,
            source_type: ParameterType::Constant,
            source_value: source_value,
        }
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
    pub fn target_value(&self) -> i32 {
        self.target_value
    }

    pub fn source_type(&self) -> &ParameterType {
        &self.source_type
    }

    pub fn source_value(&self) -> i32 {
        self.source_value
    }

    #[allow(dead_code)]
    pub fn mutate_trigger_division_by_zero(&mut self) {
        self.instruction_id = InstructionId::Divide;
        self.source_type = ParameterType::Constant;
        self.source_value = 0;
    }

    pub fn mutate_randomize_instruction<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
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
            // InstructionId::Logarithm,
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

    pub fn mutate_source_value(&mut self, mutation: &MutateValue) -> bool {
        let is_call = self.instruction_id == InstructionId::Call;
        if is_call {
            return false;
        }
        let (status, new_value) = self.mutate_value(mutation, self.source_value);
        self.source_value = new_value;
        status
    }

    pub fn mutate_target_value(&mut self, mutation: &MutateValue) -> bool {
        let (status, new_value) = self.mutate_value(mutation, self.target_value);
        self.target_value = new_value;
        status
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as underflow, overflow.
    pub fn mutate_value(&mut self, mutation: &MutateValue, mut value: i32) -> (bool, i32) {
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

    pub fn mutate_source_type(&mut self) -> bool {
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

    pub fn mutate_enabled(&mut self) -> bool {
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

    pub fn mutate_swap_source_target_value(&mut self) -> bool {
        let is_call = self.instruction_id == InstructionId::Call;
        if is_call {
            return false;
        }
        let tmp = self.source_value;
        self.source_value = self.target_value;
        self.target_value = tmp;
        true
    }

    pub fn mutate_pick_next_program<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        let is_call = self.instruction_id == InstructionId::Call;
        if !is_call {
            // Only a call instruction can be modified.
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
                        if self.source_value < 0 {
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
                    if self.source_value < 1 {
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
                        if self.source_value < 0 {
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

    pub fn to_parameter_vec(&self) -> Vec<InstructionParameter> {
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
            InstructionId::Call => {
                let parameter0 = InstructionParameter {
                    parameter_type: ParameterType::Register,
                    parameter_value: self.target_value.abs() as i64,
                };
                let parameter1 = InstructionParameter {
                    parameter_type: ParameterType::Constant,
                    parameter_value: (self.source_value.abs()) as i64,
                };
                return vec![parameter0, parameter1];
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
