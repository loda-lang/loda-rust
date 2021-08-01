use num_bigint::BigInt;

pub trait CheckValue {
    fn is_valid(&self, value: &BigInt) -> bool;

    fn clone_boxed(&self) -> Box<dyn CheckValue>;
}

impl Clone for Box<dyn CheckValue> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

#[derive(Clone)]
pub struct OperationUnlimited {}

impl OperationUnlimited {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct OperationLimitBits {
    max_bits: u32,
}

impl OperationLimitBits {
    pub fn new(max_bits: u32) -> Self {
        Self {
            max_bits: max_bits,
        }
    }
}

impl CheckValue for OperationLimitBits {
    fn is_valid(&self, value: &BigInt) -> bool {
        if value.bits() >= self.max_bits.into() {
            return false;
        }
        true
    }

    fn clone_boxed(&self) -> Box<dyn CheckValue> {
        Box::new(self.clone())
    }
}

impl CheckValue for OperationUnlimited {
    fn is_valid(&self, _value: &BigInt) -> bool {
        true
    }

    fn clone_boxed(&self) -> Box<dyn CheckValue> {
        Box::new(self.clone())
    }
}

pub enum CheckValueError {
    InputOutOfRange,
    OutputOutOfRange,
}

pub struct PerformCheckValue {}

impl PerformCheckValue {
    pub fn check_input(check_value: &dyn CheckValue, value: &BigInt) -> Result<(), CheckValueError> {
        match check_value.is_valid(value) {
            true => Ok(()),
            false => Err(CheckValueError::InputOutOfRange)
        }
    }

    pub fn check_output(check_value: &dyn CheckValue, value: &BigInt) -> Result<(), CheckValueError> {
        match check_value.is_valid(value) {
            true => Ok(()),
            false => Err(CheckValueError::OutputOutOfRange)
        }
    }
}
