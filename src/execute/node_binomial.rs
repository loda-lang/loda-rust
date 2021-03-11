use super::{Node,RegisterIndex,RegisterValue,ProgramState};
use num_integer::{binomial, Integer};
use num_bigint::BigInt;
use num_traits::{Zero, One, Signed};

fn perform_operation(x: RegisterValue, y: RegisterValue) -> RegisterValue {
    let input_n: &BigInt = &x.0;
    let input_k: &BigInt = &y.0;

    // TODO: deal with infinity

    // positive n or zero
    if input_n.is_zero() || input_n.is_positive() {
        if input_k.is_negative() || input_k > input_n {
            return RegisterValue::zero();
        }

        // Inside pascals triangle
        let n: BigInt = input_n.clone();
        let mut k: BigInt = input_k.clone();
        let k2: BigInt = k.clone() * 2;
        if k2 > n {
            k = n.clone() - k.clone();
        }
        let value: BigInt = binomial(n, k);
        return RegisterValue(value);
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
            return RegisterValue::zero();
        }
    }

    if k.is_negative() || k > n {
        return RegisterValue::zero();
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
        // TODO: deal with overflow
    }
    RegisterValue(value * sign)
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
    fn shorthand(&self) -> &str {
        "binomial register"
    }

    fn formatted_instruction(&self) -> String {
        format!("bin {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState) {
        let lhs: RegisterValue = state.get_register_value(self.target.clone());
        let rhs: RegisterValue = state.get_register_value(self.source.clone());
        let value = perform_operation(lhs, rhs);
        state.set_register_value(self.target.clone(), value);
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
        register_vec.push(self.source.clone());
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
    fn shorthand(&self) -> &str {
        "binomial constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("bin {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState) {
        let lhs: RegisterValue = state.get_register_value(self.target.clone());
        let rhs: RegisterValue = self.source.clone();
        let value = perform_operation(lhs, rhs);
        state.set_register_value(self.target.clone(), value);
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(left: i64, right: i64) -> String {
        let value: RegisterValue = perform_operation(
            RegisterValue::from_i64(left),
            RegisterValue::from_i64(right)
        );
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
}
