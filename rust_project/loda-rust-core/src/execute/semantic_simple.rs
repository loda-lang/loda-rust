use super::EvalError;
use num_bigint::BigInt;
use num_bigint::BigUint;
use num_traits::{One, Zero, Signed, ToPrimitive};
use num_integer::Integer;
use std::cmp::Ordering;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SEMANTIC_SIMPLE_CONFIG_UNLIMITED: SemanticSimpleConfigUnlimited = SemanticSimpleConfigUnlimited {};

    pub static ref SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL: SemanticSimpleConfigLimited = SemanticSimpleConfigLimited::new(96);
}

#[derive(Debug)]
pub enum SemanticSimpleError {
    InputOutOfRange,
    OutputOutOfRange,
    DivisionByZero,
}

impl From<SemanticSimpleError> for EvalError {
    fn from(error: SemanticSimpleError) -> EvalError {
        match error {
            SemanticSimpleError::InputOutOfRange  => EvalError::InputOutOfRange,
            SemanticSimpleError::OutputOutOfRange => EvalError::OutputOutOfRange,
            SemanticSimpleError::DivisionByZero   => EvalError::DivisionByZero,
        }
    }
}

pub trait SemanticSimpleConfig {
    // #[inline(always)]
    fn value_max_bits(&self) -> Option<u64>;

    fn compute_add(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let z: BigInt = x + y;
        if let Some(value_max_bits) = self.value_max_bits() {
            if z.bits() >= value_max_bits {
                return Err(SemanticSimpleError::OutputOutOfRange);
            }
        }
        Ok(z)
    }

    fn compute_subtract(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let z: BigInt = x - y;
        if let Some(value_max_bits) = self.value_max_bits() {
            if z.bits() >= value_max_bits {
                return Err(SemanticSimpleError::OutputOutOfRange);
            }
        }
        Ok(z)
    }

    fn compute_truncate(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let z: BigInt = x - y;
        if !z.is_positive() {
            return Ok(BigInt::zero());
        }
        if let Some(value_max_bits) = self.value_max_bits() {
            if z.bits() >= value_max_bits {
                return Err(SemanticSimpleError::OutputOutOfRange);
            }
        }
        Ok(z)
    }

    fn compute_multiply(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let z: BigInt = x * y;
        if let Some(value_max_bits) = self.value_max_bits() {
            if z.bits() >= value_max_bits {
                return Err(SemanticSimpleError::OutputOutOfRange);
            }
        }
        Ok(z)
    }

    fn compute_divide(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        if y.is_zero() {
            return Err(SemanticSimpleError::DivisionByZero);
        }
        Ok(x / y)
    }

    fn compute_divide_if(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
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

    fn compute_divide_if_repeat(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let mut last_result: BigInt = x.clone();
        loop {
            let result: BigInt = self.compute_divide_if(&last_result, y)?;
            if result.abs() == last_result.abs() {
                break;
            }
            last_result = result;
        }
        Ok(last_result)
    }

    fn compute_modulo(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        if y.is_zero() {
            return Err(SemanticSimpleError::DivisionByZero);
        }
        Ok(x % y)
    }

    fn compute_gcd(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        // https://en.wikipedia.org/wiki/Binary_GCD_algorithm
        Ok(x.gcd(y))
    }

    fn compute_compare(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        if x == y {
            return Ok(BigInt::one());
        } else {
            return Ok(BigInt::zero());
        }
    }

    fn compute_min(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        Ok(x.min(y).clone())
    }

    fn compute_max(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        Ok(x.max(y).clone())
    }

    fn compute_logarithm(&self, n: &BigInt, base: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if n.bits() >= value_max_bits || base.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }

        if !n.is_positive() {
            return Err(SemanticSimpleError::InputOutOfRange);
        }
        if !base.is_positive() {
            return Err(SemanticSimpleError::InputOutOfRange);
        }
        if base.is_one() {
            return Err(SemanticSimpleError::InputOutOfRange);
        }

        // Cast from signed integers to BigUint
        let base: BigUint = match base.to_biguint() {
            Some(value) => value,
            None => return Err(SemanticSimpleError::InputOutOfRange),
        };
        let mut current_n: BigUint = match n.to_biguint() {
            Some(value) => value,
            None => return Err(SemanticSimpleError::InputOutOfRange),
        };
    
        let mut count: u64 = 0;
        while current_n >= base {
            current_n /= &base;
            count += 1;
        }
    
        let result: BigInt = BigInt::from(count);
        Ok(result)
    }

    fn compute_nthroot(&self, n: &BigInt, base: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if n.bits() >= value_max_bits || base.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }

        if n.is_negative() {
            return Err(SemanticSimpleError::InputOutOfRange);
        }
        if !base.is_positive() {
            return Err(SemanticSimpleError::InputOutOfRange);
        }
        let max_base: BigInt = BigInt::from(u32::MAX);
        if base > &max_base {
            return Err(SemanticSimpleError::InputOutOfRange);
        }
        let exponent: u32 = base.to_u32().unwrap();
    
        // Cast from signed integers to BigUint
        let n: BigUint = match n.to_biguint() {
            Some(value) => value,
            None => return Err(SemanticSimpleError::InputOutOfRange),
        };
    
        // Initialize binary search bounds
        let mut low = BigUint::zero();
        let mut high = n.clone();
        let mut result = BigUint::zero();
    
        while low <= high {
            let mid: BigUint = (&low + &high) >> 1; // Equivalent to (low + high) / 2
            let mid_pow = mid.pow(exponent); // Compute mid^base
    
            match mid_pow.cmp(&n) {
                Ordering::Less => {
                    result = mid.clone();
                    low = mid + 1u32;
                }
                Ordering::Greater => high = mid - 1u32,
                Ordering::Equal => return Ok(BigInt::from(mid)),
            }
        }
    
