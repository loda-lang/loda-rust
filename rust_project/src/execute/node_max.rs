use super::{EvalError, MyCache, Node, ProgramState, RegisterIndex, RegisterValue};
use num_bigint::BigInt;

fn perform_operation(x: RegisterValue, y: RegisterValue) -> RegisterValue {
    let xx: &BigInt = &x.0;
    let yy: &BigInt = &y.0;
    let result: &BigInt = xx.max(yy);
    RegisterValue(result.clone())
}

pub struct NodeMaxRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeMaxRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMaxRegister {
    fn shorthand(&self) -> &str {
        "max register"
    }

    fn formatted_instruction(&self) -> String {
        format!("max {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut MyCache) -> Result<(), EvalError> {
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

pub struct NodeMaxConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeMaxConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMaxConstant {
    fn shorthand(&self) -> &str {
        "max constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("max {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut MyCache) -> Result<(), EvalError> {
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
        assert_eq!(process(100, 900), "900");
        assert_eq!(process(1001, -1), "1001");
        assert_eq!(process(-1, -1), "-1");
        assert_eq!(process(100, -100), "100");
        assert_eq!(process(-100, 100), "100");
    }
}
