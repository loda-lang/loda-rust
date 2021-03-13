use super::{Node,RegisterIndex,RegisterValue,ProgramState};
use num_bigint::BigInt;
use num_traits::{ToPrimitive, One, Zero, Signed};
use num_integer::Integer;

// x raised to the power of y
// x is the base value.
// y is the power value.
// Ruby: x ** y
// Math syntax: x ^ y.
fn perform_operation(x: RegisterValue, y: RegisterValue) -> RegisterValue {
    let base: &BigInt = &x.0;
    let exponent: &BigInt = &y.0;
    
    // TODO: deal with infinity in base or exponent
    
    if base.is_zero() {
        if exponent.is_positive() {
            return RegisterValue::zero();
        }
        if exponent.is_zero() {
            return RegisterValue::one();
        }
        panic!("0 raised to a negative number, yields a divison by zero");
    }

    if base.is_one() {
        // 1^x is always 1
        return RegisterValue::one(); 
    }
    if base.abs().is_one() {
        // (-1)^x, alternates between +1 and -1
        if exponent.is_even() {
            return RegisterValue::one();
        } else {
            return RegisterValue::minus_one();
        }
    }

    if exponent.is_negative() {
        // The actual result of raising to a negative number
        // is a tiny positive number, between 0 and 1.
        // Example: 
        //  ((30) ** (-1)) => (1/30)
        //  ((-2) ** (-3)) => (1/-8)
        //  (( 2) ** (-3))  => (1/8)
        return RegisterValue::zero();
    }
    if exponent.is_one() {
        return x.clone();
    }

    // Prevent invoking pow, if the exponent is higher than an u32.
    let exponent_u32: u32 = match exponent.to_u32() {
        Some(value) => value,
        None => {
            panic!("NodePower exponent is higher than a 32bit unsigned integer. This is a max that the pow() function can handle. Aborting.");
        }
    };
    if exponent_u32 > 1000000 {
        warn!("WARNING: NodePower exponent is higher than 1000000. This is a HUGE number.");
    }
    let result: BigInt = base.pow(exponent_u32);
    RegisterValue(result)
}


pub struct NodePowerRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodePowerRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodePowerRegister {
    fn shorthand(&self) -> &str {
        "power register"
    }

    fn formatted_instruction(&self) -> String {
        format!("pow {},{}", self.target, self.source)
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

pub struct NodePowerConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodePowerConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodePowerConstant {
    fn shorthand(&self) -> &str {
        "power constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("pow {},{}", self.target, self.source)
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
    fn test_10000_exponent_zero() {
        assert_eq!(process(1, 0), "1");
        assert_eq!(process(2, 0), "1");
        assert_eq!(process(3, 0), "1");
        assert_eq!(process(4, 0), "1");
        assert_eq!(process(-1, 0), "1");
        assert_eq!(process(-2, 0), "1");
        assert_eq!(process(-3, 0), "1");
        assert_eq!(process(-4, 0), "1");
    }

    #[test]
    fn test_10001_exponent_positive() {
        assert_eq!(process(1, 1), "1");
        assert_eq!(process(2, 1), "2");
        assert_eq!(process(3, 1), "3");
        assert_eq!(process(4, 1), "4");
        assert_eq!(process(-1, 1), "-1");
        assert_eq!(process(-2, 1), "-2");
        assert_eq!(process(-3, 1), "-3");
        assert_eq!(process(-4, 1), "-4");

        assert_eq!(process(1, 2), "1");
        assert_eq!(process(2, 2), "4");
        assert_eq!(process(3, 2), "9");
        assert_eq!(process(4, 2), "16");
        assert_eq!(process(-1, 2), "1");
        assert_eq!(process(-2, 2), "4");
        assert_eq!(process(-3, 2), "9");
        assert_eq!(process(-4, 2), "16");
    }

    #[test]
    fn test_10002_exponent_negative() {
        assert_eq!(process(-3, -1), "0");
        assert_eq!(process(-2, -1), "0");
        assert_eq!(process(-1, -1), "-1");
        //assert_eq!(process(0, -1), "BOOM");
        assert_eq!(process(1, -1), "1");
        assert_eq!(process(2, -1), "0");
        assert_eq!(process(3, -1), "0");

        assert_eq!(process(-3, -2), "0");
        assert_eq!(process(-2, -2), "0");
        assert_eq!(process(-1, -2), "1");
        //assert_eq!(process(0, -2), "BOOM");
        assert_eq!(process(1, -2), "1");
        assert_eq!(process(2, -2), "0");
        assert_eq!(process(3, -2), "0");

        assert_eq!(process(-3, -3), "0");
        assert_eq!(process(-2, -3), "0");
        assert_eq!(process(-1, -3), "-1");
        //assert_eq!(process(0, -3), "BOOM");
        assert_eq!(process(1, -3), "1");
        assert_eq!(process(2, -3), "0");
        assert_eq!(process(3, -3), "0");
    }

    #[test]
    fn test_10003_minus1_plus1_alternation() {
        assert_eq!(process(-1, -4), "1");
        assert_eq!(process(-1, -3), "-1");
        assert_eq!(process(-1, -2), "1");
        assert_eq!(process(-1, -1), "-1");
        assert_eq!(process(-1,  0), "1");
        assert_eq!(process(-1,  1), "-1");
        assert_eq!(process(-1,  2), "1");
        assert_eq!(process(-1,  3), "-1");
    }

    #[test]
    #[should_panic]
    fn test_20000_way_too_high_exponent() {
        let max: u32 = u32::MAX;
        let max_plus1: i64 = (max as i64) + 1;
        perform_operation(RegisterValue::from_i64(1234), RegisterValue::from_i64(max_plus1));
    }

    #[test]
    #[should_panic]
    fn test_20001_baze0_negative_exponent_division_by_zero() {
        perform_operation(RegisterValue::zero(), RegisterValue::from_i64(-666));
    }
}
