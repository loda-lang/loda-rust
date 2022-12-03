use super::{EvalError, NodeLoopLimit, RegisterIndex, RegisterValue, RunMode};
use super::NodeRegisterLimit;
use super::BoxCheckValue;
use crate::parser::{InstructionParameter, ParameterType};
use num_bigint::{BigInt, ToBigInt};
use num_traits::{Signed, ToPrimitive, Zero};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::iter::FromIterator;
use lazy_static::lazy_static;

lazy_static! {
    static ref BIGINT_ZERO: BigInt = BigInt::zero();
}

/// The register 0 is for input data.
const INPUT_REGISTER: u64 = 0;

/// The register 0 is for output data.
const OUTPUT_REGISTER: u64 = 0;


const MAX_NUMBER_OF_REGISTERS: u64 = 10000;


#[derive(Clone)]
pub struct ProgramState {
    memory_full: HashMap<u64, BigInt>,
    step_count: u64,
    run_mode: RunMode,
    step_count_limit: u64,
    node_register_limit: NodeRegisterLimit,
    node_loop_limit: NodeLoopLimit,
    check_value: BoxCheckValue,
}

impl ProgramState {
    pub fn new(
        run_mode: RunMode, 
        step_count_limit: u64, 
        node_register_limit: NodeRegisterLimit,
        node_loop_limit: NodeLoopLimit,
    ) -> Self {
        let check_value: BoxCheckValue = node_register_limit.create_boxed_check_value();

        Self {
            memory_full: HashMap::new(),
            step_count: 0,
            run_mode: run_mode,
            step_count_limit: step_count_limit,
            node_register_limit: node_register_limit,
            node_loop_limit: node_loop_limit,
            check_value: check_value,
        }
    }

    pub fn check_value(&self) -> &BoxCheckValue {
        &self.check_value
    }

    pub fn node_register_limit(&self) -> &NodeRegisterLimit {
        &self.node_register_limit
    }

    pub fn node_loop_limit(&self) -> &NodeLoopLimit {
        &self.node_loop_limit
    }

    pub fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    pub fn get_u64(&self, address: u64) -> &BigInt {
        match self.memory_full.get(&address) {
            Some(value) => { return value; },
            None => { return &BIGINT_ZERO; }
        }
    }

    pub fn get_i64(&self, address: i64) -> Result<&BigInt, EvalError> {
        if address < 0 {
            return Err(EvalError::CannotConvertI64ToAddress);
        }
        Ok(self.get_u64(address as u64))
    }

    pub fn get_bigint(&self, address: &BigInt) -> Result<&BigInt, EvalError> {
        let address_u64: u64 = match address.to_u64() {
            Some(value) => value,
            None => {
                return Err(EvalError::CannotConvertBigIntToAddress);
            }
        };
        Ok(self.get_u64(address_u64))
    }

    pub fn get(&self, parameter: &InstructionParameter, get_address: bool) -> Result<BigInt, EvalError> {
        match parameter.parameter_type {
            ParameterType::Constant => {
                if get_address {
                    return Err(EvalError::CannotGetAddressOfConstant);
                }
                match parameter.parameter_value.to_bigint() {
                    Some(value) => { return Ok(value); },
                    None => { return Err(EvalError::CannotConvertParameterValueToBigInt); }
                }
            },
            ParameterType::Direct => {
                if get_address {
                    match parameter.parameter_value.to_bigint() {
                        Some(value) => { return Ok(value); }
                        None => { return Err(EvalError::CannotConvertParameterValueToBigInt); }
                    }
                }
                let inner_value: &BigInt = self.get_i64(parameter.parameter_value)?;
                return Ok(inner_value.clone());
            },
            ParameterType::Indirect => {
                let inner_value: &BigInt = self.get_i64(parameter.parameter_value)?;
                if get_address {
                    return Ok(inner_value.clone());
                }
                let inner_value2: &BigInt = self.get_bigint(inner_value)?;
                return Ok(inner_value2.clone());
            }
        }
    }
    
    pub fn set(&mut self, parameter: &InstructionParameter, set_value: BigInt) -> Result<(), EvalError> {
        match parameter.parameter_type {
            ParameterType::Constant => {
                return Err(EvalError::CannotSetValueOfConstant);
            },
            ParameterType::Direct => {
                self.set_i64(parameter.parameter_value, set_value)?;
                return Ok(());
            },
            ParameterType::Indirect => {
                let address_ref: &BigInt = self.get_i64(parameter.parameter_value)?;
                let address: BigInt = address_ref.clone();
                self.set_bigint(&address, set_value)?;
                return Ok(());
            }
        }
    }

