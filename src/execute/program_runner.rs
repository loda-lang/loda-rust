use super::{EvalError, Program, ProgramState, RegisterIndex, RegisterValue, RunMode};

pub struct ProgramRunner {
    program: Program,
    register_count: u8,
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

    pub fn run(&self, input: RegisterValue, run_mode: RunMode) -> Result<RegisterValue, EvalError> {
        let mut state = ProgramState::new(self.register_count, run_mode);
        state.set_register_value(RegisterIndex(0), input);
        self.program.run(&mut state)?;
        let value: RegisterValue = state.get_register_value(RegisterIndex(1));
        Ok(value)
    }

    pub fn run_terms(&self, count: u64) -> Result<Vec<i64>, EvalError> {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        let mut sequence: Vec<i64> = vec!();
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = self.run(input, RunMode::Silent)?;
            let value: i64 = output.to_i64();
            sequence.push(value);
        }
        Ok(sequence)
    }
}
