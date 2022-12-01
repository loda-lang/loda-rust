use num_bigint::BigInt;

use super::UnofficialFunctionId;
use std::error::Error;

pub trait UnofficialFunction: Send + Sync {
    fn id(&self) -> UnofficialFunctionId;
    fn name(&self) -> &'static str;
    fn execute(&self, input: Vec<BigInt>) -> Result<Vec<BigInt>, Box<dyn Error>>;
}
