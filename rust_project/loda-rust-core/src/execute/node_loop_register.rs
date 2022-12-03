use super::{EvalError, Node, NodeLoopLimit, Program, ProgramCache, ProgramSerializer, ProgramState, ProgramRunnerManager, RegisterIndex, RunMode, ValidateCallError, LOOP_RANGE_MAX_BITS};
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Signed};

pub struct NodeLoopRegister {
    register_start: RegisterIndex,
    register_with_range_length: RegisterIndex,
    program: Program,
}

impl NodeLoopRegister {
    pub fn new(register_start: RegisterIndex, register_with_range_length: RegisterIndex, program: Program) -> Self {
        Self {
            register_start: register_start,
            register_with_range_length: register_with_range_length,
            program: program,
        }
    }
}

impl Node for NodeLoopRegister {
    fn formatted_instruction(&self) -> String {
        format!("lpb ${},{}", self.register_start, self.register_with_range_length)
    }

    fn serialize(&self, serializer: &mut ProgramSerializer) {
        serializer.append_raw(self.formatted_instruction());
        serializer.indent_increment();
        self.program.serialize(serializer);
        serializer.indent_decrement();
        serializer.append_raw("lpe");
    }

    fn eval(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> Result<(), EvalError> {
        if state.run_mode() == RunMode::Verbose {
            let snapshot = state.memory_full_to_string();
            let instruction = self.formatted_instruction();
            println!("{:12} {} => {}", instruction, snapshot, snapshot);
        }

        let initial_value_inner: &BigInt = state.get_u64(self.register_with_range_length.0);
        let initial_range_length: u64;
        if initial_value_inner.is_positive() {
            if initial_value_inner.bits() >= LOOP_RANGE_MAX_BITS {
                // Loop range length is beyond the max length.
                return Err(EvalError::LoopRangeLengthExceededLimit);
            } else {
                // Value is between 0 and 2^LOOP_RANGE_MAX_BITS.
                initial_range_length = match initial_value_inner.to_u64() {
                    Some(value) => value,
                    None => {
                        return Err(EvalError::LoopRangeLengthExceededLimit);
                    }
                }
            }
        } else {
            // Value is negative. Clamp to 0 length.
            initial_range_length = 0;
        }
        if state.run_mode() == RunMode::Verbose {
            debug!("initial_range_length: {}", initial_range_length);
        }

        let mut currently_smallest_range_length: u64 = initial_range_length;

        let limit: NodeLoopLimit = state.node_loop_limit().clone();
        let mut cycles = 0;
        loop {
            let old_state: ProgramState = state.clone();

            self.program.run(state, cache)?;

            let value_inner: &BigInt = state.get_u64(self.register_with_range_length.0);
            let range_length: u64;
            if value_inner.is_positive() {
                if value_inner.bits() >= LOOP_RANGE_MAX_BITS {
                    // Range length is beyond the max length.
                    return Err(EvalError::LoopRangeLengthExceededLimit);
                } else {
                    // Value is between 0 and 2^LOOP_RANGE_MAX_BITS.
                    range_length = match value_inner.to_u64() {
                        Some(value) => value,
                        None => {
                            return Err(EvalError::LoopRangeLengthExceededLimit);
                        }
                    }
                }
            } else {
                // Value is negative. Clamp to 0 length.
                range_length = 0;
            }
            if state.run_mode() == RunMode::Verbose {
                debug!("range_length: {}", range_length);
            }

            currently_smallest_range_length = u64::min(
                range_length, 
                currently_smallest_range_length
            );

            let is_less: bool = state.is_less_range(
                &old_state, 
                self.register_start.0,
                currently_smallest_range_length
            );
            if state.run_mode() == RunMode::Verbose {
                debug!("is_less: {}  currently_smallest_range_length: {}", is_less, currently_smallest_range_length);
            }

            if !is_less {

                if state.run_mode() == RunMode::Verbose {
                    let before = state.memory_full_to_string();
                    let after = old_state.memory_full_to_string();
                    println!("{:12} {} => {}  break", "lpe", before, after);
                }

                // When the loop reaches its end, the previous state is restored.
                let mut new_state: ProgramState = old_state.clone();
                new_state.set_step_count(state.step_count());
                *state = new_state;
                break;
            }

            // Prevent looping for too long
            match limit {
                NodeLoopLimit::Unlimited => {},
                NodeLoopLimit::LimitCount(limit_count) => {
                    cycles += 1;
                    if cycles > limit_count {
                        return Err(EvalError::LoopCountExceededLimit);
                    }
                }
            }
            if state.run_mode() == RunMode::Verbose {
                let before = state.memory_full_to_string();
                let after = old_state.memory_full_to_string();
                println!("{:12} {} => {}  continue", "lpe", before, after);
            }
        }

        state.increment_step_count()?;
        Ok(())
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
