use super::UnofficialFunctionId;
use std::error::Error;

pub trait UnofficialFunction: Send + Sync {
    fn id(&self) -> UnofficialFunctionId;
    fn name(&self) -> &'static str;
    fn execute(&self) -> Result<String, Box<dyn Error>>;
}
