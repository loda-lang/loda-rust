use super::{EvalError, Node, ProgramCache, ProgramState, RegisterIndex, RegisterValue};
use std::collections::HashSet;
use num_bigint::BigInt;

struct OperationUnlimited {}

struct OperationLimited32Bit {}

struct OperationLimitedNBit {
    max_bits: u32,
}

enum CheckValueError {
    OutOfRange,
}

trait CheckValue {
    fn check_value(&self, value: &BigInt) -> Result<(), CheckValueError>;
}

impl CheckValue for OperationLimited32Bit {
    fn check_value(&self, value: &BigInt) -> Result<(), CheckValueError> {
        if value.bits() >= 32 {
            return Err(CheckValueError::OutOfRange);
        }
        Ok(())
    }
}

impl CheckValue for OperationLimitedNBit {
    fn check_value(&self, value: &BigInt) -> Result<(), CheckValueError> {
        if value.bits() >= self.max_bits.into() {
            return Err(CheckValueError::OutOfRange);
        }
        Ok(())
    }
}

impl CheckValue for OperationUnlimited {
    fn check_value(&self, _value: &BigInt) -> Result<(), CheckValueError> {
        Ok(())
    }
}

impl From<CheckValueError> for EvalError {
    fn from(_err: CheckValueError) -> EvalError {
        EvalError::AddOutOfRange
    }
}

fn perform_operation<T: CheckValue>(checker: &T, x: &RegisterValue, y: &RegisterValue) -> Result<RegisterValue, EvalError> {
    let xx: &BigInt = &x.0;
    let yy: &BigInt = &y.0;
    checker.check_value(xx)?;
    checker.check_value(yy)?;
    let zz: BigInt = xx + yy;
    checker.check_value(&zz)?;
    Ok(RegisterValue(zz))
}


enum AddError {
    InputOutOfRange,
    OutputOutOfRange,
}

impl From<AddError> for EvalError {
    fn from(_err: AddError) -> EvalError {
        EvalError::AddOutOfRange
    }
}

fn old_perform_operation(x: &RegisterValue, y: &RegisterValue) -> Result<RegisterValue, AddError> {
    let xx: &BigInt = &x.0;
    let yy: &BigInt = &y.0;
    if xx.bits() >= 32 {
        return Err(AddError::InputOutOfRange);
    }
    if yy.bits() >= 32 {
        return Err(AddError::InputOutOfRange);
    }
    let zz: BigInt = xx + yy;
    if zz.bits() >= 32 {
        return Err(AddError::OutputOutOfRange);
    }
    Ok(RegisterValue(zz))
}

#[allow(dead_code)]
pub struct NodeAddRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeAddRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeAddRegister {
    fn formatted_instruction(&self) -> String {
        format!("add {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = state.get_register_value_ref(&self.source);
        // let checker = OperationUnlimited {};
        let checker = OperationLimited32Bit {};
        let value = perform_operation(&checker, lhs, rhs)?;
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
        register_vec.push(self.source.clone());
    }

    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        if register_set.contains(&self.source) {
            register_set.insert(self.target.clone());
        }
    }
}

#[allow(dead_code)]
pub struct NodeAddConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeAddConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeAddConstant {
    fn formatted_instruction(&self) -> String {
        format!("add {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = &self.source;
        let value = old_perform_operation(lhs, rhs)?;
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(left: i64, right: i64) -> String {
        let result = old_perform_operation(
            &RegisterValue::from_i64(left),
            &RegisterValue::from_i64(right)
        );
        match result {
            Ok(value) => return value.to_string(),
            Err(AddError::InputOutOfRange) => return "BOOM-INPUT".to_string(),
            Err(AddError::OutputOutOfRange) => return "BOOM-OUTPUT".to_string()
        }
    }

    #[test]
    fn test_10000() {
        assert_eq!(process(100, 900), "1000");
        assert_eq!(process(1001, -1), "1000");
        assert_eq!(process(-1, -1), "-2");
        assert_eq!(process(100, -100), "0");
        assert_eq!(process(-100, 100), "0");
    }

    #[test]
    fn test_10001_out_of_range() {
        {
            assert_eq!(process(0x7fffffff, 0), "2147483647");
            assert_eq!(process(0x80000000, 0), "BOOM-INPUT");
            assert_eq!(process(0, 0x80000000), "BOOM-INPUT");
            assert_eq!(process(-0x80000000, 0), "BOOM-INPUT");
            assert_eq!(process(0, -0x80000000), "BOOM-INPUT");
        }
        {
            assert_eq!(process(0x6fffffff, 0x10000000), "2147483647");
            assert_eq!(process(0x70000000, 0x10000000), "BOOM-OUTPUT");
        }
        {
            assert_eq!(process(-0x6fffffff, -0x10000000), "-2147483647");
            assert_eq!(process(-0x70000000, -0x10000000), "BOOM-OUTPUT");
        }
    }
}
