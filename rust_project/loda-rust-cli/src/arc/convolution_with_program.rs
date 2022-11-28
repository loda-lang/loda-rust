use super::{Bitmap, BitmapToNumber, convolution3x3};
use anyhow::Context;
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use num_bigint::{BigInt, BigUint};
use num_bigint::ToBigInt;

pub trait ConvolutionWithProgram {
    fn conv3x3_program(&self, program_runner: &ProgramRunner) -> anyhow::Result<Bitmap>;
}

impl ConvolutionWithProgram for Bitmap {
    fn conv3x3_program(&self, program_runner: &ProgramRunner) -> anyhow::Result<Bitmap> {
        // let mut cache = ProgramCache::new();
        // let step_count_limit: u64 = 1000000000;
        // let mut step_count: u64 = 0;

        // let step_count_limit: u64 = 1000000000;
        // let mut cache = ProgramCache::new();
        // let mut step_count: u64 = 0;
        // let input_raw_int: BigInt = 42u32.to_bigint().unwrap();
        // let input = RegisterValue(input_raw_int);
        // let result_run = program_runner.run(
        //     &input, 
        //     RunMode::Silent, 
        //     &mut step_count, 
        //     step_count_limit,
        //     NodeRegisterLimit::Unlimited,
        //     NodeLoopLimit::Unlimited,
        //     &mut cache
        // );


        let result = convolution3x3(&self, |bm| {
            let step_count_limit: u64 = 1000000000;
            let mut cache = ProgramCache::new();
            let mut step_count: u64 = 0;

            let input_raw_uint: BigUint = bm.to_number()?;
            let input_raw_int: BigInt = match input_raw_uint.to_bigint() {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Integrity error. Couldn't convert BigUint to BigInt. input {}", input_raw_uint));
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
            let output: RegisterValue = result_run
                .with_context(|| format!("run failed for input {}", input_raw_uint))?;

            let output_i64: i64 = match output.try_to_i64() {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("output value {} is out of range i64 when computing term for input {}", output, input_raw_uint));
                }
            };
            if output_i64 < 0 || output_i64 > 255 {
                return Err(anyhow::anyhow!("output value {} is out of range [0..255] when computing term for input {}", output, input_raw_uint));
            }
            let output: u8 = output_i64 as u8;
            Ok(output)
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loda_rust_core::execute::ProgramId;
    use crate::arc::BitmapTryCreate;
    use crate::config::Config;
    use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
    use std::path::PathBuf;
    
    #[test]
    fn test_10000_callback() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,2,3,4,
            5,6,7,8,
            9,10,11,12,
            13,14,15,16,
        ];
        let input: Bitmap = Bitmap::try_create(4, 4, pixels).expect("bitmap");

        let program_content: &str = "
        mov $1,$0
        mod $1,256 ; width
        div $0,256
        mov $2,$0
        mod $2,256 ; height
        div $0,256
        mov $3,$1
        mul $3,$2 ; number of pixels
        mov $5,255 ; inital value
        lpb $3
            mov $6,$0
            mod $6,256
            min $5,$6  ; pick the lowest pixel value 
            div $0,256
            sub $3,1
        lpe
        mov $0,$5
        ";

        let config = Config::load();
        let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    
        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            loda_programs_oeis_dir,
        );
        let result_parse = dm.parse(ProgramId::ProgramWithoutId, &program_content.to_string());

        let program_runner: ProgramRunner = result_parse.expect("ProgramRunner");
    
        // Act
        let output: Bitmap = input.conv3x3_program(&program_runner).expect("bitmap");

        // Assert
        assert_eq!(output.width(), 2);
        assert_eq!(output.height(), 2);
        assert_eq!(output.get(0, 0), Some(1));
        assert_eq!(output.get(1, 0), Some(2));
        assert_eq!(output.get(0, 1), Some(5));
        assert_eq!(output.get(1, 1), Some(6));
    }
}
