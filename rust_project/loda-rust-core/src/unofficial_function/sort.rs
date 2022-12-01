use super::{UnofficialFunction, UnofficialFunctionId};
use num_bigint::BigInt;

pub struct SortFunction {
    id: u32,
    inout_count: u8,
}

impl SortFunction {
    pub fn new(id: u32, inout_count: u8) -> Self {
        Self {
            id,
            inout_count,
        }
    }
}

impl UnofficialFunction for SortFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: self.inout_count, outputs: self.inout_count }
    }

    fn name(&self) -> String {
        format!("Sort {} values in ascending order", self.inout_count)
    }

    fn run(&self, mut input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        input.sort();
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::perform_run;

    #[test]
    fn test_ok() {
        {
            let f = SortFunction::new(0, 4);
            let v = perform_run(Box::new(f), vec![2, -1, -2, 0, 1]).expect("output");
            assert_eq!(v, vec![-2, -1, 0, 1, 2]);
        }
        {
            let f = SortFunction::new(0, 3);
            let v = perform_run(Box::new(f), vec![1, 2, 3]).expect("output");
            assert_eq!(v, vec![1, 2, 3]);
        }
        {
            let f = SortFunction::new(0, 3);
            let v = perform_run(Box::new(f), vec![3, 2, 1]).expect("output");
            assert_eq!(v, vec![1, 2, 3]);
        }
    }
}
