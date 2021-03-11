use std::rc::Rc;
use super::{Node,RegisterIndex,RegisterValue,Program,ProgramState,ProgramRunner,ProgramRunnerManager};

pub struct NodeCallConstant {
    target: RegisterIndex,
    program_id: u64,
    program_runner_rc: Rc::<ProgramRunner>,
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

    fn eval(&self, state: &mut ProgramState) {
        // TODO: warn if no program have been installed.

        let input: RegisterValue = state.get_register_value(self.target.clone());
        let output: RegisterValue = self.program_runner_rc.run(input, state.run_mode());
        state.set_register_value(self.target.clone(), output);
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
    }

    fn update_call(&mut self, program_manager: &mut ProgramRunnerManager) {
        let program_id: u64 = self.program_id;

        let program_runner: Rc::<ProgramRunner> = match program_manager.get(program_id) {
            Some(value) => value,
            None => {
                println!("NodeCall. Unable to find program_id: {}", program_id);
                panic!("NodeCall. Unable to get program");
            }
        };

        self.program_runner_rc = program_runner;
        // println!("NodeCall: update_call. program_id: {}", program_id);
    }

    fn accumulate_call_dependencies(&self, program_id_vec: &mut Vec<u64>) {
        program_id_vec.push(self.program_id);
    }
}
