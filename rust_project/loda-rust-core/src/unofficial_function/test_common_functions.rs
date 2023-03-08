#[cfg(test)]
mod tests {
    use crate::execute::ProgramId;
    use crate::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
    use crate::execute::NodeRegisterLimit;
    use crate::unofficial_function::UnofficialFunctionRegistry;
    use crate::control::{DependencyManager,DependencyManagerFileSystemMode};
    use crate::unofficial_function::register_common_functions;
    use num_bigint::{BigInt, ToBigInt};
    use std::path::PathBuf;

    #[test]
    fn test_10000_sum() {
        let program = "
        mov $0,1000
        mov $1,100
        mov $2,10
        mov $3,1
        f41 $0,1000 ; Sum of 4 values
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, 1111);
    }

    #[test]
    fn test_10001_product() {
        let program = "
        mov $0,2
        mov $1,3
        mov $2,4
        f31 $0,1010 ; Product of 3 values
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, 24);
    }

    #[test]
    fn test_10002_sort() {
        let program = "
        mov $0,89
        mov $1,100
        mov $2,-20
        mov $3,98
        f44 $0,1020 ; Sorting 4 values
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, -20);
    }

    #[test]
    fn test_20000_indirect_memory_access_ok() {
        let program = "
        mov $8,5
        mov $5,3
        mov $6,4
        f21 $$8,1010 ; Product of 2 values
        mov $0,$5
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, 12);
    }

    #[test]
    fn test_20001_indirect_memory_access_ok() {
        let program = "
        mov $8,1
        mov $1,3
        mov $2,4
        f21 $$8,1010 ; Product of 2 values
        mov $0,$1
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, 12);
    }

    #[test]
    fn test_20002_indirect_memory_access_ok() {
        let program = "
        mov $8,0
        mov $0,3
        mov $1,4
        f21 $$8,1010 ; Product of 2 values
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, 12);
    }

    #[test]
    fn test_20003_indirect_memory_access_error_negative_address() {
        let program = "
        mov $8,-1
        f21 $$8,1010 ; Product of 2 values
        ";
        run(program, 0).expect_err("negative address");
    }

    #[test]
    fn test_30000_assert_equal_ok() {
        let program = "
        mov $0,89
        mov $1,89
        f20 $0,1030 ; Assert input[0] is equal to input[1].
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, 89);
    }

    #[test]
    fn test_30001_assert_equal_error() {
        let program = "
        mov $0,89
        mov $1,42
        f20 $0,1030 ; Assert input[0] is equal to input[1].
        ";
        _ = run(program, 0).expect_err("should fail");
    }


    #[test]
    fn test_30002_assert_different_ok() {
        let program = "
        mov $0,-89
        mov $1,-42
        f20 $0,1031 ; Assert input[0] is different than input[1].
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, -89);
    }

    #[test]
    fn test_30003_assert_different_error() {
        let program = "
        mov $0,-89
        mov $1,-89
        f20 $0,1031 ; Assert input[0] is different than input[1].
        ";
        _ = run(program, 0).expect_err("should fail");
    }

    #[test]
    fn test_30004_assert_less_than_ok() {
        let program = "
        mov $0,-1000
        mov $1,-50
        f20 $0,1032 ; Assert input[0] is less than input[1].
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, -1000);
    }

    #[test]
    fn test_30005_assert_less_than_error() {
        let program = "
        mov $0,-50
        mov $1,-1000
        f20 $0,1032 ; Assert input[0] is less than input[1].
        ";
        _ = run(program, 0).expect_err("should fail");
    }

    #[test]
    fn test_30006_assert_less_than_or_equal_ok() {
        let program = "
        mov $0,9
        mov $1,9
        f20 $0,1033 ; Assert input[0] is less than or equal to input[1].
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, 9);
    }

    #[test]
    fn test_30007_assert_less_than_or_equal_error() {
        let program = "
        mov $0,10
        mov $1,9
        f20 $0,1033 ; Assert input[0] is less than or equal to input[1].
        ";
        _ = run(program, 0).expect_err("should fail");
    }

    #[test]
    fn test_30008_assert_greater_than_ok() {
        let program = "
        mov $0,-50
        mov $1,-1000
        f20 $0,1034 ; Assert input[0] is greater than input[1].
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, -50);
    }

    #[test]
    fn test_30009_assert_greater_than_error() {
        let program = "
        mov $0,-1000
        mov $1,-50
        f20 $0,1034 ; Assert input[0] is greater than input[1].
        ";
        _ = run(program, 0).expect_err("should fail");
    }

    #[test]
    fn test_30010_assert_greater_than_or_equal_ok() {
        let program = "
        mov $0,10
        mov $1,10
        f20 $0,1035 ; Assert input[0] is greater than or equal to input[1].
        ";
        let v: i64 = run(program, 0).expect("output");
        assert_eq!(v, 10);
    }

    #[test]
    fn test_30011_assert_greater_than_or_equal_error() {
        let program = "
        mov $0,9
        mov $1,10
        f20 $0,1035 ; Assert input[0] is greater than or equal to input[1].
        ";
        _ = run(program, 0).expect_err("should fail");
    }

    /// Run program with 1 input and 1 output
    fn run<S: AsRef<str>>(program: S, input: i64) -> anyhow::Result<i64> {
        let program_str: &str = program.as_ref();

        let registry = UnofficialFunctionRegistry::new();
        register_common_functions(&registry);

        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::Virtual,
            PathBuf::from("non-existing-dir"),
            registry,
        );
        let result_parse = dm.parse(ProgramId::ProgramWithoutId, program_str);

        let program_runner: ProgramRunner = result_parse.expect("ProgramRunner");

        let step_count_limit: u64 = 1000000000;
        let mut cache = ProgramCache::new();
        let mut step_count: u64 = 0;

        // Input
        let input_bigint: BigInt = match input.to_bigint() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Cannot convert {} to BigInt.", input));
            }
        };
        let input_registervalue = RegisterValue(input_bigint);
        
        // Run
        let result_run = program_runner.run(
            input_registervalue,
            RunMode::Silent, 
            &mut step_count, 
            step_count_limit,
            NodeRegisterLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            &mut cache
        );

        // Output
        let output: RegisterValue = match result_run {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("run failed for input {:?}, error: {:?}", input, error));
            }
        };
        let output_i64: i64 = match output.try_to_i64() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("output value {} is out of range i64 when computing term for input {}", output, input));
            }
        };
        Ok(output_i64)
    }
}
