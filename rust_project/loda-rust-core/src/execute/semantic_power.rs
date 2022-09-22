use super::EvalError;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, One, Zero, Signed};
use num_integer::Integer;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SEMANTIC_POWER_CONFIG_UNLIMITED: SemanticPowerConfigUnlimited = SemanticPowerConfigUnlimited {};

    pub static ref SEMANTIC_POWER_CONFIG_LIMIT_SMALL: SemanticPowerConfigLimited = SemanticPowerConfigLimited::new(100);
}

pub enum SemanticPowerError {
    PowerZeroDivision,
    PowerExponentTooHigh,
    PowerExceededLimit,
}

impl From<SemanticPowerError> for EvalError {
    fn from(error: SemanticPowerError) -> EvalError {
        match error {
            SemanticPowerError::PowerZeroDivision => EvalError::PowerZeroDivision,
            SemanticPowerError::PowerExponentTooHigh => EvalError::PowerExponentTooHigh,
            SemanticPowerError::PowerExceededLimit => EvalError::PowerExceededLimit
        }
    }
}

pub trait SemanticPowerConfig {
    // #[inline(always)]
    fn power_max_bits(&self) -> Option<u128>;

    /// x raised to the power of y
    /// 
    /// x is the base value.
    /// 
    /// y is the power value.
    /// 
    /// Ruby: x ** y
    /// 
    /// Math syntax: x ^ y.
    fn compute_power(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticPowerError> {
        let base: &BigInt = x;

        let exponent: &BigInt = y;
        
        if base.is_zero() {
            if exponent.is_positive() {
                return Ok(BigInt::zero());
            }
            if exponent.is_zero() {
                return Ok(BigInt::one());
            }
            return Err(SemanticPowerError::PowerZeroDivision);
        }
    
        if base.is_one() {
            // 1^x is always 1
            return Ok(BigInt::one());
        }
        if base.abs().is_one() {
            // (-1)^x, alternates between +1 and -1
            if exponent.is_even() {
                return Ok(BigInt::one());
            } else {
                return Ok(-BigInt::one());
            }
        }
    
        if exponent.is_negative() {
            // The actual result of raising to a negative number
            // is a tiny positive number, between 0 and 1.
            // Example: 
            //  ((30) ** (-1)) => (1/30)
            //  ((-2) ** (-3)) => (1/-8)
            //  (( 2) ** (-3))  => (1/8)
            return Ok(BigInt::zero());
        }
        if exponent.is_one() {
            return Ok(x.clone());
        }
    
        // Prevent invoking pow, if the exponent is higher than an u32.
        let exponent_u32: u32 = match exponent.to_u32() {
            Some(value) => value,
            None => {
                // Power `exponent` is higher than a 32bit unsigned integer. This is beyond what the pow() function can handle.
                // debug!("power exponent exceeded 32bits: {}", exponent);
                return Err(SemanticPowerError::PowerExponentTooHigh);
            }
        };
    
        // Ensure that the result of pow doesn't exceed the limit (optionally)
        if let Some(power_max_bits) = self.power_max_bits() {
            // There is no floating point logarithm for BigInt.
            // so it's a rough estimate of the number of bits in the result.
            let result_size: u128 = (base.bits() as u128) * (exponent_u32 as u128);
            if result_size > power_max_bits {
                // debug!("power result size exceeded max bits: {}", result_size);
                return Err(SemanticPowerError::PowerExceededLimit);
            }
        }
    
        let result: BigInt = base.pow(exponent_u32);
        Ok(result)
    }
}

pub struct SemanticPowerConfigUnlimited {}

impl SemanticPowerConfig for SemanticPowerConfigUnlimited {
    fn power_max_bits(&self) -> Option<u128> {
        None
    }
}

pub struct SemanticPowerConfigLimited {
    power_max_bits: u128,
}

impl SemanticPowerConfigLimited {
    fn new(power_max_bits: u128) -> Self {
        Self {
            power_max_bits: power_max_bits,
        }
    }
}

impl SemanticPowerConfig for SemanticPowerConfigLimited {
    fn power_max_bits(&self) -> Option<u128> {
        Some(self.power_max_bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigInt;

    fn process(left: i64, right: i64) -> String {
        let config = SemanticPowerConfigUnlimited {};
        process_inner(left, right, &config)
    }

    fn process_limit(left: i64, right: i64, limit: u32) -> String {
        let config = SemanticPowerConfigLimited::new(limit as u128);
        process_inner(left, right, &config)
    }

    fn process_inner(left: i64, right: i64, semantic_power_config: &dyn SemanticPowerConfig) -> String {
        let x = left.to_bigint().unwrap();
        let y = right.to_bigint().unwrap();
        let result = semantic_power_config.compute_power(&x, &y);
        match result {
            Ok(value) => return value.to_string(),
            Err(SemanticPowerError::PowerZeroDivision) => return "ZeroDivision".to_string(),
            Err(SemanticPowerError::PowerExponentTooHigh) => return "ExponentTooHigh".to_string(),
            Err(SemanticPowerError::PowerExceededLimit) => return "ExceededLimit".to_string(),
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
}
