use super::{EvalError, NodeLoopLimit, RegisterIndex, RegisterValue, RunMode};
use super::node_binomial::NodeBinomialLimit;
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
    static ref OUT_OF_BOUNDS_RETURN_VALUE: RegisterValue = RegisterValue::zero();
    static ref BIGINT_ZERO: BigInt = BigInt::zero();
}

/// The register 0 is for input data.
const INPUT_REGISTER: u64 = 0;

/// The register 0 is for output data.
const OUTPUT_REGISTER: u64 = 0;


const MAX_NUMBER_OF_REGISTERS: u64 = 10000;


#[derive(Clone)]
pub struct ProgramState {
    register_vec: Vec<RegisterValue>,
    memory_full: HashMap<u64, BigInt>,
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
        // Register 0 is for output value
        // So there must be a least 1 register.
        let register_count: u8 = u8::max(register_count, 1);

        let mut register_vec: Vec<RegisterValue> = vec!();
        for _ in 0..register_count {
            register_vec.push(RegisterValue::zero());
        }

        let check_value: BoxCheckValue = node_register_limit.create_boxed_check_value();

        Self {
            register_vec: register_vec,
            memory_full: HashMap::new(),
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
        // panic!("TODO: replace u8 addresses with u64");
        let index = register_index.0 as usize;
        if index >= self.register_vec.len() {
            // Accessing a register outside bounds always returns zero
            return &OUT_OF_BOUNDS_RETURN_VALUE;
        }    
        return &self.register_vec[index];
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

    fn register_index_from(&self, value: i64) -> Result<RegisterIndex, EvalError> {
        if value < 0 {
            debug!("value out of range, too low");
            return Err(EvalError::CannotConvertBigIntToRegisterIndex);
        }
        if value > 255 {
            debug!("value out of range, too high");
            return Err(EvalError::CannotConvertBigIntToRegisterIndex);
        }
        Ok(RegisterIndex(value as u8))
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
            ParameterType::Register => {
                if get_address {
                    match parameter.parameter_value.to_bigint() {
                        Some(value) => { return Ok(value); }
                        None => { return Err(EvalError::CannotConvertParameterValueToBigInt); }
                    }
                }
                let inner_value: &BigInt = self.get_i64(parameter.parameter_value)?;
                // let index: RegisterIndex = self.register_index_from(parameter.parameter_value)?;
                // let value: &RegisterValue = self.get_register_value_ref(&index);
                // let inner_value: BigInt = value.0.clone();
                return Ok(inner_value.clone());
            },
            ParameterType::Indirect => {
                let inner_value: &BigInt = self.get_i64(parameter.parameter_value)?;

                // let index: RegisterIndex = self.register_index_from(parameter.parameter_value)?;
                // let value: &RegisterValue = self.get_register_value_ref(&index);
                if get_address {
                    // let inner_value: BigInt = value.0.clone();
                    return Ok(inner_value.clone());
                }
                // let optional_inner_value: Option<i64> = value.try_to_i64();
                // let inner_value: i64 = match optional_inner_value {
                //     Some(value) => value,
                //     None => {
                //         return Err(EvalError::CannotConvertBigIntToRegisterIndex);
                //     }
                // };
                let inner_value2: &BigInt = self.get_bigint(inner_value)?;
                // let index2: RegisterIndex = self.register_index_from(inner_value)?;
                // let value2: &RegisterValue = self.get_register_value_ref(&index2);
                // let inner_value: BigInt = value2.0.clone();
                return Ok(inner_value2.clone());
            }
        }
    }
    
    pub fn set(&mut self, parameter: &InstructionParameter, set_value: BigInt) -> Result<(), EvalError> {
        match parameter.parameter_type {
            ParameterType::Constant => {
                return Err(EvalError::CannotSetValueOfConstant);
            },
            ParameterType::Register => {
                self.set_i64(parameter.parameter_value, set_value)?;
                // let index: RegisterIndex = self.register_index_from(parameter.parameter_value)?;
                // self.set_register_value(index, RegisterValue(set_value));
                return Ok(());
            },
            ParameterType::Indirect => {
                let address_ref: &BigInt = self.get_i64(parameter.parameter_value)?;
                let address: BigInt = address_ref.clone();
                self.set_bigint(&address, set_value)?;

                // let index: RegisterIndex = self.register_index_from(parameter.parameter_value)?;
                // let value: &RegisterValue = self.get_register_value_ref(&index);
                // let optional_inner_value: Option<i64> = value.try_to_i64();
                // let inner_value: i64 = match optional_inner_value {
                //     Some(value) => value,
                //     None => {
                //         return Err(EvalError::CannotConvertBigIntToRegisterIndex);
                //     }
                // };
                // let index2: RegisterIndex = self.register_index_from(inner_value)?;
                // self.set_register_value(index2, RegisterValue(set_value));
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

    pub fn get_output_value_legacy(&self) -> &RegisterValue {
        // panic!("TODO: replace u8 addresses with u64");
        assert!(self.register_vec.len() >= 1);
        return &self.register_vec[OUTPUT_REGISTER as usize];
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

    pub fn set_register_value(&mut self, register_index: RegisterIndex, register_value: RegisterValue) {
        // panic!("TODO: replace u8 addresses with u64");
        let index = register_index.0 as usize;
        if index >= self.register_vec.len() {
            panic!("set_register_value. index is outside the number of registers.");
        }
        self.register_vec[index] = register_value;
    }

    /// Write a value to register 0, the input register.
    pub fn set_input_value(&mut self, register_value: &RegisterValue) {
        self.memory_full.insert(INPUT_REGISTER, register_value.0.clone());
    }
   
    pub fn set_register_range_to_zero(&mut self, register_index: RegisterIndex, count: u8) {
        // panic!("TODO: replace u8 addresses with u64");
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

    /// Make the internal state human readable
    pub fn register_vec_to_string(&self) -> String {
        // panic!("TODO: replace u8 addresses with u64");
        let strings: Vec<String> = self.register_vec.iter().map(|register_value| {
            register_value.0.to_string()
        }).collect();
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

    pub fn is_less_range_legacy(&self, other_state: &ProgramState, register_index: RegisterIndex, range_length: u8) -> bool {
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

    /// Similar to `is_less_range()`, but with a range of 1.
    /// 
    /// This function is simpler than its counterpart `is_less_range`.
    pub fn is_less_single(&self, other_state: &ProgramState, register_index: RegisterIndex) -> bool {
        // panic!("TODO: replace u8 addresses with u64");
        self.is_less_range(other_state, register_index, 1)
    }

    pub fn is_less_single_legacy(&self, other_state: &ProgramState, register_index: RegisterIndex) -> bool {
        // panic!("TODO: replace u8 addresses with u64");
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
    fn test_10001_initialize_with_too_few_registers() {
        let state = ProgramState::new(
            0, 
            RunMode::Silent, 
            1000, 
            NodeRegisterLimit::Unlimited,
            NodeBinomialLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            NodePowerLimit::Unlimited,
        );
        assert_eq!(state.register_vec_to_string(), "[0]")
    }

    #[test]
    fn test_10002_initialize_with_the_minimum_number_of_registers() {
        let state = ProgramState::new(
            1, 
            RunMode::Silent, 
            1000,
            NodeRegisterLimit::Unlimited,
            NodeBinomialLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            NodePowerLimit::Unlimited,
        );
        assert_eq!(state.register_vec_to_string(), "[0]")
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
