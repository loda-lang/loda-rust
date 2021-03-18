use super::{EvalError, Node, ProgramState, RegisterIndex, RegisterValue};
use num_bigint::BigInt;
use num_traits::{Zero, One, Signed};

fn perform_operation(x: RegisterValue, y: RegisterValue) -> Result<RegisterValue,EvalError> {
    let n: &BigInt = &x.0;
    let base: &BigInt = &y.0;
    if !n.is_positive() {  
        // Same as  `if(n <= 0)`
        debug!("NodeLogarithm, n must be 1 or greater");
        return Err(EvalError::LogDomainError);
    }
    if n.is_one() {
        return Ok(RegisterValue::zero());
    }
    if *base <= BigInt::one() {
        // Same as `if(base < 2)`
        debug!("NodeLogarithm, base must be 2 or greater");
        return Err(EvalError::LogDomainError);
    }
    let mut m = BigInt::one();
    let mut value = BigInt::zero();
    while m < *n {
        m *= base;
        value += 1;
    }
    if m != *n {
        value -= 1;
    }
    Ok(RegisterValue(value))
}

pub struct NodeLogarithmRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeLogarithmRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeLogarithmRegister {
    fn shorthand(&self) -> &str {
        "logarithm register"
    }

    fn formatted_instruction(&self) -> String {
        format!("log {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState) -> Result<(), EvalError> {
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

pub struct NodeLogarithmConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeLogarithmConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeLogarithmConstant {
    fn shorthand(&self) -> &str {
        "logarithm constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("log {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState) -> Result<(), EvalError> {
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
    fn test_10000_base2() {
        assert_eq!(process(-2, 2), "BOOM");
        assert_eq!(process(-1, 2), "BOOM");
        assert_eq!(process(0, 2), "BOOM");
        assert_eq!(process(1, 2), "0");
        assert_eq!(process(2, 2), "1");
        assert_eq!(process(3, 2), "1");
        assert_eq!(process(4, 2), "2");
        assert_eq!(process(8, 2), "3");
        assert_eq!(process(16, 2), "4");
        assert_eq!(process(31, 2), "4");
        assert_eq!(process(32, 2), "5");
    }

    #[test]
    fn test_10001_base10() {
        assert_eq!(process(-2, 10), "BOOM");
        assert_eq!(process(-1, 10), "BOOM");
        assert_eq!(process(0, 10), "BOOM");
        assert_eq!(process(1, 10), "0");
        assert_eq!(process(9, 10), "0");
        assert_eq!(process(10, 10), "1");
        assert_eq!(process(99, 10), "1");
        assert_eq!(process(100, 10), "2");
    }

    #[test]
    fn test_10002_parameters_too_low() {
        assert_eq!(process(0, 0), "BOOM");
        assert_eq!(process(100, 1), "BOOM");
        assert_eq!(process(100, 0), "BOOM");
        assert_eq!(process(100, -1), "BOOM");
        assert_eq!(process(100, -2), "BOOM");
        assert_eq!(process(100, -3), "BOOM");
        assert_eq!(process(2, -666), "BOOM");
    }

    #[test]
    fn test_10003_n_equals_one() {
        assert_eq!(process(1, 4), "0");
        assert_eq!(process(1, 3), "0");
        assert_eq!(process(1, 2), "0");
        assert_eq!(process(1, 0), "0");
        {
            // TODO: this is what should happen
            // TODO: assert_eq!(process(1, 1), "BOOM");
            // TODO: assert_eq!(process(1, -1), "BOOM");
            // TODO: assert_eq!(process(1, -666), "BOOM");
        }
        {
            // TODO: This is what actually happens, which is wrong.
            assert_eq!(process(1, 1), "0");
            assert_eq!(process(1, -1), "0");
            assert_eq!(process(1, -666), "0");
        }
    }
}