        Ok(BigInt::from(result))
    }

    fn compute_digitsum(&self, n: &BigInt, base: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if n.bits() >= value_max_bits || base.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }

        if !base.is_positive() {
            return Err(SemanticSimpleError::InputOutOfRange);
        }
        if base.is_one() {
            return Err(SemanticSimpleError::InputOutOfRange);
        }
        // "base" is 2 or greater

        let is_negative_result: bool = n.is_negative();
    
        // Cast base and n from BigInt to BigUint
        let base: BigUint = match base.to_biguint() {
            Some(value) => value,
            None => return Err(SemanticSimpleError::InputOutOfRange),
        };
    
        let mut remaining: BigUint = match n.abs().to_biguint() {
            Some(value) => value,
            None => return Err(SemanticSimpleError::InputOutOfRange),
        };
    
        let mut sum = BigUint::zero();
    
        // Calculate the digit sum by repeatedly dividing by the base
        while remaining > BigUint::zero() {
            let digit = &remaining % &base;
            sum += &digit;
            remaining /= &base;
        }

        // Adjust sign of the result
        let mut result = BigInt::from(sum);
        if is_negative_result {
            result = -result;
        }
        Ok(result)
    }

    fn compute_digitalroot(&self, n: &BigInt, base: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if n.bits() >= value_max_bits || base.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }

        if !base.is_positive() {
            return Err(SemanticSimpleError::InputOutOfRange);
        }
        if base.is_one() {
            return Err(SemanticSimpleError::InputOutOfRange);
        }
        // "base" is 2 or greater

        let is_negative_result: bool = n.is_negative();
    
        // Cast base and n from BigInt to BigUint
        let base: BigUint = match base.to_biguint() {
            Some(value) => value,
            None => return Err(SemanticSimpleError::InputOutOfRange),
        };
    
        let mut remaining: BigUint = match n.abs().to_biguint() {
            Some(value) => value,
            None => return Err(SemanticSimpleError::InputOutOfRange),
        };
    
        let mut sum = BigUint::zero();    
        loop {
            while remaining > BigUint::zero() {
                let digit: BigUint = &remaining % &base;
                sum += &digit;
                remaining /= &base;
            }

            if sum < base {
                // Stop iterating when the sum is less than the base
                break;
            }
            remaining = sum.clone();
            sum = BigUint::zero();
        }

        // Adjust sign of the result
        let mut result = BigInt::from(sum);
        if is_negative_result {
            result = -result;
        }
        Ok(result)
    }

    fn compute_equal(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        if x == y {
            return Ok(BigInt::one());
        } else {
            return Ok(BigInt::zero());
        }
    }

    fn compute_notequal(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        if x == y {
            return Ok(BigInt::zero());
        } else {
            return Ok(BigInt::one());
        }
    }

    fn compute_lessorequal(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        if x <= y {
            return Ok(BigInt::one());
        } else {
            return Ok(BigInt::zero());
        }
    }

    fn compute_greaterorequal(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        if x >= y {
            return Ok(BigInt::one());
        } else {
            return Ok(BigInt::zero());
        }
    }

    fn compute_bitwiseand(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let x_abs: BigInt = x.abs();
        let y_abs: BigInt = y.abs();
        let mut x_and_y: BigInt = x_abs & y_abs;
        if x.is_negative() && y.is_negative() {
            x_and_y = -x_and_y;
        }
        Ok(x_and_y)
    }

    fn compute_bitwiseor(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let x_abs: BigInt = x.abs();
        let y_abs: BigInt = y.abs();
        let mut x_or_y: BigInt = x_abs | y_abs;
        if x.is_negative() || y.is_negative() {
            x_or_y = -x_or_y;
        }
        Ok(x_or_y)
    }

    fn compute_bitwisexor(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(value_max_bits) = self.value_max_bits() {
            if x.bits() >= value_max_bits || y.bits() >= value_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let x_abs: BigInt = x.abs();
        let y_abs: BigInt = y.abs();
        let mut x_xor_y: BigInt = x_abs ^ y_abs;
        if x.is_negative() != y.is_negative() {
            x_xor_y = -x_xor_y;
        }
        Ok(x_xor_y)
    }
}

pub struct SemanticSimpleConfigUnlimited {}

impl SemanticSimpleConfig for SemanticSimpleConfigUnlimited {
    fn value_max_bits(&self) -> Option<u64> {
        None
    }
}

pub struct SemanticSimpleConfigLimited {
    value_max_bits: u64,
}

impl SemanticSimpleConfigLimited {
    fn new(value_max_bits: u64) -> Self {
        Self {
            value_max_bits: value_max_bits,
        }
    }
}

impl SemanticSimpleConfig for SemanticSimpleConfigLimited {
    fn value_max_bits(&self) -> Option<u64> {
        Some(self.value_max_bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigInt;

    enum ComputeMode {
        Add,
        Subtract,
        Truncate,
        Multiply,
        Divide,
        DivideIf,
        DivideIfRepeat,
        Modulo,
        GCD,
        Compare,
        Min,
        Max,
        Logarithm,
        NthRoot,
        DigitSum,
        DigitalRoot,
        Equal,
        NotEqual,
        LessOrEqual,
        GreaterOrEqual,
        BitwiseAnd,
        BitwiseOr,
        BitwiseXor,
    }

    fn compute(config: &dyn SemanticSimpleConfig, mode: ComputeMode, left: i64, right: i64) -> String {
        let x = left.to_bigint().unwrap();
        let y = right.to_bigint().unwrap();
        self::compute_bigint(config, mode, x, y)
    }

    fn compute_with_strings(config: &dyn SemanticSimpleConfig, mode: ComputeMode, left: &str, right: &str) -> String {
        let left_bigint = BigInt::parse_bytes(left.as_bytes(), 10).unwrap();
        let right_bigint = BigInt::parse_bytes(right.as_bytes(), 10).unwrap();
        self::compute_bigint(config, mode, left_bigint, right_bigint)
    }

    fn compute_bigint(config: &dyn SemanticSimpleConfig, mode: ComputeMode, x: BigInt, y: BigInt) -> String {
        let result = match mode {
            ComputeMode::Add            => config.compute_add(&x, &y),
            ComputeMode::Subtract       => config.compute_subtract(&x, &y),
            ComputeMode::Truncate       => config.compute_truncate(&x, &y),
            ComputeMode::Multiply       => config.compute_multiply(&x, &y),
            ComputeMode::Divide         => config.compute_divide(&x, &y),
            ComputeMode::DivideIf       => config.compute_divide_if(&x, &y),
            ComputeMode::DivideIfRepeat => config.compute_divide_if_repeat(&x, &y),
            ComputeMode::Modulo         => config.compute_modulo(&x, &y),
            ComputeMode::GCD            => config.compute_gcd(&x, &y),
            ComputeMode::Compare        => config.compute_compare(&x, &y),
            ComputeMode::Min            => config.compute_min(&x, &y),
            ComputeMode::Max            => config.compute_max(&x, &y),
            ComputeMode::Logarithm      => config.compute_logarithm(&x, &y),
            ComputeMode::NthRoot        => config.compute_nthroot(&x, &y),
            ComputeMode::DigitSum       => config.compute_digitsum(&x, &y),
            ComputeMode::DigitalRoot    => config.compute_digitalroot(&x, &y),
            ComputeMode::Equal          => config.compute_equal(&x, &y),
            ComputeMode::NotEqual       => config.compute_notequal(&x, &y),
            ComputeMode::LessOrEqual    => config.compute_lessorequal(&x, &y),
            ComputeMode::GreaterOrEqual => config.compute_greaterorequal(&x, &y),
            ComputeMode::BitwiseAnd     => config.compute_bitwiseand(&x, &y),
            ComputeMode::BitwiseOr      => config.compute_bitwiseor(&x, &y),
            ComputeMode::BitwiseXor     => config.compute_bitwisexor(&x, &y),
        };
        match result {
            Ok(value) => return value.to_string(),
            Err(SemanticSimpleError::InputOutOfRange)  => return "InputOutOfRange".to_string(),
            Err(SemanticSimpleError::OutputOutOfRange) => return "OutputOutOfRange".to_string(),
            Err(SemanticSimpleError::DivisionByZero)   => return "DivisionByZero".to_string(),
        }
    }

    fn compute_add(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::Add, left, right)
    }

    #[test]
    fn test_10000_add_basic() {
        assert_eq!(compute_add(100, 900), "1000");
        assert_eq!(compute_add(1001, -1), "1000");
        assert_eq!(compute_add(-1, -1), "-2");
        assert_eq!(compute_add(100, -100), "0");
        assert_eq!(compute_add(-100, 100), "0");
    }

    #[test]
    fn test_10001_add_out_of_range() {
        {
            assert_eq!(compute_add(0x7fffffff, 0), "2147483647");
            assert_eq!(compute_add(0x80000000, 0), "InputOutOfRange");
            assert_eq!(compute_add(0, 0x80000000), "InputOutOfRange");
            assert_eq!(compute_add(-0x80000000, 0), "InputOutOfRange");
            assert_eq!(compute_add(0, -0x80000000), "InputOutOfRange");
        }
        {
            assert_eq!(compute_add(0x6fffffff, 0x10000000), "2147483647");
            assert_eq!(compute_add(0x70000000, 0x10000000), "OutputOutOfRange");
        }
        {
            assert_eq!(compute_add(-0x6fffffff, -0x10000000), "-2147483647");
            assert_eq!(compute_add(-0x70000000, -0x10000000), "OutputOutOfRange");
        }
    }

    fn compute_subtract(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::Subtract, left, right)
    }

