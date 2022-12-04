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
    use super::super::perform_run;

    #[test]
    fn test_ok() {
        {
            let f = SumFunction::new(0, 2);
            let v = perform_run(Box::new(f), vec![1001, -1]).expect("output");
            assert_eq!(v, vec![1000]);
        }
        {
            let f = SumFunction::new(0, 4);
            let v = perform_run(Box::new(f), vec![100, 1, 1000, 10]).expect("output");
            assert_eq!(v, vec![1111]);
        }
    }
}
