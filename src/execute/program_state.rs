use super::{RegisterIndex, RegisterValue, RunMode};

#[derive(Clone)]
pub struct ProgramState {
    register_vec: Vec<RegisterValue>,
    eval_count: u64,
    run_mode: RunMode,
}

impl ProgramState {
    pub fn new(register_count: u8, run_mode: RunMode) -> Self {
        let mut register_vec: Vec<RegisterValue> = vec!();
        for _ in 0..register_count {
            register_vec.push(RegisterValue::zero());
        }
        Self {
            register_vec: register_vec,
            eval_count: 0,
            run_mode: run_mode,
        }
    }

    pub fn increment_eval_count(&mut self) {
        self.eval_count += 1;
    }

    pub fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    pub fn get_register_value(&self, register_index: RegisterIndex) -> RegisterValue {
        let index = register_index.0 as usize;
        if index >= self.register_vec.len() {
            panic!("get_register_value. index is outside the number of registers.");
        }
        return self.register_vec[index].clone();
    }

    pub fn set_register_value(&mut self, register_index: RegisterIndex, register_value: RegisterValue) {
        let index = register_index.0 as usize;
        if index >= self.register_vec.len() {
            panic!("set_register_value. index is outside the number of registers.");
        }
        self.register_vec[index] = register_value;
    }

    pub fn set_register_range_to_zero(&mut self, register_index: RegisterIndex, count: u8) {
        let number_of_registers: usize = self.register_vec.len(); 
        let mut index = register_index.0 as usize;
        for _ in 0..count {
            if index >= number_of_registers {
                // Do nothing when the index is outside the number of registers.
                return;
            }
            self.register_vec[index] = RegisterValue::zero();
            index += 1;
        }
    }

    // Make the internal state human readable
    pub fn register_vec_to_string(&self) -> String {
        let strings: Vec<String> = self.register_vec.iter().map(|register_value| {
            register_value.0.to_string()
        }).collect();
        let strings_joined: String = strings.join(",");
        format!("[{}]", strings_joined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_program_state() -> ProgramState {
        let mut state = ProgramState::new(4, RunMode::Silent);
        state.set_register_value(RegisterIndex(0), RegisterValue::from_i64(100));
        state.set_register_value(RegisterIndex(1), RegisterValue::from_i64(101));
        state.set_register_value(RegisterIndex(2), RegisterValue::from_i64(102));
        state.set_register_value(RegisterIndex(3), RegisterValue::from_i64(103));
        state
    }

    #[test]
    fn test_10000_register_vec_to_string() {
        let state = mock_program_state();
        assert_eq!(state.register_vec_to_string(), "[100,101,102,103]");
    }

    #[test]
    fn test_10001_set_register_range_to_zero() {
        {
            // clear 0 registers is the same as doing nothing
            let mut state = mock_program_state();
            state.set_register_range_to_zero(RegisterIndex(1), 0);
            assert_eq!(state.register_vec_to_string(), "[100,101,102,103]");
        }
        {
            // clear inside the range
            let mut state = mock_program_state();
            state.set_register_range_to_zero(RegisterIndex(1), 2);
            assert_eq!(state.register_vec_to_string(), "[100,0,0,103]");
        }
        {
            // clear inside the range
            let mut state = mock_program_state();
            state.set_register_range_to_zero(RegisterIndex(3), 1);
            assert_eq!(state.register_vec_to_string(), "[100,101,102,0]");
        }
        {
            // clear starting inside the range, and ending outside the range
            let mut state = mock_program_state();
            state.set_register_range_to_zero(RegisterIndex(3), 2);
            assert_eq!(state.register_vec_to_string(), "[100,101,102,0]");
        }
        {
            // clear outside range, is the same as doing nothing
            let mut state = mock_program_state();
            state.set_register_range_to_zero(RegisterIndex(100), 1);
            assert_eq!(state.register_vec_to_string(), "[100,101,102,103]");
        }
    }
}
