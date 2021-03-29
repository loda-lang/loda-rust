use super::{EvalError, MyCache, Program, ProgramId, ProgramState, RegisterIndex, RegisterValue, RunMode};

pub struct ProgramRunner {
    program_id: ProgramId,
    program: Program,
    register_count: u8,
}

impl ProgramRunner {
    pub fn new(program_id: ProgramId, program: Program) -> Self {
        // Determine the number of registeres to allocate before running the program
        let max_register_index: u8 = program.max_register_index();
        let register_count: u8 = max_register_index + 1;

        Self {
            program_id: program_id,
            program: program,
            register_count: register_count,
        }
    }

    pub fn run(&self, input: RegisterValue, run_mode: RunMode, step_count: &mut u64, step_count_limit: u64, cache: &mut MyCache) -> Result<RegisterValue, EvalError> {
        // TODO: lookup (programid+index) in cache

        cache.increment_hit();

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

        // TODO: if this is an existing+verified program, then save result in cache
        // TODO: if this is an mining-candidate program, then don't save result in cache
        cache.increment_miss();

        Ok(value)
    }

    pub fn run_terms(&self, count: u64) -> Result<Vec<i64>, EvalError> {
        if count >= 0x7fff_ffff_ffff_ffff {
            panic!("Value is too high. Cannot be converted to 64bit signed integer.");
        }
        let mut sequence: Vec<i64> = vec!();
        let mut cache = MyCache::new();
        let step_count_limit: u64 = 10000;
        let mut step_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = self.run(
                input, 
                RunMode::Silent, 
                &mut step_count, 
                step_count_limit,
                &mut cache,
            )?;
            let value: i64 = output.to_i64();
            sequence.push(value);
        }
        Ok(sequence)
    }
}
