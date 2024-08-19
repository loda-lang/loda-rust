use super::{SemanticBinomialConfig, SemanticBinomialError};
use super::{SemanticPowerConfig, SemanticPowerError};
use super::{SemanticSimpleConfig, SemanticSimpleError};
use super::{EvalError, semantic_binomial, semantic_power, semantic_simple};
use num_bigint::BigInt;

pub trait Semantics {
    fn add(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn subtract(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn truncate(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn multiply(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn divide(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn divide_if(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn modulo(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn power(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn gcd(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn binomial(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn compare(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn min(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn max(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn logarithm(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn nthroot(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn digitsum(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn digitalroot(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn equal(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn notequal(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
    fn lessorequal(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError>;
}

pub struct SemanticsWithoutLimits {}

impl Semantics for SemanticsWithoutLimits {
    fn add(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_add(x, y);
        let value = result?;
        Ok(value)
    }

    fn subtract(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_subtract(x, y);
        let value = result?;
        Ok(value)
    }

    fn truncate(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_truncate(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn multiply(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_multiply(x, y);
        let value = result?;
        Ok(value)
    }

    fn divide(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_divide(x, y);
        let value = result?;
        Ok(value)
    }

    fn divide_if(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_divide_if(x, y);
        let value = result?;
        Ok(value)
    }

    fn modulo(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_modulo(x, y);
        let value = result?;
        Ok(value)
    }

    fn power(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticPowerError> = semantic_power::SEMANTIC_POWER_CONFIG_UNLIMITED.compute_power(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn gcd(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_gcd(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn binomial(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticBinomialError> = semantic_binomial::SEMANTIC_BINOMIAL_CONFIG_UNLIMITED.compute_binomial(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn compare(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_compare(x, y);
        let value = result?;
        Ok(value)
    }

    fn min(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_min(x, y);
        let value = result?;
        Ok(value)
    }

    fn max(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_max(x, y);
        let value = result?;
        Ok(value)
    }

    fn logarithm(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_logarithm(x, y);
        let value = result?;
        Ok(value)
    }

    fn nthroot(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_nthroot(x, y);
        let value = result?;
        Ok(value)
    }

    fn digitsum(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_digitsum(x, y);
        let value = result?;
        Ok(value)
    }

    fn digitalroot(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_digitalroot(x, y);
        let value = result?;
        Ok(value)
    }

    fn equal(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_equal(x, y);
        let value = result?;
        Ok(value)
    }

    fn notequal(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_notequal(x, y);
        let value = result?;
        Ok(value)
    }

    fn lessorequal(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_UNLIMITED.compute_lessorequal(x, y);
        let value = result?;
        Ok(value)
    }
}

pub struct SemanticsWithSmallLimits {}

impl Semantics for SemanticsWithSmallLimits {
    fn add(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_add(x, y);
        let value = result?;
        Ok(value)
    }

    fn subtract(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_subtract(x, y);
        let value = result?;
        Ok(value)
    }

    fn truncate(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_truncate(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn multiply(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_multiply(x, y);
        let value = result?;
        Ok(value)
    }

    fn divide(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_divide(x, y);
        let value = result?;
        Ok(value)
    }

    fn divide_if(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_divide_if(x, y);
        let value = result?;
        Ok(value)
    }

    fn modulo(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_modulo(x, y);
        let value = result?;
        Ok(value)
    }

    fn power(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticPowerError> = semantic_power::SEMANTIC_POWER_CONFIG_LIMIT_SMALL.compute_power(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn gcd(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_gcd(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn binomial(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticBinomialError> = semantic_binomial::SEMANTIC_BINOMIAL_CONFIG_LIMIT_SMALL.compute_binomial(x, y);
        let value = result?;
        Ok(value)
    }
    
    fn compare(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_compare(x, y);
        let value = result?;
        Ok(value)
    }

    fn min(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_min(x, y);
        let value = result?;
        Ok(value)
    }

    fn max(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_max(x, y);
        let value = result?;
        Ok(value)
    }

    fn logarithm(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_logarithm(x, y);
        let value = result?;
        Ok(value)
    }

    fn nthroot(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_nthroot(x, y);
        let value = result?;
        Ok(value)
    }

    fn digitsum(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_digitsum(x, y);
        let value = result?;
        Ok(value)
    }

    fn digitalroot(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_digitalroot(x, y);
        let value = result?;
        Ok(value)
    }

    fn equal(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_equal(x, y);
        let value = result?;
        Ok(value)
    }

    fn notequal(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_notequal(x, y);
        let value = result?;
        Ok(value)
    }

    fn lessorequal(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
        let result: Result<BigInt, SemanticSimpleError> = semantic_simple::SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL.compute_lessorequal(x, y);
        let value = result?;
        Ok(value)
    }
}
