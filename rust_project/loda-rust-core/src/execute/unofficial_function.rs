use std::error::Error;

pub trait UnofficialFunction: Send + Sync {
    fn function_id(&self) -> u64;
    fn name(&self) -> &'static str;
    fn inputs(&self) -> u8;
    fn outputs(&self) -> u8;
    fn execute(&self) -> Result<String, Box<dyn Error>>;
}
