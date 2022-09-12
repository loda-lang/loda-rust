use super::EvalError;
use num_bigint::BigInt;
use num_traits::Signed;
use num_traits::Zero;

pub struct Semantics {}

impl Semantics {
    pub fn move_value(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(y.clone())
    }

    pub fn add(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(x + y)
    }

    pub fn subtract(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(x - y)
    }

    pub fn truncate(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let value: BigInt = x - y;
        if !value.is_positive() {
            return Ok(BigInt::zero());
        }
        Ok(value)
    }
    
    pub fn multiply(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(x * y)
    }

    pub fn divide(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if y.is_zero() {
            return Err(EvalError::DivisionByZero);
        }
        Ok(x / y)
    }

    pub fn divide_if(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if y.is_zero() {
            return Ok(x.clone());
        }
        let remain: BigInt = x % y;
        if remain.is_zero() {
            return Ok(x / y);
        } else {
            return Ok(x.clone());
        }
    }

    pub fn modulo(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if y.is_zero() {
            return Err(EvalError::DivisionByZero);
        }
        Ok(x % y)
    }
    
    // TODO: pow
    // TODO: gcd
    // TODO: bin
    // TODO: cmp
    // TODO: min
    // TODO: max
}
