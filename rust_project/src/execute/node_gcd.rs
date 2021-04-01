use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::Zero;

fn perform_operation(x: RegisterValue, y: RegisterValue) -> Result<RegisterValue,EvalError> {
    let xx: &BigInt = &x.0;
    let yy: &BigInt = &y.0;
    if xx.is_zero() && yy.is_zero() {
        debug!("NodeGCD, both parameters must not be zero at the same time");
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
        let lhs: RegisterValue = state.get_register_value(self.target.clone());
        let rhs: RegisterValue = state.get_register_value(self.source.clone());
        let value: RegisterValue = perform_operation(lhs, rhs)?;
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
        register_vec.push(self.source.clone());
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
        let lhs: RegisterValue = state.get_register_value(self.target.clone());
        let rhs: RegisterValue = self.source.clone();
        let value: RegisterValue = perform_operation(lhs, rhs)?;
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
        let result = perform_operation(
            RegisterValue::from_i64(left),
            RegisterValue::from_i64(right)
        );
        match result {
            Ok(value) => value.to_string(),
            Err(_) => "BOOM".to_string()
        }
    }

    #[test]
    fn test_10000() {
        assert_eq!(process(0, 0), "BOOM");
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
}
