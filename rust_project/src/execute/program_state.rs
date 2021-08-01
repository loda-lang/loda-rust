use super::{EvalError, NodeLoopLimit, RegisterIndex, RegisterValue, RunMode};
use super::node_binomial::NodeBinomialLimit;
use super::node_power::NodePowerLimit;
use super::NodeRegisterLimit;
use super::BoxCheckValue;
use num_bigint::BigInt;
use num_traits::Signed;
use std::cmp::Ordering;
use lazy_static::lazy_static;

lazy_static! {
    static ref OUT_OF_BOUNDS_RETURN_VALUE: RegisterValue = RegisterValue::zero();
}

// The register 0 is for input data.
const INPUT_REGISTER: usize = 0;

// The register 1 is for output data.
const OUTPUT_REGISTER: usize = 1;


#[derive(Clone)]
pub struct ProgramState {
    register_vec: Vec<RegisterValue>,
    step_count: u64,
    run_mode: RunMode,
    step_count_limit: u64,
    node_register_limit: NodeRegisterLimit,
    node_binomial_limit: NodeBinomialLimit,
    node_loop_limit: NodeLoopLimit,
    node_power_limit: NodePowerLimit,
    check_value: BoxCheckValue,
}

impl ProgramState {
    pub fn new(
        register_count: u8, 
        run_mode: RunMode, 
        step_count_limit: u64, 
        node_register_limit: NodeRegisterLimit,
        node_binomial_limit: NodeBinomialLimit, 
        node_loop_limit: NodeLoopLimit,
        node_power_limit: NodePowerLimit
    ) -> Self {
        // Register 0 is for input value
        // Register 1 is for output value
        // So there must be a least 2 registers.
        assert!(register_count >= 2);

        let mut register_vec: Vec<RegisterValue> = vec!();
        for _ in 0..register_count {
            register_vec.push(RegisterValue::zero());
        }

        let check_value: BoxCheckValue = node_register_limit.create_boxed_check_value();

        Self {
            register_vec: register_vec,
            step_count: 0,
            run_mode: run_mode,
            step_count_limit: step_count_limit,
            node_register_limit: node_register_limit,
            node_binomial_limit: node_binomial_limit,
            node_loop_limit: node_loop_limit,
            node_power_limit: node_power_limit,
            check_value: check_value,
        }
    }

    pub fn check_value(&self) -> &BoxCheckValue {
        &self.check_value
    }

    pub fn node_register_limit(&self) -> &NodeRegisterLimit {
        &self.node_register_limit
    }

    pub fn node_binomial_limit(&self) -> &NodeBinomialLimit {
        &self.node_binomial_limit
    }

    pub fn node_loop_limit(&self) -> &NodeLoopLimit {
        &self.node_loop_limit
    }

    pub fn node_power_limit(&self) -> &NodePowerLimit {
        &self.node_power_limit
    }

    pub fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    pub fn get_register_value_ref(&self, register_index: &RegisterIndex) -> &RegisterValue {
        let index = register_index.0 as usize;
        if index >= self.register_vec.len() {
            // Accessing a register outside bounds always returns zero
            return &OUT_OF_BOUNDS_RETURN_VALUE;
        }    
        return &self.register_vec[index];
    }    

    // Read the value of register 1, the output register.
    pub fn get_output_value(&self) -> &RegisterValue {
        assert!(self.register_vec.len() >= 2);
        return &self.register_vec[OUTPUT_REGISTER];
    }    

    pub fn set_register_value(&mut self, register_index: RegisterIndex, register_value: RegisterValue) {
        let index = register_index.0 as usize;
        if index >= self.register_vec.len() {
            panic!("set_register_value. index is outside the number of registers.");
        }
        self.register_vec[index] = register_value;
    }

