use super::EvalError;
use num_integer::{binomial, Integer};
use num_bigint::{BigInt, ToBigInt};
use num_traits::{Zero, One, Signed};
use lazy_static::lazy_static;

const BINOMIAL_INTERNAL_VALUE_BITS: u64 = 96;

lazy_static! {
    static ref BINOMIAL_MAX_N: BigInt = 50.to_bigint().unwrap();
}

pub fn semantic_binomial(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
    let input_n: &BigInt = x;
    let input_k: &BigInt = y;

    // positive n or zero
    if input_n.is_zero() || input_n.is_positive() {
        if input_n > &BINOMIAL_MAX_N {
            // debug!("too high a N value: bin({:?},{:?})", input_n, input_k);
            return Err(EvalError::BinomialDomainError);
        }

        if input_k.is_negative() || input_k > input_n {
            return Ok(BigInt::zero());
        }

        // Inside pascals triangle
        let n: BigInt = input_n.clone();
        let mut k: BigInt = input_k.clone();
        let k2: BigInt = k.clone() * 2;
        if k2 > n {
            k = n.clone() - k.clone();
        }
        let value: BigInt = binomial(n, k);
        return Ok(value);
    }

    if &input_n.abs() > &BINOMIAL_MAX_N {
        // debug!("too low a N value: bin({:?},{:?})", input_n, input_k);
        return Err(EvalError::BinomialDomainError);
    }

    let mut n: BigInt = input_n.clone();
    let mut k: BigInt = input_k.clone();

    // negative n
    // https://arxiv.org/pdf/1105.3689.pdf
    let mut sign: i64 = 1;
    if input_k.is_zero() || input_k.is_positive() {
        if input_k.is_odd() {
            sign = -1;
        }
        n = -n.clone() + k.clone() - 1;
    } else {
        if input_k <= input_n {
            let n_minus_k: BigInt = n.clone() - k.clone();
            if n_minus_k.is_odd() {
                sign = -1;
            }
            let n_old: BigInt = n.clone();
            n = -k.clone() - 1;
            k = n_old - k;
        } else {
            return Ok(BigInt::zero());
        }
    }

    if k.is_negative() || k > n {
        return Ok(BigInt::zero());
    }

    let k2: BigInt = k.clone() * 2;
    if k2 > n {
        let n_minus_k: BigInt = n.clone() - k.clone();
        k = n_minus_k;
    }

    let mut value = BigInt::one();
    let mut i: BigInt = BigInt::zero();
    while i < k {
        let n_minus_i: BigInt = n.clone() - i.clone();
        value *= n_minus_i;
        if value.bits() > BINOMIAL_INTERNAL_VALUE_BITS {
            // debug!("too high an internal value: bin({:?},{:?}) value: {:?}", input_n, input_k, value);
            return Err(EvalError::BinomialDomainError);
        }
        i += 1;
        value = value / i.clone();
    }
    Ok(value * sign)
}

pub enum SemanticBinomialError {
    TooHighNValue,
    TooLowNValue,
    InternalValueExceededLimit,
}

impl From<SemanticBinomialError> for EvalError {
    fn from(_err: SemanticBinomialError) -> EvalError {
        EvalError::BinomialDomainError
    }
}

pub trait SemanticBinomialConfig {
    // #[inline(always)]
    fn binomial_max_n(&self) -> Option<&BigInt>;

    // #[inline(always)]
    fn binomial_internal_value_bits(&self) -> Option<u64>;

