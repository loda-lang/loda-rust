use super::EvalError;
use num_integer::{binomial, Integer};
use num_bigint::{BigInt, ToBigInt};
use num_traits::{Zero, One, Signed};
use lazy_static::lazy_static;

lazy_static! {
    static ref BINOMIAL_MAX_N: BigInt = 20.to_bigint().unwrap();
    static ref BINOMIAL_MAX_INTERNAL_VALUE: BigInt = (0xffff_ffff_ffff as i64).to_bigint().unwrap();
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
        if &value.abs() > &BINOMIAL_MAX_INTERNAL_VALUE {
            // debug!("too high an internal value: bin({:?},{:?}) value: {:?}", input_n, input_k, value);
            return Err(EvalError::BinomialDomainError);
        }
        i += 1;
        value = value / i.clone();
    }
    Ok(value * sign)
}
