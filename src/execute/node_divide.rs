use super::{Node,RegisterIndex,RegisterValue,ProgramState};
use num_bigint::BigInt;
use num_traits::Zero;

fn perform_operation(x: RegisterValue, y: RegisterValue) -> RegisterValue {
    let xx: &BigInt = &x.0;
    let yy: &BigInt = &y.0;
    // TODO: deal with infinity
    if yy.is_zero() {
        // TODO: indicate division by zero
        error!("division by zero");
        return RegisterValue::from_i64(0xfffffff)
    }
    RegisterValue(xx / yy)
}

pub struct NodeDivideRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeDivideRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeDivideRegister {
    fn shorthand(&self) -> &str {
        "divide register"
    }

    fn formatted_instruction(&self) -> String {
        format!("div {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState) {
        let lhs: RegisterValue = state.get_register_value(self.target.clone());
        let rhs: RegisterValue = state.get_register_value(self.source.clone());
        let value = perform_operation(lhs, rhs);
        state.set_register_value(self.target.clone(), value);
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
        register_vec.push(self.source.clone());
    }
}

pub struct NodeDivideConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeDivideConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeDivideConstant {
    fn shorthand(&self) -> &str {
        "divide constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("div {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState) {
        let lhs: RegisterValue = state.get_register_value(self.target.clone());
        let rhs: RegisterValue = self.source.clone();
        let value = perform_operation(lhs, rhs);
        state.set_register_value(self.target.clone(), value);
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
        let v = value.to_i64();
        if v >= 0xffffff {
            return "BOOM".to_string();
        }
        v.to_string()
    }

    #[test]
    fn test_10000() {
        assert_eq!(process(50, 10), "5");
        assert_eq!(process(100, 1), "100");
        assert_eq!(process(-1, -1), "1");
        assert_eq!(process(3, -3), "-1");
        assert_eq!(process(-3, 3), "-1");
        assert_eq!(process(100, 0), "BOOM");
        assert_eq!(process(-100, 0), "BOOM");
    }
}
