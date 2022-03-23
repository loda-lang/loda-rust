use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use std::collections::HashSet;
use num_integer::{binomial, Integer};
use num_bigint::{BigInt, ToBigInt};
use num_traits::{Zero, One, Signed};

#[derive(Clone)]
pub enum NodeBinomialLimit {
    Unlimited,
    LimitN(u8)
}

enum BinomialError {
    TooHighNValue,
    TooLowNValue,
}

impl From<BinomialError> for EvalError {
    fn from(_err: BinomialError) -> EvalError {
        EvalError::BinomialDomainError
    }
}

fn perform_operation(x: &RegisterValue, y: &RegisterValue, limit: &NodeBinomialLimit) -> Result<RegisterValue, BinomialError> {
    let input_n: &BigInt = &x.0;
    let input_k: &BigInt = &y.0;

    // positive n or zero
    if input_n.is_zero() || input_n.is_positive() {
        match limit {
            NodeBinomialLimit::Unlimited => {},
            NodeBinomialLimit::LimitN(max_n) => {
                if *input_n > max_n.to_bigint().unwrap() {
                    // debug!("too high a N value: bin({:?},{:?})", input_n, input_k);
                    return Err(BinomialError::TooHighNValue);
                }
            }
        }
        if input_k.is_negative() || input_k > input_n {
            return Ok(RegisterValue::zero());
        }

        // Inside pascals triangle
        let n: BigInt = input_n.clone();
        let mut k: BigInt = input_k.clone();
        let k2: BigInt = k.clone() * 2;
        if k2 > n {
            k = n.clone() - k.clone();
        }
        let value: BigInt = binomial(n, k);
        return Ok(RegisterValue(value));
    }

    match limit {
        NodeBinomialLimit::Unlimited => {},
        NodeBinomialLimit::LimitN(max_n) => {
            if input_n.abs() > max_n.to_bigint().unwrap() {
                // debug!("too low a N value: bin({:?},{:?})", input_n, input_k);
                return Err(BinomialError::TooLowNValue);
            }
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
            return Ok(RegisterValue::zero());
        }
    }

    if k.is_negative() || k > n {
        return Ok(RegisterValue::zero());
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
    Ok(RegisterValue(value * sign))
}

pub struct NodeBinomialRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeBinomialRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeBinomialRegister {
    fn formatted_instruction(&self) -> String {
        format!("bin {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = state.get_register_value_ref(&self.source);
        let value: RegisterValue = perform_operation(lhs, rhs, state.node_binomial_limit())?;
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
        register_vec.push(self.source.clone());
    }

    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        if register_set.contains(&self.source) {
            register_set.insert(self.target.clone());
        }
    }
}

pub struct NodeBinomialConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeBinomialConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeBinomialConstant {
    fn formatted_instruction(&self) -> String {
        format!("bin {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = &self.source;
        let value: RegisterValue = perform_operation(lhs, rhs, state.node_binomial_limit())?;
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(left: i64, right: i64) -> String {
        let limit = NodeBinomialLimit::Unlimited;
        process_inner(left, right, &limit)
    }

    fn process_limit(left: i64, right: i64, limit: u8) -> String {
        let limit = NodeBinomialLimit::LimitN(limit);
        process_inner(left, right, &limit)
    }

    fn process_inner(left: i64, right: i64, limit: &NodeBinomialLimit) -> String {
        let result = perform_operation(
            &RegisterValue::from_i64(left),
            &RegisterValue::from_i64(right),
            limit,
        );
        let value: RegisterValue = match result {
            Ok(value) => value,
            Err(BinomialError::TooHighNValue) => return "TOOHIGH".to_string(),
            Err(BinomialError::TooLowNValue) => return "TOOLOW".to_string(),
        };
        let v = value.to_i64();
        if v >= 0xffffff {
            return "BOOM".to_string();
        }
        v.to_string()
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
        assert_eq!(process_limit(3, 1, 3), "3");
        assert_eq!(process_limit(4, 1, 3), "TOOHIGH");

        assert_eq!(process_limit(80, 1, 80), "80");
        assert_eq!(process_limit(81, 1, 80), "TOOHIGH");
    }

    #[test]
    fn test_20002_check_lower_limit() {
        assert_eq!(process_limit(-3, 1, 3), "-3");
        assert_eq!(process_limit(-4, 1, 3), "TOOLOW");

        assert_eq!(process_limit(-80, 1, 80), "-80");
        assert_eq!(process_limit(-81, 1, 80), "TOOLOW");
    }
}