    fn semantic_binomial(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticBinomialError> {
        let input_n: &BigInt = x;
        let input_k: &BigInt = y;
    
        // positive n or zero
        if input_n.is_zero() || input_n.is_positive() {
            if let Some(binomial_max_n) = self.binomial_max_n() {
                if input_n > binomial_max_n {
                    // debug!("too high a N value: bin({:?},{:?})", input_n, input_k);
                    return Err(SemanticBinomialError::TooHighNValue);
                }
            }
    
            if input_k.is_negative() || input_k > input_n {
                return Ok(BigInt::zero());
            }
    
            // Inside pascals triangle
            let n: BigInt = input_n.clone();
            let mut k: BigInt = input_k.clone();
            let k2: BigInt = k.clone() * 2;
            if k2 > n {
                k = n.clone() - k.clone();
            }
            let value: BigInt = binomial(n, k);
            return Ok(value);
        }
    
        if let Some(binomial_max_n) = self.binomial_max_n() {
            if &input_n.abs() > binomial_max_n {
                // debug!("too low a N value: bin({:?},{:?})", input_n, input_k);
                return Err(SemanticBinomialError::TooLowNValue);
            }
        }
    
        let mut n: BigInt = input_n.clone();
        let mut k: BigInt = input_k.clone();
    
        // negative n
        // https://arxiv.org/pdf/1105.3689.pdf
        let mut sign: i64 = 1;
        if input_k.is_zero() || input_k.is_positive() {
            if input_k.is_odd() {
                sign = -1;
            }
            n = -n.clone() + k.clone() - 1;
        } else {
            if input_k <= input_n {
                let n_minus_k: BigInt = n.clone() - k.clone();
                if n_minus_k.is_odd() {
                    sign = -1;
                }
                let n_old: BigInt = n.clone();
                n = -k.clone() - 1;
                k = n_old - k;
            } else {
                return Ok(BigInt::zero());
            }
        }
    
        if k.is_negative() || k > n {
            return Ok(BigInt::zero());
        }
    
        let k2: BigInt = k.clone() * 2;
        if k2 > n {
            let n_minus_k: BigInt = n.clone() - k.clone();
            k = n_minus_k;
        }
    
        let mut value = BigInt::one();
        let mut i: BigInt = BigInt::zero();
        while i < k {
            let n_minus_i: BigInt = n.clone() - i.clone();
            value *= n_minus_i;
            if let Some(binomial_internal_value_bits) = self.binomial_internal_value_bits() {
                if value.bits() > binomial_internal_value_bits {
                    // debug!("too extreme an internal value: bin({:?},{:?}) value: {:?}", input_n, input_k, value);
                    return Err(SemanticBinomialError::InternalValueExceededLimit);
                }
            }
            i += 1;
            value = value / i.clone();
        }
        Ok(value * sign)
    }    
}

struct SemanticBinomialConfigUnlimited {}

impl SemanticBinomialConfig for SemanticBinomialConfigUnlimited {
    fn binomial_max_n(&self) -> Option<&BigInt> {
        None
    }

    fn binomial_internal_value_bits(&self) -> Option<u64> {
        None
    }
}

struct SemanticBinomialConfigLimited {
    binomial_max_n: BigInt,
    binomial_internal_value_bits: u64,
}

impl SemanticBinomialConfigLimited {
    fn new(binomial_max_n: BigInt, binomial_internal_value_bits: u64) -> Self {
        Self {
            binomial_max_n: binomial_max_n,
            binomial_internal_value_bits: binomial_internal_value_bits,
        }
    }
}

impl SemanticBinomialConfig for SemanticBinomialConfigLimited {
    fn binomial_max_n(&self) -> Option<&BigInt> {
        Some(&self.binomial_max_n)
    }

