use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use num_bigint::BigInt;
use num_traits::Zero;

fn perform_operation(x: RegisterValue, y: RegisterValue) -> Result<RegisterValue,EvalError> {
    let yy: &BigInt = &y.0;
    if yy.is_zero() {
        debug!("NodeDivideIf, division by zero");
        return Err(EvalError::DivisionByZero);
    }
    let xx: &BigInt = &x.0;
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
    fn test_10000_zero() {
        assert_eq!(process(100, 0), "BOOM");
        assert_eq!(process(-100, 0), "BOOM");
    }

    #[test]
    fn test_10001_remainder_zero() {
        assert_eq!(process(50, 10), "5");
        assert_eq!(process(100, 1), "100");
        assert_eq!(process(42, -1), "-42");
        assert_eq!(process(-1, -1), "1");
        assert_eq!(process(3, -3), "-1");
        assert_eq!(process(-3, 3), "-1");
    }

    #[test]
    fn test_10002_cannot_be_divided() {
        assert_eq!(process(33, 10), "33");
        assert_eq!(process(100, 33), "100");
        assert_eq!(process(-100, -33), "-100");
    }
}
