use super::{EvalError, Node, ProgramState, RegisterIndex, RegisterValue};
use num_bigint::BigInt;

fn perform_operation(x: RegisterValue, y: RegisterValue) -> RegisterValue {
    let xx: &BigInt = &x.0;
    let yy: &BigInt = &y.0;
    RegisterValue(xx - yy)
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
    fn shorthand(&self) -> &str {
        "subtract register"
    }

    fn formatted_instruction(&self) -> String {
        format!("sub {},{}", self.target, self.source)
    }

    fn eval_advanced(&self, state: &mut ProgramState) -> Result<(), EvalError> {
        let lhs: RegisterValue = state.get_register_value(self.target.clone());
        let rhs: RegisterValue = state.get_register_value(self.source.clone());
        let value = perform_operation(lhs, rhs);
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
        register_vec.push(self.source.clone());
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
    fn shorthand(&self) -> &str {
        "subtract constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("sub {},{}", self.target, self.source)
    }

    fn eval_advanced(&self, state: &mut ProgramState) -> Result<(), EvalError> {
        let lhs: RegisterValue = state.get_register_value(self.target.clone());
        let rhs: RegisterValue = self.source.clone();
        let value = perform_operation(lhs, rhs);
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
        let value: RegisterValue = perform_operation(
            RegisterValue::from_i64(left),
            RegisterValue::from_i64(right)
        );
        value.to_string()
    }

    #[test]
    fn test_10000() {
        assert_eq!(process(1001, 1), "1000");
        assert_eq!(process(999, -1), "1000");
        assert_eq!(process(-100, -100), "0");
        assert_eq!(process(100, 300), "-200");
        assert_eq!(process(0, 10), "-10");
    }
}
