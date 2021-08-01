use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use super::{BoxCheckValue, PerformCheckValue};
use std::collections::HashSet;
use num_bigint::BigInt;

fn perform_operation(check: &BoxCheckValue, x: &RegisterValue, y: &RegisterValue) -> Result<RegisterValue, EvalError> {
    let xx: &BigInt = &x.0;
    check.input(xx)?;

    let yy: &BigInt = &y.0;
    check.input(yy)?;

    let zz: BigInt = xx - yy;
    check.output(&zz)?;

    Ok(RegisterValue(zz))
}

pub struct NodeSubtractRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeSubtractRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeSubtractRegister {
    fn formatted_instruction(&self) -> String {
        format!("sub {},{}", self.target, self.source)
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
        if self.target == self.source {
            // Subtracting itself from itself, always result in 0
            register_set.remove(&self.target);
            return;
        }
        if register_set.contains(&self.source) {
            register_set.insert(self.target.clone());
        }
    }    
}

pub struct NodeSubtractConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeSubtractConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeSubtractConstant {
    fn formatted_instruction(&self) -> String {
        format!("sub {},{}", self.target, self.source)
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
            Err(EvalError::InputOutOfRange) => return "BOOM-INPUT".to_string(),
            Err(EvalError::OutputOutOfRange) => return "BOOM-OUTPUT".to_string(),
            Err(_) => return "BOOM-OTHER".to_string()
        }
    }

    #[test]
    fn test_10000() {
        assert_eq!(process(1001, 1), "1000");
        assert_eq!(process(999, -1), "1000");
        assert_eq!(process(-100, -100), "0");
        assert_eq!(process(100, 300), "-200");
        assert_eq!(process(0, 10), "-10");
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
            assert_eq!(process(0x6fffffff, -0x10000000), "2147483647");
            assert_eq!(process(0x70000000, -0x10000000), "BOOM-OUTPUT");
        }
        {
            assert_eq!(process(-0x6fffffff, 0x10000000), "-2147483647");
            assert_eq!(process(-0x70000000, 0x10000000), "BOOM-OUTPUT");
        }
    }
}
