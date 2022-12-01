use crate::execute::{UnofficialFunction, UnofficialFunctionId};
use num_bigint::BigInt;
use num_traits::Zero;
use std::ops::Add;

pub struct SumFunction {
    id: u32,
    inputs: u8,
}

impl SumFunction {
    pub fn new(id: u32, inputs: u8) -> Self {
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
        let mut sum = BigInt::zero();
        for i in input {
            sum = sum.add(i);
        }
        let output: Vec<BigInt> = vec![sum];
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::execute::{UnofficialFunction, UnofficialFunctionId, UnofficialFunctionRegistry};
    // use num_bigint::{BigInt, ToBigInt};
    // use num_traits::Zero;
    // use std::ops::Add;
    // use std::path::PathBuf;

    // trait TestRun {
    //     fn test_run();    
    // }

    // impl TestRun for dyn UnofficialFunction {
    //     fn test_run() {
    //         println!("hello world");
    //     }
    // }


    // fn run(f: Box<dyn UnofficialFunction>) {

    // }

    #[test]
    fn test_1() -> anyhow::Result<()> {
        let _f = SumFunction::new(1234, 2);

        // f.test_run();
        // run(Box::new(f));
        // Assert
        // let input_vec: Vec<BigInt> = vec![1000.to_bigint().unwrap(), 1.to_bigint().unwrap()];
        // let output_vec: Vec<BigInt> = unofficial_function.run(input_vec).expect("output");
        // let expected_output_vec: Vec<BigInt> = vec![1001.to_bigint().unwrap()];
        // assert_eq!(output_vec, expected_output_vec);
        Ok(())
    }
}
