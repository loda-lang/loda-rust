use super::EvalError;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, One, Zero, Signed};
use num_integer::Integer;

/// x raised to the power of y
/// 
/// x is the base value.
/// 
/// y is the power value.
/// 
/// Ruby: x ** y
/// 
/// Math syntax: x ^ y.
pub fn semantic_power(
    x: &BigInt, 
    y: &BigInt
) -> Result<BigInt, EvalError> {
    let base: &BigInt = x;

    let exponent: &BigInt = y;
    
    if base.is_zero() {
        if exponent.is_positive() {
            return Ok(BigInt::zero());
        }
        if exponent.is_zero() {
            return Ok(BigInt::one());
        }
        return Err(EvalError::PowerZeroDivision);
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
            // NodePower `exponent` is higher than a 32bit unsigned integer. This is beyond what the pow() function can handle.
            return Err(EvalError::PowerExponentTooHigh);
        }
    };

    let result: BigInt = base.pow(exponent_u32);
    Ok(result)
}