    // Write a value to register 0, the input register.
    pub fn set_input_value(&mut self, register_value: &RegisterValue) {
        assert!(self.register_vec.len() >= 2);
        self.register_vec[INPUT_REGISTER] = register_value.clone();
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
    // Returns `true` if the range of registers have a lower value.
    // Returns `false` if the range of registers have the same value or greater value, o.
    // Returns `false` if a register is encountered with a negative value.
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
            let a_value: &BigInt = &a.0;
            if a_value.is_negative() {
                // Negative value encountered
                return false;
            }
            let b: &RegisterValue = &other_state.register_vec[index];
            let b_value: &BigInt = &b.0;
            let ordering: Ordering = a_value.cmp(&b_value);
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
        let a_value: &BigInt = &a.0;
        if a_value.is_negative() {
            // Negative value encountered
            return false;
        }
        let b: &RegisterValue = &other_state.register_vec[index];
        let b_value: &BigInt = &b.0;
        let ordering: Ordering = a_value.cmp(&b_value);
        match ordering {
            Ordering::Less => return true,
            Ordering::Greater => return false,
            Ordering::Equal => return false,
        }
    }
}

impl ProgramState {
    pub fn step_count_limit(&self) -> u64 {
        self.step_count_limit
    }

    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    pub fn increment_step_count(&mut self) -> Result<(), EvalError> {
        let count: u64 = self.step_count + 1;
        self.step_count = count;

        if count >= self.step_count_limit {
            return Err(EvalError::StepCountExceededLimit);
        }
        Ok(())
    }

    pub fn set_step_count(&mut self, count: u64) {
        self.step_count = count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_program_state() -> ProgramState {
        let mut state = ProgramState::new(
            4, 
            RunMode::Silent, 
            1000, 
            NodeRegisterLimit::Unlimited,
            NodeBinomialLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            NodePowerLimit::Unlimited,
        );
        state.set_register_value(RegisterIndex(0), RegisterValue::from_i64(100));
        state.set_register_value(RegisterIndex(1), RegisterValue::from_i64(101));
        state.set_register_value(RegisterIndex(2), RegisterValue::from_i64(102));
        state.set_register_value(RegisterIndex(3), RegisterValue::from_i64(103));
        state
    }

    fn empty_program_state() -> ProgramState {
        ProgramState::new(
            4, 
            RunMode::Silent, 
            1000, 
            NodeRegisterLimit::Unlimited,
            NodeBinomialLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            NodePowerLimit::Unlimited,
        )
    }

    #[test]
    fn test_10000_register_vec_to_string() {
        let state = mock_program_state();
        assert_eq!(state.register_vec_to_string(), "[100,101,102,103]");
    }

    #[test]
    #[should_panic]
    fn test_10001_initialize_with_too_few_registers() {
        ProgramState::new(
            0, 
            RunMode::Silent, 
            1000, 
            NodeRegisterLimit::Unlimited,
            NodeBinomialLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            NodePowerLimit::Unlimited,
        );
    }

    #[test]
    #[should_panic]
    fn test_10002_initialize_with_too_few_registers() {
        ProgramState::new(
            1, 
            RunMode::Silent, 
            1000,
            NodeRegisterLimit::Unlimited,
            NodeBinomialLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            NodePowerLimit::Unlimited,
        );
    }

    #[test]
    fn test_20001_set_register_range_to_zero() {
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
    fn test_30000_is_less_range_returns_false() {
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
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(50));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            let mut state1 = mock_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-50));
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), false);
        }
    }

    #[test]
    fn test_30001_is_less_range_returns_true() {
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
    }

    #[test]
    fn test_30002_is_less_single_returns_false() {
        {
            let state = empty_program_state();
            assert_eq!(state.is_less_single(&state, RegisterIndex(0)), false);
        }
        {
            let crazy_index_out_of_bounds = RegisterIndex(100);
            let state = empty_program_state();
            assert_eq!(state.is_less_single(&state, crazy_index_out_of_bounds), false);
        }
        {
            let mut state0 = empty_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(51));
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(50));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = empty_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(50));
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(50));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = empty_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-50));
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(49));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = empty_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-50));
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = empty_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(50));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = empty_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-49));
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(-50));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
    }

    #[test]
    fn test_30003_is_less_single_returns_true() {
        {
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(1));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
        {
            let mut state0 = empty_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(1));
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(2));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
        {
            let mut state0 = empty_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(1));
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(100));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
        {
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(100));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
        {
            let mut state0 = empty_program_state();
            state0.set_register_value(RegisterIndex(0), RegisterValue::from_i64(99));
            let mut state1 = empty_program_state();
            state1.set_register_value(RegisterIndex(0), RegisterValue::from_i64(100));
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
    }
}
