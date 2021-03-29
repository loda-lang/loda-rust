use super::{EvalError, Program, ProgramState, RegisterIndex, RegisterValue, RunMode};

pub struct ProgramRunner {
    program: Program,
    register_count: u8,
    // TODO: reference to cache
}

impl ProgramRunner {
    pub fn new(program: Program) -> Self {
        // Determine the number of registeres to allocate before running the program
        let max_register_index: u8 = program.max_register_index();
        let register_count: u8 = max_register_index + 1;

        Self {
            program: program,
            register_count: register_count,
        }
    }

    pub fn run(&self, input: RegisterValue, run_mode: RunMode, step_count: &mut u64, step_count_limit: u64) -> Result<RegisterValue, EvalError> {
        // Initial state
        let mut state = ProgramState::new(self.register_count, run_mode, step_count_limit);
        state.set_step_count(*step_count);
        state.set_register_value(RegisterIndex(0), input);

        // Invoke the actual run() function
        let run_result = self.program.run(&mut state);

        // Update statistics, no matter if run succeeded or failed
        *step_count = state.step_count();

        // In case run failed, then return the error
        if let Err(error) = run_result {
            return Err(error);
        }

        // In case run succeeded, then return register 1.
        let value: RegisterValue = state.get_register_value(RegisterIndex(1));
        Ok(value)
    }

    pub fn run_terms(&self, count: u64) -> Result<Vec<i64>, EvalError> {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        let mut sequence: Vec<i64> = vec!();
        let step_count_limit: u64 = 10000;
        let mut step_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = self.run(input, RunMode::Silent, &mut step_count, step_count_limit)?;
            let value: i64 = output.to_i64();
            sequence.push(value);
        }
        Ok(sequence)
    }
}
