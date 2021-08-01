use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use super::{BoxCheckValue, PerformCheckValue};
use std::collections::HashSet;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::Zero;

fn perform_operation(check: &BoxCheckValue, x: &RegisterValue, y: &RegisterValue) -> Result<RegisterValue,EvalError> {
    let xx: &BigInt = &x.0;
    check.input(xx)?;

    let yy: &BigInt = &y.0;
    check.input(yy)?;

    if xx.is_zero() && yy.is_zero() {
        return Err(EvalError::GCDDomainError);
    }
    // https://en.wikipedia.org/wiki/Binary_GCD_algorithm
    let zz = xx.gcd(yy);
    Ok(RegisterValue(zz))
}

pub struct NodeGCDRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeGCDRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeGCDRegister {
    fn formatted_instruction(&self) -> String {
        format!("gcd {},{}", self.target, self.source)
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

pub struct NodeGCDConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeGCDConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeGCDConstant {
    fn formatted_instruction(&self) -> String {
        format!("gcd {},{}", self.target, self.source)
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
            Err(EvalError::InputOutOfRange) => return "BOOM-INPUT".to_string(),
            Err(EvalError::GCDDomainError) => return "BOOM-ZERO".to_string(),
            Err(_) => return "BOOM-OTHER".to_string()
        }
    }

    #[test]
    fn test_10000() {
        assert_eq!(process(0, 0), "BOOM-ZERO");
        assert_eq!(process(0, 1), "1");
        assert_eq!(process(1, 0), "1");
        assert_eq!(process(1, 1), "1");
        assert_eq!(process(2, 2), "2");
        assert_eq!(process(6, 4), "2");
        assert_eq!(process(100, 55), "5");
        assert_eq!(process(-100, 55), "5");
        assert_eq!(process(-100, -55), "5");
        assert_eq!(process(-100, 1), "1");
        assert_eq!(process(43, 41), "1");
    }

    #[test]
    fn test_10001_outofrange() {
        assert_eq!(process(0x80000000, 1), "BOOM-INPUT");
        assert_eq!(process(-0x80000000, 1), "BOOM-INPUT");
        assert_eq!(process(0x80000000, 0x80000000), "BOOM-INPUT");
        assert_eq!(process(-0x80000000, -0x80000000), "BOOM-INPUT");
        assert_eq!(process(1, 0x80000000), "BOOM-INPUT");
        assert_eq!(process(1, -0x80000000), "BOOM-INPUT");
    }
}
