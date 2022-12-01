use super::{UnofficialFunction, UnofficialFunctionId};
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
        let mut x = BigInt::zero();
        for i in input {
            x = x.add(i);
        }
        Ok(vec![x])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::{BigInt, ToBigInt};
    use num_traits::ToPrimitive;

    fn run(f: Box<dyn UnofficialFunction>, input: Vec<i32>) -> anyhow::Result<Vec<i32>> {
        let input_vec: Vec<BigInt> = input.iter().map(|v| v.to_bigint().unwrap() ).collect();
        let output_bigints: Vec<BigInt> = f.run(input_vec)?;
        let output: Vec<i32> = output_bigints.iter().map(|v| v.to_i32().unwrap() ).collect();
        Ok(output)
    }

    #[test]
    fn test_ok() {
        {
            let f = SumFunction::new(0, 2);
            let v = run(Box::new(f), vec![1001, -1]).expect("output");
            assert_eq!(v, vec![1000]);
        }
        {
            let f = SumFunction::new(0, 4);
            let v = run(Box::new(f), vec![100, 1, 1000, 10]).expect("output");
            assert_eq!(v, vec![1111]);
        }
    }
}
