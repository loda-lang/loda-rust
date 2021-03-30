use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use num_bigint::{BigInt, ToBigInt};
use num_traits::{ToPrimitive, Signed};

pub struct NodeClearConstant {
    target: RegisterIndex,
    clear_count: u8,
}

impl NodeClearConstant {
    pub fn new(target: RegisterIndex, clear_count: u8) -> Self {
        Self {
            target: target,
            clear_count: clear_count,
        }
    }
}

impl Node for NodeClearConstant {
    fn shorthand(&self) -> &str {
        "clear constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("clr {},{}", self.target, self.clear_count)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        state.set_register_range_to_zero(self.target.clone(), self.clear_count);
        Ok(())
    }

    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {
        // This operation does not affect the number of registers to be allocated.
        // The default value of an uninitialized register is zero.
        // And accessing a register outside the allocated registers just yields zero.
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
    fn shorthand(&self) -> &str {
        "clear register"
    }

    fn formatted_instruction(&self) -> String {
        format!("clr {},{}", self.target, self.register_with_clear_count)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let value: RegisterValue = state.get_register_value(self.register_with_clear_count.clone());
        let value_inner: &BigInt = &value.0;
        let clear_count: u8;
        let max_clear_count_bigint: BigInt = 255.to_bigint().unwrap();
        if value_inner.is_positive() {
            if value_inner > &max_clear_count_bigint {
                error!("Range length is beyond the ProgramState max length. Clamping range to 255.");
                clear_count = 255;
            } else {
                // Value is between 0 and 255, so it can be casted to an unsigned byte.
                clear_count = value_inner.to_u8().unwrap();
            }
        } else {
            // Value is negative. Clamp to 0 length.
            clear_count = 0;
        }
        debug!("clear_count: {}", clear_count);
        
        state.set_register_range_to_zero(self.target.clone(), clear_count);
        Ok(())
    }
    
    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {
        // This operation does not affect the number of registers to be allocated.
        // The default value of an uninitialized register is zero.
        // And accessing a register outside the allocated registers just yields zero.
    }
}
