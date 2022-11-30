#[cfg(test)]
mod tests {
    use loda_rust_core::execute::ProgramId;
    use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
    use loda_rust_core::execute::NodeRegisterLimit;
    use loda_rust_core::execute::{UnofficialFunction, UnofficialFunctionId, UnofficialFunctionRegistry};
    use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
    use crate::config::Config;
    use num_bigint::BigInt;
    use num_bigint::ToBigInt;
    use std::path::PathBuf;
    use std::error::Error;
    use std::sync::Arc;

    struct HelloWorldFunction;

    impl UnofficialFunction for HelloWorldFunction {
        fn id(&self) -> UnofficialFunctionId {
            UnofficialFunctionId::InputOutput { id: 1234, inputs: 1, outputs: 1 }
        }

        fn name(&self) -> &'static str {
            "Hello World"
        }

        fn execute(&self) -> Result<String, Box<dyn Error>> {
            debug!("execute");
            Ok("executed".to_string())
        }
    }

    #[test]
    fn test_10000_registry_lookup() -> anyhow::Result<()> {
        let registry = UnofficialFunctionRegistry::new();
        let plugin = HelloWorldFunction {};
        registry.register(Arc::new(Box::new(plugin)));

        // Act
        let key = UnofficialFunctionId::InputOutput { 
            id: 1234, 
            inputs: 1, 
            outputs: 1 
        };
        let unofficial_function: Arc<Box<dyn UnofficialFunction>> = registry.lookup(key).expect("unofficial_function");

        // Assert
        let execute_output: String = unofficial_function.execute().expect("string");
        assert_eq!(execute_output, "executed");
        Ok(())
    }

    #[test]
    fn test_10001_registry_clone() -> anyhow::Result<()> {
        // Arrange
        let registry_original = UnofficialFunctionRegistry::new();
        let registry: UnofficialFunctionRegistry = registry_original.clone();
        let plugin = HelloWorldFunction {};
        registry.register(Arc::new(Box::new(plugin)));
        
        // Act
        // Check that the original registry contains the item
        let key = UnofficialFunctionId::InputOutput { 
            id: 1234, 
            inputs: 1, 
            outputs: 1 
        };
        let unofficial_function: Arc<Box<dyn UnofficialFunction>> = registry_original.lookup(key).expect("unofficial_function");

        // Assert
        let execute_output: String = unofficial_function.execute().expect("string");
        assert_eq!(execute_output, "executed");
        Ok(())
    }

    // #[test]
    fn test_20000_simple() -> anyhow::Result<()> {
        let program_content: &str = "
        f11 $0,1234
        ";

        let config = Config::load();
        let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    
        let registry = UnofficialFunctionRegistry::new();
        let plugin = HelloWorldFunction {};
        registry.register(Arc::new(Box::new(plugin)));

        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            loda_programs_oeis_dir,
            registry,
        );
        let result_parse = dm.parse(ProgramId::ProgramWithoutId, &program_content.to_string());

        let program_runner: ProgramRunner = result_parse.expect("ProgramRunner");

        let step_count_limit: u64 = 1000000000;
        let mut cache = ProgramCache::new();
        let mut step_count: u64 = 0;

        let input_original: i32 = 132;
        let input_raw_int: BigInt = match input_original.to_bigint() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Integrity error. Couldn't convert to BigInt."));
            }
        };
        let input = RegisterValue(input_raw_int);
        
        let result_run = program_runner.run(
            &input, 
            RunMode::Silent, 
            &mut step_count, 
            step_count_limit,
            NodeRegisterLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            &mut cache
        );
        let output: RegisterValue = match result_run {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("run failed for input {:?}, error: {:?}", input_original, error));
            }
        };

        let output_i64: i64 = match output.try_to_i64() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("output value {} is out of range i64 when computing term for input {}", output, input_original));
            }
        };
        if output_i64 < 0 || output_i64 > 255 {
            return Err(anyhow::anyhow!("output value {} is out of range [0..255] when computing term for input {}", output, input_original));
        }
        let output: u8 = output_i64 as u8;

        assert_eq!(output, 42);

        Ok(())
    }
}
