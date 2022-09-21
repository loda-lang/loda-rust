use super::{SemanticBinomialConfig, SemanticBinomialError};
use super::{SemanticPowerConfig, SemanticPowerError};
use super::{SemanticSimpleConfig, SemanticSimpleError};
use super::{EvalError, semantic_binomial, semantic_power, semantic_simple};
use num_bigint::BigInt;

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
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_add(x, y);
        let value = result?;
        Ok(value)
    }

    fn subtract(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_subtract(x, y);
        let value = result?;
        Ok(value)
    }

    fn truncate(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_truncate(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn multiply(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_multiply(x, y);
        let value = result?;
        Ok(value)
    }

    fn divide(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_divide(x, y);
        let value = result?;
        Ok(value)
    }

    fn divide_if(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_divide_if(x, y);
        let value = result?;
        Ok(value)
    }

    fn modulo(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_modulo(x, y);
        let value = result?;
        Ok(value)
    }

    fn power(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticPowerError> = semantic_power::SEMANTIC_POWER_CONFIG_LIMIT_SMALL.compute_power(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn gcd(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_gcd(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn binomial(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticBinomialError> = semantic_binomial::SEMANTIC_BINOMIAL_CONFIG_LIMIT_SMALL.compute_binomial(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn compare(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_compare(x, y);
        let value = result?;
        Ok(value)
    }

    fn min(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_min(x, y);
        let value = result?;
        Ok(value)
    }

    fn max(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_max(x, y);
        let value = result?;
        Ok(value)
    }
}
