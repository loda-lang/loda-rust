use super::UnofficialFunctionId;
use num_bigint::BigInt;
use std::error::Error;

pub trait UnofficialFunction: Send + Sync {
    fn id(&self) -> UnofficialFunctionId;
    fn name(&self) -> String;
    fn run(&self, input: Vec<BigInt>) -> Result<Vec<BigInt>, Box<dyn Error>>;
}
