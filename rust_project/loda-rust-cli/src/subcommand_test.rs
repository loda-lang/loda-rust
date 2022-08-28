//! The `loda-rust test-xyz` subcommands, runs automated tests.
use std::error::Error;

pub struct SubcommandTest {}

impl SubcommandTest {
    pub fn test_integration_with_lodacpp() -> Result<(), Box<dyn Error>> {
        println!("run tests here");
        Ok(())
    }
}
