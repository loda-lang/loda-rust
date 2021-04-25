use super::{EvalError, ProgramCache, Program, ProgramId, ProgramSerializer, ProgramState, RegisterIndex, RegisterValue, RunMode};
use super::node_binomial::NodeBinomialLimit;
use super::node_power::NodePowerLimit;
use super::node_move::NodeMoveRegister;
use std::collections::HashSet;

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

    pub fn run(
        &self, 
        input: &RegisterValue, 
        run_mode: RunMode, 
        step_count: &mut u64, 
        step_count_limit: u64, 
        node_binomial_limit: NodeBinomialLimit, 
        node_power_limit: NodePowerLimit, 
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
            self.register_count, 
            run_mode, 
            step_count_limit, 
            node_binomial_limit,
            node_power_limit,
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
        let output: RegisterValue = state.get_output_value().clone();

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

    pub fn live_registers(&self) -> HashSet<RegisterIndex> {
        let mut register_set: HashSet<RegisterIndex> = HashSet::new();
        register_set.insert(RegisterIndex(0));
        self.program.live_register_indexes(&mut register_set);
        register_set
    }

    #[allow(dead_code)]
    pub fn has_live_registers(&self) -> bool {
        self.live_registers().contains(&RegisterIndex(1))
    }

    // While mining. Many programs gets rejected, because there is no connection from the 
    // input register to the output register. These defunct programs can be turned into 
    // working programs, by doing this trick:
    //
    // When detecting there no live output register, then append a move instruction 
    // that takes data from the lowest live register, and places it in the output register.
    // There may still be something meaningful in one of the other live registers.
    //
    // When there is zero live registers, then there is no way to get to the output register, 
    // and this program is truely defunct.
    pub fn mining_trick_attempt_fixing_the_output_register(&mut self) -> bool {
        let live_registers: HashSet<RegisterIndex> = self.live_registers();
        if live_registers.is_empty() {
            // There is no live registers to pick from.
            return false;
        }
        let target: RegisterIndex = RegisterIndex(1);
        if live_registers.contains(&target) {
            // There is live data in the output register.
            // No need to apply the trick.
            return true;
        }

        // There is no live data in the output register.
        // Append a `mov` instruction to the program that moves 
        // data to the output register.

        // Pick the lowest register index from the hash.
        let source: RegisterIndex = live_registers.into_iter()
            .min_by(|a, b| a.partial_cmp(&b).expect("Found a NaN"))
            .expect("There was no minimum");

        let node = NodeMoveRegister::new(target, source);
        let node_wrapped = Box::new(node);
        self.program.push_boxed(node_wrapped);

        // Determine the number of registeres to allocate before running the program
        let max_register_index: u8 = self.program.max_register_index();
        self.register_count = max_register_index + 1;

        true
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
        let step_count_limit: u64 = 10000;
        let mut step_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let result = self.run(
                &input, 
                RunMode::Silent, 
                &mut step_count, 
                step_count_limit,
                NodeBinomialLimit::Unlimited,
                NodePowerLimit::Unlimited,
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
