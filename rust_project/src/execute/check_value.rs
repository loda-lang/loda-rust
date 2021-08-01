use num_bigint::BigInt;
use super::EvalError;

pub trait CheckValue {
    fn is_valid(&self, value: &BigInt) -> bool;

    fn clone_boxed(&self) -> BoxCheckValue;
}

pub type BoxCheckValue = Box<dyn CheckValue>;

impl Clone for BoxCheckValue {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

pub trait PerformCheckValue {
    fn input(&self, value: &BigInt) -> Result<(), EvalError>;
    fn output(&self, value: &BigInt) -> Result<(), EvalError>;
}

impl PerformCheckValue for BoxCheckValue {
    fn input(&self, value: &BigInt) -> Result<(), EvalError> {
        match self.is_valid(value) {
            true => Ok(()),
            false => Err(EvalError::InputOutOfRange)
        }
    }

    fn output(&self, value: &BigInt) -> Result<(), EvalError> {
        match self.is_valid(value) {
            true => Ok(()),
            false => Err(EvalError::OutputOutOfRange)
        }
    }
}

#[derive(Clone)]
pub struct CheckValueUnlimited {}

impl CheckValueUnlimited {
    pub fn new() -> Self {
        Self {}
    }
}

impl CheckValue for CheckValueUnlimited {
    fn is_valid(&self, _value: &BigInt) -> bool {
        true
    }

    fn clone_boxed(&self) -> BoxCheckValue {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct CheckValueLimitBits {
    max_bits: u32,
}

impl CheckValueLimitBits {
    pub fn new(max_bits: u32) -> Self {
        Self {
            max_bits: max_bits,
        }
    }
}

impl CheckValue for CheckValueLimitBits {
    fn is_valid(&self, value: &BigInt) -> bool {
        if value.bits() >= self.max_bits.into() {
            return false;
        }
        true
    }

    fn clone_boxed(&self) -> BoxCheckValue {
        Box::new(self.clone())
    }
}
