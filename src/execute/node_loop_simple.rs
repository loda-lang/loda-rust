use super::{EvalError, Node, Program, ProgramRunnerManager, ProgramState, RegisterIndex, RunMode, ValidateCallError};

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
    fn shorthand(&self) -> &str {
        "loop simple"
    }

    fn formatted_instruction(&self) -> String {
        String::from("")
    }

    fn eval_advanced(&self, state: &mut ProgramState) -> Result<(), EvalError> {
        if state.run_mode() == RunMode::Verbose {
            let snapshot = state.register_vec_to_string();
            let instruction = format!("lpb {}", self.register);
            println!("{:12} {} => {}", instruction, snapshot, snapshot);
        }

        let mut cycles = 0;
        loop {
            let old_state: ProgramState = state.clone();

            self.program.run(state);

            let is_less: bool = state.is_less_single(
                &old_state, 
                self.register.clone()
            );

            if !is_less {
                if state.run_mode() == RunMode::Verbose {
                    println!("LOOP CYCLE EXIT");
                }

                // When the loop reaches its end, the previous state is restored.
                *state = old_state.clone();
                break;
            }


            cycles += 1;
            if cycles > 1000 {
                panic!("looped too many times");
                // TODO: propagate info about problematic loops all the way
                // to caller and their caller, and let them decide what to do about it.
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
