use super::{CreateError, EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use crate::parser::InstructionParameter;
use std::collections::HashSet;
use num_bigint::{BigInt, ToBigInt};
use num_traits::{ToPrimitive, Signed, Zero};

pub struct NodeClear {
    target: InstructionParameter,
    source: InstructionParameter,
}

impl NodeClear {
    pub fn new(target: InstructionParameter, source: InstructionParameter) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeClear {
    fn formatted_instruction(&self) -> String {
        format!("clr {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let target: BigInt = state.get(&self.target, true)?;
        let target_u64: u64 = match target.to_u64() {
            Some(value) => value,
            None => {
                return Err(EvalError::CannotConvertBigIntToAddress);
            }
        };

        let source: BigInt = state.get(&self.source, false)?;
        if source.is_negative() || source.is_zero() {
            // clear instruction with source being negative or zero. Does nothing.
            return Ok(());
        }
        let source_u64: u64 = match source.to_u64() {
            Some(value) => value,
            None => {
                return Err(EvalError::ClearRangeLengthExceedsLimit);
            }
        };

        state.set_register_range_to_zero(RegisterIndex(target_u64), source_u64)?;
        Ok(())
    }

    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {
        // This operation does not affect the number of registers to be allocated.
        // The default value of an uninitialized register is zero.
        // And accessing a register outside the allocated registers just yields zero.
    }

    fn live_register_indexes(&self, _register_set: &mut HashSet<RegisterIndex>) {
    }    
}

pub struct NodeClearConstant {
    target: RegisterIndex,
    clear_count: u8,
}

impl NodeClearConstant {
    pub fn create(target: RegisterIndex, source: RegisterValue) -> Result<Self, CreateError> {
        if source.0.is_negative() {
            // clear instruction with source being a negative number. Makes no sense.
            return Err(CreateError::ClearRangeLengthMustBeNonNegative);
        }
        let limit_bigint: BigInt = 255.to_bigint().unwrap();
        if source.0 > limit_bigint {
            // clear instruction with source being an unusual high value.
            return Err(CreateError::ClearRangeLengthExceedsLimit);
        }
        let clear_count: u8 = source.0.to_u8().unwrap();
        let node = Self::new(target, clear_count);
        Ok(node)
    }

    pub fn new(target: RegisterIndex, clear_count: u8) -> Self {
        Self {
            target: target,
            clear_count: clear_count,
        }
    }
}

impl Node for NodeClearConstant {
    fn formatted_instruction(&self) -> String {
        format!("clr {},{}", self.target, self.clear_count)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        state.set_register_range_to_zero(self.target.clone(), self.clear_count as u64)?;
        Ok(())
    }

    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {
        // This operation does not affect the number of registers to be allocated.
        // The default value of an uninitialized register is zero.
        // And accessing a register outside the allocated registers just yields zero.
    }

    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        let mut index: u64 = self.target.0;
        for _ in 0..(self.clear_count) {
            let register_index = RegisterIndex(index);
            register_set.remove(&register_index);
            if index == u64::MAX {
                break;
            }
            index += 1;
        }
    }    
}

pub struct NodeClearRegister {
    target: RegisterIndex,
    register_with_clear_count: RegisterIndex,
}

impl NodeClearRegister {
    pub fn new(target: RegisterIndex, register_with_clear_count: RegisterIndex) -> Self {
        Self {
            target: target,
            register_with_clear_count: register_with_clear_count,
        }
    }
}

impl Node for NodeClearRegister {
    fn formatted_instruction(&self) -> String {
        format!("clr {},{}", self.target, self.register_with_clear_count)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        //panic!("TODO: replace u8 addresses with u64");
        let value_inner: &BigInt = state.get_u64(self.register_with_clear_count.0 as u64);
        let clear_count: u8;
        let max_clear_count_bigint: BigInt = 255.to_bigint().unwrap();
        if value_inner.is_positive() {
            if value_inner > &max_clear_count_bigint {
                // debug!("Range length is beyond the ProgramState max length. Clamping range to 255.");
                clear_count = 255;
            } else {
                // Value is between 0 and 255, so it can be casted to an unsigned byte.
                clear_count = value_inner.to_u8().unwrap();
            }
        } else {
            // Value is negative. Clamp to 0 length.
            clear_count = 0;
        }
        // debug!("clear_count: {}", clear_count);
        
        //panic!("TODO: replace u8 addresses with u64");
        state.set_register_range_to_zero(self.target.clone(), clear_count as u64)?;
        Ok(())
    }
    
    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {
        // This operation does not affect the number of registers to be allocated.
        // The default value of an uninitialized register is zero.
        // And accessing a register outside the allocated registers just yields zero.
    }

    fn live_register_indexes(&self, _register_set: &mut HashSet<RegisterIndex>) {
        // It cannot be determined if this clears the live registers
        // Registers lower than the target register is unaffected by clear.
        // Registers greater than or equal to the target register may be cleared.
    }
}
