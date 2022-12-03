use super::{EvalError, Node, NodeLoopLimit, ProgramCache, Program, ProgramRunnerManager, ProgramSerializer, ProgramState, RunMode, ValidateCallError, RegisterIndexAndType, LOOP_RANGE_MAX_BITS};
use crate::parser::{Instruction, InstructionParameter, ParameterType};
use super::compiletime_error::*;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Signed};

pub struct NodeLoopSlow {
    target: InstructionParameter,
    source: InstructionParameter,
    program: Program,
}

impl NodeLoopSlow {
    pub fn new(instruction: Instruction, program: Program) -> Result<NodeLoopSlow, CreateInstructionError> {
        assert!(!instruction.parameter_vec.is_empty());
        let target: InstructionParameter = instruction.parameter_vec.first().unwrap().clone();
        let _ = RegisterIndexAndType::from_parameter(&instruction, &target)?;

        let source: InstructionParameter;
        if instruction.parameter_vec.len() == 2 {
            let parameter1: &InstructionParameter = instruction.parameter_vec.last().unwrap();
            source = parameter1.clone();
        } else {
            source = InstructionParameter::new(ParameterType::Constant, 1);
        }

        let instance = Self {
            target: target,
            source: source,
            program: program,
        };
        Ok(instance)
    }
}

impl Node for NodeLoopSlow {
    fn formatted_instruction(&self) -> String {
        format!("lpb {},{}", self.target, self.source)
    }

    fn serialize(&self, serializer: &mut ProgramSerializer) {
        serializer.append_raw(self.formatted_instruction());
        serializer.indent_increment();
        self.program.serialize(serializer);
        serializer.indent_decrement();
        serializer.append_raw("lpe");
    }

    fn eval(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> Result<(), EvalError> {
        if state.run_mode() == RunMode::Verbose {
            let snapshot = state.memory_full_to_string();
            let instruction = self.formatted_instruction();
            println!("{:12} {} => {}", instruction, snapshot, snapshot);
        }

        let limit: NodeLoopLimit = state.node_loop_limit().clone();
        let mut cycles = 0;
        loop {
            let old_state: ProgramState = state.clone();

            let old_target: BigInt = state.get(&self.target, true)?;
            let old_target_u64: u64 = match old_target.to_u64() {
                Some(value) => value,
                None => {
                    return Err(EvalError::CannotConvertBigIntToAddress);
                }
            };

            let old_source: BigInt = state.get(&self.source, false)?;
            let old_range_length: u64;
            if old_source.is_positive() {
                match old_source.to_u64() {
                    Some(value) => {
                        old_range_length = value;
                    },
                    None => {
                        return Err(EvalError::LoopRangeLengthExceededLimit);
                    }
                };
            } else {
                // `lpb` instruction with source being negative or zero. Does nothing.
                old_range_length = 0;
            }

            self.program.run(state, cache)?;
    
            let new_target: BigInt = state.get(&self.target, true)?;
            let new_target_u64: u64 = match new_target.to_u64() {
                Some(value) => value,
                None => {
                    return Err(EvalError::CannotConvertBigIntToAddress);
                }
            };

            let new_source: BigInt = state.get(&self.source, false)?;
            let new_range_length: u64;
            if new_source.is_positive() {
                match new_source.to_u64() {
                    Some(value) => {
                        new_range_length = value;
                    },
                    None => {
                        return Err(EvalError::LoopRangeLengthExceededLimit);
                    }
                };
            } else {
                // `lpb` instruction with source being negative or zero. Does nothing.
                new_range_length = 0;
            }

            let range_length: u64 = u64::min(new_range_length, old_range_length);
            if range_length >= (1 << LOOP_RANGE_MAX_BITS) {
                // Range length is beyond the max length.
                return Err(EvalError::LoopRangeLengthExceededLimit);
            }
    
            if state.run_mode() == RunMode::Verbose {
                println!("LOOP: old_target={}, old_range={}", old_target_u64, old_range_length);
                println!("LOOP: new_target={}, new_range={}", new_target_u64, new_range_length);

                let snapshot0 = old_state.memory_full_to_string();
                let snapshot1 = state.memory_full_to_string();
                println!("LOOP: old={} new={}", snapshot0, snapshot1);
            }
            let is_less: bool = state.is_less_twostartindexes_range(
                &old_state, 
                new_target_u64, 
                old_target_u64, 
                range_length
            );

            if !is_less {
                if state.run_mode() == RunMode::Verbose {
                    println!("LOOP CYCLE EXIT");
                }

                // When the loop reaches its end, the previous state is restored.
                let mut new_state: ProgramState = old_state.clone();
                new_state.set_step_count(state.step_count());
                *state = new_state;
                break;
            }

            // Prevent looping for too long
            match limit {
                NodeLoopLimit::Unlimited => {},
                NodeLoopLimit::LimitCount(limit_count) => {
                    cycles += 1;
                    if cycles > limit_count {
                        return Err(EvalError::LoopCountExceededLimit);
                    }
                }
            }
            if state.run_mode() == RunMode::Verbose {
                println!("lpe");
            }
        }

        state.increment_step_count()?;
        Ok(())
    }

    fn update_call(&mut self, program_manager: &mut ProgramRunnerManager) {
        self.program.update_call(program_manager);
    }

    fn accumulate_call_dependencies(&self, program_id_vec: &mut Vec<u64>) {
        self.program.accumulate_call_dependencies(program_id_vec);
    }

    fn validate_call_nodes(&self) -> Result<(), ValidateCallError> {
        self.program.validate_call_nodes()
    }
}
