use std::fmt;
use num_bigint::{BigInt, ToBigInt};
use num_traits::{ToPrimitive, One, Zero};

#[derive(Clone, Debug, PartialEq)]
pub struct RegisterValue(pub BigInt);

impl RegisterValue {
    pub fn zero() -> Self {
        RegisterValue(BigInt::zero())
    }

    pub fn one() -> Self {
        RegisterValue(BigInt::one())
    }

    pub fn minus_one() -> Self {
        RegisterValue(-BigInt::one())
    }

    pub fn from_i64(value: i64) -> Self {
        let value_bigint: BigInt = value.to_bigint().unwrap();
        RegisterValue(value_bigint)
    }

    #[allow(dead_code)]
    pub fn to_i64(&self) -> i64 {
        self.0.to_i64().unwrap()
    }
}

impl fmt::Display for RegisterValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
