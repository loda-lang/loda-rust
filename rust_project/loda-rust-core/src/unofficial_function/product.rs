use super::{UnofficialFunction, UnofficialFunctionId};
use num_bigint::BigInt;
use num_traits::{Zero, One};
use std::ops::Mul;

pub struct ProductFunction {
    id: u32,
    inputs: u8,
}

impl ProductFunction {
    pub fn new(id: u32, inputs: u8) -> Self {
        Self {
            id,
            inputs,
        }
    }
}

impl UnofficialFunction for ProductFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: self.inputs, outputs: 1 }
    }

    fn name(&self) -> String {
        format!("Product of {} values", self.inputs)
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.is_empty() {
            return Ok(vec![BigInt::zero()]);
        }
        let mut x = BigInt::one();
        for i in input {
            x = x.mul(i);
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
            let f = ProductFunction::new(0, 0);
            let v = perform_run(Box::new(f), vec!()).expect("output");
            assert_eq!(v, vec![0]);
        }
        {
            let f = ProductFunction::new(0, 1);
            let v = perform_run(Box::new(f), vec![-42]).expect("output");
            assert_eq!(v, vec![-42]);
        }
        {
            let f = ProductFunction::new(0, 2);
            let v = perform_run(Box::new(f), vec![2, 3]).expect("output");
            assert_eq!(v, vec![6]);
        }
        {
            let f = ProductFunction::new(0, 3);
            let v = perform_run(Box::new(f), vec![2, 3, -4]).expect("output");
            assert_eq!(v, vec![-24]);
        }
    }
}