    /// Read the value of register 0, the output reference to its `BigInt`.
    pub fn get_output_value_bigint(&self) -> &BigInt {
        self.get_u64(OUTPUT_REGISTER)
    }
    
    /// Read the value of register 0, the output register.
    /// 
    /// This is inefficient. It performs a `BigInt.clone()` operation.
    /// If you are going to discard the `ProgramState` immediately after getting the output,
    /// then consider using the `remove_output_value` function, that does no `BigInt.clone()`.
    pub fn get_output_value(&self) -> RegisterValue {
        let output_value: &BigInt = self.get_output_value_bigint();
        RegisterValue(output_value.clone())
    }

    /// Take ownership of the content in register `$0`, and removes it from the internal `memory_full`.
    /// 
    /// Warning. If you are to use `ProgramState` after invoking this function,
    /// then the `$0` address is cleared. In this case it's wiser to use the `get_output_value()` function.
    /// 
    /// Why use `remove_output_value()`?
    /// This eliminates one `BigInt.clone()` operation.
    pub fn remove_output_value(&mut self) -> RegisterValue {
        match self.memory_full.remove(&OUTPUT_REGISTER) {
            Some(value) => { 
                return RegisterValue(value); 
            },
            None => { 
                return RegisterValue(BIGINT_ZERO.clone()); 
            }
        }
    }

    pub fn set_bigint(&mut self, address: &BigInt, value: BigInt) -> Result<(), EvalError> {
        let address_u64: u64 = match address.to_u64() {
            Some(value) => value,
            None => {
                return Err(EvalError::CannotConvertBigIntToAddress);
            }
        };
        self.set_u64(address_u64, value)
    }

    pub fn set_i64(&mut self, address: i64, value: BigInt) -> Result<(), EvalError> {
        if address < 0 {
            return Err(EvalError::AddressWithNegativeValue);
        }
        self.set_u64(address as u64, value)
    }

    pub fn set_u64(&mut self, address: u64, value: BigInt) -> Result<(), EvalError> {
        if address >= MAX_NUMBER_OF_REGISTERS {
            return Err(EvalError::AddressIsOutsideMaxCapacity);
        }
        self.memory_full.insert(address, value);
        Ok(())
    }

    /// Write a value to register 0, the input register.
    pub fn set_input_value(&mut self, register_value: &RegisterValue) {
        self.memory_full.insert(INPUT_REGISTER, register_value.0.clone());
    }
   
    pub fn set_register_range_to_zero(&mut self, register_index: RegisterIndex, count: u64) -> Result<(), EvalError> {
        let mut index = register_index.0;
        let sum: u128 = (index as u128) + (count as u128);
        if sum >= (MAX_NUMBER_OF_REGISTERS as u128) {
            return Err(EvalError::AddressIsOutsideMaxCapacity);
        }
        for _ in 0..count {
            self.memory_full.remove(&index);
            index += 1;
        }
        Ok(())
    }

    /// Make the internal state human readable
    pub fn memory_full_to_string(&self) -> String {
        let key_refs: Vec<&u64> = Vec::from_iter(self.memory_full.keys());
        let mut keys: Vec<u64> = key_refs.iter().map(|&key| *key).collect();
        keys.sort();

        let mut strings = Vec::<String>::new();
        for key in keys {
            match self.memory_full.get(&key) {
                Some(value) => {
                    strings.push(format!("{}:{}", key, value));
                },
                None => {
                    strings.push(format!("{}:N/A", key));
                }
            }
        }

        let strings_joined: String = strings.join(",");
        format!("[{}]", strings_joined)
    }

    /// Compare a range of registers.
    /// 
    /// Returns `true` if the range of registers have a lower value.
    /// 
    /// Returns `false` if the range of registers have the same value or greater value.
    /// 
    /// Returns `false` if a register is encountered with a negative value.
    pub fn is_less_twostartindexes_range(&self, other_state: &ProgramState, start_index0: u64, start_index1: u64, range_length: u64) -> bool {
        for i in 0..range_length {
            let a_value: &BigInt = self.get_u64(start_index0 + i);
            if a_value.is_negative() {
                // Negative value encountered
                return false;
            }
            let b_value: &BigInt = other_state.get_u64(start_index1 + i);
            let ordering: Ordering = a_value.cmp(&b_value);
            match ordering {
                Ordering::Less => return true,
                Ordering::Greater => return false,
                Ordering::Equal => continue,
            }
        }
        false
    }

    pub fn is_less_range(&self, other_state: &ProgramState, start_index: u64, range_length: u64) -> bool {
        self.is_less_twostartindexes_range(other_state, start_index, start_index, range_length)
    }

