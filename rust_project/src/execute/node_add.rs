use super::{EvalError, Node, ProgramCache, ProgramState, RegisterIndex, RegisterValue};
use std::collections::HashSet;
use num_bigint::BigInt;

fn perform_operation(x: &RegisterValue, y: &RegisterValue) -> RegisterValue {
    let xx: &BigInt = &x.0;
    let yy: &BigInt = &y.0;
    RegisterValue(xx + yy)
}

#[allow(dead_code)]
pub struct NodeAddRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeAddRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeAddRegister {
    fn formatted_instruction(&self) -> String {
        format!("add {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = state.get_register_value_ref(&self.source);
        let value = perform_operation(lhs, rhs);
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

#[allow(dead_code)]
pub struct NodeAddConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeAddConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeAddConstant {
    fn formatted_instruction(&self) -> String {
        format!("add {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = &self.source;
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
            &RegisterValue::from_i64(left),
            &RegisterValue::from_i64(right)
        );
        value.to_string()
    }

    #[test]
    fn test_10000() {
        assert_eq!(process(100, 900), "1000");
        assert_eq!(process(1001, -1), "1000");
        assert_eq!(process(-1, -1), "-2");
        assert_eq!(process(100, -100), "0");
        assert_eq!(process(-100, 100), "0");
    }
}
