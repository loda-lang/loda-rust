use super::{EvalError, NodeLoopLimit, ProgramCache, Program, ProgramId, ProgramSerializer, ProgramState, RegisterValue, RunMode};
use super::NodeRegisterLimit;
use std::fmt;

pub struct ProgramRunner {
    program_id: ProgramId,
    program: Program,
}

impl ProgramRunner {
    pub fn new(program_id: ProgramId, program: Program) -> Self {
        Self {
            program_id: program_id,
            program: program,
        }
    }

    pub fn run(
        &self, 
        input: &RegisterValue, 
        run_mode: RunMode, 
        step_count: &mut u64, 
        step_count_limit: u64, 
        node_register_limit: NodeRegisterLimit, 
        node_loop_limit: NodeLoopLimit,
        cache: &mut ProgramCache
    ) -> Result<RegisterValue, EvalError> {
        let step_count_before: u64 = *step_count;

        // Lookup (programid+input) in cache
        // No need to compute anything if it has been computed recently
        if let ProgramId::ProgramOEIS(program_oeis) = self.program_id {
            if let Some(cache_value) = cache.get(program_oeis, &(input.0)) {
                let value = RegisterValue(cache_value.value.clone());
                *step_count = step_count_before + cache_value.step_count;
                cache.register_cache_hit();
                return Ok(value);
            }
        }

        // Initial state
        let mut state = ProgramState::new(
            run_mode, 
            step_count_limit, 
            node_register_limit,
            node_loop_limit,
        );
        state.set_step_count(step_count_before);
        state.set_input_value(input);

        // Invoke the actual run() function
        let run_result = self.program.run(&mut state, cache);

        // Update statistics, no matter if run succeeded or failed
        let step_count_after: u64 = state.step_count();
        *step_count = step_count_after;

        // In case run failed, then return the error
        if let Err(error) = run_result {
            return Err(error);
        }
        
        // In case run succeeded, then return output.
        let output: RegisterValue = state.get_output_value();

        // Update cache
        match self.program_id {
            ProgramId::ProgramOEIS(program_oeis) => {
                // If this is an existing+verified program, then save the result in cache.

                // Compute the number of steps used.
                assert!(step_count_after >= step_count_before);
                let computed_step_count: u64 = step_count_after - step_count_before;

                // Cache the computed value.
                cache.set(program_oeis, &(input.0), &(output.0), computed_step_count);
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

        Ok(output)
    }

    pub fn serialize(&self, serializer: &mut ProgramSerializer) {
        self.program.serialize(serializer);
    }

    #[cfg(test)]
    pub fn inspect(&self, count: u64) -> String {
        let mut cache = ProgramCache::new();
        self.inspect_advanced(count, &mut cache)
    }

    #[cfg(test)]
    pub fn inspect_advanced(&self, count: u64, cache: &mut ProgramCache) -> String {
        assert!(count < 0x7fff_ffff_ffff_ffff);
        let mut string_vec: Vec<String> = vec!();
        let step_count_limit: u64 = 30000;
        let mut step_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let result = self.run(
                &input, 
                RunMode::Silent, 
                &mut step_count, 
                step_count_limit,
                NodeRegisterLimit::Unlimited,
                NodeLoopLimit::Unlimited,
                cache,
            );
            match result {
                Ok(output) => {
                    string_vec.push(output.to_string());
                },
                Err(_) => {
                    string_vec.push("BOOM".to_string());
                    break;
                }
            }
        }
        string_vec.join(",")
    }
}

impl fmt::Display for ProgramRunner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut serializer = ProgramSerializer::new();
        self.serialize(&mut serializer);
        write!(f, "{}", serializer.to_string())
    }
}
