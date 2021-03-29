use std::rc::Rc;
use super::{EvalError, Node, RegisterIndex, RegisterValue, Program, ProgramState, ProgramRunner, ProgramRunnerManager, ValidateCallError};

pub struct NodeCallConstant {
    target: RegisterIndex,
    program_id: u64,
    program_runner_rc: Rc::<ProgramRunner>,
    link_established: bool,
}

impl NodeCallConstant {
    pub fn new(target: RegisterIndex, program_id: u64) -> Self {
        let dummy_program = Program::new();
        let program_runner = ProgramRunner::new(dummy_program);
        let program_runner_rc = Rc::new(program_runner);

        Self {
            target: target,
            program_id: program_id,
            program_runner_rc: program_runner_rc,
            link_established: false,
        }
    }
}

impl Node for NodeCallConstant {
    fn shorthand(&self) -> &str {
        "call constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("cal {},{}", self.target, self.program_id)
    }

    fn eval(&self, state: &mut ProgramState) -> Result<(), EvalError> {
        if !self.link_established {
            panic!("No link have been establish. This node cannot do its job.");
        }
        let input: RegisterValue = state.get_register_value(self.target.clone());
        let eval_count_limit: u64 = state.eval_count_limit();
        let mut eval_count: u64 = state.eval_count();

        // Invoke the actual run() function
        let run_result = self.program_runner_rc.run(
            input, 
            state.run_mode(), 
            &mut eval_count, 
            eval_count_limit
        );

        // Update statistics, no matter if run succeeded or failed
        state.set_eval_count(eval_count);

        let output: RegisterValue = match run_result {
            Ok(value) => value,
            Err(error) => {
                // In case run failed, then return the error
                return Err(error);
            }
        };

        // In case run succeeded, then pass on the outputted value.
        state.set_register_value(self.target.clone(), output);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
    }

    fn update_call(&mut self, program_manager: &mut ProgramRunnerManager) {
        if self.link_established {
            panic!("The link have already been establish. Double assigning a link should not happen.");
        }
        let program_id: u64 = self.program_id;

        let program_runner: Rc::<ProgramRunner> = match program_manager.get(program_id) {
            Some(value) => value,
            None => {
                panic!("NodeCall. Unable to get program_id: {}", program_id);
            }
        };

        self.program_runner_rc = program_runner;
        self.link_established = true;
        debug!("NodeCall: update_call. program_id: {}", program_id);
    }

    fn accumulate_call_dependencies(&self, program_id_vec: &mut Vec<u64>) {
        program_id_vec.push(self.program_id);
    }

    fn validate_call_nodes(&self) -> Result<(), ValidateCallError> {
        if !self.link_established {
            // There is no connection with the program that we depend on.
            // Without the working dependency, this node cannot do its job correctly.
            return Err(ValidateCallError {});
        }
        Ok(())
    }
}
