use super::{EvalError, NodeLoopLimit, RegisterIndex, RegisterValue, RunMode};
use super::node_power::NodePowerLimit;
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
    node_power_limit: NodePowerLimit,
    check_value: BoxCheckValue,
}

impl ProgramState {
    pub fn new(
        run_mode: RunMode, 
        step_count_limit: u64, 
        node_register_limit: NodeRegisterLimit,
        node_loop_limit: NodeLoopLimit,
        node_power_limit: NodePowerLimit
    ) -> Self {
        let check_value: BoxCheckValue = node_register_limit.create_boxed_check_value();

        Self {
            memory_full: HashMap::new(),
            step_count: 0,
            run_mode: run_mode,
            step_count_limit: step_count_limit,
            node_register_limit: node_register_limit,
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

    pub fn node_loop_limit(&self) -> &NodeLoopLimit {
        &self.node_loop_limit
    }

    pub fn node_power_limit(&self) -> &NodePowerLimit {
        &self.node_power_limit
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

    /// Read the value of register 0, the output register.
    pub fn get_output_value_bigint(&self) -> &BigInt {
        self.get_u64(OUTPUT_REGISTER)
    }
    
    pub fn get_output_value(&self) -> RegisterValue {
        let output_value: &BigInt = self.get_output_value_bigint();
        RegisterValue(output_value.clone())
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
        // panic!("TODO: replace u8 addresses with u64");
        let mut index = register_index.0 as u64;
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
    pub fn is_less_range(&self, other_state: &ProgramState, register_index: RegisterIndex, range_length: u8) -> bool {
        // panic!("TODO: replace u8 addresses with u64");
        let start_index: u64 = register_index.0 as u64;
        for i in 0..range_length {
            let index: u64 = start_index + (i as u64);
            let a_value: &BigInt = self.get_u64(index);
            if a_value.is_negative() {
                // Negative value encountered
                return false;
            }
            let b_value: &BigInt = other_state.get_u64(index);
            let ordering: Ordering = a_value.cmp(&b_value);
            match ordering {
                Ordering::Less => return true,
                Ordering::Greater => return false,
                Ordering::Equal => continue,
            }
        }
        false
    }

    /// Similar to `is_less_range()`, but with a range of 1.
    /// 
    /// This function is simpler than its counterpart `is_less_range`.
    pub fn is_less_single(&self, other_state: &ProgramState, register_index: RegisterIndex) -> bool {
        // panic!("TODO: replace u8 addresses with u64");
        self.is_less_range(other_state, register_index, 1)
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
            NodePowerLimit::Unlimited,
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
            NodePowerLimit::Unlimited,
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
            NodePowerLimit::Unlimited,
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
            NodePowerLimit::Unlimited,
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
            set_value_not_failable(&mut state1, 0, 50);
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            set_value_not_failable(&mut state0, 0, -50);
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 0, 49);
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            set_value_not_failable(&mut state0, 0, -50);
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 0, -49);
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            set_value_not_failable(&mut state0, 0, -49);
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 0, 50);
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), false);
        }
        {
            // compare 1 register
            let mut state0 = mock_program_state();
            set_value_not_failable(&mut state0, 0, -49);
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 0, -50);
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), false);
        }
    }

    #[test]
    fn test_30001_is_less_range_returns_true() {
        {
            // compare 1 register
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 1);
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 1), true);
        }
        {
            // compare 2 registers
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 1);
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 2), true);
        }
        {
            // compare 2 registers
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 1, 1);
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 2), true);
        }
        {
            // compare 4 registers
            let state0 = mock_program_state();
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 3, 104);
            assert_eq!(state0.is_less_range(&state1, RegisterIndex(0), 4), true);
        }
        {
            // compare 4 registers, across end of vector boundary
            let state0 = mock_program_state();
            let mut state1 = mock_program_state();
            set_value_not_failable(&mut state1, 3, 104);
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
            set_value_not_failable(&mut state0, 0, 51);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 50);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 50);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 50);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, -50);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 49);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, -50);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, -49);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, -49);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 50);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, -49);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, -50);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), false);
        }
    }

    #[test]
    fn test_30003_is_less_single_returns_true() {
        {
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 1);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 1);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 2);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 1);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 100);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
        {
            let state0 = empty_program_state();
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 100);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
        {
            let mut state0 = empty_program_state();
            set_value_not_failable(&mut state0, 0, 99);
            let mut state1 = empty_program_state();
            set_value_not_failable(&mut state1, 0, 100);
            assert_eq!(state0.is_less_single(&state1, RegisterIndex(0)), true);
        }
    }
}
