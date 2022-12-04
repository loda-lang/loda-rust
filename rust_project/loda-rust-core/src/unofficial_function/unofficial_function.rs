use super::UnofficialFunctionId;
use num_bigint::BigInt;

pub trait UnofficialFunction: Send + Sync {
    fn id(&self) -> UnofficialFunctionId;
    fn name(&self) -> String;
    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>>;
}
