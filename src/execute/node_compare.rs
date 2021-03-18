use super::{EvalError, Node, ProgramState, RegisterIndex, RegisterValue};
use num_bigint::BigInt;

fn perform_operation(x: RegisterValue, y: RegisterValue) -> RegisterValue {
    let xx: &BigInt = &x.0;
    let yy: &BigInt = &y.0;
    if xx == yy {
        RegisterValue::one()
    } else {
        RegisterValue::zero()
    }
}

pub struct NodeCompareRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeCompareRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeCompareRegister {
    fn shorthand(&self) -> &str {
        "compare register"
    }

    fn formatted_instruction(&self) -> String {
        format!("cmp {},{}", self.target, self.source)
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

pub struct NodeCompareConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeCompareConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeCompareConstant {
    fn shorthand(&self) -> &str {
        "compare constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("cmp {},{}", self.target, self.source)
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
        assert_eq!(process(100, 100), "1");
        assert_eq!(process(-33, -33), "1");
        assert_eq!(process(-1, 1), "0");
        assert_eq!(process(100, -100), "0");
        assert_eq!(process(0, 1), "0");
    }
}
