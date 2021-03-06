use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use super::{BoxCheckValue, PerformCheckValue};
use std::collections::HashSet;
use num_bigint::BigInt;
use num_traits::Zero;

fn perform_operation(check: &BoxCheckValue, x: &RegisterValue, y: &RegisterValue) -> Result<RegisterValue,EvalError> {
    let yy: &BigInt = &y.0;
    check.input(yy)?;
    if yy.is_zero() {
        return Ok(x.clone());
    }

    let xx: &BigInt = &x.0;
    check.input(xx)?;

    let remain: BigInt = xx % yy;
    if remain.is_zero() {
        return Ok(RegisterValue(xx / yy));
    } else {
        return Ok(x.clone());
    }
}

pub struct NodeDivideIfRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeDivideIfRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeDivideIfRegister {
    fn formatted_instruction(&self) -> String {
        format!("dif {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = state.get_register_value_ref(&self.source);
        let value: RegisterValue = perform_operation(state.check_value(), lhs, rhs)?;
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

pub struct NodeDivideIfConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeDivideIfConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeDivideIfConstant {
    fn formatted_instruction(&self) -> String {
        format!("dif {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = &self.source;
        let value: RegisterValue = perform_operation(state.check_value(), lhs, rhs)?;
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
            Ok(value) => value.to_string(),
            Err(EvalError::InputOutOfRange) => "BOOM-INPUT".to_string(),
            Err(_) => return "BOOM-OTHER".to_string()
        }
    }

    #[test]
    fn test_10000_remainder_zero() {
        assert_eq!(process(50, 10), "5");
        assert_eq!(process(100, 1), "100");
        assert_eq!(process(42, -1), "-42");
        assert_eq!(process(-1, -1), "1");
        assert_eq!(process(3, -3), "-1");
        assert_eq!(process(-3, 3), "-1");
    }

    #[test]
    fn test_10001_cannot_be_divided() {
        assert_eq!(process(33, 10), "33");
        assert_eq!(process(100, 33), "100");
        assert_eq!(process(-100, -33), "-100");
    }

    #[test]
    fn test_10002_divisionbyzero() {
        assert_eq!(process(100, 0), "100");
        assert_eq!(process(0, 0), "0");
        assert_eq!(process(-100, 0), "-100");
    }

    #[test]
    fn test_10003_inputoutofrange() {
        assert_eq!(process(0x7fffffff, 0x7fffffff), "1");
        assert_eq!(process(-0x7fffffff, -0x7fffffff), "1");
        assert_eq!(process(0x80000000, 1), "BOOM-INPUT");
        assert_eq!(process(-0x80000000, 1), "BOOM-INPUT");
        assert_eq!(process(0x80000001, 2), "BOOM-INPUT");
        assert_eq!(process(-0x80000001, 2), "BOOM-INPUT");
        assert_eq!(process(1, 0x7fffffff), "1");
        assert_eq!(process(1, -0x7fffffff), "1");
        assert_eq!(process(1, 0x80000000), "BOOM-INPUT");
        assert_eq!(process(1, -0x80000000), "BOOM-INPUT");
        assert_eq!(process(1, 0x80000001), "BOOM-INPUT");
        assert_eq!(process(1, -0x80000001), "BOOM-INPUT");
    }
}
