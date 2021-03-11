use super::{Node,RegisterIndex,RegisterValue,ProgramState};
use num_bigint::BigInt;
use num_traits::{Zero, One};

fn perform_operation(x: RegisterValue, y: RegisterValue) -> RegisterValue {
    let n: &BigInt = &x.0;
    let base: &BigInt = &y.0;
    // TODO: deal with infinity

    // if ( n <= 0 ) {  // TODO: deal with negative n
    //   return NUM_INF;
    // }
    if n.is_one() {
        return RegisterValue::zero();
    }
    // if ( base < 2 ) { // TODO: deal with base below 2
    //   return NUM_INF;
    // }
    let mut m = BigInt::one();
    let mut value = BigInt::zero();
    while m < *n {
        m *= base;
        // TODO: deal with overflow
        value += 1;
    }
    if m != *n {
        value -= 1;
    }
    RegisterValue(value)
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
    fn test_10000_base2() {
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
        assert_eq!(process(1, 10), "0");
        assert_eq!(process(9, 10), "0");
        assert_eq!(process(10, 10), "1");
        assert_eq!(process(99, 10), "1");
        assert_eq!(process(100, 10), "2");
    }
}
