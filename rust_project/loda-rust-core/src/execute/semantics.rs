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

pub trait Semantics {
    fn move_value(&self, _x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;

    fn add(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;

    fn subtract(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;

    fn truncate(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    
    fn multiply(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;

    fn divide(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;

    fn divide_if(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;

    fn modulo(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;

    fn power(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    
    fn gcd(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    
    fn binomial(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    
    fn compare(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;

    fn min(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;

    fn max(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;

}

pub struct SemanticsImpl {}

impl Semantics for SemanticsImpl {
    fn move_value(&self, _x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(y.clone())
    }

    fn add(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if x.bits() > ADD_SUB_BITS {
            return Err(EvalError::AddSubtractExceededLimit);
        }
        if y.bits() > ADD_SUB_BITS {
            return Err(EvalError::AddSubtractExceededLimit);
        }
        Ok(x + y)
    }

    fn subtract(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if x.bits() > ADD_SUB_BITS {
            return Err(EvalError::AddSubtractExceededLimit);
        }
        if y.bits() > ADD_SUB_BITS {
            return Err(EvalError::AddSubtractExceededLimit);
        }
        Ok(x - y)
    }

    fn truncate(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
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
    
    fn multiply(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if x.bits() > MULTIPLY_BITS {
            return Err(EvalError::MultipliplyExceededLimit);
        }
        if y.bits() > MULTIPLY_BITS {
            return Err(EvalError::MultipliplyExceededLimit);
        }
        Ok(x * y)
    }

    fn divide(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if y.is_zero() {
            return Err(EvalError::DivisionByZero);
        }
        Ok(x / y)
    }

    fn divide_if(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
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

    fn modulo(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if y.is_zero() {
            return Err(EvalError::DivisionByZero);
        }
        Ok(x % y)
    }

    fn power(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticPowerError> = semantic_power::SEMANTIC_POWER_CONFIG_LIMIT_SMALL.compute_power(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn gcd(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        // https://en.wikipedia.org/wiki/Binary_GCD_algorithm
        Ok(x.gcd(y))
    }
    
    fn binomial(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticBinomialError> = semantic_binomial::SEMANTIC_BINOMIAL_CONFIG_LIMIT_SMALL.compute_binomial(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn compare(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        if x == y {
            return Ok(BigInt::one());
        } else {
            return Ok(BigInt::zero());
        }
    }

    fn min(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(x.min(y).clone())
    }

    fn max(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        Ok(x.max(y).clone())
    }
}
