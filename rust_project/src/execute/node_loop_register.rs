use super::{EvalError, ProgramCache, Node, Program, ProgramState, ProgramRunnerManager, RegisterIndex, RegisterValue, RunMode, ValidateCallError};
use num_bigint::{BigInt, ToBigInt};
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
    fn shorthand(&self) -> &str {
        "loop register"
    }

    fn formatted_instruction(&self) -> String {
        String::from("")
    }

    fn eval(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> Result<(), EvalError> {
        if state.run_mode() == RunMode::Verbose {
            let snapshot = state.register_vec_to_string();
            let instruction = format!("lpb {},{}", self.register_start, self.register_with_range_length);
            println!("{:12} {} => {}", instruction, snapshot, snapshot);
        }

        let max_range_length_bigint: BigInt = 255.to_bigint().unwrap();

        let initial_value: RegisterValue = state.get_register_value(self.register_with_range_length.clone());
        let initial_value_inner: &BigInt = &initial_value.0;
        let initial_range_length: u8;
        if initial_value_inner.is_positive() {
            if initial_value_inner > &max_range_length_bigint {
                error!("Range length is beyond the ProgramState max length. Clamping range to 255.");
                initial_range_length = 255;
            } else {
                // Value is between 0 and 255, so it can be casted to an unsigned byte.
                initial_range_length = initial_value_inner.to_u8().unwrap();
            }
        } else {
            // Value is negative. Clamp to 0 length.
            initial_range_length = 0;
        }
        debug!("initial_range_length: {}", initial_range_length);

        let mut currently_smallest_range_length: u8 = initial_range_length;

        let mut cycles = 0;
        loop {
            let old_state: ProgramState = state.clone();

            self.program.run(state, cache)?;

            let value: RegisterValue = state.get_register_value(self.register_with_range_length.clone());
            let value_inner: &BigInt = &value.0;
            let range_length: u8;
            if value_inner.is_positive() {
                if value_inner > &max_range_length_bigint {
                    error!("Range length is beyond the ProgramState max length. Clamping range to 255.");
                    range_length = 255;
                } else {
                    // Value is between 0 and 255, so it can be casted to an unsigned byte.
                    range_length = value_inner.to_u8().unwrap();
                }
            } else {
                // Value is negative. Clamp to 0 length.
                range_length = 0;
            }
            debug!("range_length: {}", range_length);

            currently_smallest_range_length = u8::min(
                range_length, 
                currently_smallest_range_length
            );

            let is_less: bool = state.is_less_range(
                &old_state, 
                self.register_start.clone(),
                currently_smallest_range_length
            );
            debug!("is_less: {}  currently_smallest_range_length: {}", is_less, currently_smallest_range_length);

            if !is_less {

                if state.run_mode() == RunMode::Verbose {
                    let before = state.register_vec_to_string();
                    let after = old_state.register_vec_to_string();
                    println!("{:12} {} => {}  break", "lpe", before, after);
                }

                // When the loop reaches its end, the previous state is restored.
                let mut new_state: ProgramState = old_state.clone();
                new_state.set_step_count(state.step_count());
                *state = new_state;
                break;
            }


            cycles += 1;
            if cycles > 1000 {
                return Err(EvalError::LoopCountExceededLimit);
            }
            if state.run_mode() == RunMode::Verbose {
                let before = state.register_vec_to_string();
                let after = old_state.register_vec_to_string();
                println!("{:12} {} => {}  continue", "lpe", before, after);
            }
        }
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        // Loop doesn't modify any registers
        self.program.accumulate_register_indexes(register_vec);
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
