use super::{UnofficialFunction, UnofficialFunctionId};
use num_bigint::BigInt;

pub struct DebugFunction {
    id: u32,
}

impl DebugFunction {
    pub fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for DebugFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        "Debug, prints the content of a memory cell".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        for i in input {
            println!("DebugFunction: {:?}", i);
        }
        Ok(vec!())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::perform_run;

    #[test]
    fn test_ok() {
        let f = DebugFunction::new(0);
        let v = perform_run(Box::new(f), vec![1001]).expect("output");
        assert_eq!(v, vec!());
    }
}
