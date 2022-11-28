use crate::config::Config;
use super::{Bitmap, Convolution3x3};
use super::{BitmapToNumber, NumberToBitmap};
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use std::time::Instant;
use std::rc::Rc;
use std::path::PathBuf;
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


        let result = self.convolution3x3(|bm| {
            let step_count_limit: u64 = 1000000000;
            let mut cache = ProgramCache::new();
            let mut step_count: u64 = 0;

            let input_raw_uint: BigUint = bm.to_number().unwrap();
            let input_raw_int: BigInt = input_raw_uint.to_bigint().unwrap();
            let input = RegisterValue(input_raw_int);
            // let result_run = program_runner.run(
            //     &input, 
            //     RunMode::Silent, 
            //     &mut step_count, 
            //     step_count_limit,
            //     NodeRegisterLimit::Unlimited,
            //     NodeLoopLimit::Unlimited,
            //     &mut cache
            // );
            // let output: RegisterValue = match result_run {
            //     Ok(value) => value,
            //     Err(error) => {
            //         panic!("Failure while computing term {}, error: {:?}", index, error);
            //     }
            // };

            42
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use loda_rust_core::execute::ProgramId;

    use super::*;
    use crate::arc::BitmapTryCreate;

    // #[test]
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
            min $5,$6
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
