use super::EvalError;
use num_integer::{binomial, Integer};
use num_bigint::BigInt;
use num_traits::{Zero, One, Signed};

pub fn semantic_binomial(x: &BigInt, y: &BigInt) -> Result<BigInt, EvalError> {
    let input_n: &BigInt = x;
    let input_k: &BigInt = y;

    // positive n or zero
    if input_n.is_zero() || input_n.is_positive() {
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
        i += 1;
        value = value / i.clone();
    }
    Ok(value * sign)
}
