use super::{EvalError, Node, NodeLoopLimit, ProgramCache, Program, ProgramRunnerManager, ProgramSerializer, ProgramState, RegisterIndex, RunMode, ValidateCallError};
use std::collections::HashSet;

pub struct NodeLoopSimple {
    register: RegisterIndex,
    program: Program,
}

impl NodeLoopSimple {
    pub fn new(register: RegisterIndex, program: Program) -> Self {
        Self {
            register: register,
            program: program,
        }
    }
}

impl Node for NodeLoopSimple {
    fn formatted_instruction(&self) -> String {
        format!("lpb ${}", self.register)
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

            self.program.run(state, cache)?;

            let is_less: bool = state.is_less_single(
                &old_state, 
                self.register.clone()
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

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        // Loop doesn't modify any registers
        self.program.accumulate_register_indexes(register_vec);
    }

    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        self.program.live_register_indexes(register_set);
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
