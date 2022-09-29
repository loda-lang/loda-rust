use super::{EvalError, Node, NodeLoopLimit, ProgramCache, Program, ProgramRunnerManager, ProgramSerializer, ProgramState, RunMode, ValidateCallError, RegisterIndex, RegisterIndexAndType};
use crate::parser::{Instruction, InstructionParameter, ParameterType};
use super::compiletime_error::*;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Signed, Zero};

pub struct NodeLoopSlow {
    instruction: Instruction,
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
            instruction: instruction,
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
        let mut range_length: u64 = u64::max_value();
        loop {
            let old_state: ProgramState = state.clone();

            self.program.run(state, cache)?;

            let target: BigInt = state.get(&self.target, true)?;
            let target_u64: u64 = match target.to_u64() {
                Some(value) => value,
                None => {
                    return Err(EvalError::CannotConvertBigIntToAddress);
                }
            };

            let source: BigInt = state.get(&self.source, false)?;
            if source.is_positive() {
                match source.to_u64() {
                    Some(value) => {
                        range_length = u64::min(value, range_length);
                    },
                    None => {
                        return Err(EvalError::LoopRangeLengthExceededLimit);
                    }
                };
            } else {
                // `lpb` instruction with source being negative or zero. Does nothing.
                range_length = 0;
            }
    
            if state.run_mode() == RunMode::Verbose {
                println!("LOOP: target={}, range={}", target_u64, range_length);

                let snapshot0 = old_state.memory_full_to_string();
                let snapshot1 = state.memory_full_to_string();
                println!("LOOP: old={} new={}", snapshot0, snapshot1);
            }
            let is_less: bool = state.is_less_range(
                &old_state, 
                RegisterIndex(target_u64), 
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
