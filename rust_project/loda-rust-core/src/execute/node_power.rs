use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use super::{BoxCheckValue, PerformCheckValue};
use std::collections::HashSet;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, One, Zero, Signed};
use num_integer::Integer;

#[derive(Clone)]
pub enum NodePowerLimit {
    Unlimited,
    LimitBits(u32)
}

// x raised to the power of y
// x is the base value.
// y is the power value.
// Ruby: x ** y
// Math syntax: x ^ y.
fn perform_operation(
    check: &BoxCheckValue, 
    limit: &NodePowerLimit,
    x: &RegisterValue, 
    y: &RegisterValue
) -> Result<RegisterValue, EvalError> {
    let base: &BigInt = &x.0;
    check.input(base)?;

    let exponent: &BigInt = &y.0;
    check.input(exponent)?;
    
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
            // NodePower `exponent` is higher than a 32bit unsigned integer. This is beyond what the pow() function can handle.
            return Err(EvalError::PowerExponentTooHigh);
        }
    };

    // Ensure that the result of pow doesn't exceed the limit
    match limit {
        NodePowerLimit::Unlimited => {},
        NodePowerLimit::LimitBits(max_bits) => {
            // There is no floating point logarithm for BigInt.
            // so it's a rough estimate of the number of bits in the result.
            let result_size: u128 = (base.bits() as u128) * (exponent_u32 as u128);
            if result_size > (*max_bits as u128) {
                return Err(EvalError::PowerExceededLimit);
            }
        }
    }

    let result: BigInt = base.pow(exponent_u32);
    check.output(&result)?;
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
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = state.get_register_value_ref(&self.source);
        let value: RegisterValue = perform_operation(
            state.check_value(), 
            state.node_power_limit(), 
            lhs, 
            rhs
        )?;
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
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = &self.source;
        let value: RegisterValue = perform_operation(
            state.check_value(), 
            state.node_power_limit(), 
            lhs, 
            rhs
        )?;
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
    }

    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        if self.source.0.is_zero() {
            // n^0, always result in 1
            register_set.remove(&self.target);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::CheckValueLimitBits;
    use super::super::CheckValueUnlimited;

    fn process(left: i64, right: i64) -> String {
        let check_value: BoxCheckValue = Box::new(CheckValueUnlimited::new());
        let limit = NodePowerLimit::Unlimited;
        process_inner(left, right, &check_value, &limit)
    }

    fn process_limit(left: i64, right: i64, limit: u32) -> String {
        let check_value: BoxCheckValue = Box::new(CheckValueUnlimited::new());
        // let check_value: BoxCheckValue = Box::new(CheckValueLimitBits::new(32));
        let limit = NodePowerLimit::LimitBits(limit);
        process_inner(left, right, &check_value, &limit)
    }

    fn process_checkvalue_limit(left: i64, right: i64, checkvalue: u32, limit: u32) -> String {
        let check_value: BoxCheckValue = Box::new(CheckValueLimitBits::new(checkvalue));
        let limit = NodePowerLimit::LimitBits(limit);
        process_inner(left, right, &check_value, &limit)
    }

    fn process_inner(left: i64, right: i64, check_value: &BoxCheckValue, limit: &NodePowerLimit) -> String {
        let result = perform_operation(
            &check_value,
            &limit,
            &RegisterValue::from_i64(left),
            &RegisterValue::from_i64(right),
        );
        match result {
            Ok(value) => return value.to_string(),
            Err(EvalError::InputOutOfRange) => return "BOOM-INPUT".to_string(),
            Err(EvalError::OutputOutOfRange) => return "BOOM-OUTPUT".to_string(),
            Err(EvalError::PowerZeroDivision) => return "ZeroDivision".to_string(),
            Err(EvalError::PowerExponentTooHigh) => return "ExponentTooHigh".to_string(),
            Err(EvalError::PowerExceededLimit) => return "ExceededLimit".to_string(),
            Err(_) => return "BOOM-OTHER".to_string()
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

    #[test]
    fn test_30000_exceed_limit() {
        // 2 bits for representing the base
        assert_eq!(process_limit(2, 7, 16), "128");
        assert_eq!(process_limit(2, 8, 16), "256");
        assert_eq!(process_limit(2, 9, 16), "ExceededLimit");
        assert_eq!(process_limit(-2, 7, 16), "-128");
        assert_eq!(process_limit(-2, 8, 16), "256");
        assert_eq!(process_limit(-2, 9, 16), "ExceededLimit");
        assert_eq!(process_limit(3, 7, 16), "2187");
        assert_eq!(process_limit(3, 8, 16), "6561");
        assert_eq!(process_limit(3, 9, 16), "ExceededLimit");
        assert_eq!(process_limit(-3, 7, 16), "-2187");
        assert_eq!(process_limit(-3, 8, 16), "6561");
        assert_eq!(process_limit(-3, 9, 16), "ExceededLimit");

        // 3 bits for representing the base
        assert_eq!(process_limit(4, 4, 16), "256");
        assert_eq!(process_limit(4, 5, 16), "1024");
        assert_eq!(process_limit(4, 6, 16), "ExceededLimit");
        assert_eq!(process_limit(-4, 4, 16), "256");
        assert_eq!(process_limit(-4, 5, 16), "-1024");
        assert_eq!(process_limit(-4, 6, 16), "ExceededLimit");
        assert_eq!(process_limit(7, 4, 16), "2401");
        assert_eq!(process_limit(7, 5, 16), "16807");
        assert_eq!(process_limit(7, 6, 16), "ExceededLimit");
        assert_eq!(process_limit(-7, 4, 16), "2401");
        assert_eq!(process_limit(-7, 5, 16), "-16807");
        assert_eq!(process_limit(-7, 6, 16), "ExceededLimit");

        // 4 bits for representing the base
        assert_eq!(process_limit(8, 4, 20), "4096");
        assert_eq!(process_limit(8, 5, 20), "32768");
        assert_eq!(process_limit(8, 6, 20), "ExceededLimit");
        assert_eq!(process_limit(-8, 4, 20), "4096");
        assert_eq!(process_limit(-8, 5, 20), "-32768");
        assert_eq!(process_limit(-8, 6, 20), "ExceededLimit");
        assert_eq!(process_limit(15, 4, 20), "50625");
        assert_eq!(process_limit(15, 5, 20), "759375");
        assert_eq!(process_limit(15, 6, 20), "ExceededLimit");
        assert_eq!(process_limit(-15, 4, 20), "50625");
        assert_eq!(process_limit(-15, 5, 20), "-759375");
        assert_eq!(process_limit(-15, 6, 20), "ExceededLimit");

        // 8 bits for representing the base
        assert_eq!(process_limit(255, 2, 16), "65025");
        assert_eq!(process_limit(255, 2, 15), "ExceededLimit");
        assert_eq!(process_limit(-255, 2, 16), "65025");
        assert_eq!(process_limit(-255, 2, 15), "ExceededLimit");
    }

    #[test]
    fn test_30001_out_of_range() {
        {
            assert_eq!(process_checkvalue_limit(127, 1, 8, 32), "127");
            assert_eq!(process_checkvalue_limit(128, 1, 8, 32), "BOOM-INPUT");
        }
        {
            assert_eq!(process_checkvalue_limit(1, 127, 8, 32), "1");
            assert_eq!(process_checkvalue_limit(1, 128, 8, 32), "BOOM-INPUT");
        }
        {
            assert_eq!(process_checkvalue_limit(2, 6, 8, 32), "64");
            assert_eq!(process_checkvalue_limit(2, 7, 8, 32), "BOOM-OUTPUT");
        }
    }
}
