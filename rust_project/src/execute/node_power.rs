use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use std::collections::HashSet;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, One, Zero, Signed};
use num_integer::Integer;

// x raised to the power of y
// x is the base value.
// y is the power value.
// Ruby: x ** y
// Math syntax: x ^ y.
fn perform_operation(x: RegisterValue, y: RegisterValue) -> Result<RegisterValue,EvalError> {
    let base: &BigInt = &x.0;
    let exponent: &BigInt = &y.0;
    
    if base.is_zero() {
        if exponent.is_positive() {
            return Ok(RegisterValue::zero());
        }
        if exponent.is_zero() {
            return Ok(RegisterValue::one());
        }
        return Err(EvalError::PowerZeroDivision);
    }

    if base.is_one() {
        // 1^x is always 1
        return Ok(RegisterValue::one());
    }
    if base.abs().is_one() {
        // (-1)^x, alternates between +1 and -1
        if exponent.is_even() {
            return Ok(RegisterValue::one());
        } else {
            return Ok(RegisterValue::minus_one());
        }
    }

    if exponent.is_negative() {
        // The actual result of raising to a negative number
        // is a tiny positive number, between 0 and 1.
        // Example: 
        //  ((30) ** (-1)) => (1/30)
        //  ((-2) ** (-3)) => (1/-8)
        //  (( 2) ** (-3))  => (1/8)
        return Ok(RegisterValue::zero());
    }
    if exponent.is_one() {
        return Ok(x.clone());
    }

    // Prevent invoking pow, if the exponent is higher than an u32.
    let exponent_u32: u32 = match exponent.to_u32() {
        Some(value) => value,
        None => {
            warn!("NodePower exponent is higher than a 32bit unsigned integer. This is beyond what the pow() function can handle.");
            return Err(EvalError::PowerExponentTooHigh);
        }
    };
    if exponent_u32 > 1000000 {
        warn!("WARNING: NodePower exponent is higher than 1000000. This is a HUGE number.");
    }
    let result: BigInt = base.pow(exponent_u32);
    Ok(RegisterValue(result))
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
    fn formatted_instruction(&self) -> String {
        format!("pow {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
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

    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        if register_set.contains(&self.source) {
            register_set.insert(self.target.clone());
        } else {
            register_set.remove(&self.target);
        }
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
    fn formatted_instruction(&self) -> String {
        format!("pow {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
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
            Ok(value) => return value.to_string(),
            Err(err) => {
                match err {
                    EvalError::PowerZeroDivision => return "ZeroDivision".to_string(),
                    EvalError::PowerExponentTooHigh => return "ExponentTooHigh".to_string(),
                    _ => return "BOOM".to_string()
                }
            }
        }
    }

    #[test]
    fn test_10000_exponent_zero() {
        assert_eq!(process(-4, 0), "1");
        assert_eq!(process(-3, 0), "1");
        assert_eq!(process(-2, 0), "1");
        assert_eq!(process(-1, 0), "1");
        assert_eq!(process(0, 0), "1");
        assert_eq!(process(1, 0), "1");
        assert_eq!(process(2, 0), "1");
        assert_eq!(process(3, 0), "1");
        assert_eq!(process(4, 0), "1");
    }

    #[test]
    fn test_10001_exponent_positive() {
        assert_eq!(process(-4, 1), "-4");
        assert_eq!(process(-3, 1), "-3");
        assert_eq!(process(-2, 1), "-2");
        assert_eq!(process(-1, 1), "-1");
        assert_eq!(process(0, 1), "0");
        assert_eq!(process(1, 1), "1");
        assert_eq!(process(2, 1), "2");
        assert_eq!(process(3, 1), "3");
        assert_eq!(process(4, 1), "4");

        assert_eq!(process(-4, 2), "16");
        assert_eq!(process(-3, 2), "9");
        assert_eq!(process(-2, 2), "4");
        assert_eq!(process(-1, 2), "1");
        assert_eq!(process(0, 2), "0");
        assert_eq!(process(1, 2), "1");
        assert_eq!(process(2, 2), "4");
        assert_eq!(process(3, 2), "9");
        assert_eq!(process(4, 2), "16");
    }

    #[test]
    fn test_10002_exponent_negative() {
        assert_eq!(process(-3, -1), "0");
        assert_eq!(process(-2, -1), "0");
        assert_eq!(process(-1, -1), "-1");
        assert_eq!(process(0, -1), "ZeroDivision");
        assert_eq!(process(1, -1), "1");
        assert_eq!(process(2, -1), "0");
        assert_eq!(process(3, -1), "0");

        assert_eq!(process(-3, -2), "0");
        assert_eq!(process(-2, -2), "0");
        assert_eq!(process(-1, -2), "1");
        assert_eq!(process(0, -2), "ZeroDivision");
        assert_eq!(process(1, -2), "1");
        assert_eq!(process(2, -2), "0");
        assert_eq!(process(3, -2), "0");

        assert_eq!(process(-3, -3), "0");
        assert_eq!(process(-2, -3), "0");
        assert_eq!(process(-1, -3), "-1");
        assert_eq!(process(0, -3), "ZeroDivision");
        assert_eq!(process(1, -3), "1");
        assert_eq!(process(2, -3), "0");
        assert_eq!(process(3, -3), "0");

        assert_eq!(process(0, -666), "ZeroDivision");
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
    fn test_20000_way_too_high_exponent() {
        let max: u32 = u32::MAX;
        let max_plus1: i64 = (max as i64) + 1;
        assert_eq!(process(1234, max_plus1), "ExponentTooHigh");
    }
}