    fn binomial_internal_value_bits(&self) -> Option<u64> {
        Some(self.binomial_internal_value_bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::ToPrimitive;

    fn process(left: i64, right: i64) -> String {
        let config = SemanticBinomialConfigUnlimited {};
        process_inner(left, right, &config)
    }

    fn process_limit(left: i64, right: i64, max_n: u8, internal_value_bits: u64) -> String {
        let max_n: BigInt = max_n.to_bigint().unwrap();
        let config = SemanticBinomialConfigLimited::new(max_n, internal_value_bits);
        process_inner(left, right, &config)
    }

    fn process_inner(left: i64, right: i64, semantic_binomial_config: &dyn SemanticBinomialConfig) -> String {
        let x = left.to_bigint().unwrap();
        let y = right.to_bigint().unwrap();
        let result = semantic_binomial_config.semantic_binomial(&x, &y);
        let value_bigint: BigInt = match result {
            Ok(value) => value,
            Err(SemanticBinomialError::TooHighNValue) => return "TOOHIGH".to_string(),
            Err(SemanticBinomialError::TooLowNValue) => return "TOOLOW".to_string(),
            Err(SemanticBinomialError::InternalValueExceededLimit) => return "INTERNEXCEEDEDLIMIT".to_string(),
        };
        let value_i64 = match value_bigint.to_i64() {
            Some(value) => value,
            None => {
                return "BOOM".to_string();
            }
        };
        value_i64.to_string()
    }

    #[test]
    fn test_10000_positive() {
        let pascals_triangle = [
            (0, 0, 1),
            (1, 0, 1), (1, 1, 1),
            (2, 0, 1), (2, 1, 2), (2, 2, 1),
            (3, 0, 1), (3, 1, 3), (3, 2, 3), (3, 3, 1),
            (4, 0, 1), (4, 1, 4), (4, 2, 6), (4, 3, 4), (4, 4, 1),
            (5, 0, 1), (5, 1, 5), (5, 2, 10), (5, 3, 10), (5, 5, 1), (5, 5, 1),
            (6, 0, 1), (6, 1, 6), (6, 2, 15), (6, 3, 20), (6, 4, 15), (6, 5, 6), (6, 6, 1),
        ];

        for item in pascals_triangle.iter() {
            let actual: String = process(item.0, item.1);
            let expected_s = item.2.to_string();
            assert_eq!(actual, expected_s);
        }
    }

    #[test]
    fn test_10001_k_outside_bounds() {
        assert_eq!(process(0, -2), "0");
        assert_eq!(process(0, -1), "0");
        assert_eq!(process(0, 0), "1"); // inside triangle
        assert_eq!(process(0, 1), "0");
        assert_eq!(process(0, 2), "0");

        assert_eq!(process(1, -2), "0");
        assert_eq!(process(1, -1), "0");
        assert_eq!(process(1, 0), "1"); // inside triangle
        assert_eq!(process(1, 1), "1"); // inside triangle
        assert_eq!(process(1, 2), "0");
        assert_eq!(process(1, 3), "0");
    }

    #[test]
    fn test_10002_n_minus1() {
        assert_eq!(process(-1, -4), "-1");
        assert_eq!(process(-1, -3), "1");
        assert_eq!(process(-1, -2), "-1");
        assert_eq!(process(-1, -1), "1");
        assert_eq!(process(-1, 0), "1");
        assert_eq!(process(-1, 1), "-1");
        assert_eq!(process(-1, 2), "1");
        assert_eq!(process(-1, 3), "-1");
    }

    #[test]
    fn test_10002_n_minus2() {
        assert_eq!(process(-2, -5), "-4");
        assert_eq!(process(-2, -4), "3");
        assert_eq!(process(-2, -3), "-2");
        assert_eq!(process(-2, -2), "1");
        assert_eq!(process(-2, -1), "0");
        assert_eq!(process(-2, 0), "1");
        assert_eq!(process(-2, 1), "-2");
        assert_eq!(process(-2, 2), "3");
        assert_eq!(process(-2, 3), "-4");
    }

    #[test]
    fn test_10003_n_minus3() {
        assert_eq!(process(-3, -5), "6");
        assert_eq!(process(-3, -4), "-3");
        assert_eq!(process(-3, -3), "1");
        assert_eq!(process(-3, -2), "0");
        assert_eq!(process(-3, -1), "0");
        assert_eq!(process(-3, 0), "1");
        assert_eq!(process(-3, 1), "-3");
        assert_eq!(process(-3, 2), "6");
        assert_eq!(process(-3, 3), "-10");
    }

    #[test]
    fn test_20001_check_upper_limit() {
        assert_eq!(process_limit(3, 1, 3, 100), "3");
        assert_eq!(process_limit(4, 1, 3, 100), "TOOHIGH");

        assert_eq!(process_limit(80, 1, 80, 100), "80");
        assert_eq!(process_limit(81, 1, 80, 100), "TOOHIGH");
    }

    #[test]
    fn test_20002_check_lower_limit() {
        assert_eq!(process_limit(-3, 1, 3, 100), "-3");
        assert_eq!(process_limit(-4, 1, 3, 100), "TOOLOW");

        assert_eq!(process_limit(-80, 1, 80, 100), "-80");
        assert_eq!(process_limit(-81, 1, 80, 100), "TOOLOW");
    }

    #[test]
    fn test_20003_check_internal_value() {
        assert_eq!(process_limit(-9, 9, 80, 17), "INTERNEXCEEDEDLIMIT");
        assert_eq!(process_limit(-9, 9, 80, 18), "-24310");
    }
}
