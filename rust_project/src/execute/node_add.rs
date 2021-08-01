use super::{EvalError, Node};
use super::{ProgramCache, ProgramState};
use super::{RegisterIndex, RegisterValue};
use super::{BoxCheckValue, CheckValueError, PerformCheckValue};
use std::collections::HashSet;
use num_bigint::BigInt;

impl From<CheckValueError> for EvalError {
    fn from(_err: CheckValueError) -> EvalError {
        EvalError::AddOutOfRange
    }
}

fn perform_operation(check: &BoxCheckValue, x: &RegisterValue, y: &RegisterValue) -> Result<RegisterValue, CheckValueError> {
    let xx: &BigInt = &x.0;
    check.input(xx)?;

    let yy: &BigInt = &y.0;
    check.input(yy)?;

    let zz: BigInt = xx + yy;
    check.output(&zz)?;

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
        let value = perform_operation(state.check_value(), lhs, rhs)?;
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
        let value = perform_operation(state.check_value(), lhs, rhs)?;
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
    use super::super::CheckValueLimitBits;

    fn process(left: i64, right: i64) -> String {
        let check_value: BoxCheckValue = Box::new(CheckValueLimitBits::new(32));
        let result = perform_operation(
            &check_value,
            &RegisterValue::from_i64(left),
            &RegisterValue::from_i64(right)
        );
        match result {
            Ok(value) => return value.to_string(),
            Err(CheckValueError::InputOutOfRange) => return "BOOM-INPUT".to_string(),
            Err(CheckValueError::OutputOutOfRange) => return "BOOM-OUTPUT".to_string(),
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