    #[test]
    fn test_20000_subtract_basic() {
        assert_eq!(compute_subtract(1001, 1), "1000");
        assert_eq!(compute_subtract(999, -1), "1000");
        assert_eq!(compute_subtract(-100, -100), "0");
        assert_eq!(compute_subtract(100, 300), "-200");
        assert_eq!(compute_subtract(0, 10), "-10");
    }

    #[test]
    fn test_20001_subtract_out_of_range() {
        {
            assert_eq!(compute_subtract(0x7fffffff, 0), "2147483647");
            assert_eq!(compute_subtract(0x80000000, 0), "InputOutOfRange");
            assert_eq!(compute_subtract(0, 0x80000000), "InputOutOfRange");
            assert_eq!(compute_subtract(-0x80000000, 0), "InputOutOfRange");
            assert_eq!(compute_subtract(0, -0x80000000), "InputOutOfRange");
        }
        {
            assert_eq!(compute_subtract(0x6fffffff, -0x10000000), "2147483647");
            assert_eq!(compute_subtract(0x70000000, -0x10000000), "OutputOutOfRange");
        }
        {
            assert_eq!(compute_subtract(-0x6fffffff, 0x10000000), "-2147483647");
            assert_eq!(compute_subtract(-0x70000000, 0x10000000), "OutputOutOfRange");
        }
    }

