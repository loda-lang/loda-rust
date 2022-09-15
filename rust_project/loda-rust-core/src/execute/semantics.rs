use super::{EvalError, semantic_binomial, semantic_power};
use num_bigint::{BigInt, ToBigInt};
use num_traits::Signed;
use num_traits::Zero;
use num_traits::One;
use num_integer::Integer;
use lazy_static::lazy_static;

lazy_static! {
    static ref MULTIPLICATION_LIMIT: BigInt = (0xffff_ffff_ffff as i64).to_bigint().unwrap();
    static ref ADD_SUB_LIMIT: BigInt = (0xffff_ffff_ffff as i64).to_bigint().unwrap();
}

pub struct Semantics {}

impl Semantics {
    pub fn move_value(_x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(y.clone())
    }

    pub fn add(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if &x.abs() > &ADD_SUB_LIMIT {
            return Err(EvalError::MultiplicationExceededLimit);
        }
        if &y.abs() > &ADD_SUB_LIMIT {
            return Err(EvalError::MultiplicationExceededLimit);
        }
        Ok(x + y)
    }

    pub fn subtract(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if &x.abs() > &ADD_SUB_LIMIT {
            return Err(EvalError::MultiplicationExceededLimit);
        }
        if &y.abs() > &ADD_SUB_LIMIT {
            return Err(EvalError::MultiplicationExceededLimit);
        }
        Ok(x - y)
    }

    pub fn truncate(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if &x.abs() > &ADD_SUB_LIMIT {
            return Err(EvalError::MultiplicationExceededLimit);
        }
        if &y.abs() > &ADD_SUB_LIMIT {
            return Err(EvalError::MultiplicationExceededLimit);
        }
        let value: BigInt = x - y;
        if !value.is_positive() {
            return Ok(BigInt::zero());
        }
        Ok(value)
    }
    
    pub fn multiply(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if &x.abs() > &MULTIPLICATION_LIMIT {
            return Err(EvalError::MultiplicationExceededLimit);
        }
        if &y.abs() > &MULTIPLICATION_LIMIT {
            return Err(EvalError::MultiplicationExceededLimit);
        }
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

    pub fn power(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        semantic_power(x, y)
    }
    
    pub fn gcd(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        // https://en.wikipedia.org/wiki/Binary_GCD_algorithm
        Ok(x.gcd(y))
    }
    
    pub fn binomial(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        semantic_binomial(x, y)
    }
    
    pub fn compare(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if x == y {
            return Ok(BigInt::one());
        } else {
            return Ok(BigInt::zero());
        }
    }

    pub fn min(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(x.min(y).clone())
    }

    pub fn max(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(x.max(y).clone())
    }
}
