use super::{EvalError, Node, ProgramState, RegisterIndex, RegisterValue};
use num_bigint::BigInt;
use num_traits::Zero;

fn perform_operation(x: RegisterValue, y: RegisterValue) -> Result<RegisterValue,EvalError> {
    let yy: &BigInt = &y.0;
    if yy.is_zero() {
        debug!("NodeModulo, division by zero");
        return Err(EvalError::DivisionByZero);
    }
    let xx: &BigInt = &x.0;
    Ok(RegisterValue(xx % yy))
}

pub struct NodeModuloRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeModuloRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeModuloRegister {
    fn shorthand(&self) -> &str {
        "modulo register"
    }

    fn formatted_instruction(&self) -> String {
        format!("mod {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState) {
        match self.eval_advanced(state) {
            Ok(_) => {},
            Err(err) => {
                panic!("Unable to perform operation. error: {:?}", err);
            }
        }
    }

    fn eval_advanced(&self, state: &mut ProgramState) -> Result<(), EvalError> {
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

pub struct NodeModuloConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeModuloConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeModuloConstant {
    fn shorthand(&self) -> &str {
        "modulo constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("mod {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState) {
        match self.eval_advanced(state) {
            Ok(_) => {},
            Err(err) => {
                panic!("Unable to perform operation. error: {:?}", err);
            }
        }
    }

    fn eval_advanced(&self, state: &mut ProgramState) -> Result<(), EvalError> {
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
        assert_eq!(process(100, 0), "BOOM");
        assert_eq!(process(-100, 0), "BOOM");

        assert_eq!(process(50, 10), "0");
        assert_eq!(process(100, 1), "0");
        assert_eq!(process(-1, -1), "0");
        assert_eq!(process(3, -3), "0");
        assert_eq!(process(-3, 3), "0");

        assert_eq!(process(10, 3), "1");
        assert_eq!(process(99, 10), "9");
        assert_eq!(process( 999,  10), "9");
        assert_eq!(process(-999,  10), "-9");
        assert_eq!(process(-999, -10), "-9");
        assert_eq!(process( 999, -10), "9");
    }
}
