use anyhow::Context;

use super::{EvalError, Node, NodeLoopLimit, Program, ProgramCache, ProgramSerializer, ProgramState, ProgramRunnerManager, RegisterIndex, RunMode, ValidateCallError, LOOP_RANGE_MAX_BITS};

pub struct NodeLoopConstant {
    register_start: RegisterIndex,
    range_length: u64,
    program: Program,
}

impl NodeLoopConstant {
    pub fn new(register_start: RegisterIndex, range_length: u64, program: Program) -> Self {
        Self {
            register_start: register_start,
            range_length: range_length,
            program: program,
        }
    }
}

impl Node for NodeLoopConstant {
    fn formatted_instruction(&self) -> String {
        if self.range_length == 1 {
            return format!("lpb ${}", self.register_start)
        }
        format!("lpb ${},{}", self.register_start, self.range_length)
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

        if self.range_length >= (1 << LOOP_RANGE_MAX_BITS) {
            // Range length is beyond the max length.
            let error = Err(EvalError::LoopRangeLengthExceededLimit);
            return error.context("NodeLoopConstant range length exceeded LOOP_RANGE_MAX_BITS");
        }

        let limit: NodeLoopLimit = state.node_loop_limit().clone();
        let mut cycles = 0;
        loop {
            let old_state: ProgramState = state.clone();

            self.program.run(state, cache)?;

            let is_less: bool = state.is_less_range(
                &old_state, 
                self.register_start.0, 
                self.range_length
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
                        let error = Err(EvalError::LoopCountExceededLimit);
                        return error.context("NodeLoopConstant loop count exceeded limit");
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
