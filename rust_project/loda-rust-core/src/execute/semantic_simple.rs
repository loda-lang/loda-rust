use super::EvalError;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, One, Zero, Signed};
use num_integer::Integer;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SEMANTIC_SIMPLE_CONFIG_UNLIMITED: SemanticSimpleConfigUnlimited = SemanticSimpleConfigUnlimited {};

    pub static ref SEMANTIC_SIMPLE_CONFIG_LIMIT_SMALL: SemanticSimpleConfigLimited = SemanticSimpleConfigLimited::new(96, 96);
}

#[derive(Debug)]
pub enum SemanticSimpleError {
    InputOutOfRange,
    OutputOutOfRange,
}

impl From<SemanticSimpleError> for EvalError {
    fn from(error: SemanticSimpleError) -> EvalError {
        match error {
            SemanticSimpleError::InputOutOfRange => EvalError::InputOutOfRange,
            SemanticSimpleError::OutputOutOfRange => EvalError::OutputOutOfRange,
        }
    }
}

pub trait SemanticSimpleConfig {
    // #[inline(always)]
    fn input_max_bits(&self) -> Option<u64>;

    // #[inline(always)]
    fn output_max_bits(&self) -> Option<u64>;

    fn compute_add(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(input_max_bits) = self.input_max_bits() {
            if x.bits() >= input_max_bits || y.bits() >= input_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let z: BigInt = x + y;
        if let Some(output_max_bits) = self.output_max_bits() {
            if z.bits() >= output_max_bits {
                return Err(SemanticSimpleError::OutputOutOfRange);
            }
        }
        Ok(z)
    }

    fn compute_subtract(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(input_max_bits) = self.input_max_bits() {
            if x.bits() >= input_max_bits || y.bits() >= input_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let z: BigInt = x - y;
        if let Some(output_max_bits) = self.output_max_bits() {
            if z.bits() >= output_max_bits {
                return Err(SemanticSimpleError::OutputOutOfRange);
            }
        }
        Ok(z)
    }

    fn compute_truncate(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(input_max_bits) = self.input_max_bits() {
            if x.bits() >= input_max_bits || y.bits() >= input_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let z: BigInt = x - y;
        if !z.is_positive() {
            return Ok(BigInt::zero());
        }
        if let Some(output_max_bits) = self.output_max_bits() {
            if z.bits() >= output_max_bits {
                return Err(SemanticSimpleError::OutputOutOfRange);
            }
        }
        Ok(z)
    }

    fn compute_multiply(&self, x: &BigInt, y: &BigInt) -> Result<BigInt, SemanticSimpleError> {
        if let Some(input_max_bits) = self.input_max_bits() {
            if x.bits() >= input_max_bits || y.bits() >= input_max_bits {
                return Err(SemanticSimpleError::InputOutOfRange);
            }
        }
        let z: BigInt = x * y;
        if let Some(output_max_bits) = self.output_max_bits() {
            if z.bits() >= output_max_bits {
                return Err(SemanticSimpleError::OutputOutOfRange);
            }
        }
        Ok(z)
    }
}

pub struct SemanticSimpleConfigUnlimited {}

impl SemanticSimpleConfig for SemanticSimpleConfigUnlimited {
    fn input_max_bits(&self) -> Option<u64> {
        None
    }

    fn output_max_bits(&self) -> Option<u64> {
        None
    }
}

pub struct SemanticSimpleConfigLimited {
    input_max_bits: u64,
    output_max_bits: u64,
}

impl SemanticSimpleConfigLimited {
    fn new(input_max_bits: u64, output_max_bits: u64) -> Self {
        Self {
            input_max_bits: input_max_bits,
            output_max_bits: output_max_bits,
        }
    }
}

impl SemanticSimpleConfig for SemanticSimpleConfigLimited {
    fn input_max_bits(&self) -> Option<u64> {
        Some(self.input_max_bits)
    }

    fn output_max_bits(&self) -> Option<u64> {
        Some(self.output_max_bits)
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
    }

    fn compute(config: &dyn SemanticSimpleConfig, mode: ComputeMode, left: i64, right: i64) -> String {
        let x = left.to_bigint().unwrap();
        let y = right.to_bigint().unwrap();
        let result = match mode {
            ComputeMode::Add      => config.compute_add(&x, &y),
            ComputeMode::Subtract => config.compute_subtract(&x, &y),
            ComputeMode::Truncate => config.compute_truncate(&x, &y),
            ComputeMode::Multiply => config.compute_multiply(&x, &y),
        };
        match result {
            Ok(value) => return value.to_string(),
            Err(SemanticSimpleError::InputOutOfRange) => return "InputOutOfRange".to_string(),
            Err(SemanticSimpleError::OutputOutOfRange) => return "OutputOutOfRange".to_string(),
            Err(error) => return format!("{:?}", error),
        }
    }

    fn compute_add(left: i64, right: i64) -> String {
        let limit: u64 = 32;
        let config = SemanticSimpleConfigLimited::new(limit, limit);
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
        let limit: u64 = 32;
        let config = SemanticSimpleConfigLimited::new(limit, limit);
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
        let limit: u64 = 32;
        let config = SemanticSimpleConfigLimited::new(limit, limit);
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
        let limit: u64 = 32;
        let config = SemanticSimpleConfigLimited::new(limit, limit);
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

}
