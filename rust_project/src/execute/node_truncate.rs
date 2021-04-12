use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use std::collections::HashSet;
use num_bigint::BigInt;
use num_traits::Signed;

fn perform_operation(x: &RegisterValue, y: &RegisterValue) -> RegisterValue {
    let a: &BigInt = &x.0;
    let b: &BigInt = &y.0;
    let value: BigInt = a - b;
    if value.is_positive() {
        RegisterValue(value)
    } else {
        RegisterValue::zero()
    }
}

pub struct NodeTruncateRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeTruncateRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeTruncateRegister {
    fn formatted_instruction(&self) -> String {
        format!("trn {},{}", self.target, self.source)
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
        if self.target == self.source {
            // Truncating itself with itself, always result in 0
            register_set.remove(&self.target);
            return;
        }
        if register_set.contains(&self.source) {
            register_set.insert(self.target.clone());
        }
    }    
}

pub struct NodeTruncateConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeTruncateConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeTruncateConstant {
    fn formatted_instruction(&self) -> String {
        format!("trn {},{}", self.target, self.source)
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
        // greater than 0
        assert_eq!(process(1, 0), "1");
        assert_eq!(process(22, 0), "22");
        assert_eq!(process(22, 1), "21");
        assert_eq!(process(22, 2), "20");
        assert_eq!(process(22, 21), "1");
        assert_eq!(process(-22, -23), "1");

        // zero
        assert_eq!(process(0, 0), "0");
        assert_eq!(process(22, 22), "0");
        assert_eq!(process(-22, -22), "0");

        // less than 0
        assert_eq!(process(22, 23), "0");
        assert_eq!(process(22, 100), "0");
        assert_eq!(process(-22, 100), "0");
    }
}
