use super::{EvalError, RegisterIndex, RegisterValue, RunMode};
use num_bigint::BigInt;
use num_traits::Signed;
use std::cmp::Ordering;

#[derive(Clone)]
pub struct ProgramState {
    register_vec: Vec<RegisterValue>,
    eval_count: u64,
    run_mode: RunMode,
    eval_count_limit: u64,
}

impl ProgramState {
    pub fn new(register_count: u8, run_mode: RunMode, eval_count_limit: u64) -> Self {
        let mut register_vec: Vec<RegisterValue> = vec!();
        for _ in 0..register_count {
            register_vec.push(RegisterValue::zero());
        }
        Self {
            register_vec: register_vec,
            eval_count: 0,
            run_mode: run_mode,
            eval_count_limit: eval_count_limit,
        }
    }

    pub fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    pub fn get_register_value(&self, register_index: RegisterIndex) -> RegisterValue {
        let index = register_index.0 as usize;
        if index >= self.register_vec.len() {
            // Accessing a register outside bounds always returns zero
            return RegisterValue::zero();
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

    // Compare a range of registers.
    // LODA's `Memory.is_less` is always invoked with absolute values.
    // Unlike LODA, here the absolute value happens inside this `is_less` function.
    pub fn is_less_range(&self, other_state: &ProgramState, register_index: RegisterIndex, range_length: u8) -> bool {
        let vector_length: usize = self.register_vec.len();
        if vector_length != other_state.register_vec.len() {
            panic!("inconsistency. The vector lengths must be the same");
        }
        let start_index: usize = register_index.0 as usize;
        for i in 0..(range_length as usize) {
            let index: usize = start_index + i;
            if index >= vector_length {
                // Reached end of the vector
                return false;
            }
            let a: &RegisterValue = &self.register_vec[index];
            let b: &RegisterValue = &other_state.register_vec[index];
            let a_abs: BigInt = a.0.abs();
            let b_abs: BigInt = b.0.abs();
            let ordering: Ordering = a_abs.cmp(&b_abs);
            match ordering {
                Ordering::Less => return true,
                Ordering::Greater => return false,
                Ordering::Equal => continue,
            }
        }
        false
    }

    // Similar to `is_less_range()`, but with a range of 1.
    // This function is simpler than its counterpart `is_less_range`.
    pub fn is_less_single(&self, other_state: &ProgramState, register_index: RegisterIndex) -> bool {
        let vector_length: usize = self.register_vec.len();
        if vector_length != other_state.register_vec.len() {
            panic!("inconsistency. The vector lengths must be the same");
        }
        let index: usize = register_index.0 as usize;
        if index >= vector_length {
            // Reached end of the vector
            return false;
        }
        let a: &RegisterValue = &self.register_vec[index];
        let b: &RegisterValue = &other_state.register_vec[index];
        let a_abs: BigInt = a.0.abs();
        let b_abs: BigInt = b.0.abs();
        let ordering: Ordering = a_abs.cmp(&b_abs);
        match ordering {
            Ordering::Less => return true,
            Ordering::Greater => return false,
            Ordering::Equal => return false,
        }
    }
}

impl ProgramState {
    pub fn eval_count_limit(&self) -> u64 {
        self.eval_count_limit
    }

    pub fn eval_count(&self) -> u64 {
        self.eval_count
    }

    pub fn increment_eval_count(&mut self) -> Result<(), EvalError> {
        let count: u64 = self.eval_count + 1;
        self.eval_count = count;

        if count >= self.eval_count_limit {
            error!("eval_count exceeded limit");
            return Err(EvalError::EvalCountExceededLimit);
        }
        Ok(())
    }

    pub fn replace_eval_count(&mut self, count: u64) {
        // debug!("replacing eval count: {} -> {}", self.eval_count, count);
        self.eval_count = count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_program_state() -> ProgramState {
        let mut state = ProgramState::new(4, RunMode::Silent, 1000);
        state.set_register_value(RegisterIndex(0), RegisterValue::from_i64(100));
        state.set_register_value(RegisterIndex(1), RegisterValue::from_i64(101));
        state.set_register_value(RegisterIndex(2), RegisterValue::from_i64(102));
        state.set_register_value(RegisterIndex(3), RegisterValue::from_i64(103));
        state
    }

    fn empty_program_state() -> ProgramState {
        ProgramState::new(4, RunMode::Silent, 1000)
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

    #[test]
    fn test_20000_is_less_range_returns_false() {
        {
            // compare 0 registers
            let zero_length: u8 = 0;
            let state = empty_program_state();
            assert_eq!(state.is_less_range(&state, RegisterIndex(0), zero_length), false);
        }
        {
            // compare 1 register
            let state = empty_program_state();
            assert_eq!(state.is_less_range(&state, RegisterIndex(0), 1), false);
        }
        {
            // compare 4 registers
            let state = empty_program_state();
            assert_eq!(state.is_less_range(&state, RegisterIndex(0), 4), false);
        }
        {
            // compare 4 registers
            let state = mock_program_state();
            assert_eq!(state.is_less_range(&state, RegisterIndex(0), 4), false);
        }
        {
            // compare 4 registers
            let crazy_index_out_of_bounds = RegisterIndex(100);
            let state = mock_program_state();
            assert_eq!(state.is_less_range(&state, crazy_index_out_of_bounds, 4), false);
        }
        {
            // compare a crazy number of registers
            let crazy_length_out_of_bounds: u8 = 100;
            let state = mock_program_state();
            assert_eq!(state.is_less_range(&state, RegisterIndex(0), crazy_length_out_of_bounds), false);
        }
        {
            // compare 1 register
            let state0 = mock_program_state();
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(50));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-50));
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(49));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-50));
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), false);
        }
    }

    #[test]
    fn test_20001_is_less_range_returns_true() {
        {
            // compare 1 register
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(1));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), true);
        }
        {
            // compare 2 registers
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(1));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 2), true);
        }
        {
            // compare 2 registers
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(1), RegisterValue::from_i64(1));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 2), true);
        }
        {
            // compare 4 registers
            let state0 = mock_program_state();
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(3), RegisterValue::from_i64(104));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 4), true);
        }
        {
            // compare 4 registers, across end of vector boundary
            let state0 = mock_program_state();
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(3), RegisterValue::from_i64(104));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(2), 4), true);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(50));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), true);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-50));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), true);
        }
    }

    #[test]
    fn test_20002_is_less_single_returns_false() {
        {
            let state = empty_program_state();
            assert_eq!(state.is_less_single(&state, RegisterIndex(0)), false);
        }
        {
            let crazy_index_out_of_bounds = RegisterIndex(100);
            let state = mock_program_state();
            assert_eq!(state.is_less_single(&state, crazy_index_out_of_bounds), false);
        }
        {
            let state0 = mock_program_state();
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(50));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = mock_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-50));
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(49));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = mock_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-50));
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
    }

    #[test]
    fn test_20003_is_less_single_returns_true() {
        {
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(1));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
        {
            let mut state0 = mock_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(50));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
        {
            let mut state0 = mock_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-50));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
    }
}
