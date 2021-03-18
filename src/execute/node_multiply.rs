use super::{EvalError, Node, ProgramState, RegisterIndex, RegisterValue};
use num_bigint::BigInt;

fn perform_operation(x: RegisterValue, y: RegisterValue) -> RegisterValue {
    let xx: &BigInt = &x.0;
    let yy: &BigInt = &y.0;
    RegisterValue(xx * yy)
}

pub struct NodeMultiplyRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeMultiplyRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMultiplyRegister {
    fn shorthand(&self) -> &str {
        "multiply register"
    }

    fn formatted_instruction(&self) -> String {
        format!("mul {},{}", self.target, self.source)
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

pub struct NodeMultiplyConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeMultiplyConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMultiplyConstant {
    fn shorthand(&self) -> &str {
        "multiply constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("mul {},{}", self.target, self.source)
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
        assert_eq!(process(1, 1000), "1000");
        assert_eq!(process(1000, 1), "1000");
        assert_eq!(process(-1, -1), "1");
        assert_eq!(process(0, 0), "0");
        assert_eq!(process(0, 1), "0");
        assert_eq!(process(1, 0), "0");
        assert_eq!(process(1, 1), "1");
        assert_eq!(process(-500, 2), "-1000");
    }
}
