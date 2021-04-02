use super::{EvalError, Node, Program, ProgramCache, ProgramSerializer, ProgramState, ProgramRunnerManager, RegisterIndex, RunMode, ValidateCallError};
use std::collections::HashSet;

pub struct NodeLoopConstant {
    register_start: RegisterIndex,
    range_length: u8,
    program: Program,
}

impl NodeLoopConstant {
    pub fn new(register_start: RegisterIndex, range_length: u8, program: Program) -> Self {
        Self {
            register_start: register_start,
            range_length: range_length,
            program: program,
        }
    }
}

impl Node for NodeLoopConstant {
    fn formatted_instruction(&self) -> String {
        format!("lpb {},{}", self.register_start, self.range_length)
    }

    fn serialize(&self, serializer: &mut ProgramSerializer) {
        serializer.append(self.formatted_instruction());
        serializer.indent_increment();
        self.program.serialize(serializer);
        serializer.indent_decrement();
        serializer.append("lpe");
    }

    fn eval(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> Result<(), EvalError> {
        if state.run_mode() == RunMode::Verbose {
            let snapshot = state.register_vec_to_string();
            let instruction = self.formatted_instruction();
            println!("{:12} {} => {}", instruction, snapshot, snapshot);
        }

        let mut cycles = 0;
        loop {
            let old_state: ProgramState = state.clone();

            self.program.run(state, cache)?;

            let is_less: bool = state.is_less_range(
                &old_state, 
                self.register_start.clone(), 
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


            cycles += 1;
            if cycles > 1000 {
                return Err(EvalError::LoopCountExceededLimit);
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
