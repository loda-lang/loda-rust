#[cfg(test)]
mod tests {
    use loda_rust_core::execute::ProgramId;
    use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
    use loda_rust_core::execute::NodeRegisterLimit;
    use loda_rust_core::unofficial_function::UnofficialFunctionRegistry;
    use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
    use loda_rust_core::unofficial_function::register_common_functions;
    use crate::config::Config;
    use crate::arc::register_arc_functions;
    use num_bigint::{BigInt, ToBigInt};
    use std::path::PathBuf;

    #[test]
    fn test_20000_simple() -> anyhow::Result<()> {
        let program: &str = "
        mov $0,100
        mov $1,10
        mov $2,1
        f31 $0,1 ; Sum of 3 values

        ;mov $0,257
        ;mov $1,1
        ;mov $2,0
        ;f31 $0,100001

        ;mov $0,257
        ;mov $1,1
        ;mov $2,0
        ;f11 $0,100002
        ";

        let config = Config::load();
        let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    
        let registry = UnofficialFunctionRegistry::new();
        register_common_functions(&registry);
        register_arc_functions(&registry);

        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            loda_programs_oeis_dir,
            registry,
        );
        let result_parse = dm.parse(ProgramId::ProgramWithoutId, program);

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
            input, 
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