    /// Similar to `is_less_range()`, but with a range of 1.
    /// 
    /// This function is simpler than its counterpart `is_less_range`.
    pub fn is_less_single(&self, other_state: &ProgramState, start_index: u64) -> bool {
        self.is_less_range(other_state, start_index, 1)
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

    fn set_value_not_failable(state: &mut ProgramState, address: u64, value: i64) {
        let value_bigint = value.to_bigint().expect("should not fail");
        state.set_u64(address, value_bigint).expect("should not fail");
    }

    fn mock_program_state() -> ProgramState {
        let mut state = ProgramState::new(
            RunMode::Silent, 
            1000, 
            NodeRegisterLimit::Unlimited,
            NodeLoopLimit::Unlimited,
        );
        set_value_not_failable(&mut state, 0, 100);
        set_value_not_failable(&mut state, 1, 101);
        set_value_not_failable(&mut state, 2, 102);
        set_value_not_failable(&mut state, 3, 103);
        state
    }

    fn empty_program_state() -> ProgramState {
        ProgramState::new(
            RunMode::Silent, 
            1000, 
            NodeRegisterLimit::Unlimited,
            NodeLoopLimit::Unlimited,
        )
    }

    #[test]
    fn test_10000_memory_full_to_string() {
        let state = mock_program_state();
        assert_eq!(state.memory_full_to_string(), "[0:100,1:101,2:102,3:103]");
    }

    #[test]
    fn test_10001_initialize_with_too_few_registers() {
        let state = ProgramState::new(
            RunMode::Silent, 
            1000, 
            NodeRegisterLimit::Unlimited,
            NodeLoopLimit::Unlimited,
        );
        assert_eq!(state.memory_full_to_string(), "[]")
    }

    #[test]
    fn test_10002_initialize_with_the_minimum_number_of_registers() {
        let state = ProgramState::new(
            RunMode::Silent, 
            1000,
            NodeRegisterLimit::Unlimited,
            NodeLoopLimit::Unlimited,
        );
        assert_eq!(state.memory_full_to_string(), "[]")
    }

    #[test]
    fn test_20001_set_register_range_to_zero() {
        {
            // clear 0 registers is the same as doing nothing
            let mut state = mock_program_state();
            state.set_register_range_to_zero(RegisterIndex(1), 0).expect("should not fail");
            assert_eq!(state.memory_full_to_string(), "[0:100,1:101,2:102,3:103]");
        }
        {
            // clear inside the range
            let mut state = mock_program_state();
            state.set_register_range_to_zero(RegisterIndex(1), 2).expect("should not fail");
            assert_eq!(state.memory_full_to_string(), "[0:100,3:103]");
        }
        {
            // clear inside the range
            let mut state = mock_program_state();
            state.set_register_range_to_zero(RegisterIndex(3), 1).expect("should not fail");
            assert_eq!(state.memory_full_to_string(), "[0:100,1:101,2:102]");
        }
        {
            // clear starting inside the range, and ending outside the range
            let mut state = mock_program_state();
            state.set_register_range_to_zero(RegisterIndex(3), 2).expect("should not fail");
            assert_eq!(state.memory_full_to_string(), "[0:100,1:101,2:102]");
        }
        {
            // clear outside range, is the same as doing nothing
            let mut state = mock_program_state();
            state.set_register_range_to_zero(RegisterIndex(100), 1).expect("should not fail");
            assert_eq!(state.memory_full_to_string(), "[0:100,1:101,2:102,3:103]");
        }
    }

    #[test]
    fn test_30000_is_less_range_returns_false() {
        {
            // compare 0 registers
            let zero_length: u64 = 0;
            let state = empty_program_state();
            assert_eq!(state.is_less_range(&state, 0, zero_length), false);
        }
        {
            // compare 1 register
            let state = empty_program_state();
            assert_eq!(state.is_less_range(&state, 0, 1), false);
        }
        {
            // compare 4 registers
            let state = empty_program_state();
            assert_eq!(state.is_less_range(&state, 0, 4), false);
        }
        {
            // compare 4 registers
            let state = mock_program_state();
            assert_eq!(state.is_less_range(&state, 0, 4), false);
        }
        {
            // compare 4 registers
            let crazy_high_index = 1000;
            let state = mock_program_state();
            assert_eq!(state.is_less_range(&state, crazy_high_index, 4), false);
        }
        {
            // compare a crazy number of registers
            let crazy_length: u64 = 1000;
            let state = mock_program_state();
            assert_eq!(state.is_less_range(&state, 0, crazy_length), false);
        }
        {
            // compare 1 register
            let state0 = mock_program_state();
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 0, 50);
            assert_eq!(state0.is_less_range(&state1, 0, 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            set_value_not_failable(&mut state0, 0, -50);
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 0, 49);
            assert_eq!(state0.is_less_range(&state1, 0, 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            set_value_not_failable(&mut state0, 0, -50);
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 0, -49);
            assert_eq!(state0.is_less_range(&state1, 0, 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            set_value_not_failable(&mut state0, 0, -49);
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 0, 50);
            assert_eq!(state0.is_less_range(&state1, 0, 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            set_value_not_failable(&mut state0, 0, -49);
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 0, -50);
            assert_eq!(state0.is_less_range(&state1, 0, 1), false);
        }
    }

    #[test]
    fn test_30001_is_less_range_returns_true() {
        {
            // compare 1 register
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 1);
            assert_eq!(state0.is_less_range(&state1, 0, 1), true);
        }
        {
            // compare 2 registers
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 1);
            assert_eq!(state0.is_less_range(&state1, 0, 2), true);
        }
        {
            // compare 2 registers
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 1, 1);
            assert_eq!(state0.is_less_range(&state1, 0, 2), true);
        }
        {
            // compare 4 registers
            let state0 = mock_program_state();
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 3, 104);
            assert_eq!(state0.is_less_range(&state1, 0, 4), true);
        }
        {
            // compare 4 registers, across end of vector boundary
            let state0 = mock_program_state();
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 3, 104);
            assert_eq!(state0.is_less_range(&state1, 2, 4), true);
        }
    }

    #[test]
    fn test_30002_is_less_single_returns_false() {
        {
            let state = empty_program_state();
            assert_eq!(state.is_less_single(&state, 0), false);
        }
        {
            let crazy_index_out_of_bounds = 100;
            let state = empty_program_state();
            assert_eq!(state.is_less_single(&state, crazy_index_out_of_bounds), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 51);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 50);
            assert_eq!(state0.is_less_single(&state1, 0), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 50);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 50);
            assert_eq!(state0.is_less_single(&state1, 0), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, -50);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 49);
            assert_eq!(state0.is_less_single(&state1, 0), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, -50);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, -49);
            assert_eq!(state0.is_less_single(&state1, 0), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, -49);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 50);
            assert_eq!(state0.is_less_single(&state1, 0), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, -49);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, -50);
            assert_eq!(state0.is_less_single(&state1, 0), false);
        }
    }

    #[test]
    fn test_30003_is_less_single_returns_true() {
        {
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 1);
            assert_eq!(state0.is_less_single(&state1, 0), true);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 1);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 2);
            assert_eq!(state0.is_less_single(&state1, 0), true);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 1);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 100);
            assert_eq!(state0.is_less_single(&state1, 0), true);
        }
        {
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 100);
            assert_eq!(state0.is_less_single(&state1, 0), true);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 99);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 100);
            assert_eq!(state0.is_less_single(&state1, 0), true);
        }
    }

    #[test]
    fn test_40000_is_less_twostartindexes_range_length1() {
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 100);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 10, 100);
            assert_eq!(state0.is_less_twostartindexes_range(&state1, 0, 10, 1), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 99);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 10, 100);
            assert_eq!(state0.is_less_twostartindexes_range(&state1, 0, 10, 1), true);
        }
    }

    #[test]
    fn test_40001_is_less_twostartindexes_range_length2() {
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 100);
            set_value_not_failable(&mut state0, 1, 100);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 10, 100);
            set_value_not_failable(&mut state1, 11, 100);
            assert_eq!(state0.is_less_twostartindexes_range(&state1, 0, 10, 2), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 100);
            set_value_not_failable(&mut state0, 1, 99);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 10, 100);
            set_value_not_failable(&mut state1, 11, 100);
            assert_eq!(state0.is_less_twostartindexes_range(&state1, 0, 10, 2), true);
        }
    }

    #[test]
    fn test_50000_remove_output_value_some() {
        // Arrange
        let mut state = empty_program_state();
        set_value_not_failable(&mut state, 0, 100);
        assert_eq!(state.memory_full_to_string(), "[0:100]");

        // Act
        let output: RegisterValue = state.remove_output_value();

        // Assert
        assert_eq!(state.memory_full_to_string(), "[]");
        assert_eq!(output, RegisterValue(100.to_bigint().unwrap()));
    }

    #[test]
    fn test_50001_remove_output_value_none() {
        // Arrange
        let mut state = empty_program_state();
        set_value_not_failable(&mut state, 1, 100);
        assert_eq!(state.memory_full_to_string(), "[1:100]");

        // Act
        let output: RegisterValue = state.remove_output_value();

        // Assert
        assert_eq!(state.memory_full_to_string(), "[1:100]");
        assert_eq!(output, RegisterValue(BigInt::zero()));
    }
}
