use super::{CacheValue, EvalError, MyCache, Program, ProgramId, ProgramState, RegisterIndex, RegisterValue, RunMode};

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

        // Lookup (programid+input) in cache
        // No need to compute anything if it has been computed recently
        if let ProgramId::ProgramOEIS(program_oeis) = self.program_id {
            if let Some(cache_value) = cache.get(program_oeis, &(input.0)) {
                let value = RegisterValue(cache_value.value.clone());
                cache.register_cache_hit();

                // TODO: how to update the step counter?
                // *step_count += cache_value.step_count  
                // The first time there is a cache miss, then its `step_count`
                // gets stored in the cache. However this `step_count` will
                // vary depending on from where the function call happened.
                // If it's deeply nested, then the step_count will be high.
                // If it's invoked from the root scope, then the `step_count` will be low.
                // So `step_count` is misleading.
                // Come up with a new way of keeping track of the step_count.
                // Use a stack with step_counts.

                return Ok(value);
            }
        }

        // Initial state
        let mut state = ProgramState::new(self.register_count, run_mode, step_count_limit);
        state.set_step_count(*step_count);
        state.set_register_value(RegisterIndex(0), input.clone());

        // Invoke the actual run() function
        let run_result = self.program.run(&mut state, cache);

        // Update statistics, no matter if run succeeded or failed
        *step_count = state.step_count();

        // In case run failed, then return the error
        if let Err(error) = run_result {
            return Err(error);
        }
        
        // In case run succeeded, then return register 1.
        let value: RegisterValue = state.get_register_value(RegisterIndex(1));

        // Update cache
        match self.program_id {
            ProgramId::ProgramOEIS(program_oeis) => {
                // If this is an existing+verified program, then save the result in cache.
                cache.set(program_oeis, &(input.0), &(value.0));
                cache.register_cache_miss_for_program_oeis();
            },
            ProgramId::ProgramWithoutId => {
                // If this is a mining-candidate program, then don't save the result in cache.
                // There are no other programs that can refer to this program's id,
                // so there is no need for caching these types of programs.
                // The result value is only used once.
                cache.register_cache_miss_for_program_without_id();
            }
        }

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
