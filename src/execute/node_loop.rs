use super::{Node,RegisterIndex,RegisterValue,Program,ProgramState,ProgramRunnerManager,RunMode};
use num_bigint::BigInt;
use num_traits::{Zero, Signed};

pub struct NodeLoopRegister {
    register: RegisterIndex,
    program: Program,
}

impl NodeLoopRegister {
    pub fn new(register: RegisterIndex, program: Program) -> Self {
        Self {
            register: register,
            program: program,
        }
    }
}

impl Node for NodeLoopRegister {
    fn shorthand(&self) -> &str {
        "loop register"
    }

    fn formatted_instruction(&self) -> String {
        format!("lpe")
    }

    fn eval(&self, state: &mut ProgramState) {
        if state.run_mode() == RunMode::Verbose {
            let snapshot = state.register_vec_to_string();
            let instruction = format!("lpb {}", self.register);
            println!("{:12} {} => {}", instruction, snapshot, snapshot);
        }

        let mut cycles = 0;
        loop {
            let value: RegisterValue = state.get_register_value(self.register.clone());
            let value_inner: &BigInt = &value.0;
            if value_inner.is_zero() || value_inner.is_negative() {
                // TODO: only print in verbose mode
                // println!("reached end");
                break;
            }

            self.program.run(state);

            cycles += 1;
            if cycles > 1000 {
                panic!("looped too many times");
                // TODO: propagate info about problematic loops all the way
                // to caller and their caller, and let them decide what to do about it.
            }
        }
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.register.clone());
        self.program.accumulate_register_indexes(register_vec);
    }

    fn update_call(&mut self, program_manager: &mut ProgramRunnerManager) {
        self.program.update_call(program_manager);
    }

    fn accumulate_call_dependencies(&self, program_id_vec: &mut Vec<u64>) {
        self.program.accumulate_call_dependencies(program_id_vec);
    }
}
