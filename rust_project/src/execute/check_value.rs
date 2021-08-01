use num_bigint::BigInt;

pub enum CheckValueError {
    OutOfRange,
}

pub trait CheckValue {
    fn check_value(&self, value: &BigInt) -> Result<(), CheckValueError>;

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
    fn check_value(&self, value: &BigInt) -> Result<(), CheckValueError> {
        if value.bits() >= self.max_bits.into() {
            return Err(CheckValueError::OutOfRange);
        }
        Ok(())
    }

    fn clone_boxed(&self) -> Box<dyn CheckValue> {
        Box::new(self.clone())
    }
}

impl CheckValue for OperationUnlimited {
    fn check_value(&self, _value: &BigInt) -> Result<(), CheckValueError> {
        Ok(())
    }

    fn clone_boxed(&self) -> Box<dyn CheckValue> {
        Box::new(self.clone())
    }
}
