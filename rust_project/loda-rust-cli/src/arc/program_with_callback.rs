#[cfg(test)]
mod tests {
    use loda_rust_core::execute::ProgramId;
    use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
    use loda_rust_core::execute::NodeRegisterLimit;
    use loda_rust_core::execute::{UnofficialFunction, UnofficialFunctionId, UnofficialFunctionRegistry};
    use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
    use crate::config::Config;
    use num_bigint::{BigInt, ToBigInt};
    use num_traits::Zero;
    use std::ops::Add;
    use std::path::PathBuf;
    use std::sync::Arc;

    struct SumFunction {
        id: u32,
        inputs: u8,
    }

    impl SumFunction {
        fn new(id: u32, inputs: u8) -> Self {
            Self {
                id,
                inputs,
            }
        }
    }

    impl UnofficialFunction for SumFunction {
        fn id(&self) -> UnofficialFunctionId {
            UnofficialFunctionId::InputOutput { id: self.id, inputs: self.inputs, outputs: 1 }
        }

        fn name(&self) -> String {
            format!("Sum of {} values", self.inputs)
        }

        fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
            println!("run input: {:?}", input);

            let mut sum = BigInt::zero();
            for i in input {
                sum = sum.add(i);
            }
            let output_vec: Vec<BigInt> = vec![sum];

            println!("run output: {:?}", output_vec);
    
            Ok(output_vec)
        }
    }

    #[test]
    fn test_10000_registry_lookup() -> anyhow::Result<()> {
        let registry = UnofficialFunctionRegistry::new();
        let plugin = SumFunction::new(1234, 2);
        registry.register(Arc::new(Box::new(plugin)));
        let key = UnofficialFunctionId::InputOutput { 
            id: 1234, 
            inputs: 2, 
            outputs: 1 
        };

        // Act
        let unofficial_function: Arc<Box<dyn UnofficialFunction>> = registry.lookup(key).expect("unofficial_function");

        // Assert
        let input_vec: Vec<BigInt> = vec![1000.to_bigint().unwrap(), 1.to_bigint().unwrap()];
        let output_vec: Vec<BigInt> = unofficial_function.run(input_vec).expect("output");
        let expected_output_vec: Vec<BigInt> = vec![1001.to_bigint().unwrap()];
        assert_eq!(output_vec, expected_output_vec);
        Ok(())
    }

    #[test]
    fn test_10001_registry_clone() -> anyhow::Result<()> {
        // Arrange
        let registry_original = UnofficialFunctionRegistry::new();
        let registry: UnofficialFunctionRegistry = registry_original.clone();
        let plugin = SumFunction::new(1234, 3);
        registry.register(Arc::new(Box::new(plugin)));
        let key = UnofficialFunctionId::InputOutput { 
            id: 1234, 
            inputs: 3, 
            outputs: 1 
        };
        
        // Act
        // Check that the original registry contains the item
        let unofficial_function: Arc<Box<dyn UnofficialFunction>> = registry_original.lookup(key).expect("unofficial_function");

        // Assert
        let input_vec: Vec<BigInt> = vec![100.to_bigint().unwrap(), 1.to_bigint().unwrap(), 10.to_bigint().unwrap()];
        let output_vec: Vec<BigInt> = unofficial_function.run(input_vec).expect("output");
        let expected_output_vec: Vec<BigInt> = vec![111.to_bigint().unwrap()];
        assert_eq!(output_vec, expected_output_vec);
        Ok(())
    }

    #[test]
    fn test_20000_simple() -> anyhow::Result<()> {
        let program_content: &str = "
        mov $0,100
        mov $1,10
        mov $2,1
        f31 $0,1234 ; Sum of 3 values
        ";

        let config = Config::load();
        let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    
        let registry = UnofficialFunctionRegistry::new();
        let plugin = SumFunction::new(1234, 3);
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

        assert_eq!(output_i64, 111);

        Ok(())
    }
}
