use super::{EvalError, Node, NodeLoopLimit, ProgramCache, Program, ProgramRunnerManager, ProgramSerializer, ProgramState, RegisterIndex, RunMode, ValidateCallError};
use anyhow::Context;
use num_bigint::BigInt;
use num_traits::{Signed, Zero, One};
use std::{cmp::Ordering, ops::Sub};

pub struct NodeUnofficialLoopSubtract {
    register: RegisterIndex,
    program: Program,
}

impl NodeUnofficialLoopSubtract {
    pub fn new(register: RegisterIndex, program: Program) -> Self {
        Self {
            register: register,
            program: program,
        }
    }
}

impl Node for NodeUnofficialLoopSubtract {
    fn formatted_instruction(&self) -> String {
        format!("lps ${}", self.register)
    }

    fn serialize(&self, serializer: &mut ProgramSerializer) {
        serializer.append_raw(self.formatted_instruction());
        serializer.indent_increment();
        self.program.serialize(serializer);
        serializer.indent_decrement();
        serializer.append_raw("lpe");
    }

    fn eval(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> anyhow::Result<()> {
        if state.run_mode() == RunMode::Verbose {
            let snapshot = state.memory_full_to_string();
            let instruction = self.formatted_instruction();
            println!("{:12} {} => {}", instruction, snapshot, snapshot);
        }

        let mut current_counter: BigInt;
        {
            let counter: &BigInt = state.get_u64(self.register.0);
            if counter.is_negative() {
                state.increment_step_count()?;
                return Ok(())
            }
            current_counter = counter.clone();
        }

        let limit: NodeLoopLimit = state.node_loop_limit().clone();
        let mut cycles = 0;
        loop {
            self.program.run(state, cache)?;

            {
                let counter: &BigInt = state.get_u64(self.register.0);
                if counter != &current_counter {
                    state.increment_step_count()?;
                    if state.run_mode() == RunMode::Verbose {
                        println!("LOOP CYCLE EXIT");
                    }
                    return Ok(())
                }
                current_counter = counter - BigInt::one();
                if current_counter.is_zero() || current_counter.is_negative() {
                    println!("reached the end of the loop");
                    break;
                }
                state.set_u64(self.register.0, current_counter.clone())
                    .context("cannot decrement counter")?;
            }

            // Prevent looping for too long
            match limit {
                NodeLoopLimit::Unlimited => {},
                NodeLoopLimit::LimitCount(limit_count) => {
                    cycles += 1;
                    if cycles > limit_count {
                        let error = Err(EvalError::LoopCountExceededLimit);
                        return error.context("NodeLoopSimple loop count exceeded limit");
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
