use super::{SemanticBinomialConfig, SemanticBinomialError};
use super::{SemanticPowerConfig, SemanticPowerError};
use super::{EvalError, semantic_binomial, semantic_power};
use num_bigint::BigInt;
use num_traits::Signed;
use num_traits::Zero;
use num_traits::One;
use num_integer::Integer;

const MULTIPLY_BITS: u64 = 96;
const ADD_SUB_BITS: u64 = 96;

pub struct Semantics {}

impl Semantics {
    pub fn move_value(_x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(y.clone())
    }

    pub fn add(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if x.bits() > ADD_SUB_BITS {
            return Err(EvalError::AddSubtractExceededLimit);
        }
        if y.bits() > ADD_SUB_BITS {
            return Err(EvalError::AddSubtractExceededLimit);
        }
        Ok(x + y)
    }

    pub fn subtract(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if x.bits() > ADD_SUB_BITS {
            return Err(EvalError::AddSubtractExceededLimit);
        }
        if y.bits() > ADD_SUB_BITS {
            return Err(EvalError::AddSubtractExceededLimit);
        }
        Ok(x - y)
    }

    pub fn truncate(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if x.bits() > ADD_SUB_BITS {
            return Err(EvalError::AddSubtractExceededLimit);
        }
        if y.bits() > ADD_SUB_BITS {
            return Err(EvalError::AddSubtractExceededLimit);
        }
        let value: BigInt = x - y;
        if !value.is_positive() {
            return Ok(BigInt::zero());
        }
        Ok(value)
    }
    
    pub fn multiply(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if x.bits() > MULTIPLY_BITS {
            return Err(EvalError::MultipliplyExceededLimit);
        }
        if y.bits() > MULTIPLY_BITS {
            return Err(EvalError::MultipliplyExceededLimit);
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
        let result: Result<BigInt, SemanticPowerError> = semantic_power::SEMANTIC_POWER_CONFIG_LIMIT_SMALL.compute_power(x, y);
        let value = result?;
        Ok(value)
    }
    
    pub fn gcd(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        // https://en.wikipedia.org/wiki/Binary_GCD_algorithm
        Ok(x.gcd(y))
    }
    
    pub fn binomial(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticBinomialError> = semantic_binomial::SEMANTIC_BINOMIAL_CONFIG_LIMIT_SMALL.compute_binomial(x, y);
        let value = result?;
        Ok(value)
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