    fn compute_truncate(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::Truncate, left, right)
    }

    #[test]
    fn test_30000_truncate_basic() {
        // greater than 0
        assert_eq!(compute_truncate(1, 0), "1");
        assert_eq!(compute_truncate(22, 0), "22");
        assert_eq!(compute_truncate(22, 1), "21");
        assert_eq!(compute_truncate(22, 2), "20");
        assert_eq!(compute_truncate(22, 21), "1");
        assert_eq!(compute_truncate(-22, -23), "1");

        // zero
        assert_eq!(compute_truncate(0, 0), "0");
        assert_eq!(compute_truncate(22, 22), "0");
        assert_eq!(compute_truncate(-22, -22), "0");

        // less than 0
        assert_eq!(compute_truncate(22, 23), "0");
        assert_eq!(compute_truncate(22, 100), "0");
        assert_eq!(compute_truncate(-22, 100), "0");
    }

    #[test]
    fn test_30001_truncate_out_of_range() {
        assert_eq!(compute_truncate(0x7fffffff, 0x7fffffff), "0");
        assert_eq!(compute_truncate(-0x7fffffff, 0x7fffffff), "0");
        assert_eq!(compute_truncate(0x80000000, 0x80000000), "InputOutOfRange");
        assert_eq!(compute_truncate(0x7fffffff, -1), "OutputOutOfRange");
        assert_eq!(compute_truncate(0x7fffffff, -0x7fffffff), "OutputOutOfRange");
    }

    fn compute_multiply(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::Multiply, left, right)
    }

    #[test]
    fn test_40000_multiply_basic() {
        assert_eq!(compute_multiply(1, 1000), "1000");
        assert_eq!(compute_multiply(1000, 1), "1000");
        assert_eq!(compute_multiply(-1, -1), "1");
        assert_eq!(compute_multiply(0, 0), "0");
        assert_eq!(compute_multiply(0, 1), "0");
        assert_eq!(compute_multiply(1, 0), "0");
        assert_eq!(compute_multiply(1, 1), "1");
        assert_eq!(compute_multiply(-500, 2), "-1000");
    }

    #[test]
    fn test_40001_multiply_out_of_range() {
        {
            assert_eq!(compute_multiply(0x7fffffff, 1), "2147483647");
            assert_eq!(compute_multiply(1, 0x7fffffff), "2147483647");
            assert_eq!(compute_multiply(-1, -0x7fffffff), "2147483647");
            assert_eq!(compute_multiply(0x7fffffff, -1), "-2147483647");
            assert_eq!(compute_multiply(1, -0x7fffffff), "-2147483647");
        }
        {
            assert_eq!(compute_multiply(0x80000000, 1), "InputOutOfRange");
            assert_eq!(compute_multiply(1, 0x80000000), "InputOutOfRange");
            assert_eq!(compute_multiply(-0x80000000, 1), "InputOutOfRange");
            assert_eq!(compute_multiply(1, -0x80000000), "InputOutOfRange");
        }
        {
            assert_eq!(compute_multiply(0x7fffffff, 2), "OutputOutOfRange");
            assert_eq!(compute_multiply(2, 0x7fffffff), "OutputOutOfRange");
            assert_eq!(compute_multiply(0x8000000, 0x10), "OutputOutOfRange");
            assert_eq!(compute_multiply(0x10, 0x8000000), "OutputOutOfRange");
            assert_eq!(compute_multiply(-0x10, -0x8000000), "OutputOutOfRange");
            assert_eq!(compute_multiply(0x10, -0x8000000), "OutputOutOfRange");
        }
    }

    fn compute_divide(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::Divide, left, right)
    }

    #[test]
    fn test_50000_divide_by0() {
        assert_eq!(compute_divide(100, 0), "DivisionByZero");
        assert_eq!(compute_divide(-100, 0), "DivisionByZero");
    }

    #[test]
    fn test_50001_divide_by1() {
        assert_eq!(compute_divide(-100, 1), "-100");
        assert_eq!(compute_divide(0, 1), "0");
        assert_eq!(compute_divide(100, 1), "100");
        assert_eq!(compute_divide(-1, -1), "1");
        assert_eq!(compute_divide(0, -1), "0");
        assert_eq!(compute_divide(1, -1), "-1");
    }

    #[test]
    fn test_50002_divide_by2() {
        assert_eq!(compute_divide(-10, 2), "-5");
        assert_eq!(compute_divide(-9, 2), "-4");
        assert_eq!(compute_divide(-4, 2), "-2");
        assert_eq!(compute_divide(-3, 2), "-1");
        assert_eq!(compute_divide(-2, 2), "-1");
        assert_eq!(compute_divide(-1, 2), "0");
        assert_eq!(compute_divide(0, 2), "0");
        assert_eq!(compute_divide(1, 2), "0");
        assert_eq!(compute_divide(2, 2), "1");
        assert_eq!(compute_divide(3, 2), "1");
        assert_eq!(compute_divide(4, 2), "2");
        assert_eq!(compute_divide(9, 2), "4");
        assert_eq!(compute_divide(10, 2), "5");
    }

    #[test]
    fn test_50003_divide_big_values() {
        assert_eq!(compute_divide(50, 10), "5");
        assert_eq!(compute_divide(3, -3), "-1");
        assert_eq!(compute_divide(-3, 3), "-1");
    }

    #[test]
    fn test_50004_divide_inputoutofrange() {
        assert_eq!(compute_divide(0x7fffffff, 0x7fffffff), "1");
        assert_eq!(compute_divide(-0x7fffffff, -0x7fffffff), "1");
        assert_eq!(compute_divide(0x80000000, 1), "InputOutOfRange");
        assert_eq!(compute_divide(-0x80000000, 1), "InputOutOfRange");
        assert_eq!(compute_divide(0x80000001, 2), "InputOutOfRange");
        assert_eq!(compute_divide(-0x80000001, 2), "InputOutOfRange");
        assert_eq!(compute_divide(1, 0x7fffffff), "0");
        assert_eq!(compute_divide(1, -0x7fffffff), "0");
        assert_eq!(compute_divide(1, 0x80000000), "InputOutOfRange");
        assert_eq!(compute_divide(1, -0x80000000), "InputOutOfRange");
        assert_eq!(compute_divide(1, 0x80000001), "InputOutOfRange");
        assert_eq!(compute_divide(1, -0x80000001), "InputOutOfRange");
    }

    fn compute_divideif(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::DivideIf, left, right)
    }

    #[test]
    fn test_60000_divideif_remainder_zero() {
        assert_eq!(compute_divideif(50, 10), "5");
        assert_eq!(compute_divideif(100, 1), "100");
        assert_eq!(compute_divideif(42, -1), "-42");
        assert_eq!(compute_divideif(-1, -1), "1");
        assert_eq!(compute_divideif(3, -3), "-1");
        assert_eq!(compute_divideif(-3, 3), "-1");
    }

    #[test]
    fn test_60001_divideif_cannot_be_divided() {
        assert_eq!(compute_divideif(33, 10), "33");
        assert_eq!(compute_divideif(100, 33), "100");
        assert_eq!(compute_divideif(-100, -33), "-100");
    }

    #[test]
    fn test_60002_divideif_divisionbyzero() {
        assert_eq!(compute_divideif(100, 0), "100");
        assert_eq!(compute_divideif(0, 0), "0");
        assert_eq!(compute_divideif(-100, 0), "-100");
    }

    #[test]
    fn test_60003_divideif_inputoutofrange() {
        assert_eq!(compute_divideif(0x7fffffff, 0x7fffffff), "1");
        assert_eq!(compute_divideif(-0x7fffffff, -0x7fffffff), "1");
        assert_eq!(compute_divideif(0x80000000, 1), "InputOutOfRange");
        assert_eq!(compute_divideif(-0x80000000, 1), "InputOutOfRange");
        assert_eq!(compute_divideif(0x80000001, 2), "InputOutOfRange");
        assert_eq!(compute_divideif(-0x80000001, 2), "InputOutOfRange");
        assert_eq!(compute_divideif(1, 0x7fffffff), "1");
        assert_eq!(compute_divideif(1, -0x7fffffff), "1");
        assert_eq!(compute_divideif(1, 0x80000000), "InputOutOfRange");
        assert_eq!(compute_divideif(1, -0x80000000), "InputOutOfRange");
        assert_eq!(compute_divideif(1, 0x80000001), "InputOutOfRange");
        assert_eq!(compute_divideif(1, -0x80000001), "InputOutOfRange");
    }

    fn compute_divideifrepeat(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::DivideIfRepeat, left, right)
    }

    #[test]
    fn test_60004_divideifrepeat_remainder_zero() {
        assert_eq!(compute_divideifrepeat(50, 5), "2");
        assert_eq!(compute_divideifrepeat(0, 3), "0");
        assert_eq!(compute_divideifrepeat(0, -3), "0");
        assert_eq!(compute_divideifrepeat(1, -1), "1");
        assert_eq!(compute_divideifrepeat(6, -3), "-2");
    }

    #[test]
    fn test_60005_divideifrepeat_divisionbyzero() {
        assert_eq!(compute_divideifrepeat(-100, 0), "-100");
        assert_eq!(compute_divideifrepeat(0, 0), "0");
        assert_eq!(compute_divideifrepeat(100, 0), "100");
    }

    #[test]
    fn test_60006_divideifrepeat_inputoutofrange() {
        assert_eq!(compute_divideifrepeat(0x7fffffff, 0x7fffffff), "1");
        assert_eq!(compute_divideifrepeat(-0x7fffffff, -0x7fffffff), "1");
        assert_eq!(compute_divideifrepeat(0x80000000, 1), "InputOutOfRange");
        assert_eq!(compute_divideifrepeat(-0x80000000, 1), "InputOutOfRange");
        assert_eq!(compute_divideifrepeat(0x80000001, 2), "InputOutOfRange");
        assert_eq!(compute_divideifrepeat(-0x80000001, 2), "InputOutOfRange");
        assert_eq!(compute_divideifrepeat(1, 0x7fffffff), "1");
        assert_eq!(compute_divideifrepeat(1, -0x7fffffff), "1");
        assert_eq!(compute_divideifrepeat(1, 0x80000000), "InputOutOfRange");
        assert_eq!(compute_divideifrepeat(1, -0x80000000), "InputOutOfRange");
        assert_eq!(compute_divideifrepeat(1, 0x80000001), "InputOutOfRange");
        assert_eq!(compute_divideifrepeat(1, -0x80000001), "InputOutOfRange");
    }

    fn compute_modulo(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::Modulo, left, right)
    }

    #[test]
    fn test_70000_modulo() {
        assert_eq!(compute_modulo(100, 0), "DivisionByZero");
        assert_eq!(compute_modulo(-100, 0), "DivisionByZero");
        assert_eq!(compute_modulo(50, 10), "0");
        assert_eq!(compute_modulo(100, 1), "0");
        assert_eq!(compute_modulo(-1, -1), "0");
        assert_eq!(compute_modulo(3, -3), "0");
        assert_eq!(compute_modulo(-3, 3), "0");
        assert_eq!(compute_modulo(99, 99), "0");
        assert_eq!(compute_modulo(99, -99), "0");
        assert_eq!(compute_modulo(-99, 99), "0");
        assert_eq!(compute_modulo(-99, -99), "0");
        assert_eq!(compute_modulo(10, 3), "1");
        assert_eq!(compute_modulo(99, 10), "9");
        assert_eq!(compute_modulo( 999,  10), "9");
        assert_eq!(compute_modulo(-999,  10), "-9");
        assert_eq!(compute_modulo(-999, -10), "-9");
        assert_eq!(compute_modulo( 999, -10), "9");
    }

    fn compute_gcd(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::GCD, left, right)
    }

    #[test]
    fn test_80000_gcd_basic() {
        assert_eq!(compute_gcd(0, 0), "0");
        assert_eq!(compute_gcd(0, 1), "1");
        assert_eq!(compute_gcd(1, 0), "1");
        assert_eq!(compute_gcd(1, 1), "1");
        assert_eq!(compute_gcd(2, 2), "2");
        assert_eq!(compute_gcd(6, 4), "2");
        assert_eq!(compute_gcd(100, 55), "5");
        assert_eq!(compute_gcd(-100, 55), "5");
        assert_eq!(compute_gcd(-100, -55), "5");
        assert_eq!(compute_gcd(-100, 1), "1");
        assert_eq!(compute_gcd(43, 41), "1");
        assert_eq!(compute_gcd(100, 0), "100");
        assert_eq!(compute_gcd(-100, 0), "100");
    }

    #[test]
    fn test_80001_gcd_outofrange() {
        assert_eq!(compute_gcd(0x80000000, 1), "InputOutOfRange");
        assert_eq!(compute_gcd(-0x80000000, 1), "InputOutOfRange");
        assert_eq!(compute_gcd(0x80000000, 0x80000000), "InputOutOfRange");
        assert_eq!(compute_gcd(-0x80000000, -0x80000000), "InputOutOfRange");
        assert_eq!(compute_gcd(1, 0x80000000), "InputOutOfRange");
        assert_eq!(compute_gcd(1, -0x80000000), "InputOutOfRange");
    }

    fn compute_compare(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::Compare, left, right)
    }

    #[test]
    fn test_90000_compare_basic() {
        assert_eq!(compute_compare(100, 100), "1");
        assert_eq!(compute_compare(-33, -33), "1");
        assert_eq!(compute_compare(-1, 1), "0");
        assert_eq!(compute_compare(100, -100), "0");
        assert_eq!(compute_compare(0, 1), "0");
    }

    #[test]
    fn test_90001_compare_outofrange() {
        assert_eq!(compute_compare(0x80000000, 0), "InputOutOfRange");
        assert_eq!(compute_compare(-0x80000000, 0), "InputOutOfRange");
        assert_eq!(compute_compare(0, 0x80000000), "InputOutOfRange");
        assert_eq!(compute_compare(0, -0x80000000), "InputOutOfRange");
        assert_eq!(compute_compare(0x80000000, 0x80000000), "InputOutOfRange");
        assert_eq!(compute_compare(-0x80000000, -0x80000000), "InputOutOfRange");
    }

    fn compute_min(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::Min, left, right)
    }

    #[test]
    fn test_100000_min() {
        assert_eq!(compute_min(100, 900), "100");
        assert_eq!(compute_min(1001, -1), "-1");
        assert_eq!(compute_min(-1, -1), "-1");
        assert_eq!(compute_min(100, -100), "-100");
        assert_eq!(compute_min(-100, 100), "-100");
    }

    fn compute_max(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(32);
        compute(&config, ComputeMode::Max, left, right)
    }

    #[test]
    fn test_110000_max() {
        assert_eq!(compute_max(100, 900), "900");
        assert_eq!(compute_max(1001, -1), "1001");
        assert_eq!(compute_max(-1, -1), "-1");
        assert_eq!(compute_max(100, -100), "100");
        assert_eq!(compute_max(-100, 100), "100");
    }

    fn compute_logarithm(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(64);
        compute(&config, ComputeMode::Logarithm, left, right)
    }

    #[test]
    fn test_120000_logarithm() {
        assert_eq!(compute_logarithm(-1, 1), "InputOutOfRange");
        assert_eq!(compute_logarithm(-1, 2), "InputOutOfRange");
        assert_eq!(compute_logarithm(0, 1), "InputOutOfRange");
        assert_eq!(compute_logarithm(0, 2), "InputOutOfRange");
        assert_eq!(compute_logarithm(1,-1), "InputOutOfRange");
        assert_eq!(compute_logarithm(1, 0), "InputOutOfRange");
        assert_eq!(compute_logarithm(1, 1), "InputOutOfRange");
        assert_eq!(compute_logarithm(1, 2), "0");
        assert_eq!(compute_logarithm(1, 3), "0");
        assert_eq!(compute_logarithm(1, 4), "0");
        assert_eq!(compute_logarithm(2,-1), "InputOutOfRange");
        assert_eq!(compute_logarithm(2, 0), "InputOutOfRange");
        assert_eq!(compute_logarithm(2, 1), "InputOutOfRange");
        assert_eq!(compute_logarithm(2, 2), "1");
        assert_eq!(compute_logarithm(2, 3), "0");
        assert_eq!(compute_logarithm(2, 4), "0");
        assert_eq!(compute_logarithm(3, 2), "1");
        assert_eq!(compute_logarithm(3, 3), "1");
        assert_eq!(compute_logarithm(3, 4), "0");
        assert_eq!(compute_logarithm(4,-1), "InputOutOfRange");
        assert_eq!(compute_logarithm(4, 0), "InputOutOfRange");
        assert_eq!(compute_logarithm(4, 1), "InputOutOfRange");
        assert_eq!(compute_logarithm(4, 2), "2");
        assert_eq!(compute_logarithm(4, 3), "1");
        assert_eq!(compute_logarithm(4, 4), "1");
        assert_eq!(compute_logarithm(4, 5), "0");
        assert_eq!(compute_logarithm(8, 2), "3");
        assert_eq!(compute_logarithm(9, 3), "2");
        assert_eq!(compute_logarithm(16, 4), "2");
        assert_eq!(compute_logarithm(16, 2), "4");
        assert_eq!(compute_logarithm(10, 10), "1");
        assert_eq!(compute_logarithm(100, 10), "2");
        assert_eq!(compute_logarithm(1000, 10), "3");
        assert_eq!(compute_logarithm(10000, 10), "4");
        assert_eq!(compute_logarithm(100000, 10), "5");
        assert_eq!(compute_logarithm(1000000, 10), "6");
        assert_eq!(compute_logarithm(10000000, 10), "7");
        assert_eq!(compute_logarithm(100000000, 10), "8");
        assert_eq!(compute_logarithm(1000000000, 10), "9");
        assert_eq!(compute_logarithm(9999999999, 10), "9");
        assert_eq!(compute_logarithm(10000000000, 10), "10");
        assert_eq!(compute_logarithm(10000000001, 10), "10");
    }

    fn compute_nthroot(left: &str, right: i64) -> String {
        let left_bigint = BigInt::parse_bytes(left.as_bytes(), 10).unwrap();
        let right_bigint = right.to_bigint().unwrap();
        let config = SemanticSimpleConfigLimited::new(256);
        compute_bigint(&config, ComputeMode::NthRoot, left_bigint, right_bigint)
    }

    #[test]
    fn test_130000_nthroot() {
        assert_eq!(compute_nthroot("-1", 0), "InputOutOfRange");
        assert_eq!(compute_nthroot("-1", 1), "InputOutOfRange");
        assert_eq!(compute_nthroot("-1", 2), "InputOutOfRange");
        assert_eq!(compute_nthroot("0", 1), "0");
        assert_eq!(compute_nthroot("0", 2), "0");
        assert_eq!(compute_nthroot("0", 3), "0");
        assert_eq!(compute_nthroot("1", 0), "InputOutOfRange");
        assert_eq!(compute_nthroot("1", 1), "1");
        assert_eq!(compute_nthroot("1", 2), "1");
        assert_eq!(compute_nthroot("1", 3), "1");
        assert_eq!(compute_nthroot("1", 4), "1");
        assert_eq!(compute_nthroot("2", -1), "InputOutOfRange");
        assert_eq!(compute_nthroot("2", 0), "InputOutOfRange");
        assert_eq!(compute_nthroot("2", 1), "2");
        assert_eq!(compute_nthroot("2", 2), "1");
        assert_eq!(compute_nthroot("2", 3), "1");
        assert_eq!(compute_nthroot("2", 4), "1");
        assert_eq!(compute_nthroot("3", 2), "1");
        assert_eq!(compute_nthroot("3", 3), "1");
        assert_eq!(compute_nthroot("3", 4), "1");
        assert_eq!(compute_nthroot("4", -1), "InputOutOfRange");
        assert_eq!(compute_nthroot("4", 0), "InputOutOfRange");
        assert_eq!(compute_nthroot("4", 1), "4");
        assert_eq!(compute_nthroot("4", 2), "2");
        assert_eq!(compute_nthroot("4", 3), "1");
        assert_eq!(compute_nthroot("4", 4), "1");
        assert_eq!(compute_nthroot("5", 2), "2");
        assert_eq!(compute_nthroot("5", 3), "1");
        assert_eq!(compute_nthroot("5", 4), "1");
        assert_eq!(compute_nthroot("5", 5), "1");
        assert_eq!(compute_nthroot("5", 6), "1");
        assert_eq!(compute_nthroot("6", 2), "2");
        assert_eq!(compute_nthroot("6", 3), "1");
        assert_eq!(compute_nthroot("6", 4), "1");
        assert_eq!(compute_nthroot("7", 2), "2");
        assert_eq!(compute_nthroot("7", 3), "1");
        assert_eq!(compute_nthroot("8", 2), "2");
        assert_eq!(compute_nthroot("8", 3), "2");
        assert_eq!(compute_nthroot("8", 4), "1");
        assert_eq!(compute_nthroot("9", 2), "3");
        assert_eq!(compute_nthroot("9", 3), "2");
        assert_eq!(compute_nthroot("9", 4), "1");
        assert_eq!(compute_nthroot("10", 2), "3");
        assert_eq!(compute_nthroot("10", 3), "2");
        assert_eq!(compute_nthroot("10", 4), "1");
        assert_eq!(compute_nthroot("10", 5), "1");
        assert_eq!(compute_nthroot("11", 2), "3");
        assert_eq!(compute_nthroot("12", 2), "3");
        assert_eq!(compute_nthroot("13", 2), "3");
        assert_eq!(compute_nthroot("14", 2), "3");
        assert_eq!(compute_nthroot("15", 2), "3");
        assert_eq!(compute_nthroot("16", 2), "4");
        assert_eq!(compute_nthroot("64", 2), "8");
        assert_eq!(compute_nthroot("64", 3), "4");
        assert_eq!(compute_nthroot("64", 4), "2");
        assert_eq!(compute_nthroot("64", 6), "2");
        assert_eq!(compute_nthroot("64", 7), "1");
        assert_eq!(compute_nthroot("80", 2), "8");
        assert_eq!(compute_nthroot("81", 2), "9");
        assert_eq!(compute_nthroot("81", 4), "3");
        assert_eq!(compute_nthroot("82", 2), "9");
        assert_eq!(compute_nthroot("82", 4), "3");
        assert_eq!(compute_nthroot("100", 2), "10");
        assert_eq!(compute_nthroot("1000", 3), "10");
        assert_eq!(compute_nthroot("10000", 4), "10");
        assert_eq!(compute_nthroot("100000", 5), "10");
        assert_eq!(compute_nthroot("1000000", 6), "10");
        assert_eq!(compute_nthroot("10000000", 7), "10");
        assert_eq!(compute_nthroot("100000000", 8), "10");
        assert_eq!(compute_nthroot("1000000000", 9), "10");
        assert_eq!(compute_nthroot("10000000000", 10), "10");
        assert_eq!(compute_nthroot("100000000000", 11), "10");
        assert_eq!(compute_nthroot("1000000000000", 12), "10");
        assert_eq!(compute_nthroot("10000000000000", 13), "10");
        assert_eq!(compute_nthroot("100000000000000", 14), "10");
        assert_eq!(compute_nthroot("1000000000000000", 15), "10");
        assert_eq!(compute_nthroot("10000000000000000", 16), "10");
        assert_eq!(compute_nthroot("100000000000000000", 17), "10");
        assert_eq!(compute_nthroot("1000000000000000000", 18), "10");
        assert_eq!(compute_nthroot("10000000000000000000", 19), "10");
        assert_eq!(compute_nthroot("100000000000000000000", 20), "10");
        assert_eq!(compute_nthroot("1000000000000000000000", 21), "10");
        assert_eq!(compute_nthroot("10000000000000000000000", 22), "10");
        assert_eq!(compute_nthroot("100000000000000000000000", 23), "10");
        assert_eq!(compute_nthroot("1000000000000000000000000", 24), "10");
        assert_eq!(compute_nthroot("10000000000000000000000000", 25), "10");
    }

    fn compute_digitsum(left: &str, right: i64) -> String {
        let left_bigint = BigInt::parse_bytes(left.as_bytes(), 10).unwrap();
        let right_bigint = right.to_bigint().unwrap();
        let config = SemanticSimpleConfigLimited::new(128);
        compute_bigint(&config, ComputeMode::DigitSum, left_bigint, right_bigint)
    }

    #[test]
    fn test_140000_digitsum() {
        assert_eq!(compute_digitsum("10", -5), "InputOutOfRange");
        assert_eq!(compute_digitsum("10", -1), "InputOutOfRange");
        assert_eq!(compute_digitsum("10", 0), "InputOutOfRange");
        assert_eq!(compute_digitsum("-1", 1), "InputOutOfRange");
        assert_eq!(compute_digitsum("0", 1), "InputOutOfRange");
        assert_eq!(compute_digitsum("1", 1), "InputOutOfRange");
        assert_eq!(compute_digitsum("2", 1), "InputOutOfRange");
        assert_eq!(compute_digitsum("5", 10), "5");
        assert_eq!(compute_digitsum("15", 10), "6");
        assert_eq!(compute_digitsum("125", 10), "8");
        assert_eq!(compute_digitsum("1235", 10), "11");
        assert_eq!(compute_digitsum("12345", 10), "15");
        assert_eq!(compute_digitsum("-5", 10), "-5");
        assert_eq!(compute_digitsum("-15", 10), "-6");
        assert_eq!(compute_digitsum("-125", 10), "-8");
        assert_eq!(compute_digitsum("-1235", 10), "-11");
        assert_eq!(compute_digitsum("-12345", 10), "-15");
        assert_eq!(compute_digitsum("1", 2), "1");
        assert_eq!(compute_digitsum("3", 2), "2");
        assert_eq!(compute_digitsum("7", 2), "3");
        assert_eq!(compute_digitsum("15", 2), "4");
        assert_eq!(compute_digitsum("31", 2), "5");
        assert_eq!(compute_digitsum("-1", 2), "-1");
        assert_eq!(compute_digitsum("-3", 2), "-2");
        assert_eq!(compute_digitsum("-7", 2), "-3");
        assert_eq!(compute_digitsum("-15", 2), "-4");
        assert_eq!(compute_digitsum("-31", 2), "-5");
        assert_eq!(compute_digitsum("19", 3), "3");
        assert_eq!(compute_digitsum("18446744073709551615", 2), "64");
        assert_eq!(compute_digitsum("18446744073709551615", 4), "96");
    }

    fn compute_digitalroot(left: &str, right: i64) -> String {
        let left_bigint = BigInt::parse_bytes(left.as_bytes(), 10).unwrap();
        let right_bigint = right.to_bigint().unwrap();
        let config = SemanticSimpleConfigLimited::new(128);
        compute_bigint(&config, ComputeMode::DigitalRoot, left_bigint, right_bigint)
    }

    #[test]
    fn test_150000_digitalroot() {
        assert_eq!(compute_digitalroot("10", -5), "InputOutOfRange");
        assert_eq!(compute_digitalroot("10", -1), "InputOutOfRange");
        assert_eq!(compute_digitalroot("10", 0), "InputOutOfRange");
        assert_eq!(compute_digitalroot("-1", 1), "InputOutOfRange");
        assert_eq!(compute_digitalroot("0", 1), "InputOutOfRange");
        assert_eq!(compute_digitalroot("1", 1), "InputOutOfRange");
        assert_eq!(compute_digitalroot("2", 1), "InputOutOfRange");
        assert_eq!(compute_digitalroot("5", 10), "5");
        assert_eq!(compute_digitalroot("15", 10), "6");
        assert_eq!(compute_digitalroot("125", 10), "8");
        assert_eq!(compute_digitalroot("1235", 10), "2");
        assert_eq!(compute_digitalroot("12345", 10), "6");
        assert_eq!(compute_digitalroot("-5", 10), "-5");
        assert_eq!(compute_digitalroot("-15", 10), "-6");
        assert_eq!(compute_digitalroot("-125", 10), "-8");
        assert_eq!(compute_digitalroot("-1235", 10), "-2");
        assert_eq!(compute_digitalroot("-12345", 10), "-6");
        assert_eq!(compute_digitalroot("1", 2), "1");
        assert_eq!(compute_digitalroot("3", 2), "1");
        assert_eq!(compute_digitalroot("7", 2), "1");
        assert_eq!(compute_digitalroot("15", 2), "1");
        assert_eq!(compute_digitalroot("31", 2), "1");
        assert_eq!(compute_digitalroot("-1", 2), "-1");
        assert_eq!(compute_digitalroot("-3", 2), "-1");
        assert_eq!(compute_digitalroot("-7", 2), "-1");
        assert_eq!(compute_digitalroot("-15", 2), "-1");
        assert_eq!(compute_digitalroot("-31", 2), "-1");
        assert_eq!(compute_digitalroot("19", 3), "1");
        assert_eq!(compute_digitalroot("18446744073709551615", 2), "1");
        assert_eq!(compute_digitalroot("18446744073709551615", 4), "3");
    }

    fn compute_equal(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(64);
        compute(&config, ComputeMode::Equal, left, right)
    }

    #[test]
    fn test_160000_equal() {
        assert_eq!(compute_equal(0, 0), "1");
        assert_eq!(compute_equal(1, 1), "1");
        assert_eq!(compute_equal(2, 2), "1");
        assert_eq!(compute_equal(-1, -1), "1");
        assert_eq!(compute_equal(-2, -2), "1");
        assert_eq!(compute_equal(1, 0), "0");
        assert_eq!(compute_equal(0, 1), "0");
        assert_eq!(compute_equal(-1, 0), "0");
        assert_eq!(compute_equal(0, -1), "0");
    }

    fn compute_notequal(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(64);
        compute(&config, ComputeMode::NotEqual, left, right)
    }

    #[test]
    fn test_170000_notequal() {
        assert_eq!(compute_notequal(0, 0), "0");
        assert_eq!(compute_notequal(1, 1), "0");
        assert_eq!(compute_notequal(2, 2), "0");
        assert_eq!(compute_notequal(-1, -1), "0");
        assert_eq!(compute_notequal(-2, -2), "0");
        assert_eq!(compute_notequal(1, 0), "1");
        assert_eq!(compute_notequal(0, 1), "1");
        assert_eq!(compute_notequal(-1, 0), "1");
        assert_eq!(compute_notequal(0, -1), "1");
    }

    fn compute_lessorequal(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(64);
        compute(&config, ComputeMode::LessOrEqual, left, right)
    }

    #[test]
    fn test_180000_lessorequal() {
        assert_eq!(compute_lessorequal(0, 0), "1");
        assert_eq!(compute_lessorequal(1, 1), "1");
        assert_eq!(compute_lessorequal(2, 2), "1");
        assert_eq!(compute_lessorequal(-1, -1), "1");
        assert_eq!(compute_lessorequal(-2, -2), "1");
        assert_eq!(compute_lessorequal(1, 0), "0");
        assert_eq!(compute_lessorequal(0, 1), "1");
        assert_eq!(compute_lessorequal(-1, 0), "1");
        assert_eq!(compute_lessorequal(0, -1), "0");
    }

    fn compute_greaterorequal(left: i64, right: i64) -> String {
        let config = SemanticSimpleConfigLimited::new(64);
        compute(&config, ComputeMode::GreaterOrEqual, left, right)
    }

    #[test]
    fn test_190000_greaterorequal() {
        assert_eq!(compute_greaterorequal(0, 0), "1");
        assert_eq!(compute_greaterorequal(1, 1), "1");
        assert_eq!(compute_greaterorequal(2, 2), "1");
        assert_eq!(compute_greaterorequal(-1, -1), "1");
        assert_eq!(compute_greaterorequal(-2, -2), "1");
        assert_eq!(compute_greaterorequal(1, 0), "1");
        assert_eq!(compute_greaterorequal(0, 1), "0");
        assert_eq!(compute_greaterorequal(-1, 0), "0");
        assert_eq!(compute_greaterorequal(0, -1), "1");
    }

    fn compute_bitwiseand(left: &str, right: &str) -> String {
        let config = SemanticSimpleConfigLimited::new(128);
        compute_with_strings(&config, ComputeMode::BitwiseAnd, left, right)
    }

    #[test]
    fn test_200000_bitwiseand() {
        assert_eq!(compute_bitwiseand("0", "0"), "0");
        assert_eq!(compute_bitwiseand("0", "1"), "0");
        assert_eq!(compute_bitwiseand("1", "0"), "0");
        assert_eq!(compute_bitwiseand("1", "1"), "1");
        assert_eq!(compute_bitwiseand("1", "2"), "0");
        assert_eq!(compute_bitwiseand("1", "3"), "1");
        assert_eq!(compute_bitwiseand("-1", "1"), "1");
        assert_eq!(compute_bitwiseand("-1", "-1"), "-1");
        assert_eq!(compute_bitwiseand("-1", "2"), "0");
        assert_eq!(compute_bitwiseand("-1", "-2"), "0");
        assert_eq!(compute_bitwiseand("9223372036854775807", "9223372036854775808"), "0");
        assert_eq!(compute_bitwiseand("9223372036854775807", "-9223372036854775808"), "0");
        assert_eq!(compute_bitwiseand("3148244321913096809130", "1574122160956548404565"), "0");
    }

    fn compute_bitwiseor(left: &str, right: &str) -> String {
        let config = SemanticSimpleConfigLimited::new(128);
        compute_with_strings(&config, ComputeMode::BitwiseOr, left, right)
    }

    #[test]
    fn test_210000_bitwiseor() {
        assert_eq!(compute_bitwiseor("0", "0"), "0");
        assert_eq!(compute_bitwiseor("0", "1"), "1");
        assert_eq!(compute_bitwiseor("1", "0"), "1");
        assert_eq!(compute_bitwiseor("1", "1"), "1");
        assert_eq!(compute_bitwiseor("1", "2"), "3");
        assert_eq!(compute_bitwiseor("1", "3"), "3");
        assert_eq!(compute_bitwiseor("-1", "1"), "-1");
        assert_eq!(compute_bitwiseor("-1", "-1"), "-1");
        assert_eq!(compute_bitwiseor("-1", "2"), "-3");
        assert_eq!(compute_bitwiseor("-1", "-2"), "-3");
        assert_eq!(compute_bitwiseor("9223372036854775807", "9223372036854775808"), "18446744073709551615");
        assert_eq!(compute_bitwiseor("-9223372036854775807", "9223372036854775808"), "-18446744073709551615");
        assert_eq!(compute_bitwiseor("-9223372036854775807", "-9223372036854775808"), "-18446744073709551615");
        assert_eq!(compute_bitwiseor("3148244321913096809130", "1574122160956548404565"), "4722366482869645213695");        
    }

    fn compute_bitwisexor(left: &str, right: &str) -> String {
        let config = SemanticSimpleConfigLimited::new(128);
        compute_with_strings(&config, ComputeMode::BitwiseXor, left, right)
    }

    #[test]
    fn test_220000_bitwisexor() {
        assert_eq!(compute_bitwisexor("0", "0"), "0");
        assert_eq!(compute_bitwisexor("0", "1"), "1");
        assert_eq!(compute_bitwisexor("1", "0"), "1");
        assert_eq!(compute_bitwisexor("1", "1"), "0");
        assert_eq!(compute_bitwisexor("1", "2"), "3");
        assert_eq!(compute_bitwisexor("1", "3"), "2");
        assert_eq!(compute_bitwisexor("-1", "1"), "0");
        assert_eq!(compute_bitwisexor("-1", "-1"), "0");
        assert_eq!(compute_bitwisexor("-1", "2"), "-3");
        assert_eq!(compute_bitwisexor("-1", "-2"), "3");
        assert_eq!(compute_bitwisexor("9223372036854775807", "9223372036854775808"), "18446744073709551615");
        assert_eq!(compute_bitwisexor("-9223372036854775807", "9223372036854775808"), "-18446744073709551615");
        assert_eq!(compute_bitwisexor("-9223372036854775807", "-9223372036854775808"), "18446744073709551615");
        assert_eq!(compute_bitwisexor("3148244321913096809130", "1574122160956548404565"), "4722366482869645213695");
    }
}
